/**
 Launch-failure and recursion-guard tests for the spawn-claude CLI.
 
 `child_process.spawn` reports a launch failure (a missing or non-executable
 launcher, a sandbox denial) as an `'error'` event, and an unanswered event kills
 the process with a raw stack trace. The CLI also writes a `running` spawn-state
 file before launching, so a launcher that never ran leaves a record that every
 hook event of every session re-reads forever.
 
 The recursion guard has to leave no trace when it refuses: no terminal window,
 no spawn-state file, and no spawn identifier on stdout. Proving the absence
 needs a launcher that records instead of opens, so the guard cases put a stub
 `terminal-exec` on PATH and read back what it was handed.
 
 The CLI runs its work at module scope, so each case runs it as a child process
 with an injected HOME: HOME keeps the spawn-state directory disposable, and an
 empty PATH makes `terminal-exec` unresolvable for the launch-failure cases.
 
 @module
 */

import {
  spawnSync,
} from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { fileURLToPath, } from 'node:url';
import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  DEFAULT_MAX_SPAWN_DEPTH,
  MAX_SPAWN_DEPTH_ENV,
  SPAWN_DEPTH_ENV,
} from '../handler/claude-spawn/spawn-recursion-guard.ts';

/**
 Absolute path of the CLI under test, resolved from this file so the child runs
 the same source regardless of the working directory.
 */
const CLI_PATH = fileURLToPath(new URL('spawn-claude.ts', import.meta.url,),);

/**
 Directory whose `node_modules` resolves the CLI's workspace dependencies.
 */
const PACKAGE_ROOT = fileURLToPath(new URL('../..', import.meta.url,),);

/**
 Subdirectory of HOME holding every spawn coordination file.
 */
const SPAWN_RESULTS_DIR = '.claude/spawn-results';

/**
 Name the CLI resolves its launcher by, and the name the stub launcher takes.
 */
const LAUNCHER_NAME = 'terminal-exec';

/**
 Prompt every run asks the CLI to hand the child session.
 */
const LAUNCH_PROMPT = 'a prompt nobody will read';

/**
 File the stub launcher appends one line per launch to, inside the temporary HOME.
 */
const LAUNCHER_CALLS_FILE = 'launcher-calls.txt';

/**
 Environment variable telling the stub launcher where to record its calls.
 */
const LAUNCHER_CALLS_ENV = 'SPAWN_CLAUDE_TEST_LAUNCHER_CALLS';

/**
 Character the stub launcher puts between the two fields it records.
 
 Not a space, because the arguments the CLI passes contain them.
 */
const LAUNCHER_CALL_FIELD_SEPARATOR = '\t';

/**
 How long a case waits for the detached stub launcher to record a launch.
 
 Long enough for a loaded machine to finish starting one Node process, and short
 enough that a launch which never happens is reported without stalling the suite.
 */
const LAUNCHER_CALL_DEADLINE_MS = 5_000;

/**
 Delay between looks for the stub launcher's record.
 */
const LAUNCHER_CALL_POLL_MS = 25;

/**
 One launch of the CLI, captured.
 */
type CliRun = {
  readonly status: number | typeof TIMED_OUT;
  readonly stderr: string;
  readonly stdout: string;
};

/**
 Sentinel for a CLI run the harness had to kill, distinct from every exit code
 the CLI can return.
 */
const TIMED_OUT: unique symbol = Symbol('spawn-claude-test/cli-run-timed-out',);

/**
 Disposable HOME whose `path` holds one session mapping, so the CLI finds a
 calling session and reaches the launcher.
 */
type TempHome = {
  readonly path: string;
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable HOME holding one session mapping.
 
 @returns temporary HOME that removes itself when disposed
 */
async function makeHomeWithSession(): Promise<TempHome> {
  /**
   Temporary HOME root.
   */
  const home = await mkdtemp(join(scratchRoot(), 'spawn-claude-',),);
  /**
   PID mapping directory the session lookup reads.
   */
  const byPid = join(
    home,
    SPAWN_RESULTS_DIR,
    '.by-pid',
  );
  mkdirSync(
    byPid,
    { recursive: true, },
  );
  writeFileSync(
    join(
      byPid,
      '1.json',
    ),
    JSON.stringify({
      sessionId: 'spawn-claude-test-parent-session',
      transcriptPath: join(
        home,
        'transcript.jsonl',
      ),
    },),
  );
  mkdirSync(
    join(
      home,
      SPAWN_RESULTS_DIR,
      'spawns',
    ),
    { recursive: true, },
  );
  return {
    path: home,
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
      await rm(
        home,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Lists spawn-state files left in a temporary HOME.
 
 @param home - temporary HOME root
 
 @returns file names under the spawns directory
 */
function spawnStateFiles(home: string,): readonly string[] {
  return readdirSync(
    join(
      home,
      SPAWN_RESULTS_DIR,
      'spawns',
    ),
  );
}

/**
 Runs the CLI once with an injected HOME and an explicit PATH.
 
 @param home - temporary HOME root
 
 @param path - PATH the CLI resolves its launcher from
 
 @param env - extra environment values for the run
 
 @returns captured exit status, stdout, and stderr
 */
function runCli({
  home,
  path = '',
  env = {},
}: {
  readonly home: string;
  readonly path?: string;
  readonly env?: Readonly<Record<string, string>>;
},): CliRun {
  /**
   Child run, bounded so a hung CLI cannot stall the suite.
   */
  const result = spawnSync(
    process.execPath,
    [
      CLI_PATH,
      LAUNCH_PROMPT,
    ],
    {
      cwd: PACKAGE_ROOT,
      encoding: 'utf8',
      env: {
        ...env,
        HOME: home,
        PATH: path,
        [LAUNCHER_CALLS_ENV]: join(
          home,
          LAUNCHER_CALLS_FILE,
        ),
      },
      input: '',
      timeout: 30_000,
    },
  );
  return {
    status: result.error === undefined
      ? (result.status ?? TIMED_OUT)
      : TIMED_OUT,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

/**
 Runs the CLI once with an injected HOME and no PATH to resolve a launcher from.
 
 @param home - temporary HOME root
 
 @returns captured exit status, stdout, and stderr
 */
function runCliWithoutLauncher(home: string,): CliRun {
  return runCli({ home, },);
}

/**
 One launch the stub launcher recorded.
 */
type LauncherCall = {
  /**
   Spawn generation the CLI handed the child, empty when it handed none.
   */
  readonly depth: string;
  /**
   Arguments the CLI asked the launcher to run.
   */
  readonly argv: string;
};

/**
 Splits one line the stub launcher recorded into its fields.
 
 @param line - one recorded line
 
 @returns the fields the stub wrote, split on the separator between them
 */
function parseLauncherCall(line: string,): LauncherCall {
  const [depth = '', argv = '',] = line.split(LAUNCHER_CALL_FIELD_SEPARATOR,);
  return {
    depth,
    argv,
  };
}

/**
 Reads the launches the stub launcher recorded, waiting for one to land.
 
 The CLI spawns the launcher detached and exits without waiting for it, so a
 launch can still be in flight when the run returns. An empty answer therefore
 means "none within the deadline", which is what the refusal cases assert.
 
 @param home - temporary HOME root
 
 @returns recorded launches in call order, empty when none arrived in time
 */
async function recordedLauncherCalls(home: string,): Promise<readonly LauncherCall[]> {
  /**
   Marker file the stub appends one line per launch to.
   */
  const callsFile = join(
    home,
    LAUNCHER_CALLS_FILE,
  );
  /**
   Moment after which a launch that never happened is reported as absent.
   */
  const stopAt = performance.now()
    + LAUNCHER_CALL_DEADLINE_MS;

  while (performance.now() < stopAt) {
    if (existsSync(callsFile,)) {
      /**
       Recorded launches, one per line.
       */
      const lines = readFileSync(
        callsFile,
        'utf8',
      )
        .split('\n',)
        .filter(function keepRecordedLine(line,): boolean {
          return line !== '';
        },);
      if (lines.length > 0)
        return lines.map(parseLauncherCall,);
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- each look must follow the last: the stub writes the record once, and looking again before that answer settles reads an empty file.
    await wait(LAUNCHER_CALL_POLL_MS,);
  }
  return [];
}

/**
 Writes an executable `terminal-exec` stub into a temporary HOME.
 
 The stub records the arguments and the spawn depth it was handed instead of
 opening a window, so a test can tell whether a launch happened at all and what
 generation it would have run.
 
 @param home - temporary HOME root
 
 @returns directory to put on PATH so the CLI resolves the stub
 */
async function writeLauncherStub(home: string,): Promise<string> {
  /**
   Directory holding the stub as the command the CLI resolves by name.
   */
  const binDirectory = join(
    home,
    'bin',
  );
  await mkdir(
    binDirectory,
    { recursive: true, },
  );

  /**
   Stub path, named exactly as the launcher the CLI spawns.
   */
  const stubPath = join(
    binDirectory,
    LAUNCHER_NAME,
  );
  writeFileSync(
    stubPath,
    [
      `#!${process.execPath}`,
      '(async function recordLaunch() {',
      "  const { appendFileSync, } = await import('node:fs');",
      '  appendFileSync(',
      `    process.env[${JSON.stringify(LAUNCHER_CALLS_ENV,)}],`,
      '    [',
      `      process.env[${JSON.stringify(SPAWN_DEPTH_ENV,)}] ?? '',`,
      "      process.argv.slice(2).join(' '),",
      `    ].join(${JSON.stringify(LAUNCHER_CALL_FIELD_SEPARATOR,)}) + '\\n',`,
      '  );',
      '})();',
      '',
    ]
      .join('\n',),
  );
  await chmod(
    stubPath,
    0o755,
  );

  return binDirectory;
}

await describe({
  name: 'spawn-claude launch failure',
  children: [
    it({
      name: 'reports the missing launcher instead of crashing on an unhandled error event',
      fn: async () => {
        await using home = await makeHomeWithSession();
        /**
         CLI run against an unresolvable launcher.
         */
        const run = runCliWithoutLauncher(home.path,);
        expect(run.stderr,)
          .toContain('terminal-exec',);
        expect(run.stderr,)
          .not
          .toContain('Unhandled',);
        expect(run.status,)
          .toBe(1,);
      },
    },),
    it({
      name: 'removes the pre-created spawn state file so no session keeps scanning it',
      fn: async () => {
        await using home = await makeHomeWithSession();
        runCliWithoutLauncher(home.path,);
        expect(spawnStateFiles(home.path,),)
          .toEqual([],);
      },
    },),
    it({
      name: 'reports no spawn identifier, because no child session exists',
      fn: async () => {
        await using home = await makeHomeWithSession();
        /**
         CLI run against an unresolvable launcher.
         */
        const run = runCliWithoutLauncher(home.path,);
        expect(run.stdout,)
          .toBe('',);
      },
    },),
  ],
},);

await describe({
  name: 'spawn-claude recursion guard',
  children: [
    //region Launches the guard allows

    it({
      name: 'launches the first generation with the depth marker the next spawn counts from',
      fn: async () => {
        await using home = await makeHomeWithSession();
        /**
         PATH that resolves the recording stub as the launcher.
         */
        const launcherPath = await writeLauncherStub(home.path,);
        /**
         CLI run from the top of a chain.
         */
        const run = runCli({
          home: home.path,
          path: launcherPath,
        },);
        expect(run.status,)
          .toBe(0,);
        expect(run.stdout,)
          .toContain('spawnId',);
        /**
         Launches the stub recorded for that run.
         */
        const calls = await recordedLauncherCalls(home.path,);
        expect(calls,)
          .toHaveLength(1,);
        expect(calls[0]?.depth,)
          .toBe('1',);
        expect(calls[0]?.argv,)
          .toContain('claude',);
        expect(spawnStateFiles(home.path,),)
          .toHaveLength(1,);
      },
    },),
    it({
      name: 'launches the child one generation below the parent that spawns it',
      fn: async () => {
        await using home = await makeHomeWithSession();
        /**
         PATH that resolves the recording stub as the launcher.
         */
        const launcherPath = await writeLauncherStub(home.path,);
        /**
         CLI run from a session two generations deep.
         */
        const run = runCli({
          home: home.path,
          path: launcherPath,
          env: { [SPAWN_DEPTH_ENV]: '2', },
        },);
        expect(run.status,)
          .toBe(0,);
        /**
         Launches the stub recorded for that run.
         */
        const calls = await recordedLauncherCalls(home.path,);
        expect(calls,)
          .toHaveLength(1,);
        expect(calls[0]?.depth,)
          .toBe('3',);
      },
    },),
    it({
      name: 'launches when the override raises the limit above this session',
      fn: async () => {
        await using home = await makeHomeWithSession();
        /**
         PATH that resolves the recording stub as the launcher.
         */
        const launcherPath = await writeLauncherStub(home.path,);
        /**
         CLI run from a session the default limit would refuse, with a raised limit.
         */
        const run = runCli({
          home: home.path,
          path: launcherPath,
          env: {
            [SPAWN_DEPTH_ENV]: '4',
            [MAX_SPAWN_DEPTH_ENV]: '5',
          },
        },);
        expect(run.status,)
          .toBe(0,);
        /**
         Launches the stub recorded for that run.
         */
        const calls = await recordedLauncherCalls(home.path,);
        expect(calls,)
          .toHaveLength(1,);
        expect(calls[0]?.depth,)
          .toBe('5',);
      },
    },),

    //endregion Launches the guard allows

    //region Launches the guard refuses

    it({
      name: 'refuses at the limit without launching a launcher or leaving a spawn record',
      fn: async () => {
        await using home = await makeHomeWithSession();
        /**
         PATH that would resolve the recording stub as the launcher.
         */
        const launcherPath = await writeLauncherStub(home.path,);
        /**
         CLI run from a session sitting at the default limit.
         */
        const run = runCli({
          home: home.path,
          path: launcherPath,
          env: { [SPAWN_DEPTH_ENV]: String(DEFAULT_MAX_SPAWN_DEPTH,), },
        },);
        expect(run.status,)
          .toBe(1,);
        expect(run.stderr,)
          .toContain(`${String(DEFAULT_MAX_SPAWN_DEPTH,)} spawn generations deep`,);
        expect(run.stderr,)
          .toContain(MAX_SPAWN_DEPTH_ENV,);
        expect(run.stderr,)
          .not
          .toContain('Unhandled',);
        expect(run.stdout,)
          .toBe('',);
        expect(await recordedLauncherCalls(home.path,),)
          .toEqual([],);
        expect(spawnStateFiles(home.path,),)
          .toEqual([],);
      },
    },),
    it({
      name: 'refuses a marker it cannot read instead of treating it as the top of a chain',
      fn: async () => {
        await using home = await makeHomeWithSession();
        /**
         PATH that would resolve the recording stub as the launcher.
         */
        const launcherPath = await writeLauncherStub(home.path,);
        /**
         CLI run with a depth marker that is not a generation count.
         */
        const run = runCli({
          home: home.path,
          path: launcherPath,
          env: { [SPAWN_DEPTH_ENV]: 'two', },
        },);
        expect(run.status,)
          .toBe(1,);
        expect(run.stderr,)
          .toContain(SPAWN_DEPTH_ENV,);
        expect(await recordedLauncherCalls(home.path,),)
          .toEqual([],);
      },
    },),

    //endregion Launches the guard refuses
  ],
},);
