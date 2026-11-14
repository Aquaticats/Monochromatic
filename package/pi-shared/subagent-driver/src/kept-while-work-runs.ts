/**
 Keeping a run whose agent ended its turn while work it started still runs: the feed line that names what is waited for, the
 message that sends the agent back once that work is gone, and the line that says what Pi made of it.

 An agent that ends its turn while a build compiles is waiting for that build, not finished with its task; a driver that closed
 that session as `idle` lost the work of two runs on 2026-11-12. Briefs tell agents that ending a turn while work runs is right,
 so this is where the driver makes that true.

 @module
 */

import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import type { CommandAnswer, } from './driver-types.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { fieldsOf, } from './read-pi-fields.ts';
import type { WatchRequest, } from './watch-request.ts';
import { wakeMessageOf, } from './work-the-run-left-running.ts';

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

  /**
   The last failure a read of the run's work ended with, or nothing while no read has failed since a good one.

   The read is asked again on every look, so one failure that lasts is reported once rather than on each of them.
   */
  unreadable?: string;
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
   Work still runs, so the run is not over and the driver keeps it. Work whose read failed keeps the run just the same, because
   nothing about it is known.
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
 Asks what the run left running, and keeps a run whose work cannot be read rather than ending it as one that finished.

 By the time this is asked the agent has ended its turn and the run is only waiting out its own idle time, so a read that fails
 says nothing about whether the run did its work: ending the run `driver-failed` instead is what reported eleven runs of this
 machine as failures on 2026-11-13 whose reviews were whole and whose Pi had exited 0. The failure goes to the feed and the
 summary under its own name.

 The run is kept all the same, because `nothing` would end it as a run that finished its turn while work of its own may still run:
 the summary of 2026-11-14 recorded `ESRCH: no such process` under `workProblem` for the run
 `lock-shim-of-pi-takes-over-a-lock-whose-holder-is-gone-2`, which ended `idle` with its build unread. A run whose work is never
 readable is ended by its own wall clock, with its worktree kept.

 The one failure that does not keep the run is the driver's own feed having failed, which the summary already reports under
 `feedProblem` and which rewrites the run's end to `driver-failed`: keeping such a run would hold it to its wall clock to learn
 what the driver already knows.

 @param watch - watch request

 @param waited - where the driver stands with the work, which this fills and clears

 @returns what stands between the run and its end, which is work that is not known to have ended

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
    /**
     What the failed read said, which names the path and the code it failed with.
     */
    const failure = String(error,);
    // The read is asked on every look, so one failure that lasts is noted once instead of burying the feed under itself.
    if (waited.unreadable !== failure) {
      waited.unreadable = failure;
      watch.record
        .noteWorkProblem(failure,);
    }
    // A driver whose own feed failed has already lost the record of the run, and the summary reports that under `feedProblem`; the
    // run's end is rewritten to `driver-failed` for it, so keeping the run would only hold it to its wall clock. Every other failed
    // read keeps it, because nothing is known about the work it left.
    /**
     Whether the driver's own record of the run has already failed.
     */
    const feedFailed = watch.record
      .feedProblem()
      .failed;
    return feedFailed ? 'nothing' : 'kept';
  }
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
