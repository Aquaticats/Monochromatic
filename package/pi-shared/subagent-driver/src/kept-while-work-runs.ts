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
};

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
  watch.record
    .note({
      kind: 'wake',
      work: ended,
      success: takenByPi(answer,),
      error: outcomeTextOf(answer,),
      preview: preview({
        value: said,
        length: FEED_PREVIEW_CHARACTERS.inboxMessage,
      },),
    },);
  delete waited.work;
  return 'resumed';
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
