/**
 Which run endings count as a run's own terms, and how a late failure changes the reason a run reports.

 @module
 */

import type {
  EndReason,
  RunFailure,
} from './driver-types.ts';

/**
 Reasons a run ends on its own terms.
 */
export const CLEAN_END_REASONS: ReadonlySet<EndReason> = new Set<EndReason>([
  'idle',
  'stop-file',
],);

/**
 The reason a run finally ends with, once the failures that only show at the end are known. A run whose Pi outlived every
 signal, whose last assistant message ended in a model error nothing recovered from, or whose feed could not be written did not
 end on its own terms, so none of those may pass for `idle`.

 @param endReason - why the run ended, as the watch found it

 @param stopped - whether Pi's leaving could be confirmed

 @param modelError - the error Pi settled on, when nothing after it recovered

 @param feedProblem - why the feed is incomplete, when writing it failed

 @returns the reason the summary and the exit status carry

 @example
 ```ts
 const reason = finalEndReason({
   endReason: 'idle',
   stopped: true,
   modelError: NO_RUN_FAILURE,
   feedProblem: NO_RUN_FAILURE,
 },);
 ```
 */
export function finalEndReason(
  {
    endReason,
    stopped,
    modelError,
    feedProblem,
  }: {
    readonly endReason: EndReason;
    readonly stopped: boolean;
    readonly modelError: RunFailure;
    readonly feedProblem: RunFailure;
  },
): EndReason {
  if (!CLEAN_END_REASONS.has(endReason,)) {
    return endReason;
  }
  if (!stopped) {
    return 'pi-not-stopped';
  }
  if (modelError.failed) {
    return 'model-error';
  }
  return feedProblem.failed ? 'driver-failed' : endReason;
}
