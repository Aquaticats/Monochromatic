/**
 A disposable repository, a recording stand-in for `gh`, and the real daily update run over them.

 The update's own commands (`toolchain:update`, `cargo-shadow update` and the rest) are replaced by small scripts that
 move a lock the way the real ones do, and two crates and one TypeScript package are checked through real `mise` tasks.
 Everything else is real: `mise`, `git`, the process groups, the run lock, the pending file and the `gh` call path.
 The repository has no remote and the stand-in `gh` is first on `PATH`, so no real issue can be opened.

 @module
 */

import {
  mkdir,
  mkdtempDisposable,
  readFile,
  realpath,
  symlink,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  assertPrivateBus,
  startPrivateBus,
} from './daily-update-private-bus-fixture.ts';
import {
  directoryOf,
  layOutRepository,
  put,
  putExecutable,
  run,
} from './daily-update-whole-path-repository-fixture.ts';
import {
  FAKE_CARGO_SHADOW,
  FAKE_GH,
  FAKE_PNPM,
} from './daily-update-whole-path-scripts-fixture.ts';

/**
 Folder of the daily update's TypeScript source, which the fixture runs directly as the unit does.
 */
const SOURCE_DIRECTORY = join(import.meta.dirname,);

/**
 A call the stand-in `gh` received.
 */
export type RecordedGhCall = {
  /**
   Arguments after `gh`.
   */
  readonly args: readonly string[];
  /**
   Text sent on standard input.
   */
  readonly stdin: string;
  /**
   The environment values the update promises to set, as `gh` saw them.
   */
  readonly environment: Readonly<Record<string, string>>;
};

/**
 What the whole run left behind.
 */
export type WholePathResult = {
  /**
   Exit status of the daily update.
   */
  readonly exitCode: number;
  /**
   Everything the update printed on standard error, which is its journal.
   */
  readonly journal: string;
  /**
   Calls the stand-in `gh` received, in order.
   */
  readonly ghCalls: readonly RecordedGhCall[];
  /**
   Titles of the notifications the update sent, one per notification, read from its journal where `notifyOwner` writes
   each title and body before it calls `notify-send`.
   */
  readonly notificationTitles: readonly string[];
  /**
   Commit subjects of the repository, newest first.
   */
  readonly commitSubjects: readonly string[];
  /**
   Text of the pending issues file, empty when the run wrote none.
   */
  readonly pendingText: string;
};

/**
 Options of the whole-path run.
 */
export type WholePathOptions = {
  /**
   Exit status the stand-in `gh` gives to every call; zero for a working GitHub.
   */
  readonly ghExitCode: number;
};

/**
 Title of the notification the daily update sends, which it also writes to its journal.
 */
const NOTIFICATION_TITLE = 'Daily update needs a look';

/**
 Reads a text file, answering the empty string when it does not exist.

 @param path - absolute path

 @returns the text, or the empty string when the file is absent

 @throws Error when the file exists but cannot be read

 @example
 ```ts
 const text = await readTextOrEmpty('/r/.cache/pending.json',);
 ```
 */
export async function readTextOrEmpty(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if ((Error.isError(error,)) && ('code' in error)
      && (error.code === 'ENOENT'))
      return '';
    throw error;
  }
}

/**
 Reads one line the stand-in `gh` recorded.

 @param line - one JSON line

 @returns the recorded call

 @throws Error when the line is not a recorded call

 @example
 ```ts
 const call = parseGhCall('{"args":[],"stdin":"","environment":{}}',);
 ```
 */
export function parseGhCall(line: string,): RecordedGhCall {
  /**
   The parsed line.
   */
  const parsed: unknown = JSON.parse(line,);
  if (
    ((typeof parsed) !== 'object') || (parsed === null)
      || (!('args' in parsed))
      || (!Array.isArray(parsed.args,))
      || (!('stdin' in parsed))
      || ((typeof parsed.stdin) !== 'string')
      || (!('environment' in parsed))
      || ((typeof parsed.environment) !== 'object')
      || (parsed.environment === null)
  ) {
    throw new Error(`the stand-in gh recorded a line that is not a call: ${line}`,);
  }
  /**
   The environment values the call carried, as text.
   */
  const environment: Record<string, string> = {};
  for (const [name, value,] of Object.entries(parsed.environment,))
    environment[name] = String(value,);
  return {
    args: parsed.args
      .map(String,),
    stdin: parsed.stdin,
    environment,
  };
}

/**
 Runs the real daily update over the disposable repository with the stand-in `gh`.

 @param ghExitCode - exit status the stand-in `gh` gives to every call; zero for a working GitHub

 @returns what the run printed, which commits it made, what `gh` was asked, and what it kept for later

 @throws Error when `mise` or `node` is not on `PATH` or a step of the fixture fails

 @example
 ```ts
 const result = await runWholePath({ ghExitCode: 0, },);
 ```
 */
export async function runWholePath({
  ghExitCode,
}: WholePathOptions,): Promise<WholePathResult> {
  /**
   The disposable folder that holds the whole run, removed when the function ends.
   */
  await using disposable = await mkdtempDisposable(join(
    scratchRoot(),
    'daily-update-whole-path-',
  ),);
  /**
   Canonical path of the fixture root.
   */
  const base = await realpath(disposable.path,);
  /**
   The repository.
   */
  const root = join(
    base,
    'repository',
  );
  /**
   The folder of stand-in programs.
   */
  const bin = join(
    base,
    'bin',
  );
  /**
   The file the stand-in `gh` records its calls in.
   */
  const ghLog = join(
    base,
    'gh-calls.jsonl',
  );
  /**
   The machine-global lock the update changes.
   */
  const globalLock = join(
    base,
    'global',
    'toolchain.lock.json',
  );
  /**
   The private session bus, which no host service can be started through.
   */
  await using bus = await startPrivateBus(process.env
    .XDG_RUNTIME_DIR
    ?? base,);
  /**
   Environment of every process: the stand-ins first on `PATH`, fixture-owned homes, no ambient credentials.
   */
  const environment: Readonly<Record<string, string>> = {
    PATH: [
      bin,
      '/usr/bin',
      '/bin',
    ].join(delimiter,),
    HOME: join(
      base,
      'home',
    ),
    XDG_CONFIG_HOME: join(
      base,
      'config',
    ),
    XDG_CACHE_HOME: join(
      base,
      'cache',
    ),
    XDG_DATA_HOME: join(
      base,
      'data',
    ),
    XDG_STATE_HOME: join(
      base,
      'state',
    ),
    GIT_CONFIG_GLOBAL: join(
      base,
      'gitconfig',
    ),
    GIT_AUTHOR_NAME: 'Fixture',
    GIT_AUTHOR_EMAIL: 'fixture@example.invalid',
    GIT_COMMITTER_NAME: 'Fixture',
    GIT_COMMITTER_EMAIL: 'fixture@example.invalid',
    MISE_TRUSTED_CONFIG_PATHS: base,
    MISE_CEILING_PATHS: dirname(base,),
    MISE_TASK_RUN_AUTO_INSTALL: 'false',
    MISE_YES: '1',
    NO_COLOR: '1',
    DBUS_SESSION_BUS_ADDRESS: bus.address,
    FIXTURE_GH_LOG: ghLog,
    FIXTURE_GH_EXIT: String(ghExitCode,),
    FIXTURE_GLOBAL_LOCK: globalLock,
  };
  // Only the program folders the fixture names are on `PATH`: the folder that holds `mise` also holds the repository's
  // Git wrapper, which needs a user service manager that a disposable repository must not start.
  await mkdir(
    bin,
    { recursive: true, },
  );
  await Promise.all([
    symlink(
      await realpath(join(
        await directoryOf('mise',),
        'mise',
      ),),
      join(
        bin,
        'mise',
      ),
    ),
    symlink(
      process.execPath,
      join(
        bin,
        'node',
      ),
    ),
  ],);
  await Promise.all([
    putExecutable({
      directory: bin,
      name: 'gh',
      content: FAKE_GH,
    },),
    putExecutable({
      directory: bin,
      name: 'cargo-shadow',
      content: FAKE_CARGO_SHADOW,
    },),
    putExecutable({
      directory: bin,
      name: 'pnpm',
      content: FAKE_PNPM,
    },),
    put({
      path: globalLock,
      content: '{"tools":{"rust":{"version":"nightly-1"}}}\n',
    },),
    put({
      path: ghLog,
      content: '',
    },),
  ],);
  // The real `notify-send` runs in this fixture, so the guard stands before anything else can start it.
  assertPrivateBus({
    environment,
    privateDirectory: bus.directory,
  },);
  await layOutRepository({
    root,
    environment,
  },);
  /**
   The daily update, run the way the task runs it.
   */
  const update = await run({
    program: process.execPath,
    args: [
      join(
        SOURCE_DIRECTORY,
        'daily-update.ts',
      ),
      '--repository',
      root,
      '--global-lock',
      globalLock,
    ],
    directory: root,
    environment,
  },);
  /**
   The repository's commit subjects.
   */
  const log = await run({
    program: 'git',
    args: [
      'log',
      '--format=%s',
    ],
    directory: root,
    environment,
  },);
  /**
   Text of the pending file, empty when absent.
   */
  const pendingText = await readTextOrEmpty(join(
    root,
    '.cache',
    'toolchain-daily-update-pending-issues.json',
  ),);
  return {
    exitCode: update.exitCode,
    journal: update.stderr,
    ghCalls: (await readFile(
      ghLog,
      'utf8',
    ))
      .split('\n',)
      .filter(function nonEmpty(line,) {
        return line !== '';
      },)
      .map(parseGhCall,),
    notificationTitles: update.stderr
      .split('\n',)
      .filter(function isTitle(line,) {
        return line === NOTIFICATION_TITLE;
      },),
    commitSubjects: log.stdout
      .split('\n',)
      .filter(function nonEmpty(line,) {
        return line !== '';
      },),
    pendingText,
  };
}
