/**
 Diagnostics tests for the completed-child scan.
 
 A spawn-state file this package cannot parse holds a completed child's result
 that will never be delivered. Skipping it silently makes that loss
 indistinguishable from an empty spawns directory, which is the ordinary state
 of every session that never spawned a child, so the scan reports the failure on
 stderr while stdout stays empty: in a hook, stdout is the protocol channel.
 
 The scan reads its directory from `$HOME` at module load, so each case runs a
 child process with a disposable HOME rather than touching the operator's own
 spawn directory.
 
 @module
 */

import {
  spawnSync,
} from 'node:child_process';
import {
  mkdirSync,
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
 Absolute path of the module under test, resolved from this file so the child
 imports the same source regardless of the working directory.
 */
const INJECT_PATH = fileURLToPath(new URL('inject.ts', import.meta.url,),);

/**
 Directory whose `node_modules` resolves the module's workspace dependencies.
 */
const PACKAGE_ROOT = fileURLToPath(new URL('../..', import.meta.url,),);

/**
 Subdirectory of HOME holding the spawn coordination files.
 */
const SPAWNS_SUBDIRECTORY = join(
  '.claude',
  'spawn-results',
  'spawns',
);

/**
 Parent session the scan is asked about; a record for it is deliverable.
 */
const PARENT_SESSION_ID = 'inject-test-parent-session';

/**
 Child script: runs the scan and reports only which arm it took, because stdout
 belongs to the hook protocol.
 */
const SCAN_SCRIPT = [
  `const { checkCompletedChildren, NOTHING_TO_REPORT, } = await import(${JSON.stringify(INJECT_PATH,)});`,
  `const result = await checkCompletedChildren({ parentSessionId: ${JSON.stringify(PARENT_SESSION_ID,)}, consume: false, },);`,
  `process.stdout.write(result === NOTHING_TO_REPORT ? 'NOTHING' : result,);`,
].join('\n',);

/**
 Disposable HOME holding this case's spawns directory, removed when the case's
 `await using` scope ends.
 */
type TempHome = {
  readonly path: string;
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable HOME for one scan case.
 
 @returns HOME that removes itself when disposed
 */
async function makeHome(): Promise<TempHome> {
  /**
   Temporary HOME root.
   */
  const path = await mkdtemp(join(scratchRoot(), 'claude-spawn-inject-',),);
  return {
    path,
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
      await rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 One scan run, captured.
 */
type ScanRun = {
  readonly stderr: string;
  readonly stdout: string;
};

/**
 Runs the scan in a child process whose HOME is `home`.
 
 @param home - disposable HOME holding the spawns directory to scan
 
 @returns captured stdout and stderr
 */
function runScan(home: string,): ScanRun {
  /**
   Child run, bounded so a hung import cannot stall the suite.
   */
  const result = spawnSync(
    process.execPath,
    [
      '--input-type=module-typescript',
      '-e',
      SCAN_SCRIPT,
    ],
    {
      cwd: PACKAGE_ROOT,
      encoding: 'utf8',
      env: {
        ...process.env,
        HOME: home,
      },
      input: '',
      timeout: 30_000,
    },
  );
  return {
    stderr: result.stderr,
    stdout: result.stdout,
  };
}

/**
 Writes one spawn-state file into a disposable HOME.
 
 @param home - disposable HOME
 
 @param filename - name of the record under the spawns directory
 
 @param content - bytes to write
 
 @returns nothing; the file is left for the scan to read
 */
function writeSpawnRecord(
  {
    home,
    filename,
    content,
  }: {
    readonly home: string;
    readonly filename: string;
    readonly content: string;
  },
): void {
  /**
   Spawns directory the scan lists.
   */
  const spawnsDirectory = join(
    home,
    SPAWNS_SUBDIRECTORY,
  );
  mkdirSync(
    spawnsDirectory,
    { recursive: true, },
  );
  writeFileSync(
    join(
      spawnsDirectory,
      filename,
    ),
    content,
  );
}

await describe({
  name: 'completed-child scan diagnostics',
  children: [
    it({
      name: 'reports an unreadable spawn-state file on stderr',
      fn: async () => {
        await using home = await makeHome();
        /**
         Record name the diagnostic must name, so the operator can find it.
         */
        const filename = 'corrupt-record.json';
        writeSpawnRecord({
          home: home.path,
          filename,
          content: '{"spawnId": "abc", "status":',
        },);

        /**
         Scan run over that record.
         */
        const run = runScan(home.path,);
        expect(run.stdout,)
          .toBe('NOTHING',);
        expect(run.stderr,)
          .toContain('spawn state unreadable',);
        expect(run.stderr,)
          .toContain(filename,);
      },
    },),
    it({
      name: 'delivers a completed child without reporting a diagnostic',
      fn: async () => {
        await using home = await makeHome();
        writeSpawnRecord({
          home: home.path,
          filename: 'completed-record.json',
          content: JSON.stringify({
            lastMessage: 'finished',
            parentSessionId: PARENT_SESSION_ID,
            sessionId: 'child-session',
            spawnId: 'spawn-id',
            status: 'stopped',
            transcriptPath: '/transcript.jsonl',
          },),
        },);

        /**
         Scan run over a record that is deliverable.
         */
        const run = runScan(home.path,);
        expect(run.stdout,)
          .toContain('finished',);
        expect(run.stderr,)
          .not
          .toContain('spawn state unreadable',);
      },
    },),
  ],
},);
