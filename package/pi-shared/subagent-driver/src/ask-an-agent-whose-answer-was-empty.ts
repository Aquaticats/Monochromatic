/**
 Asking an agent whose last answer said nothing to go on, or ending the run on that answer: the words of each ask, Pi's answer to
 it, and the bound that keeps the asks finite.

 A run of 2026-11-13 ended with an assistant message that held no text and no tool call, 61 seconds into its fifth turn
 (`pi-runs/five-library-controls-of-main-that-state-what-main-changed-are-true-again-1`): the driver closed it as `idle`, its
 summary read as a run that came back after saying it was about to start, and the place it held went to the next run while its
 report was never written. An empty answer can be a provider's or a model's hiccup that one more turn recovers, so the driver asks
 the agent, in its own session, to go on. The bound is what keeps an agent that answers with nothing from being asked without end.

 @module
 */

import {
  EMPTY_ANSWER_ASK_LIMIT,
} from './driver-constants.ts';
import type { WorkWaitedFor, } from './kept-while-work-runs.ts';
import {
  dispositionOf,
  outcomeTextOf,
  refusedByPi,
  takenByPi,
} from './what-pi-did-with-a-message.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 What asking an agent whose answer said nothing came to.
 */
export type EmptyAnswerAnswer =
  /**
   The last answer carried words, or no assistant message has come: there is nothing to ask about.
   */
  | 'nothing'
  /**
   The ask went and the run goes on until the agent's reply to it settles.
   */
    | 'asked'
  /**
   Every ask this run allows has gone, and the last answer still said nothing.
   */
    | 'used-up'
  /**
   Pi refused the ask, so the agent was never told its answer said nothing and cannot go on.
   */
    | 'refused'
  /**
   The run's own wall clock passed while the ask waited for its answer.
   */
    | 'wall-clock'
  /**
   Pi left while the ask waited for its answer.
   */
    | 'pi-exited';

/**
 The message the driver sends an agent whose last answer said nothing.

 It says only what happened and what the run needs: the answer held no text and no tool call, and the agent either goes on with
 its task or says that it has finished and gives its report. It names no count and blames neither the agent nor the provider,
 because which of them left the answer empty is not the driver's to decide.

 @returns the words of the ask

 @example
 ```ts
 const said = goOnMessage();
 ```
 */
export function goOnMessage(): string {
  return 'Your last answer was empty: it held no text and no tool call. Go on with your task, or say that you have finished and'
    + ' give your report.';
}

/**
 Asks an agent whose last answer said nothing to go on, when this run has an ask left, and says what stands between the run and
 its end.

 An ask is sent only after the driver has found nothing of the run's still running, so an agent that ended its turn with an empty
 answer while a build it started compiles is woken to that build rather than asked to repeat itself. A message Pi took into a
 turn is waited for like the wake that sends an agent back to work it left: the reply to it settles, and the silence before it is
 not the silence of a run that has nothing left.

 @param watch - watch request

 @param waited - where the driver stands with the messages it has sent, which this fills when a reply is owed

 @returns what asking came to

 @example
 ```ts
 const asked = await askAnAgentWhoseAnswerWasEmpty({ watch, waited, },);
 ```
 */
export async function askAnAgentWhoseAnswerWasEmpty({
  watch,
  waited,
}: {
  readonly watch: WatchRequest;
  readonly waited: WorkWaitedFor;
},): Promise<EmptyAnswerAnswer> {
  if (!watch.record
    .lastAnswerWasEmpty()) {
    return 'nothing';
  }
  /**
   How many times the driver has already asked the agent to go on.
   */
  const asked = watch.record
    .askedToGoOn();
  if (asked >= EMPTY_ANSWER_ASK_LIMIT) {
    return 'used-up';
  }
  /**
   The words the agent is sent.
   */
  const said = goOnMessage();
  /**
   How many settles Pi had when the message went, which its own reply is counted from.
   */
  const settledBefore = watch.record
    .settledCount();
  /**
   Pi's answer, waited for within the run's own deadline.
   */
  const answer = await watch.connection
    .send({
      command: {
        type: 'prompt',
        message: said,
      },
      deadline: watch.deadline,
    },);
  /**
   What Pi said it did with the message.
   */
  const disposition = dispositionOf(answer,);
  watch.record
    .countAskToGoOn({
      attempt: asked + 1,
      of: EMPTY_ANSWER_ASK_LIMIT,
      success: takenByPi(answer,),
      disposition,
      error: outcomeTextOf(answer,),
      said,
    },);
  if (disposition === 'handled') {
    // Pi handled the prompt without starting a turn, and no settle will come for one.
    watch.record
      .countHandledPrompt();
  }
  // A message Pi took into a turn is answered by that turn, and the run stays up until its reply settles.
  if (takenByPi(answer,) && (disposition !== 'handled')) {
    waited.wokenAtSettle = settledBefore;
  }
  if (answer.ending === 'deadline') {
    return 'wall-clock';
  }
  if (answer.ending === 'pi-exited') {
    return 'pi-exited';
  }
  if (refusedByPi(answer,)) {
    return 'refused';
  }
  return 'asked';
}
