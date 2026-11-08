/**
 Which run endings count as a run's own terms, and how a late failure changes the reason a run reports.

 @module
 */

import type {
  EndReason,
  PiExit,
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
 signal, whose last assistant message ended in a model error nothing recovered from, whose feed could not be written, or whose
 Pi left with a failing status did not end on its own terms, so none of those may pass for `idle`.

 @param endReason - why the run ended, as the watch found it

 @param stopped - whether Pi's leaving could be confirmed

 @param modelError - the error Pi settled on, when nothing after it recovered

 @param feedProblem - why the feed is incomplete, when writing it failed

 @param piExit - how Pi ended

 @returns the reason the summary and the exit status carry

 @example
 ```ts
 const reason = finalEndReason({
   endReason: 'idle',
   stopped: true,
   modelError: NO_RUN_FAILURE,
   feedProblem: NO_RUN_FAILURE,
   piExit: { code: 0, },
 },);
 ```
 */
export function finalEndReason(
  {
    endReason,
    stopped,
    modelError,
    feedProblem,
    piExit,
  }: {
    readonly endReason: EndReason;
    readonly stopped: boolean;
    readonly modelError: RunFailure;
    readonly feedProblem: RunFailure;
    readonly piExit: PiExit;
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
  if (feedProblem.failed) {
    return 'driver-failed';
  }
  /**
   The status Pi left with, when it left with one rather than a signal.
   */
  const { code, } = piExit;
  // A run that ended on its own terms was not supposed to see Pi fail afterwards, such as while it was asked for statistics.
  return ((code === undefined) || (code === 0)) ? endReason : 'pi-exited';
}
