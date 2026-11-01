/**
 Tests for the polling helpers: attempt counts, ordering and the deadline.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pollUntil,
  pollUntilDeadline,
} from './poll.ts';

/**
 Builds a check that fails a number of times, then holds, and counts its calls.
 
 @param failures - how many calls fail before the first success
 
 @returns the check and the call counter
 */
function flaky(failures: number,): { readonly calls: { count: number; }; readonly check: () => Promise<boolean>; } {
  const calls = { count: 0, };
  return {
    calls,
    check: async () => {
      calls.count++;
      return calls.count > failures;
    },
  };
}

await describe({
  name: pollUntil.name,
  children: [
    it({
      name: 'returns true as soon as a look succeeds, check first',
      fn: async ({ expect, }) => {
        const { calls, check, } = flaky(2,);
        expect(await pollUntil({ attempts: 5, intervalMs: 1, order: 'check-then-wait', check, },),).toBe(true,);
        expect(calls.count,).toBe(3,);
      },
    },),
    it({
      name: 'returns false after the last attempt when no look succeeds',
      fn: async ({ expect, }) => {
        const { calls, check, } = flaky(100,);
        expect(await pollUntil({ attempts: 4, intervalMs: 1, order: 'check-then-wait', check, },),).toBe(false,);
        expect(calls.count,).toBe(4,);
      },
    },),
    it({
      name: 'waits before the first look when the order is wait-then-check',
      fn: async ({ expect, }) => {
        const { check, } = flaky(0,);
        const started = performance.now();
        expect(await pollUntil({ attempts: 2, intervalMs: 30, order: 'wait-then-check', check, },),).toBe(true,);
        expect(performance.now() - started,).toBeGreaterThanOrEqual(25,);
      },
    },),
    it({
      name: 'makes no look when no attempt is allowed',
      fn: async ({ expect, }) => {
        const { calls, check, } = flaky(0,);
        expect(await pollUntil({ attempts: 0, intervalMs: 1, order: 'wait-then-check', check, },),).toBe(false,);
        expect(calls.count,).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: pollUntilDeadline.name,
  children: [
    it({
      name: 'looks at once and returns true when the condition already holds',
      fn: async ({ expect, }) => {
        const { calls, check, } = flaky(0,);
        expect(await pollUntilDeadline({ timeoutMs: 1_000, intervalMs: 1, check, },),).toBe(true,);
        expect(calls.count,).toBe(1,);
      },
    },),
    it({
      name: 'keeps looking until the condition holds',
      fn: async ({ expect, }) => {
        const { calls, check, } = flaky(3,);
        expect(await pollUntilDeadline({ timeoutMs: 1_000, intervalMs: 1, check, },),).toBe(true,);
        expect(calls.count,).toBe(4,);
      },
    },),
    it({
      name: 'gives up after the deadline',
      fn: async ({ expect, }) => {
        const { check, } = flaky(1_000_000,);
        const started = performance.now();
        expect(await pollUntilDeadline({ timeoutMs: 60, intervalMs: 5, check, },),).toBe(false,);
        expect(performance.now() - started,).toBeGreaterThanOrEqual(55,);
      },
    },),
  ],
},);
