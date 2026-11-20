/**
 Keeping a run whose agent ended its turn while work it started still runs: the feed line that names what is waited for, the
 message that sends the agent back once that work is gone, and the line that says what Pi made of it.

 An agent that ends its turn while a build compiles is waiting for that build, not finished with its task; a driver that closed
 that session as `idle` lost the work of two runs on 2026-11-12. Briefs tell agents that ending a turn while work runs is right,
 so this is where the driver makes that true.

 That message can meet an agent that is already awake: Pi starts a turn of its own when a tool it ran in the background reports,
 so the end of the work and the driver's message arrive together. Pi answers a prompt that meets a running agent with its busy
 error unless the prompt names a streaming behaviour, and queues it into that turn when one is named; on 2026-11-19 the driver
 read the busy error as a refusal, ended the run `prompt-rejected`, and cut off the turn the agent had already started. The
 message now asks to be steered into the running turn, and the busy error is read as the agent being awake, never as a refusal of
 the message.

 @module
 */

import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import type { CommandAnswer, } from './driver-types.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { fieldsOf, } from './read-pi-fields.ts';
import type { WatchRequest, } from './watch-request.ts';
import { wakeMessageOf, } from './work-the-run-left-running.ts';

/**
 The start of the error Pi 1.5.2 answers a prompt with when its agent is already in a turn and the prompt named no streaming
 behaviour (`prompt()` in `dist/core/agent-session.js` of `@earendil-works/pi-coding-agent`): "Agent is already processing.
 Specify streamingBehavior ('steer' or 'followUp') to queue the message." It says the agent is busy, never that the message is
 unacceptable.

 The continuation that names `streamingBehavior` is what tells that answer from the agent library's own error, which opens with
 the same words and names neither a queue nor a turn (`agent.js` of `@earendil-works/pi-agent-core`, "Agent is already processing
 a prompt. Use steer() or followUp() to queue messages, or wait for completion."). Only the session's answer says a turn of the
 agent's own took the message, so only that one is read as the agent being awake.
 */
const AGENT_ALREADY_PROCESSING = 'Agent is already processing. Specify streamingBehavior';

/**
 Where the driver stands with the work a run left running.
 */
export type WorkWaitedFor = {
  /**
   The lines that named the work when the driver began to wait, or nothing while it waits for none.
   */
  work?: readonly string[];

  /**
   How many settles Pi had produced when the driver last sent the agent back to its work, or nothing while no message is owed
   a reply.

   A settle, or a prompt Pi handled without starting a turn, is counted: what this waits for is Pi's answer to that message, and
   the run has had neither since it was woken.
   */
  wokenAtSettle?: number;
};

/**
 Tells whether the run has answered the message that last woke it.

 A run whose agent was sent back to work it can now read is not a run that has finished its turn: the reply to that message, and
 the settle after it, are what the driver is up. Counting from the settle before the message went means a fast reply satisfies
 this at once, and no reply at all keeps the run up until its own wall clock ends it.

 @param wokenAtSettle - how many settles Pi had when that message went, or nothing while no message is owed a reply

 @param settledNow - how many settles Pi has produced

 @returns whether any message the driver sent the agent has been answered

 @example
 ```ts
 const free = answeredItsWake({ settledNow: 3, },);
 ```
 */
export function answeredItsWake({
  wokenAtSettle,
  settledNow,
}: {
  readonly wokenAtSettle?: number;
  readonly settledNow: number
},): boolean {
  return (wokenAtSettle === undefined) || (settledNow > wokenAtSettle);
}

/**
 What asking about the work answered.
 */
export type WorkAsked =
  /**
   Work still runs, so the run is not over and the driver keeps it.
   */
  | 'kept'
  /**
   The work ended, so the agent was sent back to read it and the run goes on.
   */
    | 'resumed'
  /**
   The work ended and Pi refused the message that sends the agent back, so the run cannot go on.
   */
    | 'refused'
  /**
   Nothing runs, so nothing stands between the run and its end but the rest it has had.
   */
    | 'nothing';

/**
 Asks what the run left running, and does what that answer needs.

 A run with nothing running is left to the caller to end; a run whose work still runs is kept, with one line naming that work the
 first time it is seen; and a run whose work has ended is sent a message naming what ended, so the agent goes on.

 @param watch - watch request

 @param waited - where the driver stands with the work, which this fills and clears

 @returns what stands between the run and its end

 @example
 ```ts
 const asked = await askAboutItsWork({ watch, waited, },);
 ```
 */
export async function askAboutItsWork({
  watch,
  waited,
}: {
  readonly watch: WatchRequest;
  readonly waited: WorkWaitedFor;
},): Promise<WorkAsked> {
  /**
   The lines that name the work still running.
   */
  const work = (watch.workLeftRunning === undefined) ? [] : await watch.workLeftRunning();
  if (work.length > 0) {
    if (waited.work === undefined) {
      // One line at the start of the wait names what is waited for, so a reader of the feed sees why the run is still up.
      waited.work = work;
      watch.record
        .note({
          kind: 'waiting',
          work,
          problem: `the agent ended its turn while work it started still runs, so the run is kept until it ends: ${work.join(', ',)}`,
        },);
    }
    return 'kept';
  }
  /**
   The work the driver was waiting for, when it was waiting for any.
   */
  const ended = waited.work;
  if (ended === undefined) {
    return 'nothing';
  }
  /**
   The message that sends the agent back to work it can now read.
   */
  const said = wakeMessageOf({ work: ended, },);
  /**
   How many settles Pi had when that message went, which its own reply is counted from.
   */
  const settledBefore = watch.record
    .settledCount();
  /**
   Pi's answer, waited for within the run's own deadline.

   The message asks to be steered, so a turn the agent started by itself takes it instead of refusing it: Pi delivers a steering
   message after the running assistant turn and before the next request, which is the agent reading what ended and going on. Pi
   ignores the field while its agent rests, so the message starts a turn as it always did.
   */
  const answer = await watch.connection
    .send({
      command: {
        type: 'prompt',
        message: said,
        streamingBehavior: 'steer',
      },
      deadline: watch.deadline,
    },);
  /**
   What Pi said it did with the message.
   */
  const disposition = dispositionOf(answer,);
  watch.record
    .note({
      kind: 'wake',
      work: ended,
      success: takenByPi(answer,),
      disposition,
      error: outcomeTextOf(answer,),
      preview: preview({
        value: said,
        length: FEED_PREVIEW_CHARACTERS.inboxMessage,
      },),
    },);
  delete waited.work;
  // A Pi that answered that its agent is already processing refused nothing: the agent is awake in a turn of its own, which is
  // the wake the work's end began, so the run stays up until that turn settles. Ending it here is what cut off a turn of the run
  // of 2026-11-19 and reported a run that came back as one Pi refused.
  if (foundTheAgentAlreadyAwake({ answer, },)) {
    watch.record
      .note({
        kind: 'problem',
        problem: 'Pi was already in a turn of its own when the message that names the ended work went, so the message was not '
          + 'taken: the run stays up until that turn settles, and the agent was not told by it that its work ended',
      },);
    waited.wokenAtSettle = settledBefore;
    return 'resumed';
  }
  // A message Pi refused woke nobody, so nothing is owed a reply and the run cannot go on: the agent was never told its work ended.
  if (refusedByIt({ answer, },)) {
    return 'refused';
  }
  // A message Pi took into a turn is answered by that turn, and the run stays up until its reply settles. A message Pi handled
  // without starting a turn is never answered by one, so nothing is owed a reply.
  if (takenByPi(answer,) && (disposition !== 'handled')) {
    waited.wokenAtSettle = settledBefore;
  }
  return 'resumed';
}

/**
 Asks what the run left running, and keeps a read that fails beside the end reason rather than in its place.

 By the time this is asked the agent has ended its turn and the run is only waiting out its own idle time, so a read that fails
 says nothing about whether the run did its work. The failure goes to the feed and the summary under its own name, and the run
 ends as the run it was. Ending it `driver-failed` instead is what reported eleven runs of this machine as failures on
 2026-11-13 whose reviews were whole and whose Pi had exited 0.

 @param watch - watch request

 @param waited - where the driver stands with the work, which this fills and clears

 @returns what stands between the run and its end, which is nothing when the work could not be read

 @example
 ```ts
 const asked = await askAboutItsWorkOrReportAFailure({ watch, waited, },);
 ```
 */
export async function askAboutItsWorkOrReportAFailure({
  watch,
  waited,
}: {
  readonly watch: WatchRequest;
  readonly waited: WorkWaitedFor;
},): Promise<WorkAsked> {
  try {
    return await askAboutItsWork({
      watch,
      waited,
    },);
  }
  catch (error) {
    watch.record
      .noteWorkProblem(String(error,),);
    return 'nothing';
  }
}

/**
 Tells whether Pi answered that its agent is already in a turn of its own, which refuses this message for no reason of its own.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi said its agent was already processing

 @example
 ```ts
 const busy = {
   ending: 'answered',
   record: {
     success: false,
     error: "Agent is already processing. Specify streamingBehavior ('steer' or 'followUp')"
       + ' to queue the message.',
   },
 } as const;
 const awake = foundTheAgentAlreadyAwake({ answer: busy, },);
 ```
 */
function foundTheAgentAlreadyAwake({ answer, }: { readonly answer: CommandAnswer },): boolean {
  if (answer.ending !== 'answered') {
    return false;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  if (record.success !== false) {
    return false;
  }
  /**
   What Pi said of the message.
   */
  const { error, } = record;
  return ((typeof error) === 'string') && error.startsWith(AGENT_ALREADY_PROCESSING,);
}

/**
 Tells whether Pi refused the message itself, which is no answer and no run started.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered with a refusal

 @example
 ```ts
 const refused = refusedByIt({ answer: { ending: 'answered', record: { success: false } }, },);
 ```
 */
function refusedByIt({ answer, }: { readonly answer: CommandAnswer },): boolean {
  if (answer.ending !== 'answered') {
    return false;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  return record.success === false;
}

/**
 What Pi said it did with a prompt.

 @param answer - how waiting for Pi's answer ended

 @returns what Pi said, or nothing when Pi answered nothing

 @example
 ```ts
 const disposition = dispositionOf({ ending: 'answered', record: { data: { disposition: 'handled' } }, },);
 ```
 */
function dispositionOf(answer: CommandAnswer,): unknown {
  if (answer.ending !== 'answered') {
    return undefined;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  /**
   What Pi did with the prompt.
   */
  const { disposition, } = fieldsOf(record.data,);
  return disposition;
}

/**
 Tells whether Pi took a message.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered that it handled it

 @example
 ```ts
 const taken = takenByPi({ ending: 'answered', record: { success: true }, },);
 ```
 */
function takenByPi(answer: CommandAnswer,): boolean {
  if (answer.ending !== 'answered') {
    return false;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  return record.success === true;
}

/**
 What the feed says of a message: Pi's own word for a refusal, or the bound that ended the wait.

 @param answer - how waiting for Pi's answer ended

 @returns what the feed keeps

 @example
 ```ts
 const said = outcomeTextOf({ ending: 'silence', problem: 'no answer' },);
 ```
 */
function outcomeTextOf(answer: CommandAnswer,): unknown {
  if (answer.ending !== 'answered') {
    return answer.problem;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  return record.error ?? null;
}
