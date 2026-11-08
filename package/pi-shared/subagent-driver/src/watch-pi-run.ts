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
import type { ConnectionToPi, } from './connection-to-pi.ts';
import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  POLL_MILLISECONDS,
  STOP_FILE_NAME,
} from './driver-constants.ts';
import type {
  DriverOptions,
  EndReason,
  RecordOfPiRun,
  RunProgress,
} from './driver-types.ts';
import { millisecondsLeftUntil, } from './deadline.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { readOrdinaryFile, } from './read-ordinary-file.ts';

/**
 What watching a run needs.
 */
export type WatchRequest = {
  readonly connection: ConnectionToPi;
  readonly record: RecordOfPiRun;
  readonly options: DriverOptions;
  readonly inbox: string;
  readonly deadline: number;
};

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
   Where the claim goes, under a name of its own.
   */
  const claimed = join(
    watch.inbox,
    `.claimed-${randomUUID()}-${name}`,
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
 Finds a name for a kept failure that no earlier failure holds: the first failure of a message keeps its plain name, and a later
 one takes a number, so no failure replaces another.

 @param inbox - inbox folder

 @param name - message's name

 @returns the path the failure goes to
 */
async function freeFailurePath({
  inbox,
  name,
}: {
  readonly inbox: string;
  readonly name: string
},): Promise<string> {
  /**
   The names the inbox already holds.
   */
  const taken = new Set(await readdir(inbox,),);
  /**
   The plainest name for a failure of this message.
   */
  const first = join(
    inbox,
    `.failed-${name}`,
  );
  if (!taken.has(`.failed-${name}`,)) {
    return first;
  }
  /**
   The first number no kept failure of this message uses; there are always fewer of them than names in the inbox.
   */
  for (let attempt = 2; attempt <= (taken.size + 2); attempt += 1) {
    /**
     The numbered name of this attempt.
     */
    const numbered = `.failed-${name}.${String(attempt,)}`;
    if (!taken.has(numbered,)) {
      return join(
        inbox,
        numbered,
      );
    }
  }
  throw new Error(`every name for a kept failure of ${name} is taken`,);
}

/**
 Keeps a message that did not reach Pi, so it stays readable and is never sent again.

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
  await rename(
    claim,
    await freeFailurePath({
      inbox: watch.inbox,
      name,
    },),
  );
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
   Pi's answer, waited for only as long as the run has left.
   */
  const answer = await watch.connection
    .send({
      command: {
        type: sentAs,
        message: text,
      },
      answerMilliseconds: millisecondsLeftUntil(watch.deadline,),
    },);
  await (answer.success === true ? unlink(claim,) : keepFailure({
    watch,
    name,
    claim,
  },));
  watch.record
    .note({
      kind: 'inbox',
      file: name,
      sentAs,
      success: answer.success,
      error: answer.error ?? null,
      preview: preview({
        value: text,
        length: FEED_PREVIEW_CHARACTERS.inboxMessage,
      },),
    },);
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
      await keepFailure({
        watch,
        name,
        claim,
      },);
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
  if (watch.record
    .unrecoveredModelError()
    .failed) {
    return 'model-error';
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
      return progress;
    }
    /**
     The next message to deliver, if any.
     */
    const [nextMessage,] = await readyMessages(watch.inbox,);
    if (nextMessage !== undefined) {
      await deliverOrSetAside({
        watch,
        name: nextMessage,
      },);
    }
    else if (idleLongEnough(watch,)) {
      return 'idle';
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
