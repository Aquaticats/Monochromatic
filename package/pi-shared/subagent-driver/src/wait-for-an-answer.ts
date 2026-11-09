/**
 One command's wait for Pi's answer: the silence it allows, the deadline it cannot pass, and how a compaction Pi reports moves
 that silence.

 @module
 */

import { NO_RUN_FAILURE, } from './driver-constants.ts';
import type {
  CommandAnswer,
  CompactionOfPi,
  RunFailure,
} from './driver-types.ts';

/**
 How a wait for one command's answer is bounded, and what tells it Pi's session is being compacted.
 */
export type StartWaitingRequest = {
  /**
   Longest Pi may stay silent while answering. None of the time it reports a compaction counts, because Pi is at work then.
   */
  readonly answerMilliseconds: number;

  /**
   When the wait must be over whatever Pi is doing, as milliseconds since the epoch.
   */
  readonly deadline: number;

  /**
   The compaction Pi reports, read whenever a bound is due, so a compaction that begins or ends meanwhile moves the bounds.
   */
  readonly compactionOfPi: () => CompactionOfPi;
};

/**
 One command's wait for its answer.
 */
export type WaitingForAnAnswer = {
  /**
   Settles once the wait is over, with what ended it.
   */
  readonly answer: Promise<CommandAnswer>;

  /**
   Ends the wait with what came instead, once: Pi going ends every wait it holds, and a late record changes nothing.
   */
  readonly settle: (answer: CommandAnswer,) => void;

  /**
   Stops counting silence, because Pi reports a compaction, which is Pi at work rather than Pi silent.
   */
  readonly pauseSilence: () => void;

  /**
   Starts counting silence again once Pi's compaction is over, keeping what Pi said of it.
   */
  readonly resumeSilence: (compaction: RunFailure,) => void;
};

/**
 Starts waiting for one command's answer.

 @param answerMilliseconds - longest silence Pi may keep

 @param deadline - when the wait must be over whatever Pi reports

 @param compactionOfPi - what Pi reports of its own session, read whenever a bound is due

 @returns the wait, and the promise that settles when it is over

 @example
 ```ts
 const wait = startWaitingForAnAnswer({ answerMilliseconds: 30_000, deadline, compactionOfPi, },);
 ```
 */
export function startWaitingForAnAnswer(
  {
    answerMilliseconds,
    deadline,
    compactionOfPi,
  }: StartWaitingRequest,
): WaitingForAnAnswer {
  /**
   The answer, settled by whichever bound or record comes first.
   */
  const settled = Promise.withResolvers<CommandAnswer>();
  /**
   What changes while the wait runs: the silence it has served, the stretch it is serving now, what Pi said of a compaction
   that failed, whether it is over, and the bound that is due first.
   */
  const state: {
    silentMilliseconds: number;
    silentSince: number;
    compactionFailure: RunFailure;
    over: boolean;
    timer?: NodeJS.Timeout;
  } = {
    silentMilliseconds: 0,
    silentSince: Date.now(),
    compactionFailure: NO_RUN_FAILURE,
    over: false,
  };

  /**
   Stops the bound that is armed, if one is.
   */
  function clearBound(): void {
    if (state.timer !== undefined) {
      clearTimeout(state.timer,);
      delete state.timer;
    }
  }

  /**
   Ends the wait, once, with what ended it.

   @param answer - what Pi did, or what the driver's own bound said
   */
  function settle(answer: CommandAnswer,): void {
    if (state.over) {
      return;
    }
    state.over = true;
    clearBound();
    settled.resolve(answer,);
  }

  /**
   Silence served so far: none of the time Pi spent compacting counts.

   @returns the milliseconds of silence this wait has seen
   */
  function silence(): number {
    /**
     What Pi reports of its own session.
     */
    const compaction = compactionOfPi();
    return compaction.compacting
      ? state.silentMilliseconds
      : state.silentMilliseconds + (Date.now() - state.silentSince);
  }

  /**
   What the answer says of Pi's compaction as the wait ends: a failure Pi reported, or a compaction still running.

   @returns the problem in words, or the fact that Pi reported no compaction this wait saw
   */
  function compactionProblemNow(): RunFailure {
    /**
     What Pi reports of its own session.
     */
    const compaction = compactionOfPi();
    if (!compaction.compacting) {
      return state.compactionFailure;
    }
    return {
      failed: true,
      problem: `Pi was still compacting (${compaction.reason}) after ${String(Date.now() - compaction.since,)} ms`,
    };
  }

  /**
   Reports the wait as over because its deadline came, naming the compaction it waited on when one was under way.

   @param compactionProblem - what Pi reported of a compaction this wait saw
   */
  function settleAtDeadline(compactionProblem: RunFailure,): void {
    if (!compactionProblem.failed) {
      settle({
        ending: 'deadline',
        problem: 'no answer before the deadline',
      },);
      return;
    }
    settle({
      ending: 'deadline',
      problem: `no answer before the deadline; ${compactionProblem.problem}`,
      compactionProblem: compactionProblem.problem,
    },);
  }

  /**
   Reports the wait as over because Pi kept its silence.

   @param compactionFailure - what Pi said of a compaction this wait saw fail
   */
  function settleAtSilence(compactionFailure: RunFailure,): void {
    if (!compactionFailure.failed) {
      settle({
        ending: 'silence',
        problem: `no answer in ${String(answerMilliseconds,)} ms`,
      },);
      return;
    }
    settle({
      ending: 'silence',
      problem: `no answer in ${String(answerMilliseconds,)} ms`,
      compactionProblem: compactionFailure.problem,
    },);
  }

  /**
   Arms the bound due first and settles the wait when it comes. Every firing looks again, so a compaction that began or ended
   meanwhile moves the bound rather than being missed.
   */
  function arm(): void {
    clearBound();
    /**
     How long is left before the wait must be over whatever Pi does.
     */
    const untilDeadline = deadline - Date.now();
    if (untilDeadline <= 0) {
      settleAtDeadline(compactionProblemNow(),);
      return;
    }
    /**
     What Pi reports of its own session.
     */
    const compaction = compactionOfPi();
    if (compaction.compacting) {
      // Pi is compacting, so nothing it does is silence: only the deadline can end this wait until the compaction ends.
      state.timer = setTimeout(
        arm,
        untilDeadline,
      );
      return;
    }
    /**
     How long is left of the silence the caller allows.
     */
    const untilSilence = answerMilliseconds - silence();
    if (untilSilence <= 0) {
      settleAtSilence(state.compactionFailure,);
      return;
    }
    /**
     The bound that is due first: the rest of the silence the caller allows, or what the run has left.
     */
    const dueIn = Math.min(
      untilSilence,
      untilDeadline,
    );
    state.timer = setTimeout(
      arm,
      dueIn,
    );
  }

  /**
   Stops counting silence, because Pi began a compaction.
   */
  function pauseSilence(): void {
    state.silentMilliseconds += Date.now() - state.silentSince;
    arm();
  }

  /**
   Starts counting silence again once Pi's compaction is over.

   @param compaction - what Pi said of the compaction that ended, which a failure of the session is
   */
  function resumeSilence(compaction: RunFailure,): void {
    state.silentSince = Date.now();
    state.compactionFailure = compaction.failed
      ? compaction
      : state.compactionFailure;
    arm();
  }

  arm();
  return {
    answer: settled.promise,
    settle,
    pauseSilence,
    resumeSilence,
  };
}
