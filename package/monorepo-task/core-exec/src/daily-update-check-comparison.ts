/**
 Tells a check that the update broke from one that was already broken.

 @module
 */

import type {
  CheckComparison,
  CheckResult,
  CheckVerdict,
} from './daily-update-package-check-types.ts';

/**
 Key a check is stored under: its package and task.

 @param result - a result

 @returns the key

 @example
 ```ts
 resultKey(result,); // 'package/a/b:lint:clippy'
 ```
 */
export function resultKey(result: CheckResult,): string {
  return `${result.check
    .package}:${result.check
      .task}`;
}

/**
 Builds the comparison of one check with a verdict and the reason for it.

 @param after - the check's result after the update

 @param before - the check's result before the update, absent when there was none

 @param verdict - what the comparison says

 @param reason - why

 @returns the comparison

 @example
 ```ts
 verdictOf({ after, verdict: 'still-green', reason: 'passes before and after the update', },);
 ```
 */
export function verdictOf({
  after,
  before,
  verdict,
  reason,
}: {
  readonly after: CheckResult;
  readonly before?: CheckResult;
  readonly verdict: CheckVerdict;
  readonly reason: string;
},): CheckComparison {
  return {
    check: after.check,
    verdict,
    reason,
    ...before === undefined ? {} : { before, },
    after,
  };
}

/**
 Compares one check's result after the update with its result before.

 @param after - the result after the update

 @param before - the result before the update, absent when the check did not exist then

 @returns the comparison

 @example
 ```ts
 compareOne({ after, before, },).verdict; // 'newly-red' when it passed before and fails now
 ```
 */
export function compareOne({
  after,
  before,
}: {
  readonly after: CheckResult;
  readonly before?: CheckResult;
},): CheckComparison {
  if (before === undefined) {
    return verdictOf({
      after,
      verdict: 'unknown',
      reason: 'the check did not exist before the update, so there is nothing to compare with',
    },);
  }
  if ((before.status === 'timed-out') || (before.status === 'not-run')) {
    return verdictOf({
      after,
      before,
      verdict: 'unknown',
      reason: `before the update the check ${before.status}, so it is unknown whether it passed`,
    },);
  }
  if ((after.status === 'timed-out') || (after.status === 'not-run')) {
    return verdictOf({
      after,
      before,
      verdict: 'unknown',
      reason: `after the update the check ${after.status}, so it is unknown whether it passes`,
    },);
  }
  if ((before.status === 'passed') && (after.status === 'failed')) {
    return verdictOf({
      after,
      before,
      verdict: 'newly-red',
      reason: 'passed before the update and fails after it',
    },);
  }
  if ((before.status === 'failed') && (after.status === 'failed')) {
    return verdictOf({
      after,
      before,
      verdict: 'already-red',
      reason: 'failed before the update and fails after it',
    },);
  }
  if (before.status === 'failed') {
    return verdictOf({
      after,
      before,
      verdict: 'recovered',
      reason: 'failed before the update and passes after it',
    },);
  }
  return verdictOf({
    after,
    before,
    verdict: 'still-green',
    reason: 'passes before and after the update',
  },);
}

/**
 Compares the pass before the update with the pass after it.
 A check is newly red only when it passed before and failed after;
 a check that failed before is already red and never reported again;
 a check whose result on either side is not a clear pass or fail (timed out, not run, missing) is unknown.

 @param before - results before the update

 @param after - results after the update

 @returns one comparison per result after the update, in that order

 @example
 ```ts
 compareChecks({ before, after, },).filter(function isNew(entry,) { return entry.verdict === 'newly-red'; },);
 ```
 */
export function compareChecks({
  before,
  after,
}: {
  readonly before: readonly CheckResult[];
  readonly after: readonly CheckResult[];
},): readonly CheckComparison[] {
  /**
   Results before the update by key.
   */
  const earlier = new Map(before.map(function keyed(result,) {
    return [
      resultKey(result,),
      result,
    ] as const;
  },),);
  return after.map(function compareAfter(result,) {
    /**
     The same check before the update.
     */
    const prior = earlier.get(resultKey(result,),);
    return prior === undefined ? compareOne({ after: result, },) : compareOne({
      after: result,
      before: prior,
    },);
  },);
}
