/**
 Handing a message from the inbox to Pi: the send, what is done with a message Pi did not take, what a run still owes at its end,
 and what a later driver makes of a claim whose driver died.

 A message is claimed by renaming it into a folder named for the driver that claims it, so the claim keeps the message's own name
 and the name of its owner at every instant; that folder, and how a claimed message is laid down, are in `claims-of-an-inbox.ts`. A
 message is taken when Pi says it has it, kept aside under a dot name when Pi refuses it while it rests or leaves it unanswered, and
 held while Pi is at work, because Pi refuses every prompt handed to it at work and takes the same message once it rests. A driver
 that is killed leaves its claims behind, and a claim says only that delivery had begun, never whether Pi took the message: the
 next driver that reads that inbox keeps each such message as `.orphaned-<name>` and names it, so a message is never delivered
 twice on a guess and never lost.

 @module
 */

import {
  readdir,
  unlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import {
  type HeldMessage,
  ClaimOwnerUnreadableError,
  claimMessage,
  entryOrNothing,
  removeClaimFolderOf,
} from './claims-of-an-inbox.ts';
import { noteMessageNotTaken, } from './keeping-a-message-aside.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { readOrdinaryFile, } from './read-ordinary-file.ts';
import type { WatchRequest, } from './watch-request.ts';
import {
  messageToPiCameTo,
  outcomeTextOf,
} from './what-pi-did-with-a-message.ts';

/**
 The message a claim holds, as the rest of the driver names it, and the folder an inbox keeps its claims in.
 */
export {
  type HeldMessage,
  INBOX_CLAIMS_FOLDER,
} from './claims-of-an-inbox.ts';

/**
 What delivering one message came to.
 */
export type DeliveryOfAMessage =
  /**
   Pi took it, and its file is gone.
   */
  | {
    readonly ending: 'taken';
  }
  /**
   Pi refused it because it had begun a turn, so the driver holds it and sends it again once Pi rests.
   */
    | {
      readonly ending: 'at-work';
      readonly held: HeldMessage;
    }
  /**
   It was kept aside under a dot name, and named in the summary as a message Pi never took.
   */
    | {
      readonly ending: 'not-taken';
    }
  /**
   This driver cannot read the start of its own process, so it cannot claim a message under a name a later driver could prove it
   owns; the message stays in the inbox under its own name.
   */
    | {
      readonly ending: 'cannot-claim';
    };

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
      success: cameTo === 'taken',
      error: outcomeTextOf(answer,),
      preview: preview({
        value: text,
        length: FEED_PREVIEW_CHARACTERS.inboxMessage,
      },),
    },);
  if (cameTo === 'taken') {
    try {
      await unlinkClaimed(held.claim,);
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
  if (cameTo === 'at-work') {
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
    keptAs: 'failed',
  },);
  return { ending: 'not-taken', };
}

/**
 Takes a delivered message's claim away, and the folders that held it with it when each is empty. A claim that has already gone is
 another pass's work done, and nothing here fails over it.

 @param claim - where the message was
 */
async function unlinkClaimed(claim: string,): Promise<void> {
  try {
    await unlink(claim,);
  }
  catch (error) {
    if (errorCodeOf(error,) !== 'ENOENT') {
      throw error;
    }
  }
  await removeClaimFolderOf(claim,);
}

/**
 The absence of a claim: the inbox entry could not be taken at all.
 */
const NO_CLAIM: unique symbol = Symbol('inbox entry could not be claimed');

/**
 The absence of a claim this driver could name: it cannot read the start of its own process.
 */
const NO_CLAIM_OWNER: unique symbol = Symbol('cannot name a claim folder');

/**
 Claims one inbox entry, or answers the sentinel that says why it could not be taken.

 @param watch - watch request

 @param name - message's name

 @returns where the message now is, or {@link NO_CLAIM} or {@link NO_CLAIM_OWNER}
 */
async function claimOrNothing({
  watch,
  name,
}: {
  readonly watch: WatchRequest;
  readonly name: string
},): Promise<string | typeof NO_CLAIM | typeof NO_CLAIM_OWNER> {
  try {
    return await claimMessage({
      watch,
      name,
    },);
  }
  catch (error) {
    if (error instanceof ClaimOwnerUnreadableError) {
      // A message claimed under a name no later driver could verify would be one no later driver could resolve. It stays in the
      // inbox under its own name, and this run names it rather than reading as one that took it.
      return NO_CLAIM_OWNER;
    }
    watch.record
      .note({
        kind: 'problem',
        problem: `could not take ${name}: ${String(error,)}`,
      },);
    return NO_CLAIM;
  }
}

/**
 Claims one message ready in the inbox and delivers it.

 @param watch - watch request

 @param name - message's name

 @returns what the delivery came to, or that the entry could not be taken at all

 @example
 ```ts
 const delivered = await deliverReadyMessage({ watch, name: 'm', },);
 ```
 */
export async function deliverReadyMessage({
  watch,
  name,
}: {
  readonly watch: WatchRequest;
  readonly name: string
},): Promise<DeliveryOfAMessage | 'could-not-take'> {
  /**
   Where the entry moved to when it was claimed, or the sentinel that says it could not be taken at all.
   */
  const claim = await claimOrNothing({
    watch,
    name,
  },);
  if (claim === NO_CLAIM_OWNER) {
    return { ending: 'cannot-claim', };
  }
  if (claim === NO_CLAIM) {
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
      keptAs: 'failed',
    },);
    return { ending: 'not-taken', };
  }
}

/**
 Delivers a message the driver has been holding since Pi refused it while it worked.

 @param watch - watch request

 @param held - the message and where its claim is

 @returns what the delivery came to

 @example
 ```ts
 const delivered = await deliverHeldMessage({ watch, held, },);
 ```
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
      keptAs: 'failed',
    },);
    return { ending: 'not-taken', };
  }
}

/**
 The messages a run still owes Pi: the inbox ones it claimed and Pi has not taken. Disposing this, at the end of the watch or on a
 throw, keeps every one under the name it was published under and names it in the summary, so a message the run ends before it
 hands over is never read as delivered.
 */
export type MessagesTheRunOwes = {
  /**
   The messages claimed and not yet taken, oldest first.
   */
  readonly claimed: HeldMessage[];

  /**
   The messages this driver could not claim at all, by the name each was published under; each stays in the inbox under its name.
   */
  readonly unclaimable: string[];

  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Starts the messages a run still owes Pi, as a disposable it keeps and names on the way out.

 @param watch - watch request

 @returns the messages the run still owes, and what keeps them at its end

 @example
 ```ts
 await using owed = holdMessagesTheRunOwes(watch,);
 ```
 */
export function holdMessagesTheRunOwes(watch: WatchRequest,): MessagesTheRunOwes {
  /**
   The messages claimed and not yet taken.
   */
  const claimed: HeldMessage[] = [];
  /**
   The messages this driver could not claim at all.
   */
  const unclaimable: string[] = [];
  return {
    claimed,
    unclaimable,
    async [Symbol.asyncDispose]() {
      await keepMessagesNotTaken({
        watch,
        held: claimed,
      },);
      /* oxlint-disable eslint/no-await-in-loop -- one message after another, each looked at before it is named; see
         doc/lint/sequential-awaits.md */
      for (const name of unclaimable) {
        // The summary says what is true when it is written: a name this driver could not claim is named only if its message is
        // still in the inbox, because another driver may have claimed it and Pi may have taken it whole.
        /**
         What is in the inbox under that name, or nothing when it has gone.
         */
        const stillThere = await entryOrNothing(join(
          watch.inbox,
          name,
        ),);
        if ((typeof stillThere) !== 'symbol') {
          watch.record
            .noteMessageNotTaken(name,);
        }
      }
      /* oxlint-enable eslint/no-await-in-loop */
    },
  };
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
      keptAs: 'failed',
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
