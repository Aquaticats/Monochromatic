/**
 The one seam that asks the machine questions: a command that will not answer is killed at its bound rather than waited for.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { runCommandWithChildProcess, } from '../dist/final/node/index.mjs';

/**
 Milliseconds a stand-in question is given, which is far less than the sleep it is asked to run.
 */
const SHORT_BOUND_MILLISECONDS = 100;

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
