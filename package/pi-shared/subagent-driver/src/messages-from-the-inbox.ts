/**
 Handing a message from the inbox to Pi: the claim that keeps two ticks from taking one message, the send, and what is done with a
 message Pi did not take.

 A message is taken when Pi says it has it, and kept aside under a dot name when Pi refuses it while it rests, or leaves it
 unanswered. A refusal Pi gave while it had begun a turn is neither: Pi refuses every prompt handed to it at work and takes the
 same message once it rests, so the message stays claimed and goes again once Pi rests, within the run's deadline. Every message
 Pi never took is named in the summary by the name it was published under, so whoever sent it learns that it was not read.

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
import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { readOrdinaryFile, } from './read-ordinary-file.ts';
import type { WatchRequest, } from './watch-request.ts';
import {
  messageToPiCameTo,
  outcomeTextOf,
} from './what-pi-did-with-a-message.ts';

/**
 A message the driver holds: it was claimed out of the inbox and Pi has not taken it.
 */
export type HeldMessage = {
  /**
   The name the message was published under, which is the name the summary and the feed carry.
   */
  readonly name: string;

  /**
   Where the claim moved it, under a dot name no later look at the inbox takes for a message.
   */
  readonly claim: string;
};

/**
 What delivering one message came to.
 */
export type DeliveryOfAMessage =
  /**
   Pi took it, and its file is gone.
   */
  | { readonly ending: 'taken' }
  /**
   Pi refused it because it had begun a turn, so the driver holds it and sends it again once Pi rests.
   */
    | { readonly ending: 'at-work'; readonly held: HeldMessage }
  /**
   It was kept aside under a dot name, and named in the summary as a message Pi never took.
   */
    | { readonly ending: 'not-taken' };

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
 Publishes a kept message under one name, without ever replacing a file already there: a link fails when its name is taken, so the
 caller tries the next number.

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
 Keeps a message Pi never took, and names it in the summary. A message whose file cannot be kept is still named: the summary is
 what a reader of the run has, and a message that is neither with Pi nor under its name must not read as delivered.

 @param watch - watch request

 @param held - the message and where its claim is
 */
async function noteMessageNotTaken({
  watch,
  held,
}: {
  readonly watch: WatchRequest;
  readonly held: HeldMessage
},): Promise<void> {
  try {
    await keepFailure({
      watch,
      name: held.name,
      claim: held.claim,
    },);
  }
  catch (error) {
    watch.record
      .note({
        kind: 'problem',
        problem: `could not keep ${held.name}: ${String(error,)}`,
      },);
  }
  watch.record
    .noteMessageNotTaken(held.name,);
}

/**
 Delivers one message the driver has claimed, notes it on the feed, and lays it down as Pi left it.

 @param watch - watch request

 @param held - the message and where its claim is

 @returns what the delivery came to
 */
async function deliverClaimed({
  watch,
  held,
}: {
  readonly watch: WatchRequest;
  readonly held: HeldMessage
},): Promise<DeliveryOfAMessage> {
  /**
   The message, which must be an ordinary file: a named pipe would wait for a writer.
   */
  const text = await readOrdinaryFile(held.claim,);
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
  /**
   Whether Pi was working when it answered the message.
   */
  const piWorks = watch.record
    .isRunning();
  /**
   What the message came to, read from the state Pi was left in.
   */
  const cameTo = messageToPiCameTo({
    answer,
    piWorks,
  },);
  watch.record
    .note({
      kind: 'inbox',
      file: held.name,
      sentAs,
      success: cameTo.ending === 'taken',
      error: outcomeTextOf(answer,),
      preview: preview({
        value: text,
        length: FEED_PREVIEW_CHARACTERS.inboxMessage,
      },),
    },);
  if (cameTo.ending === 'taken') {
    try {
      await unlink(held.claim,);
    }
    catch (error) {
      watch.record
        .note({
          kind: 'problem',
          problem: `Pi took ${held.name}, but its file could not be removed: ${String(error,)}`,
        },);
    }
    return { ending: 'taken', };
  }
  if (cameTo.ending === 'at-work') {
    // Pi began a turn as the message arrived, so it refused a prompt handed to it at work. That is no answer about the message:
    // the claim is kept and the message goes again once Pi rests, within the run's deadline.
    return {
      ending: 'at-work',
      held,
    };
  }
  await noteMessageNotTaken({
    watch,
    held,
  },);
  return { ending: 'not-taken', };
}

/**
 Claims one message ready in the inbox and delivers it.

 @param watch - watch request

 @param name - message's name

 @returns what the delivery came to, or that the entry could not be taken at all
 */
export async function deliverReadyMessage({
  watch,
  name,
}: {
  readonly watch: WatchRequest;
  readonly name: string
},): Promise<DeliveryOfAMessage | 'could-not-take'> {
  /**
   Where the entry moved to when it was claimed.
   */
  let claim: string;
  try {
    claim = await claimMessage({
      watch,
      name,
    },);
  }
  catch (error) {
    watch.record
      .note({
        kind: 'problem',
        problem: `could not take ${name}: ${String(error,)}`,
      },);
    return 'could-not-take';
  }
  try {
    return await deliverClaimed({
      watch,
      held: {
        name,
        claim,
      },
    },);
  }
  catch (error) {
    watch.record
      .note({
        kind: 'problem',
        problem: `could not deliver ${name}: ${String(error,)}`,
      },);
    await noteMessageNotTaken({
      watch,
      held: {
        name,
        claim,
      },
    },);
    return { ending: 'not-taken', };
  }
}

/**
 Delivers a message the driver has been holding since Pi refused it while it worked.

 @param watch - watch request

 @param held - the message and where its claim is

 @returns what the delivery came to
 */
export async function deliverHeldMessage({
  watch,
  held,
}: {
  readonly watch: WatchRequest;
  readonly held: HeldMessage
},): Promise<DeliveryOfAMessage> {
  try {
    return await deliverClaimed({
      watch,
      held,
    },);
  }
  catch (error) {
    watch.record
      .note({
        kind: 'problem',
        problem: `could not deliver ${held.name}: ${String(error,)}`,
      },);
    await noteMessageNotTaken({
      watch,
      held,
    },);
    return { ending: 'not-taken', };
  }
}

/**
 Keeps every message the driver still held when the run ended, and names each in the summary, so a message the run ended before it
 could hand over is neither left without its name nor read as delivered.

 @param watch - watch request

 @param held - the messages claimed and never taken

 @example
 ```ts
 await keepMessagesNotTaken({ watch, held, },);
 ```
 */
export async function keepMessagesNotTaken({
  watch,
  held,
}: {
  readonly watch: WatchRequest;
  readonly held: readonly HeldMessage[]
},): Promise<void> {
  for (const message of held) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- one kept message after another, each renamed under a name of its own; see doc/lint/sequential-awaits.md
    await noteMessageNotTaken({
      watch,
      held: message,
    },);
  }
}

/**
 Lists the messages ready to read. A name starting with a dot is still being written, is being delivered, or was refused.

 @param inbox - inbox folder

 @returns message file names, in order

 @example
 ```ts
 const ready = await readyMessages('/runs/a-1/inbox',);
 ```
 */
export async function readyMessages(inbox: string,): Promise<readonly string[]> {
  return (await readdir(inbox,)).filter(function isReady(name,) {
    return !name.startsWith('.',);
  },)
    .toSorted();
}
