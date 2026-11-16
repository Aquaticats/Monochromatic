import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import type {
  CommandError,
  CommandSpawnError,
  CommandTimeoutError,
} from '../dist/final/node/errors.mjs';
import {
  run,
  runAllowingFailure,
} from '../dist/final/node/runner.mjs';

/**
 Name of a program no test host provides.
 */
const MISSING_PROGRAM = 'wg-quicker-no-such-program-for-the-test';

/**
 Deadline a test gives a command it means to outlive, in milliseconds.
 Short enough for a test, and the runner takes any bound the caller gives.
 */
const TEST_DEADLINE_MILLISECONDS = 200;

/**
 Runs a call that must refuse and hands back what it threw.

 @param call - Call expected to reject.

 @returns What the call threw.
 */
async function refusalOf(call: () => Promise<unknown>,): Promise<unknown> {
  try {
    await call();
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('Expected the call to refuse, but it returned',);
}

/**
 Waits, bounded, for a file to appear.

 @param path - File to look for.

 @returns Whether it appeared within the bound.
 */
async function fileAppears({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   When the wait gives up.
   */
  const deadline = performance.now() + 3_000;
  while (performance.now() < deadline) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- The file is looked for again within one bound.
      await readFile(path,);
      return true;
    }
    catch (error: unknown) {
      // Not written yet; the bound above decides when to stop looking.
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT')
        throw error;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- The pause is part of the same bounded wait.
    await wait(50,);
  }
  return false;
}

/**
 Waits, bounded, for a process to end.

 A signal ends a process a moment after it is sent, so the end is awaited rather than assumed.

 @param pid - Program number of the process.

 @param milliseconds - Longest time to wait.

 @returns Whether the process ended within that time.
 */
async function endsWithin(
  {
    pid,
    milliseconds,
  }: {
    readonly pid: number;
    readonly milliseconds: number;
  },
): Promise<boolean> {
  /**
   When the wait gives up.
   */
  const deadline = performance.now() + milliseconds;
  while (performance.now() < deadline) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- The end is polled within one bound.
    if (!(await canStillRun({ pid, },)))
      return true;
    // oxlint-disable-next-line eslint/no-await-in-loop -- The pause is part of the same bounded wait.
    await wait(10,);
  }
  return !(await canStillRun({ pid, },));
}

/**
 Ends a process the test started, tolerating one that has already ended.

 @param pid - Program number of the process.
 */
async function endProcess({ pid, }: { readonly pid: number; },): Promise<void> {
  try {
    process.kill(pid, 'SIGKILL',);
  }
  catch (error: unknown) {
    // A process that has already ended needs no signal.
    if ((error as NodeJS.ErrnoException).code !== 'ESRCH')
      throw error;
  }
}

/**
 Reports whether a program number names a process that can still run.

 A process the kernel holds only as a zombie cannot run anything, which is what a
 killed grandchild is until its new parent reaps it.

 @param pid - Program number to look for.

 @returns Whether a process that can still run has that number.

 @example
 ```ts
 await canStillRun({ pid: 4242 });
 ```
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

/**
 Makes a scratch directory and binds its removal to the caller's scope.

 @returns The directory path.

 @example
 ```ts
 await using directory = await scratchDirectory();
 ```
 */
async function scratchDirectory(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   New directory under the harness scratch root.
   */
  const path = await mkdtemp(join(scratchRoot(), 'wg-runner-',),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { force: true, recursive: true, },);
    },
  };
}

await describe({
  name: 'runner',
  children: [
    it({
      name: 'names a program that cannot be started as a spawn failure, with the command it tried and the reason the system gave',
      fn: async () => {
        const refusal = await refusalOf(() => run({ command: MISSING_PROGRAM, args: [], },),);
        expect((refusal as Error).name,).toBe('CommandSpawnError',);
        expect((refusal as Error).message,).toContain(MISSING_PROGRAM,);
        expect((refusal as Error).message,).toContain('ENOENT',);
        expect((refusal as CommandSpawnError).command,).toBe(MISSING_PROGRAM,);
      },
    },),
    it({
      name: 'refuses to report a program that cannot be started as an allowed failure, because a program that never ran is not an exit code',
      fn: async () => {
        const refusal = await refusalOf(() => runAllowingFailure({ command: MISSING_PROGRAM, args: [], },),);
        expect((refusal as Error).name,).toBe('CommandSpawnError',);
        expect((refusal as Error).message,).toContain(MISSING_PROGRAM,);
      },
    },),
    it({
      name: 'returns the exit code and both streams of a command that ran and failed, so an allowed failure stays an answer and not a refusal',
      fn: async () => {
        const result = await runAllowingFailure({
          command: 'sh',
          args: ['-c', 'echo out; echo err >&2; exit 3',],
        },);
        expect(result.exitCode,).toBe(3,);
        expect(result.stdout,).toBe('out\n',);
        expect(result.stderr,).toBe('err\n',);
      },
    },),
    it({
      name: 'still reports a command that failed as a typed command failure, so the exit code and stderr reach the person',
      fn: async () => {
        const refusal = await refusalOf(() => run({
          command: 'sh',
          args: ['-c', 'echo reason >&2; exit 4',],
        },),);
        expect((refusal as Error).name,).toBe('CommandError',);
        expect((refusal as CommandError).exitCode,).toBe(4,);
        expect((refusal as CommandError).stderr,).toBe('reason\n',);
      },
    },),
    it({
      name: 'ends a command that outlives its deadline, and names the command and the limit it passed',
      fn: async () => {
        const started = performance.now();
        const refusal = await refusalOf(() => run({
          command: 'sleep',
          args: ['30',],
          deadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
        },),);
        const elapsed = performance.now() - started;
        expect((refusal as Error).name,).toBe('CommandTimeoutError',);
        expect((refusal as CommandTimeoutError).command,).toBe('sleep',);
        expect((refusal as CommandTimeoutError).deadlineMilliseconds,).toBe(TEST_DEADLINE_MILLISECONDS,);
        expect((refusal as Error).message,).toContain(String(TEST_DEADLINE_MILLISECONDS,),);
        expect(elapsed,).toBeLessThan(10_000,);
      },
    },),
    it({
      name: 'ends a command that outlives its deadline on the allowed-failure path too, so a wedged tool cannot hold a teardown',
      fn: async () => {
        const refusal = await refusalOf(() => runAllowingFailure({
          command: 'sleep',
          args: ['30',],
          deadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
        },),);
        expect((refusal as Error).name,).toBe('CommandTimeoutError',);
      },
    },),
    it({
      name: 'ends the call at its deadline when the command exited but a process it left behind holds the output open, and keeps what the command wrote',
      fn: async () => {
        const started = performance.now();
        const result = await runAllowingFailure({
          command: 'sh',
          // The shell exits at once, and the sleep it started keeps standard output and standard error open for two seconds.
          args: ['-c', 'echo kept; sleep 2 & exit 0',],
          deadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
        },);
        const elapsed = performance.now() - started;
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toBe('kept\n',);
        expect(elapsed,).toBeLessThan(1_000,);
      },
    },),
    it({
      name: 'returns when a hook shell exits although a process it left behind holds its output open, instead of waiting the whole deadline for the pipes',
      fn: async () => {
        await using directory = await scratchDirectory();
        /**
         File the hook writes the background process number into, which this test ends afterwards.
         */
        const pidFile = join(directory.path, 'background.pid',);
        /**
         When the call started, for the wait it is expected not to take.
         */
        const started = performance.now();
        /**
         What the call answered.
         */
        const result = await runAllowingFailure({
          command: 'sh',
          // The shell exits at once, and the sleep it started keeps standard output and standard error open.
          args: ['-c', `echo kept; sleep 5 & echo $! > '${pidFile}'; exit 0`,],
          deadlineMilliseconds: 3_000,
        },);
        /**
         How long the call took.
         */
        const elapsed = performance.now() - started;
        /**
         Program number of the process the hook left behind.
         */
        const survivor = Number((await readFile(pidFile, 'utf8',)).trim(),);
        await endProcess({ pid: survivor, },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toBe('kept\n',);
        expect(elapsed,).toBeLessThan(1_000,);
      },
    },),
    it({
      name: 'leaves a program a hook started with a standard output it can still write to when the hook itself passes its deadline, because the deadline ends the command and not what it left behind',
      fn: async () => {
        await using directory = await scratchDirectory();
        /**
         File the background program writes only if its write after the deadline worked.
         */
        const marker = join(directory.path, 'wrote-after-deadline',);
        /**
         What the call threw.
         */
        const refusal = await refusalOf(() => run({
          command: 'sh',
          // The shell waits on its background program, so the deadline ends a command that is still running.
          args: ['-c', `( sleep 0.5; echo after-deadline; touch '${marker}' ) & wait`,],
          deadlineMilliseconds: TEST_DEADLINE_MILLISECONDS,
        },),);
        expect((refusal as Error).name,).toBe('CommandTimeoutError',);
        // The write comes half a second after the deadline ended the shell,
        // and a reader that closed the pipe would end the program here instead.
        expect(await fileAppears({ path: marker, },),).toBe(true,);
      },
    },),
    it({
      name: 'leaves a command that finishes inside its deadline alone, with both streams and its exit code',
      fn: async () => {
        const result = await run({
          command: 'sh',
          args: ['-c', 'echo out; echo err >&2',],
          deadlineMilliseconds: 30_000,
        },);
        expect(result.stdout,).toBe('out\n',);
        expect(result.stderr,).toBe('err\n',);
      },
    },),
  ],
},);
