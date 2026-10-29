/**
 Polling for a condition that becomes true some time after an action.
 
 @example
 ```ts
 import { pollUntil, } from './poll.ts';
 
 const appeared = await pollUntil({ attempts: 10, intervalMs: 100, order: 'wait-then-check', check: isThere, },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

/**
 Whether each attempt looks first and waits after a miss, or waits first and then looks.
 */
export type PollOrder = 'check-then-wait' | 'wait-then-check';

/**
 Looks for a condition up to a fixed number of times, with a fixed pause between looks.
 
 @param attempts - how many looks at most
 
 @param intervalMs - pause in milliseconds between looks
 
 @param order - whether to look before or after each pause
 
 @param check - one look: true when the condition holds
 
 @returns whether the condition held on some look
 
 @example
 ```ts
 const gone = await pollUntil({ attempts: 25, intervalMs: 200, order: 'wait-then-check', check: isGone, },);
 ```
 */
export async function pollUntil(
  {
    attempts,
    intervalMs,
    order,
    check,
  }: {
    readonly attempts: number;
    readonly check: () => Promise<boolean>;
    readonly intervalMs: number;
    readonly order: PollOrder;
  },
): Promise<boolean> {
  for (let attempt = 0; attempt < attempts; attempt++) {
    if (order === 'wait-then-check')
      // oxlint-disable-next-line eslint/no-await-in-loop -- each look depends on the pause before it: the looks are sequential by design
      await wait(intervalMs,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- each look reads state that the previous pause let change
    if (await check())
      return true;
    if ((order === 'check-then-wait') && (attempt < (attempts
      - 1)))
      // oxlint-disable-next-line eslint/no-await-in-loop -- the pause separates two sequential looks
      await wait(intervalMs,);
  }
  return false;
}

/**
 Looks for a condition until a deadline passes, with a fixed pause between looks. The first look is immediate and the
 last look happens after the deadline, so a condition that holds at the deadline is not missed.
 
 @param timeoutMs - milliseconds after which no further look is started
 
 @param intervalMs - pause in milliseconds between looks
 
 @param check - one look: true when the condition holds
 
 @returns whether the condition held on some look
 
 @example
 ```ts
 const there = await pollUntilDeadline({ timeoutMs: 6_000, intervalMs: 40, check: isThere, },);
 ```
 */
export async function pollUntilDeadline(
  {
    timeoutMs,
    intervalMs,
    check,
  }: {
    readonly check: () => Promise<boolean>;
    readonly intervalMs: number;
    readonly timeoutMs: number;
  },
): Promise<boolean> {
  /**
   The moment after which no further look is started.
   */
  const deadline = Date.now() + timeoutMs;
  if (await check())
    return true;
  while (Date.now() <= deadline) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- each look depends on the pause before it: the looks are sequential by design
    await wait(intervalMs,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- each look reads state that the previous pause let change
    if (await check())
      return true;
  }
  return false;
}
