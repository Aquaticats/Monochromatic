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
 How long reads of a run's work may go on failing before the run ends as one whose leftover work could not be read.

 The owner decided on 2026-11-15 (`doc/handover/cli-git-rust-implementation.md`, "Owner decisions in force", "Answers of the evening,
 the long sitting"): "A shorter limit, then end as failed". Such a run ends marked as one whose leftover work could not be read, with
 its worktree kept, and is never ended as finished.

 The span is counted by the clock, from the first read that failed to the instant the span is asked again, and the watch asks it on
 every look of its own (`POLL_MILLISECONDS`), whatever the idle silence is: a wait for the idle silence would make the span as long as
 the run's `--idle-seconds`, which is 120 s in production. Measured here on 2026-11-15, one scan of this machine's process table, 1 176
 entries of which 474 could be read, took 42, 102 and 96 ms over three rounds, so a hundred looks of the scan cost a few seconds of the
 span. Every failure this machine's record holds was a single failed read followed by an end, so thirty seconds is far past any
 passing failure: eight runs, each with one `work-unreadable` line, each ended within 50 ms of it (`${HOME}/temp/agent/pi-runs`, read
 on 2026-11-15).
 */
export const UNREADABLE_WORK_MILLISECONDS = 30_000;

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

  /**
   When the run of failed reads now standing began, or nothing while the last read answered.

   A failure that lasts past `UNREADABLE_WORK_MILLISECONDS` ends the run as one whose leftover work could not be read; a read that
   answers clears this, so the next failure is measured from its own instant.
   */
  unreadableSince?: number;
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
   The work ended, so the agent was sent back to read it and the run goes on. A wake whose send failed counts the same way: it went
   out with an acceptance that is unknown, so it is never sent again, and the run goes on until a reply settles or its own wall clock
   ends it.
   */
    | 'resumed'
  /**
   The work ended and Pi refused the message that sends the agent back, so the run cannot go on.
   */
    | 'refused'
  /**
   Nothing runs, so nothing stands between the run and its end but the rest it has had.
   */
    | 'nothing'
  /**
   The reads of the run's work have failed for longer than the span, so the run ends as one whose leftover work could not be read.
   */
    | 'work-unreadable';

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
  const work = await workTheRunStillHasRunning({ watch, },);
  return await answerAboutItsWork({
    watch,
    waited,
    work,
  },);
}

/**
 Reads what the run has running now, which is the one read of this step that may fail.

 Kept apart from what the answer is made to do, so that a caller can catch this read alone: a message that fails to go out is no
 failed read, and a run told its work ended twice is a run that may do what follows twice.

 @param watch - watch request

 @returns the lines that name the work the run still has going, empty when it has none

 @throws whatever the read failed with, which the caller decides on

 @example
 ```ts
 const work = await workTheRunStillHasRunning({ watch, },);
 ```
 */
async function workTheRunStillHasRunning({ watch, }: { readonly watch: WatchRequest },): Promise<readonly string[]> {
  return (watch.workLeftRunning === undefined) ? [] : await watch.workLeftRunning();
}

/**
 How sending the message that wakes the agent ended.
 */
type WakeSent =
  | {
    /**
     Pi answered the message.
     */
    readonly sent: true;
    readonly answer: CommandAnswer;
  }
    | {
    /**
     Sending failed, and what it said.
     */
      readonly sent: false;
      readonly problem: string;
    };

/**
 Sends the message that sends the agent back to work it can read, and says what came of it rather than throwing.

 A send that throws is its own state: the write may have failed before Pi saw the prompt, or the answer may have been lost once Pi
 had begun the turn, and nothing here can tell those apart. So the failure is answered rather than thrown, and the caller treats a
 wake that may have gone out as one it must never send again.

 @param watch - watch request

 @param said - the message

 @returns Pi's answer, or what the send failed with

 @example
 ```ts
 const sent = await answerToTheWake({ watch, said, },);
 ```
 */
async function answerToTheWake({
  watch,
  said,
}: {
  readonly watch: WatchRequest;
  readonly said: string;
},): Promise<WakeSent> {
  try {
    return {
      sent: true,
      answer: await watch.connection
        .send({
          command: {
            type: 'prompt',
            message: said,
          },
          deadline: watch.deadline,
        },),
    };
  }
  catch (error) {
    return {
      sent: false,
      problem: String(error,),
    };
  }
}

/**
 What Pi made of the message that woke the agent, in one word a reader of the feed can act on.

 @param answer - how waiting for Pi's answer ended

 @returns what Pi made of it, which is unknown for an answer that never came

 @example
 ```ts
 const acceptance = acceptanceOf({ ending: 'silence', problem: 'no answer' },);
 ```
 */
function acceptanceOf(answer: CommandAnswer,): 'taken' | 'refused' | 'unknown' {
  if (answer.ending !== 'answered') {
    return 'unknown';
  }
  return (answer.record
    .success
    === false) ? 'refused' : 'taken';
}

/**
 Says what the run's work now is, and does what that needs: keeps the run, or sends the agent back to work it can read.

 @param watch - watch request

 @param waited - where the driver stands with the work, which this fills and clears

 @param work - the lines that name the work still running, as the last read of the process table found it

 @returns what stands between the run and its end

 @example
 ```ts
 const asked = await answerAboutItsWork({ watch, waited, work: [], },);
 ```
 */
async function answerAboutItsWork({
  watch,
  waited,
  work,
}: {
  readonly watch: WatchRequest;
  readonly waited: WorkWaitedFor;
  readonly work: readonly string[];
},): Promise<WorkAsked> {
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
   How sending that message ended.
   */
  const sent = await answerToTheWake({
    watch,
    said,
  },);
  if (!sent.sent) {
    // A send that failed says nothing about whether Pi took the prompt: the write may have failed before it, or the answer may have
    // been lost once Pi had begun the turn. So the wake is not sent again and is not counted as answered: the run stays up for a
    // reply that may yet come, and its own wall clock ends a run whose reply never does. The feed keeps one line saying so, which is
    // where a reader sees a wake whose acceptance is unknown rather than one Pi refused.
    watch.record
      .note({
        kind: 'wake',
        work: ended,
        acceptance: 'unknown',
        error: sent.problem,
        preview: preview({
          value: said,
          length: FEED_PREVIEW_CHARACTERS.inboxMessage,
        },),
      },);
    delete waited.work;
    waited.wokenAtSettle = settledBefore;
    return 'resumed';
  }
  /**
   Pi's answer, waited for within the run's own deadline.
   */
  const { answer, } = sent;
  /**
   What Pi said it did with the message.
   */
  const disposition = dispositionOf(answer,);
  watch.record
    .note({
      kind: 'wake',
      work: ended,
      acceptance: acceptanceOf(answer,),
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
  // Only an answer that Pi handled without starting a turn owes no reply: it will never be answered by one. Everything else leaves
  // the run up for a turn that may still settle, because an answer that never came says nothing about whether Pi took the prompt.
  if (!(takenByPi(answer,) && (disposition === 'handled'))) {
    waited.wokenAtSettle = settledBefore;
  }
  return 'resumed';
}

/**
 What reading the run's work answered.
 */
type WorkRead =
  | {
    /**
     The read answered.
     */
    readonly read: true;
    readonly work: readonly string[];
  }
    | {
    /**
     The read failed, and the driver has already said what with in the feed and the summary.
     */
      readonly read: false;
    };

/**
 Reads what the run has running now, reporting a read that fails rather than throwing it on.

 @param watch - watch request

 @param waited - where the driver stands with the work, which keeps the last failure it reported, and when it began

 @param now - where the driver reads the time, which a test replaces so a span is measured rather than waited out

 @returns what the read found, or that it could not be read

 @example
 ```ts
 const read = await workTheRunStillHasRunningOrReportAFailure({ watch, waited, now: Date.now, },);
 ```
 */
async function workTheRunStillHasRunningOrReportAFailure({
  watch,
  waited,
  now,
}: {
  readonly watch: WatchRequest;
  readonly waited: WorkWaitedFor;
  readonly now: () => number;
},): Promise<WorkRead> {
  try {
    return {
      read: true,
      work: await workTheRunStillHasRunning({ watch, },),
    };
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
    // The span is measured from the first look that failed since the last one that answered.
    waited.unreadableSince ??= now();
    return { read: false, };
  }
}

/**
 Asks whether the run's reads of what it left running have failed for longer than the span.

 The span is counted by the clock, from the first look that failed since the last one that answered, so a run's idle silence cannot
 stretch it. The watch asks this on every look of its own while a failure stands.

 @param waited - where the driver stands with the work, which keeps when its reads began to fail

 @param now - where the driver reads the time

 @param spanMilliseconds - how long reads may fail before the run ends, which a test shortens

 @returns whether the failed reads have lasted past the span

 @example
 ```ts
 const over = theReadsHaveFailedForTooLong({ waited, now: Date.now, spanMilliseconds: UNREADABLE_WORK_MILLISECONDS, },);
 ```
 */
export function theReadsHaveFailedForTooLong({
  waited,
  now = Date.now,
  spanMilliseconds = UNREADABLE_WORK_MILLISECONDS,
}: {
  readonly waited: WorkWaitedFor;
  readonly now?: () => number;
  readonly spanMilliseconds?: number;
},): boolean {
  /**
   How long the reads now standing have failed for, or nothing while none has failed since a read that answered.
   */
  const since = waited.unreadableSince;
  return (since !== undefined) && ((now() - since) >= spanMilliseconds);
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
 what the driver already knows. Only the read is caught here: a message that fails to go out is no failed read, and it is answered
 where it is sent.

 @param watch - watch request

 @param waited - where the driver stands with the work, which this fills and clears

 @param now - where the driver reads the time, which a test replaces so a span is measured rather than waited out

 @param spanMilliseconds - how long reads may fail before the run ends, which a test shortens

 @returns what stands between the run and its end, which is work that is not known to have ended

 @example
 ```ts
 const asked = await askAboutItsWorkOrReportAFailure({ watch, waited, },);
 ```
 */
export async function askAboutItsWorkOrReportAFailure({
  watch,
  waited,
  now = Date.now,
  spanMilliseconds = UNREADABLE_WORK_MILLISECONDS,
}: {
  readonly watch: WatchRequest;
  readonly waited: WorkWaitedFor;
  readonly now?: () => number;
  readonly spanMilliseconds?: number;
},): Promise<WorkAsked> {
  /**
   What the read of the run's work found.
   */
  const read = await workTheRunStillHasRunningOrReportAFailure({
    watch,
    waited,
    now,
  },);
  if (!read.read) {
    // A driver whose own feed failed has already lost the record of the run, and the summary reports that under `feedProblem`; the
    // run's end is rewritten to `driver-failed` for it, so keeping the run would only hold it to its wall clock. Every other failed
    // read keeps it, because nothing is known about the work it left.
    /**
     Whether the driver's own record of the run has already failed.
     */
    const feedFailed = watch.record
      .feedProblem()
      .failed;
    if (feedFailed) {
      return 'nothing';
    }
    if (!theReadsHaveFailedForTooLong({
      waited,
      now,
      spanMilliseconds,
    },)) {
      return 'kept';
    }
    // A run whose work cannot be read for that long ends as one whose leftover work could not be read, with its worktree kept: it is
    // never ended as finished, and it does not hold a place in the pool until its wall clock either. The work the driver knew of goes
    // into the summary, because the summary is what a reader of the run sees, and into one feed line beside the failure.
    /**
     The work the driver knew of, which it keeps the worktree for, or nothing when it knew of none.
     */
    const known = waited.work ?? [];
    watch.record
      .noteWorkNotRead(known,);
    watch.record
      .note({
        kind: 'problem',
        problem: `the run ended after ${String(spanMilliseconds,)} ms of reads that could not say what work it left running; its `
          + `worktree is kept, and that work may still run: ${
            (known.length === 0) ? 'the driver knew of none' : known.join(', ',)
          }. The last read failed with: ${String(waited.unreadable,)}`,
      },);
    return 'work-unreadable';
  }
  // A read that succeeded is what a later failure is a change from: a failure that repeats one the driver has already reported is
  // news again once a good read has stood between them, so the note of the last failure is cleared by every good read, and so is the
  // instant its span is measured from.
  delete waited.unreadable;
  delete waited.unreadableSince;
  return await answerAboutItsWork({
    watch,
    waited,
    work: read.work,
  },);
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
