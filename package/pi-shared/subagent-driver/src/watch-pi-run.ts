/**
 The part of a run after the prompt is sent: watches the limits, delivers the inbox, and decides when the run is over.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  link,
  readdir,
  rename,
  unlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
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
import { preview, } from './previews-of-pi-messages.ts';
import { readOrdinaryFile, } from './read-ordinary-file.ts';
import { askAboutItsWork, } from './kept-while-work-runs.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 The code the operating system gave a failure, when it gave one.

 @param error - what was caught

 @returns the code, or nothing when the failure carries none
 */
function codeOf(error: unknown,): string {
  if (((typeof error) !== 'object') || (error === null)
    || (!('code' in error))) {
    return '';
  }
  return ((typeof error.code) === 'string') ? error.code : '';
}

/**
 Publishes a kept failure under one name, without ever replacing a file already there: a link fails when its name is taken, so
 the caller tries the next number.

 @param claim - where the message now is

 @param target - the name this attempt would keep it under

 @returns what this attempt achieved, so the caller knows whether to try another name
 */
async function publishFailure({
  claim,
  target,
}: {
  readonly claim: string;
  readonly target: string
},): Promise<'published' | 'name-taken' | 'kept-by-rename'> {
  try {
    await link(
      claim,
      target,
    );
    await unlink(claim,);
    return 'published';
  }
  catch (error) {
    /**
     What the operating system said about this name.
     */
    const code = codeOf(error,);
    if (code === 'EEXIST') {
      return 'name-taken';
    }
    if ((code === 'EPERM') || (code === 'EACCES')) {
      // A folder cannot be linked, and holding one of Pi's messages is not at stake in it.
      await rename(
        claim,
        target,
      );
      return 'kept-by-rename';
    }
    throw error;
  }
}

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
 Keeps a message that did not reach Pi, so it stays readable and is never sent again. Publication never replaces an earlier
 kept failure: a link fails when its name is taken, so the next number is tried instead.

 @param watch - watch request

 @param name - message's name

 @param claim - where the message now is
 */
async function keepFailure({
  watch,
  name,
  claim,
}: {
  readonly watch: WatchRequest;
  readonly name: string;
  readonly claim: string
},): Promise<void> {
  /**
   The most numbered names to try before giving up: there are always fewer kept failures than names in the inbox.
   */
  const attempts = (await readdir(watch.inbox,)).length + 2;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    /**
     The name this attempt would keep the failure under.
     */
    const target = join(
      watch.inbox,
      (attempt === 1) ? `.failed-${name}` : `.failed-${name}.${String(attempt,)}`,
    );
    /**
     What this attempt achieved.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one name after another until one is free, and a kept failure must never replace another; see doc/lint/sequential-awaits.md
    const outcome = await publishFailure({
      claim,
      target,
    },);
    if ((outcome === 'published') || (outcome === 'kept-by-rename')) {
      return;
    }
  }
  throw new Error(`every name for a kept failure of ${name} is taken`,);
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
   How the message reaches Pi: a steer while it works, a new prompt while it rests.
   */
  const sentAs = watch.record
    .isRunning() ? 'steer' : 'prompt';
  /**
   Pi's answer, waited for within the run's own deadline.
   */
  const answer = await watch.connection
    .send({
      command: {
        type: sentAs,
        message: text,
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
 Tells whether Pi has rested for long enough that the run is over.

 @param watch - watch request

 @returns whether the idle wait has passed
 */
function idleLongEnough(watch: WatchRequest,): boolean {
  return (!watch.record
    .isRunning())
    && (watch.record
      .settledCount()
      >= 1)
    && (watch.record
      .silentMilliseconds()
      > (watch.options
        .idleSeconds
        * MILLISECONDS_PER_SECOND));
}

/**
 Tells whether the run may end as one that came back with nothing to report: Pi has rested long enough, and no failure it settled
 on is still unanswered.

 A failure stays until a reply succeeds, so a prompt Pi only handled leaves the run ending on that failure rather than making it
 look like a run that finished its task with nothing to say.

 @param watch - watch request

 @returns whether nothing stands between the run and its end but the rest it has had
 */
function mayEndIdle(watch: WatchRequest,): boolean {
  /**
   The failure the run settled on, when nothing after it recovered.
   */
  const failure = watch.record
    .unrecoveredModelError();
  return (!failure.failed) && idleLongEnough(watch,);
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
   Where the watch stands with the model error it last answered, so one error is answered by one attempt.
   */
  const retry: RetryWaiting = {};
  /**
   The work this run left running, once the driver has seen it and is waiting for it to end; nothing until then.
   */
  const waited: { work?: readonly string[] } = {};
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
    else if ((retry.owed === undefined) && (mayEndIdle(watch,))) {
      /**
       What the run left running, asked before it may end as a run that finished its turn: an agent that ended its turn while a
       build compiled is waiting for that build, so the run is kept, and sent back to it once it is over.
       */
      const asked = await askAboutItsWork({
        watch,
        waited,
      },);
      if (asked !== 'nothing') {
        continue;
      }
      return 'idle';
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
