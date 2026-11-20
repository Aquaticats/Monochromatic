/**
 The part of a run after the prompt is sent: watches the limits, delivers the inbox, and decides when the run is over.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  readdir,
  rename,
  unlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  FEED_PREVIEW_CHARACTERS,
  POLL_MILLISECONDS,
  STOP_FILE_NAME,
} from './driver-constants.ts';
import type {
  CommandAnswer,
  EndReason,
  RunProgress,
} from './driver-types.ts';
import {
  answerModelError,
  type RetryWaiting,
} from './answer-a-model-error.ts';
import {
  keepFailure,
  returnToInbox,
} from './an-inbox-message-pi-did-not-take.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { readOrdinaryFile, } from './read-ordinary-file.ts';
import {
  askAboutItsWorkOrReportAFailure,
  foundTheAgentAlreadyAwake,
  type WorkWaitedFor,
} from './kept-while-work-runs.ts';
import type { WatchRequest, } from './watch-request.ts';
import { mayEndIdle, } from './when-a-run-may-end.ts';

/**
 Claims one inbox entry by renaming it, so that a producer publishing a newer message under the same name cannot be deleted or
 kept in its place. The claim's own name starts with a dot, so no later look at the inbox takes it for a message.

 @param watch - watch request

 @param name - message's name

 @returns where the message now is
 */
async function claimMessage({
  watch,
  name,
}: {
  readonly watch: WatchRequest;
  readonly name: string
},): Promise<string> {
  /**
   Where the claim goes, under a name of its own that does not grow with the message's own, so any name a producer may publish
   stays claimable.
   */
  const claimed = join(
    watch.inbox,
    `.claimed-${randomUUID()}`,
  );
  await rename(
    join(
      watch.inbox,
      name,
    ),
    claimed,
  );
  return claimed;
}


/**
 Delivers one claimed inbox message. The claim goes once Pi has it; a refused or unanswered one is kept under a dot name, which
 the watch skips, so it is neither lost nor sent again.

 @param watch - watch request

 @param name - message's name

 @param claim - where the message now is
 */
async function deliverMessage({
  watch,
  name,
  claim,
}: {
  readonly watch: WatchRequest;
  readonly name: string;
  readonly claim: string
},): Promise<void> {
  /**
   The message, which must be an ordinary file: a named pipe would wait for a writer.
   */
  const text = await readOrdinaryFile(claim,);
  /**
   How the driver's own read of the feed expected the message to be taken: a steer while Pi works, a new prompt while it rests.
   */
  const sentAs = watch.record
    .isRunning() ? 'steer' : 'prompt';
  /**
   Pi's answer, waited for within the run's own deadline.

   The message is a prompt that names steering whichever state that read gave. Pi queues it into a turn that is running and takes it
   as a new turn while its agent rests, so a turn Pi starts between the read and this send takes it too (`prompt()` in
   `dist/core/agent-session.js` of `@earendil-works/pi-coding-agent` reads the field only while its agent run is active). A steer
   command sent instead would wait in a queue that a Pi which settled meanwhile never drains.
   */
  const answer = await watch.connection
    .send({
      command: {
        type: 'prompt',
        message: text,
        streamingBehavior: 'steer',
      },
      deadline: watch.deadline,
    },);
  if (takenByPi(answer,)) {
    try {
      await unlink(claim,);
    }
    catch (error) {
      watch.record
        .note({
          kind: 'problem',
          problem: `Pi took ${name}, but its file could not be removed: ${String(error,)}`,
        },);
    }
  }
  else if (foundTheAgentAlreadyAwake({ answer, },)) {
    // Pi said only that its agent is busy, never that the message is unacceptable, so the message is not kept aside as one Pi
    // refused: it goes back to the inbox and the next look sends it again.
    await returnToInbox({
      watch,
      name,
      claim,
    },);
  }
  else {
    await keepFailure({
      watch,
      name,
      claim,
    },);
  }
  watch.record
    .note({
      kind: 'inbox',
      file: name,
      sentAs,
      success: takenByPi(answer,),
      error: outcomeTextOf(answer,),
      preview: preview({
        value: text,
        length: FEED_PREVIEW_CHARACTERS.inboxMessage,
      },),
    },);
}

/**
 Tells whether Pi took the message.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered that it took it
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

 @returns the text, or nothing when nothing failed
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

/**
 Delivers one inbox entry, or keeps it aside when it cannot be read, such as a folder dropped there by mistake.

 @param watch - watch request

 @param name - entry's name
 */
async function deliverOrSetAside({
  watch,
  name,
}: {
  readonly watch: WatchRequest;
  readonly name: string
},): Promise<void> {
  try {
    /**
     Where the entry moved to when it was claimed.
     */
    const claim = await claimMessage({
      watch,
      name,
    },);
    try {
      await deliverMessage({
        watch,
        name,
        claim,
      },);
    }
    catch (error) {
      watch.record
        .note({
          kind: 'problem',
          problem: `could not deliver ${name}: ${String(error,)}`,
        },);
      try {
        await keepFailure({
          watch,
          name,
          claim,
        },);
      }
      catch (keepError) {
        // The message stays where the claim put it, under a dot name, so it is neither lost nor sent again.
        watch.record
          .note({
            kind: 'problem',
            problem: `could not keep ${name}: ${String(keepError,)}`,
          },);
      }
    }
  }
  catch (error) {
    watch.record
      .note({
        kind: 'problem',
        problem: `could not take ${name}: ${String(error,)}`,
      },);
  }
}

/**
 Looks once at every limit that could end a run.

 @param watch - watch request

 @param wallDeadline - when the run must be over

 @returns why the run is over, or that it goes on
 */
async function progressAgainstLimits(
  {
    watch,
    wallDeadline,
  }: {
    readonly watch: WatchRequest;
    readonly wallDeadline: number
  },
): Promise<RunProgress> {
  if (watch.connection
    .hasExited()) {
    return 'pi-exited';
  }
  if (Date.now() >= wallDeadline) {
    return 'wall-clock';
  }
  if ((await readdir(watch.options
    .runFolder,)).includes(STOP_FILE_NAME,)) {
    return 'stop-file';
  }
  return watch.record
    .turnCount()
    >= watch.options
      .turnCap ? 'turn-cap' : 'still-running';
}

/**
 Lists the messages ready to read. A name starting with a dot is still being written, is being delivered, or was refused.

 @param inbox - inbox folder

 @returns message file names, in order
 */
async function readyMessages(inbox: string,): Promise<readonly string[]> {
  return (await readdir(inbox,)).filter(function isReady(name,) {
    return !name.startsWith('.',);
  },)
    .toSorted();
}

/**
 Checks the limits and delivers the next inbox message, on every tick, until something ends the run. One message per tick keeps
 them in order, each after Pi has answered the one before. A message is waited for only as long as the run has left, so a
 stalled answer cannot keep the stop file or the wall clock from being seen.

 @param watch - watch request

 @returns why the run ended

 @example
 ```ts
 const endReason = await watchUntilEnded(watch,);
 ```
 */
export async function watchUntilEnded(watch: WatchRequest,): Promise<EndReason> {
  /**
   The record whose settles determine when an empty inbox can permit an idle end.
   */
  const { record, } = watch;
  /**
   Where the watch stands with the model error it last answered, so one error is answered by one attempt.
   */
  const retry: RetryWaiting = {};
  /**
   The work this run left running, once the driver has seen it and is waiting for it to end; nothing until then.
   */
  const waited: WorkWaitedFor = {};
  /* oxlint-disable eslint/no-await-in-loop -- a look at the limits, then one message and Pi's answer before the next look; two
     deliveries at once would reorder the inbox. See doc/lint/sequential-awaits.md. */
  for (;;) {
    // The wait is what keeps the driver up between looks: it is the only handle a run has while Pi says nothing.
    await wait(POLL_MILLISECONDS,);
    /**
     Where the run stands against its limits.
     */
    const progress = await progressAgainstLimits({
      watch,
      wallDeadline: watch.deadline,
    },);
    if (progress !== 'still-running') {
      // A run the wall clock ended while it waited for work it left running did not end with nothing to do, and its summary
      // must not read as one that did.
      if ((progress === 'wall-clock') && (waited.work !== undefined)) {
        watch.record
          .note({
            kind: 'problem',
            problem: `the run ended on its wall clock while it waited for the work it left running: ${waited.work
              .join(', ',)}`,
          },);
      }
      return progress;
    }
    /**
     Where the run stands with the model error Pi settled on, if it settled on one.
     */
    const afterModelError = await answerModelError({
      watch,
      retry,
    },);
    if (afterModelError !== 'still-running') {
      return afterModelError;
    }
    /**
     Settles the inbox scan can account for. A settle during the scan needs a new look for messages published before it.
     */
    const settledBeforeInbox = record.settledCount();
    /**
     The next message to deliver, if any. An inbox message waits out a retry that is about to go, so a prompt meant for a resting
     Pi cannot start a run the retry prompt then lands in the middle of.
     */
    const [nextMessage,] = (retry.owed === undefined) ? await readyMessages(watch.inbox,) : [];
    if (nextMessage !== undefined) {
      await deliverOrSetAside({
        watch,
        name: nextMessage,
      },);
    }
    // A run waiting out a delay before another attempt is not idle: it is about to answer the error it settled on.
    else if ((retry.owed === undefined)
      && (mayEndIdle({
        watch,
        ...(waited.wokenAtSettle === undefined ? {} : { wokenAtSettle: waited.wokenAtSettle, }),
      },))) {
      if (record.settledCount() !== settledBeforeInbox) {
        continue;
      }
      /**
       What the run left running, asked before it may end as a run that finished its turn: an agent that ended its turn while a
       build compiled is waiting for that build, so the run is kept, and sent back to it once it is over. A read of that work that
       fails is reported beside the end reason, which stays `idle`: the agent finished its turn either way.
       */
      const asked = await askAboutItsWorkOrReportAFailure({
        watch,
        waited,
      },);
      if (asked === 'refused') {
        // Pi would not take the message that sends the agent back to work it left, so the run cannot go on and must not read as
        // one that finished.
        return 'prompt-rejected';
      }
      if (asked !== 'nothing') {
        continue;
      }
      return 'idle';
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
