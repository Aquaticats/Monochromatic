/**
 Tests for background job execution in the built bash-poke artifact.

 @module
 */

import { mkdtemp, readFile, rm, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  constants,
  readClosePayload,
  resolveShell,
  startJob,
  type ShellInvocation,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Milliseconds a cancelled long job is allowed to take before the case fails.
 */
const CANCEL_DEADLINE_MS = 5_000;

/**
 Milliseconds given to a process group to disappear after a force kill.
 */
const GROUP_SETTLE_MS = 250;

/**
 Resolved shell used by every case, so nothing depends on a hardcoded path.
 */
const shell: ShellInvocation = await resolveShell({
  pathValue: process.env.PATH ?? '',
}, );

/**
 Disposable working directory for one case.
 */
type DisposableWorkDir = {
  /**
   Absolute directory jobs run in and spool below.
   */
  readonly path: string;

  /**
   Removal hook.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable working directory.
 
 @returns directory removed on disposal
 
 @example
 ```ts
 await using work = await createDisposableWorkDir();
 ```
 */
async function createDisposableWorkDir(): Promise<DisposableWorkDir> {
  /**
   Absolute directory unique to the current case.
   */
  const path = await mkdtemp(join(scratchRoot(), 'bash-poke-job-', ), );
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { recursive: true, force: true, }, );
    },
  };
}

/**
 Reports whether a process id still exists.
 
 @param pid - process id to probe
 
 @returns whether signalling it would reach a live process
 
 @example
 ```ts
 processExists(1);
 ```
 */
function processExists(pid: number, ): boolean {
  try {
    process.kill(pid, 0, );
    return true;
  }
  catch (error: unknown) {
    return !String(error, ).includes('ESRCH');
  }
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    //region readClosePayload

    describe({
      name: readClosePayload.name,
      children: [
        it({
          name: 'reads an exit code and ignores a null signal',
          fn: async () => {
            expect(readClosePayload({ payload: [0, null, ], }, ), ).toEqual({ code: 0, });
            expect(readClosePayload({ payload: [3, null, ], }, ), ).toEqual({ code: 3, });
          },
        }, ),
        it({
          name: 'reads a signal name and ignores a null code',
          fn: async () => {
            expect(readClosePayload({ payload: [null, 'SIGTERM', ], }, ), )
              .toEqual({ signal: 'SIGTERM', });
          },
        }, ),
        it({
          name: 'returns nothing for a payload that is not a list',
          fn: async () => {
            expect(readClosePayload({ payload: 'close', }, ), ).toEqual({});
            expect(readClosePayload({ payload: undefined, }, ), ).toEqual({});
          },
        }, ),
        it({
          name: 'ignores values of the wrong kind inside a list',
          fn: async () => {
            expect(readClosePayload({ payload: ['0', 1, ], }, ), ).toEqual({});
          },
        }, ),
      ],
    }, ),

    //endregion readClosePayload

    //region startJob

    describe({
      name: startJob.name,
      children: [
        it({
          name: 'records stdout and reports a zero exit code',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'printf hello',
              cwd: work.path,
              shell,
              headChars: 100,
              tailChars: 100,
              killGraceMs: 500,
              tmp: work.path,
            }, );
            const outcome = await job.finished;
            expect(outcome.exitCode, ).toBe(0);
            expect(outcome.cancelled, ).toBe(false);
            expect(outcome.spawnError, ).toBeUndefined();
            expect(job.recorder.snapshot().text, ).toBe('hello');
          },
        }, ),
        it({
          name: 'records stderr in the same buffer',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'printf oops 1>&2',
              cwd: work.path,
              shell,
              headChars: 100,
              tailChars: 100,
              killGraceMs: 500,
              tmp: work.path,
            }, );
            await job.finished;
            expect(job.recorder.snapshot().text, ).toBe('oops');
          },
        }, ),
        it({
          name: 'reports a nonzero exit code',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'exit 3',
              cwd: work.path,
              shell,
              headChars: 100,
              tailChars: 100,
              killGraceMs: 500,
              tmp: work.path,
            }, );
            const outcome = await job.finished;
            expect(outcome.exitCode, ).toBe(3);
          },
        }, ),
        it({
          name: 'reports a command that could not be spawned',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'printf never',
              cwd: work.path,
              shell: { command: '/nonexistent-bash-poke-shell', args: ['-c', ], },
              headChars: 100,
              tailChars: 100,
              killGraceMs: 500,
              tmp: work.path,
            }, );
            const outcome = await job.finished;
            expect(outcome.exitCode, ).toBeUndefined();
            expect(outcome.spawnError, ).toContain('ENOENT');
          },
        }, ),
        it({
          name: 'runs in the working directory it was given',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'pwd',
              cwd: work.path,
              shell,
              headChars: 200,
              tailChars: 200,
              killGraceMs: 500,
              tmp: work.path,
            }, );
            await job.finished;
            expect(job.recorder.snapshot().text.trim(), ).toBe(work.path);
          },
        }, ),
        it({
          name: 'uses the injected clock for the start timestamp',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'true',
              cwd: work.path,
              shell,
              headChars: 10,
              tailChars: 10,
              killGraceMs: 500,
              tmp: work.path,
              now: function fixedClock(): number {
                return 12_345;
              },
            }, );
            expect(job.startedAt, ).toBe(12_345);
            await job.finished;
          },
        }, ),
        it({
          name: 'notifies the chunk callback while output arrives',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const chunks: number[] = [];
            const job = await startJob({
              command: 'printf one; printf two',
              cwd: work.path,
              shell,
              headChars: 100,
              tailChars: 100,
              killGraceMs: 500,
              tmp: work.path,
              onChunk(): void {
                chunks.push(1, );
              },
            }, );
            await job.finished;
            expect(chunks.length > 0, ).toBe(true);
            expect(job.recorder.snapshot().text, ).toBe('onetwo');
          },
        }, ),
        it({
          name: 'spools complete output beyond the poke budgets',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'i=0; while [ $i -lt 40 ]; do printf abcdefghij; i=$((i+1)); done',
              cwd: work.path,
              shell,
              headChars: 10,
              tailChars: 10,
              killGraceMs: 500,
              tmp: work.path,
            }, );
            await job.finished;

            /**
             Spool path, which must exist whenever the temp root is writable.
             */
            const {spoolPath} = job;
            if (spoolPath === undefined)
              throw new Error('expected a spool path');
            const spooled = await readFile(spoolPath, 'utf8', );
            expect(spooled, ).toHaveLength(400);
            expect(
              spoolPath.startsWith(join(work.path, constants.SPOOL_DIR_NAME, ), ),
            ).toBe(true);

            /**
             Bounded snapshot, which must have elided the middle.
             */
            const snapshot = job.recorder.snapshot();
            expect(snapshot.totalChars, ).toBe(400);
            expect(snapshot.truncated, ).toBe(true);
            expect(snapshot.text.length < 400, ).toBe(true);
          },
        }, ),
        it({
          name: 'ends a long job early when cancelled and marks it cancelled',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const startedAt = Date.now();
            const job = await startJob({
              command: 'sleep 30',
              cwd: work.path,
              shell,
              headChars: 10,
              tailChars: 10,
              killGraceMs: 200,
              tmp: work.path,
            }, );
            job.cancel();
            const outcome = await job.finished;
            expect(outcome.cancelled, ).toBe(true);
            expect((Date.now() - startedAt) < CANCEL_DEADLINE_MS, ).toBe(true);
          },
        }, ),
        it({
          name: 'ignores a second cancellation of the same job',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'sleep 30',
              cwd: work.path,
              shell,
              headChars: 10,
              tailChars: 10,
              killGraceMs: 100,
              tmp: work.path,
            }, );
            job.cancel();
            job.cancel();
            const outcome = await job.finished;
            expect(outcome.cancelled, ).toBe(true);
          },
        }, ),
        it({
          name: 'cancels a job started with no grace period immediately',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'sleep 30',
              cwd: work.path,
              shell,
              headChars: 10,
              tailChars: 10,
              killGraceMs: 0,
              tmp: work.path,
            }, );
            job.cancel();
            const outcome = await job.finished;
            expect(outcome.cancelled, ).toBe(true);
          },
        }, ),
        it({
          name: 'reaches a grandchild through the process group',
          fn: async () => {
            await using work = await createDisposableWorkDir();
            const job = await startJob({
              command: 'sleep 30 & printf %s "$!"; wait',
              cwd: work.path,
              shell,
              headChars: 100,
              tailChars: 100,
              killGraceMs: 100,
              tmp: work.path,
            }, );
            // The grandchild's pid arrives on stdout before the shell waits.
            await wait(300, );

            /**
             Grandchild pid printed by the job, which must die with its group.
             */
            const grandchildPid = Math.trunc(
              Number(job.recorder.snapshot().text.trim(), ),
            );
            expect(Number.isNaN(grandchildPid, ), ).toBe(false);
            expect(processExists(grandchildPid, ), ).toBe(true);

            job.cancel();
            await job.finished;
            await wait(GROUP_SETTLE_MS, );
            expect(processExists(grandchildPid, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    //endregion startJob
  ],
}, );
