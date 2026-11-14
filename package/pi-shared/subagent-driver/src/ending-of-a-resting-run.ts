/**
 The ending of a run whose agent rests: what the run left running, what its last answer said, and which end those two give it.

 The watch decides a run may end as one whose agent finished its turn, and everything after that decision awaits: the read of the
 work the run left, and the ask of an agent whose last answer said nothing. The decisions are taken again on the state those
 awaits leave, because a turn Pi began in that window is a Pi at work, and a run is neither ended `idle` nor asked to go on while
 its agent works. The review of this change reproduced an end `empty-answer` reached that way, with its turn thrown away, by
 scripting a Pi that began a turn during the read.

 @module
 */

import { askAnAgentWhoseAnswerWasEmpty, } from './ask-an-agent-whose-answer-was-empty.ts';
import type { RunProgress, } from './driver-types.ts';
import {
  askAboutItsWorkOrReportAFailure,
  type WorkWaitedFor,
} from './kept-while-work-runs.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 Takes a resting run from the decision that it may end to the end it earns.

 A run with work of its own still running is kept, and sent back to that work once it ends; a run whose agent began a turn while
 the work was being read goes on with that turn; and a run with nothing running ends as `idle`, on an empty last answer, or on
 whichever ending the ask met.

 @param watch - watch request

 @param waited - where the driver stands with the work it waits for and the messages it has sent

 @returns why the run is over, or that it goes on

 @example
 ```ts
 const ending = await endingOfARestingRun({ watch, waited, },);
 ```
 */
export async function endingOfARestingRun({
  watch,
  waited,
}: {
  readonly watch: WatchRequest;
  readonly waited: WorkWaitedFor;
},): Promise<RunProgress> {
  /**
   What the run left running, and what asking about it came to. A read of that work that fails is reported beside the ending.
   */
  const asked = await askAboutItsWorkOrReportAFailure({
    watch,
    waited,
  },);
  if (asked === 'refused') {
    // Pi would not take the message that sends the agent back to work it left, so the run cannot go on and must not read as one
    // that finished.
    return 'prompt-rejected';
  }
  if (asked !== 'nothing') {
    return 'still-running';
  }
  /**
   Whether Pi still rests with no model error standing: the read above awaited, and Pi may have begun a turn in that window, as a
   steer delivered earlier fires after the settle that opened it, and such a turn may have settled on a model error before the read
   returned. Both endings below turn on a Pi at rest with no error the loop still owes an attempt.

   Only the rest and the error are read again, never the silence the watch decided on: a read that fails writes its own line to the
   feed, and that writing restarts the silence, so a run whose work cannot be read would never end while it kept failing. Its agent
   did finish its turn, which is what the silence was there to see.
   */
  const stillRests = !watch.record
    .isRunning();
  /**
   The error Pi settled on, when the turn the read saw ended in one and nothing after it recovered.
   */
  const failure = watch.record
    .unrecoveredModelError();
  if ((!stillRests) || failure.failed) {
    return 'still-running';
  }
  /**
   What the agent's last answer came to, when it came to nothing: a run that read an empty answer as its end gave its place to the
   next run with no report written, which is what one of 2026-11-13 did. Asking goes only after the work above, so an agent that
   ended its turn with an empty answer while a build it started compiles is woken to that build, never asked to repeat itself.
   */
  const answeredNothing = await askAnAgentWhoseAnswerWasEmpty({
    watch,
    waited,
  },);
  if ((answeredNothing === 'asked') || (answeredNothing === 'still-running')) {
    return 'still-running';
  }
  if (answeredNothing === 'nothing') {
    return 'idle';
  }
  if ((answeredNothing === 'wall-clock') || (answeredNothing === 'pi-exited')) {
    return answeredNothing;
  }
  // The asks are used up, or Pi refused the one that was left, and the agent's last answer still says nothing: the run ends on
  // that answer, which is neither a run that came back nor a failure of the driver.
  return 'empty-answer';
}
