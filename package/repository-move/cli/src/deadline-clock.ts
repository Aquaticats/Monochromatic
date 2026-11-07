/**
 The run's total deadline, next to the per-transfer one.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './constants.ts';

/**
 A started clock for the whole run.
 */
export type DeadlineClock = {
  /**
   Seconds left before the total deadline; zero or negative when it has passed.
   */
  readonly remainingSeconds: () => number;

  /**
   The total the clock was started with.
   */
  readonly totalSeconds: number;
};

/**
 Starts a clock for the whole run.

 @param totalSeconds - seconds the run may take
 
 @param now - milliseconds clock, injectable for tests
 
 @returns the clock

 @example
 ```ts
 import { startDeadlineClock, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = startDeadlineClock({ totalSeconds, now, },);
 ```
 */
export function startDeadlineClock(
  {
    totalSeconds,
    now = Date.now,
  }: {
    readonly totalSeconds: number;
    readonly now?: () => number;
  },
): DeadlineClock {
  /**
   When the step started, in milliseconds since the epoch.
   */
  const started = now();

  /**
   Seconds left of the total.

   @returns seconds, possibly negative
   */
  function remainingSeconds(): number {
    return totalSeconds - ((now() - started)
      / MILLISECONDS_PER_SECOND);
  }

  return {
    totalSeconds,
    remainingSeconds,
  };
}

/**
 Picks the deadline of one step: the per-step value, or what remains of the total when that is smaller.

 @param stepSeconds - the per-step deadline
 
 @param clock - the run's clock
 
 @returns whole seconds, at least one

 @example
 ```ts
 import { boundedSeconds, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = boundedSeconds({ stepSeconds, clock, },);
 ```
 */
export function boundedSeconds(
  {
    stepSeconds,
    clock,
  }: {
    readonly stepSeconds: number;
    readonly clock: DeadlineClock;
  },
): number {
  return Math.max(
    1,
    Math.ceil(Math.min(
      stepSeconds,
      clock.remainingSeconds(),
    ),),
  );
}
