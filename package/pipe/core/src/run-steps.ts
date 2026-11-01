/**
 Runtime validation helpers for pipe step slots.
 
 Split from `run.ts` so the execution module stays under the line-count
 budget while keeping validation named and independently documented.
 
 @module
 */

import {
  PipeStepGapError,
  PipeStepOverflowError,
} from './errors.ts';

/**
 Sentinel returned by `indexOf` when no missing step slot exists.
 */
const NO_STEP_GAP = -1;

/**
 Throws when a later step appears after a missing step.
 
 @param steps - Ordered step slots from `fn1` through `fn9`, inspected with array methods only so
 no `noUncheckedIndexedAccess` guard is needed per element.
 
 @throws {@link PipeStepGapError} when step keys are not contiguous.
 
 @example
 ```ts
 assertContiguousSteps([(input) => input, undefined]);
 ```
 */
export function assertContiguousSteps(steps: readonly unknown[],): void {
  /**
   First missing step slot, or `-1` when all slots are present.
   */
  const firstGap = steps.indexOf(undefined,);

  if (firstGap === NO_STEP_GAP)
    return;

  /**
   Whether any later slot is present after the first missing slot.
   */
  const hasLaterStep = steps
    .slice(firstGap + 1,)
    .some(function stepIsPresent(step,) {
      return step !== undefined;
    },);

  if (hasLaterStep)
    throw new PipeStepGapError(firstGap,);
}

/**
 Throws when an unsupported step beyond `fn9` is present.
 
 @param overflowStep - Possible unsupported `fn10` step; presence alone (not its value) signals
 overflow because the overloads cap arity at nine.
 
 @throws {@link PipeStepOverflowError} when `fn10` is present.
 
 @example
 ```ts
 assertNoOverflowStep();
 ```
 */
export function assertNoOverflowStep(overflowStep?: unknown,): void {
  if (overflowStep !== undefined)
    throw new PipeStepOverflowError();
}
