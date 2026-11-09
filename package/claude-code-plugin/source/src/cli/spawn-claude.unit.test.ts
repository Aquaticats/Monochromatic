/**
 Launch-failure tests for the spawn-claude CLI.
 
 `child_process.spawn` reports a launch failure (a missing or non-executable
 launcher, a sandbox denial) as an `'error'` event, and an unanswered event kills
 the process with a raw stack trace. The CLI also writes a `running` spawn-state
 file before launching, so a launcher that never ran leaves a record that every
 hook event of every session re-reads forever.
 
 The CLI runs its work at module scope, so each case runs it as a child process
 with an injected HOME and an empty PATH: HOME keeps the spawn-state directory
 disposable, PATH makes `terminal-exec` unresolvable.
 
 @module
 */

import {
  spawnSync,
} from 'node:child_process';
import {
  mkdirSync,
  readdirSync,
  writeFileSync,
} from 'node:fs';
import {
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
 Runs the CLI once with an injected HOME and no PATH to resolve a launcher from.
 
 @param home - temporary HOME root
 
 @returns captured exit status, stdout, and stderr
 */
function runCliWithoutLauncher(home: string,): CliRun {
  /**
   Child run, bounded so a hung CLI cannot stall the suite.
   */
  const result = spawnSync(
    process.execPath,
    [
      CLI_PATH,
      'a prompt nobody will read',
    ],
    {
      cwd: PACKAGE_ROOT,
      encoding: 'utf8',
      env: {
        HOME: home,
        PATH: '',
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
