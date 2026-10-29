/**
 Shared helpers of the native-interoperability drivers.
 They run only inside the image `bin/interop-native-container.mjs` builds,
 where `/opt/native/bin/git` is the native wrapper and `/usr/bin/git` is Git 2.56.0.
 Every repository is restored below `/work/scratch` and never leaves the container.
 */
/// <reference types="node" />
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  readdir,
  stat,
} from 'node:fs/promises';
import { join } from 'node:path';

/** Directory whose `git` is the native wrapper. */
export const NATIVE_BIN = '/opt/native/bin';
/** Real Git 2.56.0. */
export const REAL_GIT = '/usr/bin/git';
/** The system search path without the wrapper. */
export const SYSTEM_PATH = '/usr/local/bin:/usr/bin:/bin';

/** A driver step whose result contradicts what it proves. */
export class InteropError extends Error {
  name = 'InteropError';
}

/**
 Environment of one run: no host configuration, fixed identities, the native wrapper first on `PATH` when asked.

 @param {{ home: string, wrapperBin?: string, extra?: Record<string, string> }} request -
   disposable home, the directory whose `git` the run starts (none means real Git), and added variables
 @returns {Record<string, string>} complete environment, nothing inherited from the container
 */
export function runEnvironment({
  home,
  wrapperBin,
  extra = {},
}) {
  return {
    HOME: home,
    PATH: wrapperBin === undefined ? SYSTEM_PATH : `${wrapperBin}:${SYSTEM_PATH}`,
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: '/dev/null',
    GIT_AUTHOR_NAME: 'Interop Author',
    GIT_AUTHOR_EMAIL: 'author@example.invalid',
    GIT_COMMITTER_NAME: 'Interop Committer',
    GIT_COMMITTER_EMAIL: 'committer@example.invalid',
    LC_ALL: 'C',
    ...extra,
  };
}

/**
 Outcome of one finished process.
 @typedef {{ status: number | null, signal: NodeJS.Signals | null, stdout: string, stderr: string }} RunOutcome
 */

/**
 Run one argument array without a shell and capture its output.

 @param {{ command: string, args: readonly string[], cwd: string, env: Record<string, string>, input?: string }} request -
   executable, arguments, working directory, complete environment and optional standard input
 @returns {Promise<RunOutcome>} exit state and decoded output
 */
export async function run({
  command,
  args,
  cwd,
  env,
  input,
}) {
  const child = spawn(
    command,
    [...args],
    {
      cwd,
      env,
      stdio: [
        'pipe',
        'pipe',
        'pipe',
      ],
    },
  );
  /** @type {Buffer[]} */
  const out = [];
  /** @type {Buffer[]} */
  const err = [];
  child.stdout
    .on(
    'data',
    /** @param {Buffer} chunk - bytes the process wrote to standard output */
    function collectStdout(chunk) {
      out.push(chunk);
    },
    );
  child.stderr
    .on(
    'data',
    /** @param {Buffer} chunk - bytes the process wrote to standard error */
    function collectStderr(chunk) {
      err.push(chunk);
    },
    );
  // A command that never reads its input (or replaced its process) closes the pipe early;
  // the resulting EPIPE says nothing about the command and must not end the driver.
  child.stdin
    .on(
    'error',
    function ignoreClosedInput() {
      // Nothing to do: the command already ended or replaced itself, and its exit state is read below.
    },
    );
  child.stdin
    .end(input ?? '');
  await once(
    child,
    'close',
  );
  return {
    status: child.exitCode,
    signal: child.signalCode,
    stdout: Buffer.concat(out)
      .toString('utf8'),
    stderr: Buffer.concat(err)
      .toString('utf8'),
  };
}

/**
 Run `git` through the native wrapper in a fixture and return the outcome without judging it.

 @param {{ wrapper: 'native', fixture: { repository: string, home: string }, args: readonly string[],
   extra?: Record<string, string> }} request - wrapper, repository, Git arguments and added variables
 @returns {Promise<RunOutcome>} exit state and output
 */
export function runThrough({
  wrapper,
  fixture,
  args,
  extra = {},
}) {
  if (wrapper !== 'native')
    throw new InteropError(`Only the native wrapper exists in this image, not ${String(wrapper)}.`);
  return run({
    command: 'git',
    args,
    cwd: fixture.repository,
    env: runEnvironment({
      home: fixture.home,
      wrapperBin: NATIVE_BIN,
      extra: {
        MONOCHROMATIC_WARN: 'false',
        ...extra
      },
    }),
  });
}

/**
 Run real Git in a fixture and return its trimmed output.

 @param {{ fixture: { repository: string, home: string }, args: readonly string[] }} request -
   repository and home, and the Git arguments
 @returns {Promise<string>} trimmed standard output
 */
export async function realGit({
  fixture,
  args,
}) {
  const outcome = await run({
    command: REAL_GIT,
    args,
    cwd: fixture.repository,
    env: runEnvironment({ home: fixture.home }),
  });
  if (outcome.status !== 0)
    throw new InteropError(
      `${REAL_GIT} ${args.join(' ')} exited ${String(outcome.status)} (${String(outcome.signal)}):\n`
        + `${outcome.stdout}${outcome.stderr}`,
    );
  return outcome.stdout
    .trim();
}

/**
 Whether a caught value is the error Node raises for a path that does not exist.

 @param {unknown} error - caught value
 @returns {boolean} true for an `ENOENT` error
 */
export function isMissingPath(error) {
  // `Object` wraps a primitive and turns `null` into an empty object, so any caught value can be asked for a code.
  /** @type {unknown} */
  const code = Reflect.get(
    new Object(error),
    'code',
  );
  return code === 'ENOENT';
}

/**
 Entry names of a directory, or none when it does not exist.

 @param {string} directory - directory to list
 @returns {Promise<string[]>} entry names
 */
export async function namesIn(directory) {
  try {
    return await readdir(directory);
  }
  catch (error) {
    if (isMissingPath(error))
      return [];
    throw error;
  }
}

/**
 Whether a path exists.

 @param {string} path - path to test
 @returns {Promise<boolean>} true when something is there
 */
export async function pathExists(path) {
  try {
    await stat(path);
    return true;
  }
  catch (error) {
    if (isMissingPath(error))
      return false;
    throw error;
  }
}

/**
 Everything a wrapper may leave behind in a repository after a transaction ends.

 Lock directories whose holder died are retired lazily, by the next command that acquires the lock, so a command
 that never takes that lock (a `status`, an `add` that needs no landing) leaves them in place by design.
 `staleLocks` accepts them for such a command.

 @param {{ fixture: { repository: string }, staleLocks?: boolean }} request -
   repository, and whether lock directories count as leftovers
 @returns {Promise<string[]>} descriptions of each leftover, none when the repository is quiescent
 */
export async function leftovers({
  fixture,
  staleLocks = false,
}) {
  const gitDir = join(
    fixture.repository,
    '.git',
  );
  const found = [];
  for (const name of await namesIn(join(
    gitDir,
    'cli-git-transactions',
  ))) {
    if (!(staleLocks && name.endsWith('.lock')))
      found.push(`cli-git-transactions/${name}`);
  }
  for (const name of await namesIn(join(
    gitDir,
    'cli-git',
    'shadow',
  )))
    found.push(`cli-git/shadow/${name}`);
  for (const name of await namesIn(gitDir)) {
    if (name === 'index.lock')
      found.push('index.lock');
  }
  for (const name of await namesIn(join(
    gitDir,
    'objects',
    'pack',
  ))) {
    if (name.endsWith('.keep'))
      found.push(`objects/pack/${name}`);
  }
  if ((!staleLocks) && await pathExists(join(
    gitDir,
    'cli-git-captures',
    'capture.lock',
  )))
    found.push('cli-git-captures/capture.lock');
  return found;
}

/**
 Collects every check of a driver instead of stopping at the first failure,
 so one failing check never hides the others; the driver ends with `finish`.

 @returns {{ check: (request: { condition: boolean, message: string }) => boolean, finish: () => void }} recorder
 */
export function createMatrix() {
  /** @type {string[]} */
  const failures = [];
  /** @type {string[]} */
  const passes = [];
  return {
    check({
      condition,
      message,
    }) {
      if (condition) {
        passes.push(message);
        console.log(`ok: ${message}`);
      }
      else {
        failures.push(message);
        console.log(`FAIL: ${message}`);
      }
      return condition;
    },
    finish() {
      console.log(`${String(passes.length)} checks passed, ${String(failures.length)} failed`);
      if (failures.length > 0)
        throw new InteropError(`${String(failures.length)} checks failed:\n${failures.join('\n')}`);
    },
  };
}
