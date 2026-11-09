/**
 The words the watch prints: how a length of time and a share of a sample read, and the one line a look that found nothing ends
 with. Keeping them in one module means a finding and the line that says why there is none cannot describe the same limit two
 different ways.

 @module
 */

import {
  MILLISECONDS_PER_MINUTE,
  OPEN_CALL_PERCENTILE,
  OVERDUE_PERCENTILE,
} from './overdue-constants.ts';
import type { FinishedRunEvidence, } from './overdue-limits.ts';

/**
 Reads a length of time as whole minutes, as a line prints it.

 @param milliseconds - the length of time

 @returns the length in minutes

 @example
 ```ts
 const shown = minutesOf(5_400_000,);
 ```
 */
export function minutesOf(milliseconds: number,): string {
  return String(Math.round(milliseconds / MILLISECONDS_PER_MINUTE,),);
}

/**
 Reads a share of a sample as a percentage, with a decimal only where it needs one.

 @param fraction - the share, where one is the whole

 @returns the share as text, such as `90%` or `99.9%`

 @example
 ```ts
 const shown = percentTextOf(0.999,);
 ```
 */
export function percentTextOf(fraction: number,): string {
  return `${String(
    Number((fraction * 100).toFixed(1,),),
  )}%`;
}

/**
 Describes the two limits the finished runs set.

 @param evidence - what the finished runs said

 @returns the limits in one clause, or why no limit could be read

 @example
 ```ts
 const shown = limitsTextOf(evidence,);
 ```
 */
export function limitsTextOf(evidence: FinishedRunEvidence,): string {
  /**
   What the finished runs say about how long a run lasts, when they say anything.
   */
  const runLimit = (evidence.overdueMilliseconds === undefined)
    ? ''
    : `${percentTextOf(OVERDUE_PERCENTILE,)} of ${String(evidence.finishedRuns,)} finished runs were done within ${minutesOf(evidence.overdueMilliseconds,)} min`;
  /**
   What they say about how long one call lasts, when they say anything.
   */
  const callLimit = (evidence.openCallMilliseconds === undefined)
    ? ''
    : `${percentTextOf(OPEN_CALL_PERCENTILE,)} of ${String(evidence.toolCalls,)} finished calls ended within ${minutesOf(evidence.openCallMilliseconds,)} min`;
  /**
   The limits there are.
   */
  const limits = [
    runLimit,
    callLimit,
  ].filter(function isRead(limit,) {
    return limit !== '';
  },);
  return (limits.length === 0)
    ? 'no finished run with a summary is in the runs folder, so nothing says how long work takes'
    : limits.join(', and ',);
}

/**
 Describes what the finished runs say, for a line that reports no run past a limit.

 @param evidence - what the finished runs said

 @returns what the look compared against, in one line

 @example
 ```ts
 const shown = comparedAgainstTextOf(evidence,);
 ```
 */
export function comparedAgainstTextOf(evidence: FinishedRunEvidence,): string {
  return `no Pi agent is past a limit: ${limitsTextOf(evidence,)}`;
}

/**
 Describes the end of a watch that saw no run past a limit before its deadline.

 @param evidence - what the finished runs said at the last look

 @returns the deadline line

 @example
 ```ts
 const shown = deadlineTextOf(evidence,);
 ```
 */
export function deadlineTextOf(evidence: FinishedRunEvidence,): string {
  return `the watch reached its deadline with no Pi agent past a limit: ${limitsTextOf(evidence,)}`;
}
