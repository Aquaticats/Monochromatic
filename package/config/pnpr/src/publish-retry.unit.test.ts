import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  FORBIDDEN_RETRY_DELAY_MS,
  isForbiddenPublishError,
  NPM_FORBIDDEN_MARKER,
  publishWithForbiddenRetry,
} from '../dist/final/node/index.mjs';

/**
 Fixed clock reading for deadline arithmetic.
 */
const NOW = 1_000_000;

/**
 Builds an error shaped like nano-spawn's `SubprocessError` with captured output.

 @param output - Captured npm output.

 @returns Error carrying `output`.
 */
function subprocessError(output: string,): Error {
  return Object.assign(
    new Error('Command failed with exit code 1: npm publish',),
    { output, },
  );
}

/**
 Refusal npm printed in pnpr-publish run 34922532212.

 The PUT URL in it is that run's pre-move root surface;
 from the 0.1.0-alpha.16 base the same request targets
 `https://pnpr.c.aquati.cat/npm/~monochromatic-dev/...`.
 */
const FORBIDDEN_OUTPUT = `${NPM_FORBIDDEN_MARKER}\nnpm error 403 403 Forbidden - PUT https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2fconfig-oxlint`;

/**
 Records attempts, pauses, and retry reports around a scripted sequence of attempt outcomes.

 @param outcomes - Error to throw per attempt, or `'ok'` to succeed.

 @param deadline - Retry deadline passed to the function under test.

 @returns Recorded calls and the settled error, when any.
 */
async function runScripted({
  outcomes,
  deadline,
}: {
  readonly outcomes: readonly (Error | 'ok')[];
  readonly deadline: number;
},): Promise<{
  readonly attempts: number;
  readonly sleeps: readonly number[];
  readonly retryNumbers: readonly number[];
  readonly caught: readonly unknown[];
}> {
  /**
   Attempt calls so far.
   */
  const attemptCalls: number[] = [];
  /**
   Pause durations requested.
   */
  const sleeps: number[] = [];
  /**
   Retry numbers reported.
   */
  const retryNumbers: number[] = [];
  try {
    await publishWithForbiddenRetry({
      attempt: async function scriptedAttempt() {
        /**
         Outcome for this attempt.
         */
        const outcome = outcomes[attemptCalls.length];
        attemptCalls.push(attemptCalls.length,);
        if (outcome === undefined)
          throw new Error('scripted attempts exhausted',);
        if (outcome !== 'ok')
          throw outcome;
      },
      deadline,
      now: function fixedNow() {
        return NOW;
      },
      sleep: async function recordSleep(milliseconds,) {
        sleeps.push(milliseconds,);
      },
      onRetry: function recordRetry({ retryNumber, },) {
        retryNumbers.push(retryNumber,);
      },
    },);
    return {
      attempts: attemptCalls.length,
      sleeps,
      retryNumbers,
      caught: [],
    };
  }
  catch (error: unknown) {
    return {
      attempts: attemptCalls.length,
      sleeps,
      retryNumbers,
      caught: [error,],
    };
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: isForbiddenPublishError.name,
      children: [
        it({
          name: 'recognizes captured npm output carrying E403',
          fn: async () => {
            expect(
              isForbiddenPublishError(subprocessError(FORBIDDEN_OUTPUT,),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'rejects other npm failures, errors without output, and non-errors',
          fn: async () => {
            expect(
              isForbiddenPublishError(subprocessError('npm error code E409',),),
            ).toBe(false,);
            expect(
              isForbiddenPublishError(new Error(NPM_FORBIDDEN_MARKER,),),
            ).toBe(false,);
            expect(isForbiddenPublishError({ output: NPM_FORBIDDEN_MARKER, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: publishWithForbiddenRetry.name,
      children: [
        it({
          name: 'publishes once without pausing when the first attempt succeeds',
          fn: async () => {
            /**
             Recorded run.
             */
            const run = await runScripted({
              outcomes: ['ok',],
              deadline: NOW + (3 * FORBIDDEN_RETRY_DELAY_MS),
            },);
            expect(run.attempts,).toBe(1,);
            expect(run.sleeps,).toEqual([],);
            expect(run.caught,).toEqual([],);
          },
        },),
        it({
          name: 'retries an E403 refusal after one pause and stops at the first success',
          fn: async () => {
            /**
             Recorded run.
             */
            const run = await runScripted({
              outcomes: [
                subprocessError(FORBIDDEN_OUTPUT,),
                'ok',
              ],
              deadline: NOW + (3 * FORBIDDEN_RETRY_DELAY_MS),
            },);
            expect(run.attempts,).toBe(2,);
            expect(run.sleeps,).toEqual([FORBIDDEN_RETRY_DELAY_MS,],);
            expect(run.retryNumbers,).toEqual([1,],);
            expect(run.caught,).toEqual([],);
          },
        },),
        it({
          name: 'rethrows a non-E403 failure without retrying',
          fn: async () => {
            /**
             Failure that is not an authorization refusal.
             */
            const conflict = subprocessError('npm error code E409',);
            /**
             Recorded run.
             */
            const run = await runScripted({
              outcomes: [
                conflict,
                'ok',
              ],
              deadline: NOW + (3 * FORBIDDEN_RETRY_DELAY_MS),
            },);
            expect(run.attempts,).toBe(1,);
            expect(run.sleeps,).toEqual([],);
            expect(run.caught,).toEqual([conflict,],);
          },
        },),
        it({
          name: 'makes one retry per whole pause before the deadline, then rethrows the last refusal',
          fn: async () => {
            /**
             Final refusal expected to surface.
             */
            const lastRefusal = subprocessError(FORBIDDEN_OUTPUT,);
            /**
             Recorded run.
             */
            const run = await runScripted({
              outcomes: [
                subprocessError(FORBIDDEN_OUTPUT,),
                subprocessError(FORBIDDEN_OUTPUT,),
                lastRefusal,
                'ok',
              ],
              deadline: NOW + (2 * FORBIDDEN_RETRY_DELAY_MS) + 1,
            },);
            expect(run.attempts,).toBe(3,);
            expect(run.retryNumbers,).toEqual([1, 2,],);
            expect(run.caught,).toEqual([lastRefusal,],);
          },
        },),
        it({
          name: 'attempts once without retrying once the deadline has passed',
          fn: async () => {
            /**
             Refusal after the window closed.
             */
            const refusal = subprocessError(FORBIDDEN_OUTPUT,);
            /**
             Recorded run.
             */
            const run = await runScripted({
              outcomes: [
                refusal,
                'ok',
              ],
              deadline: NOW - 1,
            },);
            expect(run.attempts,).toBe(1,);
            expect(run.sleeps,).toEqual([],);
            expect(run.caught,).toEqual([refusal,],);
          },
        },),
      ],
    },),
  ],
},);
