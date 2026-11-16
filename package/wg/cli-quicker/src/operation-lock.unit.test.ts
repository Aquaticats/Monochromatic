import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
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

import { isErrnoException, } from '../dist/final/node/errors.mjs';
import { claimOperationLock, } from '../dist/final/node/operation-lock.mjs';

/**
 Longest a case waits for a call that is expected to return on its own.

 It is far longer than the bounds the cases give the lock, so a case that reaches it
 shows a call that waited with no bound of its own.
 */
const CASE_BOUND_MILLISECONDS = 5_000;

/**
 Longest a claim process may live after a refusal, in milliseconds.

 The refusal itself takes milliseconds, and the readiness timer's default is ten seconds:
 a timer that outlives the refusal shows up as a process that lives the whole ten.
 */
const READINESS_TIMER_LEAK_MILLISECONDS = 3_000;

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
else if (mode === 'unexpected-line') {
  process.stdout.write('unexpected-output\\n',);
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
    readonly mode: 'slow' | 'ignores-sigterm' | 'unexpected-line';
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
  await using restore = {
    async [Symbol.asyncDispose](): Promise<void> {
      if (previousPath === undefined)
        delete process.env.PATH;
      else
        process.env.PATH = previousPath;
      delete process.env.WG_TEST_FLOCK_MODE;
      delete process.env.WG_TEST_FLOCK_PID_FILE;
      await rm(directory, { force: true, recursive: true, },);
    },
  };
  return await body({ directory, pidFile, },);
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
    const [state,] = stat.slice(stat.lastIndexOf(')',) + 2,)
      .split(' ',);
    return state !== 'Z';
  }
  catch (error: unknown) {
    // The kernel reports a program number it has no process for as either code, depending on the lookup.
    if (['ENOENT', 'ESRCH',].includes((error as NodeJS.ErrnoException).code ?? '',))
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
             What the claim answered, or what it threw.
             */
            const answered = { refusal: new Error('the claim returned'), };
            try {
              await claimOperationLock({
                lockPath: join(directory, 'slow.lock',),
                conflictMessage: 'busy',
                errorFactory: (message,) => new Error(message,),
                deadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
              },);
            }
            catch (error) {
              answered.refusal = isErrnoException(error) ? new Error(error.message,) : new Error(String(error,),);
            }
            /**
             How long the claim took.
             */
            const elapsed = performance.now() - started;
            expect(answered.refusal.message,).toContain(`${String(TEST_DEADLINE_MILLISECONDS,)} ms`,);
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
              (async function release(): Promise<string> {
                await lock[Symbol.asyncDispose]();
                return 'released';
              })(),
              (async function waitForCase(): Promise<string> {
                await wait(CASE_BOUND_MILLISECONDS,);
                return 'still waiting';
              })(),
            ],);
            /**
             Whether the holder the release should have ended can still run.
             */
            const stillRunning = await canStillRun({ pid: holderPid, },);
            try {
              process.kill(holderPid, 'SIGKILL',);
            }
            catch (error: unknown) {
              // The release already ended the holder.
              if ((error as NodeJS.ErrnoException).code !== 'ESRCH')
                throw error;
            }
            expect(released,).toBe('released',);
            expect(stillRunning,).toBe(false,);
          },
        },);
      },
    },),
    it({
      name: 'ends the process at once when the holder prints a line that is not its readiness line, because a refusal leaves no timer behind',
      timeout: 20_000,
      fn: async () => {
        /**
         Scratch directory of the case.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wg-operation-lock-line-',),);
        await using scratch = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        await standInFlock({ directory, },);
        /**
         Claim run in a process of its own, so what is measured is when that process can end.
         */
        const claimScript = join(directory, 'claim.ts',);
        await writeFile(
          claimScript,
          `import { claimOperationLock, } from ${JSON.stringify(new URL('../dist/final/node/operation-lock.mjs', import.meta.url,).pathname,)};
try {
  await claimOperationLock({
    lockPath: ${JSON.stringify(join(directory, 'line.lock',),)},
    conflictMessage: 'busy',
    errorFactory: (message,) => new Error(message,),
  },);
  console.log('acquired',);
}
catch (error) {
  console.log(\`refused: \${String(error,)}\`,);
}
`,
        );
        /**
         When the claim process was started.
         */
        const started = performance.now();
        /**
         The claim process.
         */
        const claim = spawn(process.execPath, [claimScript,], {
          env: {
            ...process.env,
            PATH: `${directory}:${process.env.PATH ?? ''}`,
            WG_TEST_FLOCK_MODE: 'unexpected-line',
          },
          stdio: ['ignore', 'pipe', 'pipe',],
        },);
        /**
         Everything the claim printed.
         */
        let printed = '';
        claim.stdout.on('data', (chunk: Buffer,) => {
          printed += chunk.toString('utf8',);
        },);
        await once(claim, 'close',);
        /**
         How long the claim process lived.
         */
        const elapsed = performance.now() - started;
        expect(printed,).toContain('refused: Error: Unexpected operation-lock readiness output: unexpected-output',);
        // The readiness timer must not outlive the refusal it ended: the default ten seconds of it would keep this process running.
        expect(elapsed,).toBeLessThan(READINESS_TIMER_LEAK_MILLISECONDS,);
      },
    },),
  ],
},);
