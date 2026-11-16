import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { claimOperationLock, } from '../dist/final/node/operation-lock.mjs';

/**
 Longest a case waits for a call that is expected to return on its own.

 It is far longer than the bounds the cases give the lock, so a case that reaches it
 shows a call that waited with no bound of its own.
 */
const CASE_BOUND_MILLISECONDS = 5_000;

/**
 Bound the cases give the readiness and the release wait.

 Shorter than any stand-in's delay, so a bound that exists is what ends the wait,
 and long enough that the stand-in's own start is never what the bound measures.
 */
const TEST_DEADLINE_MILLISECONDS = 300;

/**
 Delay the stand-in `flock` takes before it does anything, in milliseconds.

 Far longer than the bound the case gives the claim, so a claim without a bound waits it out.
 */
const STAND_IN_DELAY_MILLISECONDS = 2_000;

/**
 Writes a stand-in `flock` program and answers its path.

 The stand-in is what a slow or wedged holder looks like to the claim:
 it either takes the whole delay and exits without a readiness line,
 or it announces readiness and ignores `SIGTERM` until it is killed.

 @param directory - Scratch directory of the case.

 @returns The stand-in's path.
 */
async function standInFlock({ directory, }: { readonly directory: string; },): Promise<string> {
  /**
   Stand-in program, which reads its behaviour from the environment.
   */
  const path = join(directory, 'flock',);
  await writeFile(
    path,
    `#!${process.execPath}
import { writeFileSync, } from 'node:fs';
const mode = process.env.WG_TEST_FLOCK_MODE;
if (mode === 'slow') {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, ${String(STAND_IN_DELAY_MILLISECONDS,)});
  process.exit(0);
}
else if (mode === 'ignores-sigterm') {
  // The handler comes first: the claim may send its signal the moment it reads the readiness line.
  process.on('SIGTERM', () => {});
  writeFileSync(process.env.WG_TEST_FLOCK_PID_FILE, String(process.pid,),);
  process.stdout.write('wg-quicker-operation-lock-ready\\n',);
  setInterval(() => {}, 1_000,);
}
else {
  process.exit(1);
}
`,
  );
  await chmod(path, 0o700,);
  return path;
}

/**
 Runs a case's body with a stand-in `flock` first on the search path,
 and the search path put back afterwards.

 @param mode - Behaviour the stand-in takes.

 @param body - What the case does with the stand-in's directory.

 @returns What the body answered.
 */
async function withStandInFlock<Result>(
  {
    mode,
    body,
  }: {
    readonly mode: 'slow' | 'ignores-sigterm';
    readonly body: (call: {
      readonly directory: string;
      readonly pidFile: string;
    },) => Promise<Result>;
  },
): Promise<Result> {
  /**
   Scratch directory of the case.
   */
  const directory = await mkdtemp(join(scratchRoot(), 'wg-operation-lock-',),);
  /**
   File the stand-in writes its program number into.
   */
  const pidFile = join(directory, 'holder.pid',);
  /**
   Search path this case had before it started.
   */
  const previousPath = process.env.PATH;
  await standInFlock({ directory, },);
  process.env.PATH = `${directory}:${previousPath ?? ''}`;
  process.env.WG_TEST_FLOCK_MODE = mode;
  process.env.WG_TEST_FLOCK_PID_FILE = pidFile;
  try {
    return await body({ directory, pidFile, },);
  }
  finally {
    if (previousPath === undefined)
      delete process.env.PATH;
    else
      process.env.PATH = previousPath;
    delete process.env.WG_TEST_FLOCK_MODE;
    delete process.env.WG_TEST_FLOCK_PID_FILE;
    await rm(directory, { force: true, recursive: true, },);
  }
}

/**
 Reports whether a program number names a process that can still run.

 @param pid - Program number to look for.

 @returns Whether a process that can still run has that number.
 */
async function canStillRun({ pid, }: { readonly pid: number; },): Promise<boolean> {
  try {
    /**
     Stat line of the process, whose state letter follows the parenthesised name.
     */
    const stat = await readFile(`/proc/${String(pid,)}/stat`, 'utf8',);
    /**
     State letter of the process.
     */
    const state = stat.slice(stat.lastIndexOf(')',) + 2,)
      .split(' ',)[0];
    return state !== 'Z';
  }
  catch (error: unknown) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT')
      return false;
    throw error;
  }
}

await describe({
  name: claimOperationLock.name,
  // One at a time: each case owns the search path of the process.
  concurrency: 1,
  children: [
    it({
      name: 'refuses within its readiness bound when the holder takes longer than the bound, where the claim used to wait as long as the holder took',
      fn: async () => {
        await withStandInFlock({
          mode: 'slow',
          body: async ({ directory, },) => {
            /**
             When the claim started, for the bound it is expected to keep.
             */
            const started = performance.now();
            /**
             What the claim threw.
             */
            const refusal = await claimOperationLock({
              lockPath: join(directory, 'slow.lock',),
              conflictMessage: 'busy',
              errorFactory: (message,) => new Error(message,),
              deadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
            },).then(
              () => undefined,
              (error: unknown) => error,
            );
            /**
             How long the claim took.
             */
            const elapsed = performance.now() - started;
            expect(Error.isError(refusal,),).toBe(true,);
            expect((refusal as Error).message,).toContain(`${String(TEST_DEADLINE_MILLISECONDS,)} ms`,);
            expect(elapsed,).toBeLessThan(STAND_IN_DELAY_MILLISECONDS,);
          },
        },);
      },
    },),
    it({
      name: 'ends a holder that does not stop on SIGTERM, so releasing the lock cannot wait forever on it',
      fn: async () => {
        await withStandInFlock({
          mode: 'ignores-sigterm',
          body: async ({ directory, pidFile, },) => {
            /**
             The claim, whose holder ignores the polite signal.
             */
            const lock = await claimOperationLock({
              lockPath: join(directory, 'stubborn.lock',),
              conflictMessage: 'busy',
              errorFactory: (message,) => new Error(message,),
              releaseDeadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
            },);
            /**
             Program number of the stand-in holder.
             */
            const holderPid = Number((await readFile(pidFile, 'utf8',)).trim(),);
            /**
             What releasing the lock answered within the case's bound.
             */
            const released = await Promise.race([
              lock[Symbol.asyncDispose]()
                .then(() => 'released',),
              wait(CASE_BOUND_MILLISECONDS,)
                .then(() => 'still waiting',),
            ],);
            try {
              expect(released,).toBe('released',);
              expect(await canStillRun({ pid: holderPid, },),).toBe(false,);
            }
            finally {
              try {
                process.kill(holderPid, 'SIGKILL',);
              }
              catch (error: unknown) {
                // The release already ended the holder.
                if ((error as NodeJS.ErrnoException).code !== 'ESRCH')
                  throw error;
              }
            }
          },
        },);
      },
    },),
  ],
},);
