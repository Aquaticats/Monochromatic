/**
 Shared helpers of the native-interoperability drivers.
 They run only inside the image `bin/interop-native-container.ts` builds,
 where `/opt/native/bin/git` is the native wrapper and `/usr/bin/git` is Git 2.56.0.
 Every repository is restored below `/work/scratch` and never leaves the container.
 */
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  readdir,
  stat,
} from 'node:fs/promises';
import { join } from 'node:path';

/**
 Directory whose `git` is the native wrapper.
 */
export const NATIVE_BIN = '/opt/native/bin';
/**
 Real Git 2.56.0.
 */
export const REAL_GIT = '/usr/bin/git';
/**
 The system search path without the wrapper.
 */
export const SYSTEM_PATH = '/usr/local/bin:/usr/bin:/bin';

/**
 A driver step whose result contradicts what it proves.
 */
export class InteropError extends Error {
  /**
   Stable class name that tells a contradicted driver step apart from an unrelated error in a log.
   */
  override name = 'InteropError';
}

/**
 Environment of one run: no host configuration, fixed identities, the native wrapper first on `PATH` when asked.

 @param home - disposable home

 @param wrapperBin - the directory whose `git` the run starts (none means real Git)

 @param extra - added variables

 @returns complete environment, nothing inherited from the container

 @example
 ```ts
 const env = runEnvironment({ home: '/work/scratch/home', wrapperBin: NATIVE_BIN });
 ```
 */
export function runEnvironment({
  home,
  wrapperBin,
  extra = {},
}: {
  readonly home: string;
  readonly wrapperBin?: string;
  readonly extra?: Readonly<Record<string, string>>;
}): Readonly<Record<string, string>> {
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
 */
export type RunOutcome = {
  readonly status?: number;
  readonly signal?: NodeJS.Signals;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Run one argument array without a shell and capture its output.

 @param command - executable to start

 @param args - its arguments

 @param cwd - working directory

 @param env - complete environment

 @param input - optional standard input

 @returns exit state and decoded output

 @example
 ```ts
 const outcome = await run({ command: 'git', args: ['--version'], cwd: '/work/scratch', env });
 ```
 */
export async function run({
  command,
  args,
  cwd,
  env,
  input,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly env: Readonly<Record<string, string>>;
  readonly input?: string;
}): Promise<RunOutcome> {
  /**
   The started process, whose pipes are drained below.
   */
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
  /**
   Chunks the process wrote to standard output.
   */
  const out: Buffer[] = [];
  /**
   Chunks the process wrote to standard error.
   */
  const err: Buffer[] = [];
  child.stdout
    .on(
    'data',
    /**
     @param chunk - bytes the process wrote to standard output
     */
    function collectStdout(chunk: Buffer): void {
      out.push(chunk);
    },
    );
  child.stderr
    .on(
    'data',
    /**
     @param chunk - bytes the process wrote to standard error
     */
    function collectStderr(chunk: Buffer): void {
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
    ...(child.exitCode === null ? {} : { status: child.exitCode, }),
    ...(child.signalCode === null ? {} : { signal: child.signalCode, }),
    stdout: Buffer.concat(out)
      .toString('utf8'),
    stderr: Buffer.concat(err)
      .toString('utf8'),
  };
}

/**
 Run `git` through the native wrapper in a fixture and return the outcome without judging it.

 @param wrapper - which wrapper runs; only the native one exists in the image

 @param fixture - repository and home of the fixture

 @param args - Git arguments

 @param extra - added variables

 @returns exit state and output

 @example
 ```ts
 const outcome = await runThrough({ wrapper: 'native', fixture, args: ['status'] });
 ```
 */
export function runThrough({
  wrapper,
  fixture,
  args,
  extra = {},
}: {
  readonly wrapper: 'native';
  readonly fixture: {
    readonly repository: string;
    readonly home: string;
  };
  readonly args: readonly string[];
  readonly extra?: Readonly<Record<string, string>>;
}): Promise<RunOutcome> {
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

 @param fixture - repository and home of the fixture

 @param args - the Git arguments

 @returns trimmed standard output

 @example
 ```ts
 const head = await realGit({ fixture, args: ['rev-parse', 'HEAD'] });
 ```
 */
export async function realGit({
  fixture,
  args,
}: {
  readonly fixture: {
    readonly repository: string;
    readonly home: string;
  };
  readonly args: readonly string[];
}): Promise<string> {
  /**
   The finished Git run, whose nonzero exit contradicts what the driver step proves.
   */
  const outcome = await run({
    command: REAL_GIT,
    args,
    cwd: fixture.repository,
    env: runEnvironment({ home: fixture.home }),
  });
  if (outcome.status !== 0)
    throw new InteropError(
      `${REAL_GIT} ${args.join(' ')} exited ${String(outcome.status ?? 'without a status')} (${outcome.signal ?? 'no signal'}):\n`
        + `${outcome.stdout}${outcome.stderr}`,
    );
  return outcome.stdout
    .trim();
}

/**
 Whether a caught value is the error Node raises for a path that does not exist.

 @param error - caught value
 
 @returns true for an `ENOENT` error

 @example
 ```ts
 isMissingPath(new Error('boom'));
 ```
 */
export function isMissingPath(error: unknown): boolean {
  // `Object` wraps a primitive and turns `null` into an empty object, so any caught value can be asked for a code.
  /**
   The caught value's `code`, whatever its type.
   */
  const code: unknown = Reflect.get(
    new Object(error),
    'code',
  );
  return code === 'ENOENT';
}

/**
 Entry names of a directory, or none when it does not exist.

 @param directory - directory to list
 
 @returns entry names

 @example
 ```ts
 const names = await namesIn('/work/scratch/repository/.git');
 ```
 */
export async function namesIn(directory: string): Promise<string[]> {
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

 @param path - path to test
 
 @returns true when something is there

 @example
 ```ts
 const present = await pathExists('/work/scratch/repository/.git/index.lock');
 ```
 */
export async function pathExists(path: string): Promise<boolean> {
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

 @param fixture - repository to inspect

 @param staleLocks - whether lock directories count as leftovers

 @returns descriptions of each leftover, none when the repository is quiescent

 @example
 ```ts
 const left = await leftovers({ fixture, staleLocks: true });
 ```
 */
export async function leftovers({
  fixture,
  staleLocks = false,
}: {
  readonly fixture: { readonly repository: string; };
  readonly staleLocks?: boolean;
}): Promise<string[]> {
  /**
   The repository's Git directory.
   */
  const gitDir = join(
    fixture.repository,
    '.git',
  );
  /**
   Description of each leftover found, in the order the places are checked.
   */
  const found: string[] = [];
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

 @returns recorder

 @example
 ```ts
 const matrix = createMatrix();
 matrix.check({ condition: true, message: 'the repository is clean' });
 matrix.finish();
 ```
 */
export function createMatrix(): {
  readonly check: (request: {
    readonly condition: boolean;
    readonly message: string;
  }) => boolean;
  readonly finish: () => void;
} {
  /**
   Messages of the failed checks.
   */
  const failures: string[] = [];
  /**
   Messages of the passed checks.
   */
  const passes: string[] = [];
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
