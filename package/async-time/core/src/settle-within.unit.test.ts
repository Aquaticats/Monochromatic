/**
 Tests for `settleWithin`.
 
 The sharp cases are about a LEAKED TIMER, which is the entire reason this
 exists beside `withTimeout` and `wait`, and they are measured in a CHILD
 PROCESS rather than in this one.
 
 Counting `process.getActiveResourcesInfo()` in-process was tried first and
 abandoned: it does detect the leak, but this suite shares a process with
 cases that deliberately leak timers of their own, so the baseline moves
 underneath the assertion and the result depends on scheduling. The child
 process measures the property that actually matters and that a reader would
 recognise: whether a program stops when its work is done.
 
 @module
 */

import spawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  settleWithin,
  wait,
} from '../dist/final/neutral/index.mjs';

/**
 Long enough that a leaked timer holds a process well past any plausible
 startup cost, so a prompt exit cannot happen by accident.
 */
const LONG_MS = 30_000;

/**
 Short enough to keep the suite quick.
 */
const SHORT_MS = 10;

/**
 Longest a correct child is allowed to take, generously over Node startup.
 */
const EXIT_BUDGET_MS = 8_000;

/**
 Built module the child imports, resolved from this file rather than from the
 child's own cwd, which the runner does not promise.
 */
const BUILT = new URL(
  '../dist/final/neutral/index.mjs',
  import.meta.url,
).href;

/**
 Runs a program in a child process and reports how long it took to exit.
 
 @param source - module source for the child
 
 @returns Milliseconds from spawn to exit, and whether it exited at all
 
 @example
 ```ts
 const { exited, } = await timeChild({ source: 'console.log(1,);', },);
 ```
 */
async function timeChild(
  { source, }: { readonly source: string; },
): Promise<{
  readonly exited: boolean;
  readonly elapsedMs: number;
}> {
  /**
   When the child was started.
   */
  const startedAt = performance.now();

  try {
    await spawn(
      'node',
      [
        '--input-type=module',
        '--eval',
        source,
      ],
      { timeout: EXIT_BUDGET_MS, },
    );
    return {
      exited: true,
      elapsedMs: performance.now() - startedAt,
    };
  }
  catch (error) {
    // A child killed by the timeout is the FINDING here, not an accident, so
    // it is reported as a value. Anything else is rethrown.
    if (String(error,).includes('timed out',) || String(error,).includes('SIGTERM',))
      return {
        exited: false,
        elapsedMs: performance.now() - startedAt,
      };
    throw error;
  }
}

await describe({
  name: settleWithin.name,
  children: [
    it({
      name: 'REPORTS THE WINNER AS A VALUE rather than as an error, because a grace window '
        + 'running out is an ordinary outcome and expressing it as a rejection would make every '
        + 'caller catch on the expected path',
      fn: async () => {
        expect(
          await settleWithin({
            promise: wait(SHORT_MS,),
            ms: LONG_MS,
          },),
        ).toBe('settled',);

        expect(
          await settleWithin({
            promise: wait(SHORT_MS,),
            ms: SHORT_MS + LONG_MS,
          },),
        ).toBe('settled',);
      },
    },),

    it({
      name: 'REPORTS EXPIRY as expiry, so a caller that cares which side won can tell without '
        + 'inspecting an error',
      fn: async () => {
        expect(
          await settleWithin({
            promise: wait(LONG_MS,),
            ms: SHORT_MS,
          },),
        ).toBe('expired',);
      },
    },),

    it({
      name: 'LETS THE PROCESS EXIT once the work is done, which is the defect this exists for. '
        + 'The value returned looks correct whether or not the deadline was cleared, so the only '
        + 'thing that distinguishes them is whether the program stops',
      fn: async () => {
        /**
         A child that finishes 10ms of work under a 30 second deadline.
         */
        const child = await timeChild({
          source: `
            const { settleWithin, wait, } = await import(${JSON.stringify(BUILT,)});
            await settleWithin({ promise: wait(${String(SHORT_MS,)}), ms: ${String(LONG_MS,)}, },);
          `,
        },);

        expect(child.exited,).toBe(true,);
        expect(child.elapsedMs,).toBeLessThan(EXIT_BUDGET_MS,);
      },
    },),

    it({
      name: 'IS NOT THE SAME AS RACING wait, which is the positive control: the pattern this '
        + 'replaces returns the identical answer and does NOT let the process exit, so a test '
        + 'that only checked the answer would pass on the broken version',
      fn: async () => {
        /**
         The same program written the old way, which must hang.
         */
        const child = await timeChild({
          source: `
            const { wait, } = await import(${JSON.stringify(BUILT,)});
            await Promise.race([wait(${String(SHORT_MS,)}), wait(${String(LONG_MS,)}),],);
          `,
        },);

        expect(child.exited,).toBe(false,);
      },
    },),
  ],
},);
