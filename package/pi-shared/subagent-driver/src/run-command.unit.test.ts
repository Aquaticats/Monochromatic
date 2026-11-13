/**
 The one seam that asks the machine questions: a command that will not answer is killed at its bound rather than waited for.

 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  errorCodeOf,
  runCommandShowingItsOutputWithin,
  runCommandWithChildProcess,
} from '../dist/final/node/index.mjs';

/**
 The code the kernel gives a signal sent to a process that is not there, which is the answer this reads as "it has gone".
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 Milliseconds a stand-in question is given, which is far less than the sleep it is asked to run.
 */
const SHORT_BOUND_MILLISECONDS = 100;

/**
 Seconds the command asked to sleep, so a question with no bound would still be sleeping when the test ends.
 */
const LONG_SLEEP_SECONDS = 30;

/**
 A folder one test's files go in, removed when the test ends.

 @returns the folder's path, and the disposer that removes it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-shown-command-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Says whether a process is still there, which a process its group was killed for is not.

 @param pid - the process number

 @returns whether the kernel still has that process

 @example
 ```ts
 const there = await processIsThere(4242);
 ```
 */
async function processIsThere(pid: number,): Promise<boolean> {
  try {
    process.kill(pid, 0,);
    return true;
  }
  catch (error) {
    // A process the kernel no longer has is exactly what this asks about; any other failure is one this cannot answer for.
    if (errorCodeOf(error,) !== NO_SUCH_PROCESS_CODE) {
      throw error;
    }
    return false;
  }
}

await describe({
  name: runCommandShowingItsOutputWithin.name,
  children: [
    it({
      name: 'ends a command that blocks at its bound, and ends the children it started with it, so nothing of it is left running',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the blocking command writes the numbers of itself and of the child it started.
         */
        const numbers = join(
          folder.path,
          'numbers.txt',
        );
        /**
         What the bounded run answered.
         */
        const ended = await runCommandShowingItsOutputWithin({
          command: 'sh',
          words: [
            '-c',
            `sleep 30 & printf '%s %s\n' "$$" "$!" > '${numbers}'; wait`,
          ],
          timeoutMilliseconds: 300,
        },);
        expect(ended.givenUp,).toBe(true,);
        /**
         The shell's number and its child's, as the shell wrote them before it blocked.
         */
        const [shell, child,] = (await readFile(
          numbers,
          'utf8',
        )).trim()
          .split(' ',)
          .map(Number,);
        // The group is what the bound ends: the shell and the child it started are both gone, not just the process this named.
        expect(await processIsThere(shell ?? 0,),).toBe(false,);
        expect(await processIsThere(child ?? 0,),).toBe(false,);
      },
      timeout: 30_000,
    },),
    it({
      name: 'answers with the status of a command that ends inside its bound, and never calls it given up',
      fn: async () => {
        expect(await runCommandShowingItsOutputWithin({
          command: 'sh',
          words: ['-c', 'exit 3',],
          timeoutMilliseconds: 30_000,
        },),).toEqual({ status: 3, },);
      },
    },),
  ],
},);

await describe({
  name: runCommandWithChildProcess.name,
  children: [
    it({
      name: 'answers with what a command printed and the status it exited with',
      fn: async () => {
        expect(await runCommandWithChildProcess({
          command: 'sh',
          words: ['-c', 'printf "out"; printf "err" >&2; exit 3',],
        },),).toEqual({
          status: 3,
          stdout: 'out',
          stderr: 'err',
        },);
      },
    },),
    it({
      name: 'says which code a command that never ran carries, so a program this machine lacks is told from one its own bound killed',
      fn: async () => {
        /**
         What asking for a program this machine does not have answered.
         */
        const missing = await runCommandWithChildProcess({
          command: 'no-such-program-for-this-test',
          words: [],
        },);
        expect(missing.status,).toBe(-1,);
        expect(missing.problemCode,).toBe('ENOENT',);
        expect(missing.killedByItsOwnBound,).toBeUndefined();
        /**
         What a command its own bound killed answered, which carries no code: no program was missing there.
         */
        const killed = await runCommandWithChildProcess({
          command: 'sleep',
          words: [String(LONG_SLEEP_SECONDS,),],
          timeoutMilliseconds: SHORT_BOUND_MILLISECONDS,
        },);
        expect(killed.killedByItsOwnBound,).toBe(true,);
        expect(killed.problemCode,).toBeUndefined();
      },
    },),
    it({
      name: 'kills a command that runs past its bound instead of waiting for it',
      fn: async () => {
        /**
         How long the question took, which must be the bound rather than the sleep.
         */
        const startedAt = Date.now();
        /**
         What the killed command left.
         */
        const output = await runCommandWithChildProcess({
          command: 'sleep',
          words: [String(LONG_SLEEP_SECONDS,),],
          timeoutMilliseconds: SHORT_BOUND_MILLISECONDS,
        },);
        expect(Date.now() - startedAt,).toBeLessThan(LONG_SLEEP_SECONDS * 1_000,);
        expect(output.status,).toBe(-1,);
        expect(output.problem,).toBeDefined();
        // The bound is named as what ended it, so a caller never reads a command its own bound killed as one that failed.
        expect(output.killedByItsOwnBound,).toBe(true,);
        expect(output.signal,).toBe('SIGKILL',);
      },
    },),
    it({
      name: 'answers a signal from outside as the signal that ended the command, never as its own bound',
      fn: async () => {
        /**
         What a command that is signalled by something else leaves.
         */
        const output = await runCommandWithChildProcess({
          command: 'sh',
          words: ['-c', 'kill -TERM $$',],
        },);
        expect(output.status,).toBe(-1,);
        expect(output.signal,).toBe('SIGTERM',);
        expect(output.killedByItsOwnBound,).toBeUndefined();
      },
    },),
    it({
      name: 'answers that a program that is not there never ran, which a caller reports rather than reads as empty output',
      fn: async () => {
        /**
         What asking for a program that does not exist leaves.
         */
        const output = await runCommandWithChildProcess({
          command: 'a-program-that-is-not-here',
          words: [],
        },);
        expect(output.status,).toBe(-1,);
        expect(output.problem,).toContain('ENOENT',);
      },
    },),
  ],
},);
