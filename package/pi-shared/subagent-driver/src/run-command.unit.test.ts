/**
 The one seam that asks the machine questions: a command that will not answer is killed at its bound rather than waited for.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  runCommandShowingItsOutput,
  runCommandWithChildProcess,
} from '../dist/final/node/index.mjs';

/**
 Milliseconds a stand-in question is given, which is far less than the sleep it is asked to run.
 */
const SHORT_BOUND_MILLISECONDS = 100;

/**
 Milliseconds a program that ignores the polite signal is given after it, which is the grace `run-command.ts` states.
 */
const POLITE_SIGNAL_GRACE_MILLISECONDS = 5_000;

/**
 Milliseconds this test waits for the grace and the kill to have happened, above the grace itself.
 */
const AFTER_THE_GRACE_MILLISECONDS = 1_500;

/**
 A word only the child of the bound test carries, which is how a test finds that child again without a number from anywhere.
 */
const A_CHILD_OF_THIS_TEST = 'a-child-of-the-bound-test-that-ignores-its-polite-signal';

/**
 Waits a stated time, so a test can watch what a bound left behind.

 @param milliseconds - how long to wait

 @example
 ```ts
 await passTime(1_000);
 ```
 */
async function passTime(milliseconds: number,): Promise<void> {
  await new Promise(function waitItOut(resolve,) {
    setTimeout(resolve, milliseconds,);
  },);
}

/**
 Reads the process table entries, and nothing when the folder cannot be read here.

 @returns one entry per process

 @example
 ```ts
 const entries = await processEntries();
 ```
 */
async function processEntries(): Promise<readonly string[]> {
  try {
    return await readdir('/proc',);
  }
  catch {
    return [];
  }
}

/**
 Says whether any process still carries a word in its command line.

 @param word - the word a child of this test was started with

 @returns whether such a process is still there

 @example
 ```ts
 const there = await aChildStillRuns({ word: A_CHILD_OF_THIS_TEST });
 ```
 */
async function aChildStillRuns({ word, }: { readonly word: string },): Promise<boolean> {
  /**
   The process entries, and nothing at all when the folder cannot be read here.
   */
  const entries = await processEntries();
  for (const entry of entries) {
    if (!/^[0-9]+$/.test(entry,)) {
      continue;
    }
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- one process after another, and the first match ends the search
      const words = (await readFile(`/proc/${entry}/cmdline`, 'utf8',)).split('\0',);
      if (words.includes(word,)) {
        return true;
      }
    }
    catch {
      // A process of another account, or one that ended between the listing and the read, carries no word of this test.
    }
  }
  return false;
}

/**
 Seconds the command asked to sleep, so a question with no bound would still be sleeping when the test ends.
 */
const LONG_SLEEP_SECONDS = 30;

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

await describe({
  name: runCommandShowingItsOutput.name,
  children: [
    it({
      name: 'answers at its own bound when the program it started ignores the polite signal, which no wait on that program bounds',
      timeout: 20_000,
      fn: async () => {
        /**
         When the call was made.
         */
        const started = Date.now();
        /**
         What the call answered for a program that ignores the polite signal and would live for half a minute.
         */
        const ending = await runCommandShowingItsOutput({
          command: process.execPath,
          words: [
            '--input-type=module',
            '--eval',
            "process.on('SIGTERM', () => {}); setTimeout(() => process.exit(0), 30_000);",
          ],
          timeoutMilliseconds: SHORT_BOUND_MILLISECONDS,
        },);
        /**
         How long the call took to answer.
         */
        const tookMilliseconds = Date.now() - started;
        expect(ending,).toEqual({ endedByItsOwnBound: true, },);
        expect(tookMilliseconds,).toBeLessThan(1_000,);
      },
    },),
    it({
      name: 'kills the program a stated grace after the polite signal, so nothing it started outlives the caller',
      timeout: 45_000,
      fn: async () => {
        /**
         What the call answered for a program that ignores the polite signal.
         */
        const ending = await runCommandShowingItsOutput({
          command: process.execPath,
          words: [
            '--input-type=module',
            '--eval',
            "process.on('SIGTERM', () => {}); setTimeout(() => process.exit(0), 60_000);",
            A_CHILD_OF_THIS_TEST,
          ],
          timeoutMilliseconds: SHORT_BOUND_MILLISECONDS,
        },);
        expect(ending,).toEqual({ endedByItsOwnBound: true, },);
        // A grace after the polite signal the program is killed; until then it is still there.
        expect(await aChildStillRuns({ word: A_CHILD_OF_THIS_TEST, },),).toBe(true,);
        await passTime(POLITE_SIGNAL_GRACE_MILLISECONDS + AFTER_THE_GRACE_MILLISECONDS,);
        expect(await aChildStillRuns({ word: A_CHILD_OF_THIS_TEST, },),).toBe(false,);
      },
    },),
  ],
},);
