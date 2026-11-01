/**
 The one place a test starts a child process, so no child inherits a provider
 key.

 THE OWNER'S RULE: keys come from the repository's task runner, and a test's
 child must never inherit them. Every variable whose name ends in `_API_KEY`
 is removed from a child's environment (the name test is `isCredentialName`
 of `child-process-environment.ts`, which production code uses too, so the
 two cannot drift), and so is every variable whose name starts with
 `TRANSLATION_REPAIR_`, so the machine's pool and clone settings cannot reach
 a child either: a case that wants one names it in `extra`.
 `nano-spawn` only adds to the parent's environment, so a removed name is
 stated with an `undefined` value, which its merge and node's own `spawn`
 both read as absent (`child-environment.unit.test.ts` shows it with real
 children). `test-children-keyless.unit.test.ts` fails any other test file
 that imports a spawning module or calls a spawning function.

 THIS FIXTURE IS ALSO THE TEST WORLD'S TASK RUNNER. A built command refuses to
 run in a process that holds provider keys unless that process names the
 command as the one it means to start (`task-runner-guard.ts`), which the
 command's task does by setting `TRANSLATION_REPAIR_STARTED_BY` to the
 command's name. `runBuiltCommand` does the same for the command it starts,
 after it removes the names the rule names, so a case that hands its child an
 invented key still reaches what the command does; a case that wants the
 refusal says so through `startedBy`.

 A CHILD LOGS INSIDE THE WORKSPACE. The repository's logger writes beside the
 nearest ancestor `node_modules` and, where none stands above the working
 directory (a scratch directory under the system temp directory), falls back
 to the user's state directory and says so on stderr. Every child is told its
 log directory instead (`MONOCHROMATIC_LOG_DIRECTORY`, which the logger keeps
 to and never trades for a fallback): this package's gitignored
 `node_modules/.monochromatic`, unless the case names another in `extra`. So
 no test writes into the user's state and no child's stderr carries the
 logger's notice.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import nanoSpawn, { type Result, } from 'nano-spawn';

import { isCredentialName, } from '@monochromatic-dev/translation-repair-typography';
import { STARTED_BY_VARIABLE, } from '../dist/final/node/index.mjs';
import {
  builtDirectoryOf,
  packageRootOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/built-package-directory.test-fixture.ts';

/**
 Prefix of every variable the package reads for its own settings.
 */
const SETTING_PREFIX = 'TRANSLATION_REPAIR_';

/**
 Exit code reported when a child ended on a signal and so has none.
 */
const SIGNALLED = -1;

/**
 Directory every child's logger writes into unless the case names another:
 this package's gitignored `node_modules/.monochromatic`.
 */
const CHILD_LOG_DIRECTORY = fileURLToPath(new URL(
  '../node_modules/.monochromatic',
  import.meta.url,
),);

/**
 Environment for a child: the parent's, with every variable the rule removes
 stated as absent, the workspace log directory, then the case's own variables.

 @param extra - variables the case sets for its child, which win over the
 removal so a case can hand one setting down by name

 @returns Environment for node's `spawn` and for `nano-spawn` alike

 @example
 ```ts
 const env = environmentWithoutKeys({ extra: { TRANSLATION_REPAIR_RUNS_DIR: runsDir, }, },);
 ```
 */
export function environmentWithoutKeys(
  { extra = {}, }: { readonly extra?: Readonly<Record<string, string>>; } = {},
): NodeJS.ProcessEnv {
  /**
   The parent's variables, each removed one stated as absent.
   */
  const kept: NodeJS.ProcessEnv = { ...process.env, };
  for (const name of Object.keys(kept,)) {
    if (isCredentialName({ name, },) || name.startsWith(SETTING_PREFIX,))
      kept[name] = undefined;
  }
  return {
    ...kept,
    MONOCHROMATIC_LOG_DIRECTORY: CHILD_LOG_DIRECTORY,
    ...extra,
  };
}

/**
 Runs a program through `nano-spawn` with the keyless environment, as a case
 that wants the program's output and a refusal on any failed exit.

 @param file - program to run

 @param args - its arguments

 @param cwd - directory it runs in, the caller's own where absent

 @param extra - variables the case sets for it

 @returns Its result, whose output has the one final newline removed

 @throws SubprocessError when the program never started or exited with a failure

 @example
 ```ts
 const { stdout, } = await spawnKeyless({ file: 'git', args: ['status',], extra: { GIT_CONFIG_GLOBAL: devNull, }, },);
 ```
 */
export async function spawnKeyless(
  {
    file,
    args,
    cwd,
    extra = {},
  }: {
    readonly file: string;
    readonly args: readonly string[];
    readonly cwd?: string;
    readonly extra?: Readonly<Record<string, string>>;
  },
): Promise<Result> {
  return await nanoSpawn(
    file,
    args,
    {
      ...((cwd === undefined) ? {} : { cwd, }),
      env: environmentWithoutKeys({ extra, },),
    },
  );
}

/**
 What a child wrote and how it exited.

 @example
 ```ts
 const run: ChildRun = { code: 6, stdout: '', stderr: 'window-trial-probe: ...\n', };
 ```
 */
export type ChildRun = {
  /**
   Exit code, or -1 when the process was signalled.
   */
  readonly code: number;

  /**
   Everything written to stdout, whole.
   */
  readonly stdout: string;

  /**
   Everything written to stderr, whole.
   */
  readonly stderr: string;
};

/**
 Runs a program with the keyless environment and reports how it ended
 whatever its exit code, for a case that reads the finished process itself.

 @param file - program to run

 @param args - its arguments

 @param cwd - directory it runs in, the caller's own where absent

 @param extra - variables the case sets for it

 @returns Exit code, then stdout and stderr whole

 @throws Error when the process never started

 @example
 ```ts
 const done = await runKeyless({ file: process.execPath, args: [script,], cwd: scratch.path, },);
 ```
 */
export async function runKeyless(
  {
    file,
    args,
    cwd,
    extra = {},
  }: {
    readonly file: string;
    readonly args: readonly string[];
    readonly cwd?: string;
    readonly extra?: Readonly<Record<string, string>>;
  },
): Promise<ChildRun> {
  /**
   The child, its input closed.
   */
  const child = spawn(
    file,
    args,
    {
      ...((cwd === undefined) ? {} : { cwd, }),
      env: environmentWithoutKeys({ extra, },),
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Stdout as it arrives.
   */
  const out: string[] = [];
  /**
   Stderr as it arrives.
   */
  const err: string[] = [];
  child.stdout
    .setEncoding('utf8',);
  child.stdout
    .on(
    'data',
    function keepOut(chunk: string,): void {
      out.push(chunk,);
    },
    );
  child.stderr
    .setEncoding('utf8',);
  child.stderr
    .on(
    'data',
    function keepErr(chunk: string,): void {
      err.push(chunk,);
    },
    );
  // Wait for the streams to close, then read the exit off the child itself;
  // a process that never started rejects here.
  await once(
    child,
    'close',
  );
  return {
    code: child.exitCode ?? SIGNALLED,
    stdout: out.join('',),
    stderr: err.join('',),
  };
}

/**
 Which command a child's environment names as the one it means to start: the
 command it runs, as that command's own task would name it (the marker names
 the command); nothing (no marker); or another command (the marker names that
 one).
 */
export type StartedBy =
  | 'its own task'
    | 'nothing'
    | {
    /**
     Name of the other command the marker names.
     */
      readonly otherCommand: string;
    };

/**
 The marker variable a child gets, as variables to add to its environment.

 @param command - command the child runs

 @param startedBy - who the marker says started it

 @returns The marker, or no variable at all where nothing started it

 @example
 ```ts
 const marker = markerFor({ command: 'spend-report', startedBy: 'its own task', },);
 ```
 */
function markerFor(
  {
    command,
    startedBy,
  }: {
    readonly command: string;
    readonly startedBy: StartedBy;
  },
): Readonly<Record<string, string>> {
  if (startedBy === 'nothing')
    return {};
  if (startedBy === 'its own task')
    return { [STARTED_BY_VARIABLE]: command, };
  return { [STARTED_BY_VARIABLE]: startedBy.otherCommand, };
}

/**
 Runs one built command (`dist/final/node/<command>.mjs` of the package that
 owns the command) with the keyless environment and reports how it ended
 whatever its exit code, for the as-built suites.

 @param command - command's name, which names its built file

 @param packageEntry - URL of the built index of the package that owns the
 command, as `import.meta.resolve` gives it for that package's name, which the
 command's built file sits beside

 @param args - the command's arguments

 @param env - variables the case sets for it, such as the runs directory, the
 lookup cache directory or the corpus clone directory

 @param cwd - directory it runs in, the owning package's own where absent

 @param startedBy - who the child's marker says started it: the task of this
 command unless the case says otherwise

 @returns Exit code, then stdout and stderr whole

 @throws Error when the process never started

 @example
 ```ts
 const run = await runBuiltCommand({
   command: 'window-trial-probe',
   packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-bench'),
   env: { TRANSLATION_REPAIR_RUNS_DIR: runsDir, },
 },);
 ```
 */
export async function runBuiltCommand(
  {
    command,
    packageEntry,
    args = [],
    env = {},
    cwd = packageRootOf({ packageEntry, },),
    startedBy = 'its own task',
  }: {
    readonly command: string;
    readonly packageEntry: string;
    readonly args?: readonly string[];
    readonly env?: Readonly<Record<string, string>>;
    readonly cwd?: string;
    readonly startedBy?: StartedBy;
  },
): Promise<ChildRun> {
  return await runKeyless({
    file: process.execPath,
    args: [
      join(
        builtDirectoryOf({ packageEntry, },),
        `${command}.mjs`,
      ),
      ...args,
    ],
    cwd,
    extra: {
      ...env,
      ...markerFor({
        command,
        startedBy,
      },),
    },
  },);
}

/**
 Sets variables in this process, restoring what stood before. A write to the
 process environment, so a case calling it runs in a suite that runs one case
 at a time.

 @param values - variables to set, by name

 @returns Disposable that restores every variable it set

 @example
 ```ts
 using pointed = variablesSet({ values: { TRANSLATION_REPAIR_RUNS_DIR: runsDir, }, },);
 ```
 */
export function variablesSet({ values, }: { readonly values: Readonly<Record<string, string>>; },): Disposable {
  /**
   Values standing before this call, by name.
   */
  const standing = new Map<string, string>();
  for (const name of Object.keys(values,)) {
    /**
     Value standing before this call, if any.
     */
    const value = process.env[name];
    if (value !== undefined) {
      standing.set(
        name,
        value,
      );
    }
  }
  for (const [name, value,] of Object.entries(values,))
    process.env[name] = value;
  return {
    [Symbol.dispose]: function restore(): void {
      for (const name of Object.keys(values,)) {
        /**
         Value to put back, absent where none stood.
         */
        const value = standing.get(name,);
        if (value === undefined) {
          Reflect.deleteProperty(
            process.env,
            name,
          );
        }
        else
          process.env[name] = value;
      }
    },
  };
}
