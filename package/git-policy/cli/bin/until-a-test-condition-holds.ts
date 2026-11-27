/**
 A bounded wait a test uses to see a watcher's reading arrive.

 A watch reads a scope or a container once an interval, so a test that asserts on what it read has to wait for a reading
 and can never sleep for a guess: this wait looks again until the condition holds and fails at its window naming what it
 waited for (`AGENTS.md`: every wait has a deadline, and a bound that is the test's own is stated).

 This is a test helper, and nothing in the package calls it.
 */
import { wait as waitUntil, } from '@monochromatic-dev/async-time-core/ts';

/**
 Milliseconds between two looks at the condition, which is what a watcher's interval needs.
 */
const defaultPollMilliseconds = 100;

/**
 Milliseconds one wait gives a condition before it fails, which is far more than any fixture here needs.
 */
const defaultWindowMilliseconds = 10_000;

/**
 Wait, bounded, for a condition a test brings about, and fail naming what it waited for.

 @param condition - what must become true

 @param description - what the wait is for, which a failure names

 @param pollMilliseconds - how long between two looks

 @param windowMilliseconds - how long the wait may last

 @throws Error when the window passes without the condition holding

 @example
 ```ts
 await until({ condition: () => watch.lastRead() !== SCOPE_NOT_READ, description: 'the watch to read the scope' });
 ```
 */
export async function until({
  condition,
  description,
  pollMilliseconds = defaultPollMilliseconds,
  windowMilliseconds = defaultWindowMilliseconds,
}: {
  readonly condition: () => boolean;
  readonly description: string;
  readonly pollMilliseconds?: number;
  readonly windowMilliseconds?: number;
},): Promise<void> {
  /**
   When this wait gives up.
   */
  const deadline = Date.now() + windowMilliseconds;
  while (!condition()) {
    if (Date.now() > deadline)
      throw new Error(`waited ${String(windowMilliseconds)} ms for ${description}`,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- one look per pause is the wait itself
    await waitUntil(pollMilliseconds,);
  }
}
