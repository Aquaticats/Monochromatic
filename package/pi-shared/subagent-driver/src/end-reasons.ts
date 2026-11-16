/**
 Which run endings count as a run's own terms, how a late failure changes the reason a run reports, and why a prompt that never
 started a run ended it.

 @module
 */

import type {
  CommandAnswer,
  EndReason,
  PiExit,
  RunFailure,
} from './driver-types.ts';
import { didPiLeave, } from './did-pi-leave.ts';
import type { ShutDownOutcome, } from './shapes-of-the-connection.ts';

/**
 Reasons a run ends on its own terms.
 */
export const CLEAN_END_REASONS: ReadonlySet<string> = new Set<string>([
  'idle',
  'stop-file',
],);

/**
 The reason a run finally ends with, once the failures that only show at the end are known. A run whose Pi outlived every bound
 on ending it, whose last assistant message ended in a model error nothing recovered from, whose feed could not be written, or
 whose Pi left with a failing status did not end on its own terms, so none of those may pass for `idle`.

 A Pi whose exit was seen is never `pi-not-stopped`, whatever the ending's own bounds said: a process that left is not one those
 bounds ran out on. Eight of the 2,564 runs in `${HOME}/temp/agent/pi-runs` read on 2026-11-13 carried both that reason and Pi's
 exit, which is the contradiction this rule closes.

 @param endReason - why the run ended, as the watch found it

 @param stopped - whether Pi's leaving was confirmed by the time the run was put away

 @param modelError - the error Pi settled on, when nothing after it recovered

 @param feedProblem - why the feed is incomplete, when writing it failed

 @param piExit - how Pi ended

 @param sessionAtRest - why the session the run leaves still holds a value, when it does

 @returns the reason the summary and the exit status carry

 @example
 ```ts
 const reason = finalEndReason({
   endReason: 'idle',
   stopped: true,
   modelError: NO_RUN_FAILURE,
   feedProblem: NO_RUN_FAILURE,
   piExit: { code: 0, },
   sessionAtRest: NO_RUN_FAILURE,
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
    sessionAtRest,
  }: {
    readonly endReason: EndReason;
    readonly stopped: boolean;
    readonly modelError: RunFailure;
    readonly feedProblem: RunFailure;
    readonly piExit: PiExit;

    /**
     Why the session a run leaves behind still holds a value, when it does. A run whose session could not be masked did not finish
     clean whatever else it did, and the summary says where the value is.
     */
    readonly sessionAtRest: RunFailure;
  },
): EndReason {
  if (!CLEAN_END_REASONS.has(endReason,)) {
    return endReason;
  }
  if ((!stopped) && (!didPiLeave(piExit,))) {
    return 'pi-not-stopped';
  }
  if (modelError.failed) {
    return 'model-error';
  }
  if (feedProblem.failed) {
    return 'driver-failed';
  }
  if (sessionAtRest.failed) {
    return 'session-unmasked';
  }
  /**
   The status Pi left with, when it left with one rather than a signal.
   */
  const { code, } = piExit;
  // A run that ended on its own terms was not supposed to see Pi fail afterwards, such as while it was asked for statistics.
  return ((code === undefined) || (code === 0)) ? endReason : 'pi-exited';
}

/**
 Says whether a reason a summary names is one a run ends on its own terms with, which is what a run whose last message is worth
 reading must have.

 @param endReason - the reason a summary names, of any shape

 @returns whether that reason is one of {@link CLEAN_END_REASONS}

 @example
 ```ts
 const finished = endedOnItsOwnTerms('idle');
 ```
 */
export function endedOnItsOwnTerms(endReason: unknown,): boolean {
  return ((typeof endReason) === 'string') && CLEAN_END_REASONS.has(endReason,);
}

/**
 Why a run is over, when its first prompt never started a run.

 A wait that ran out of silence after a compaction Pi reported failing names the compaction: Pi refused nothing, it could not put
 its own session in order. Any other silence is a prompt Pi would not take. A wait the run's deadline ended is `wall-clock`,
 because the deadline is what ended it, and a wait Pi left is `pi-exited`.

 @param answer - how waiting for Pi's answer ended

 @param passedDeadline - whether the run's wall clock had passed when Pi refused the prompt itself

 @returns why the run ended

 @example
 ```ts
 const reason = endReasonOfPromptWait({ answer: { ending: 'silence', }, passedDeadline: false, },);
 ```
 */
export function endReasonOfPromptWait(
  {
    answer,
    passedDeadline,
  }: {
    readonly answer: CommandAnswer;
    readonly passedDeadline: boolean
  },
): EndReason {
  if (answer.ending === 'deadline') {
    return 'wall-clock';
  }
  if (answer.ending === 'pi-exited') {
    return 'pi-exited';
  }
  if ((answer.ending === 'silence') && (answer.compactionProblem !== undefined)) {
    return 'compaction-failed';
  }
  return passedDeadline ? 'wall-clock' : 'prompt-rejected';
}

/**
 What an ending took, as the run's record keeps it: the words the summary carries as `stopProblem`, and whether a bound of the
 ending ran out, which the feed reports as a problem of its own.
 */
export type EndingTaken = {
  /**
   What the summary keeps as `stopProblem`, or absent when Pi left as its input closed.
   */
  readonly problem?: string;

  /**
   Whether the ending's own bounds ran out before the driver saw Pi leave, which the feed reports as a problem.
   */
  readonly ranOutOfABound: boolean;
};

/**
 What ending Pi took, in the words the summary keeps as `stopProblem` and the feed reports as a problem.

 A run whose agent settled and whose Pi then needed a signal did the work it was given, so the ending says what it cost instead
 of turning the run into one that failed. Pi's own exit, and whether a signal ended it, is in the summary's `piExit`, so these
 words never restate it.

 @param stopped - whether Pi's leaving was known once the ending's stages were over

 @param signalsSent - the signals the driver had to send, in the order it sent them

 @param stageMilliseconds - how long each stage of the ending was given

 @param outlivedTheLastBound - whether the last stage ran out with Pi's leaving still unseen

 @param refusedSignal - why the operating system refused a signal, when it refused one

 @returns what the ending took

 @example
 ```ts
 const took = whatTheEndingTook({
   stopped: true,
   signalsSent: ['SIGTERM',],
   stageMilliseconds: 5000,
   outlivedTheLastBound: false,
 },);
 ```
 */
export function whatTheEndingTook(
  {
    stopped,
    signalsSent,
    stageMilliseconds,
    outlivedTheLastBound,
    refusedSignal,
  }: ShutDownOutcome,
): EndingTaken {
  /**
   The signals the driver had to send, in the words a reader meets them.
   */
  const signals = signalsSent.join(' and ',);
  /**
   The refused signal, as a reader has to know it: a signal the operating system refused is no answer that Pi went.
   */
  const refused = (refusedSignal === undefined) ? '' : `, and a signal had failed: ${refusedSignal}`;
  if (!stopped) {
    // A process the driver could not end names the bound it outlived, so a reader counts the ending's own time rather than
    // guessing at it.
    return {
      problem: `Pi was still running ${String(stageMilliseconds,)} ms after SIGKILL${refused}.`,
      ranOutOfABound: true,
    };
  }
  if (outlivedTheLastBound) {
    return {
      problem: `Pi stayed after its input closed${(signalsSent.length === 0) ? '' : `, the driver sent ${signals}`}, and the driver's last bound ran out before it saw Pi leave.`,
      ranOutOfABound: true,
    };
  }
  if (signalsSent.length === 0) {
    return { ranOutOfABound: false, };
  }
  return {
    problem: `Pi stayed after its input closed and left after the driver sent ${signals}.`,
    ranOutOfABound: false,
  };
}
