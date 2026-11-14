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
  INBOX_CLAIMS_FOLDER,
  NO_KEPT_NAME,
  claimMessage,
  claimOwnerIsGone,
  entryOrNothing,
  lastPartOf,
  noteMessageNotTaken,
  processFolderOf,
  removeClaimOwnerFolder,
  removeOwnerFolderOf,
} from './claims-of-an-inbox.ts';
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
 Takes a delivered message's claim away, and the owner folder with it when no other claim is left.

 @param claim - where the message was
 */
async function unlinkClaimed(claim: string,): Promise<void> {
  await unlink(claim,);
  await removeOwnerFolderOf(claim,);
}

/**
 The absence of a claim: the inbox entry could not be taken at all.
 */
const NO_CLAIM: unique symbol = Symbol('inbox entry could not be claimed');

/**
 Claims one inbox entry, or answers the sentinel when it cannot be taken at all.

 @param watch - watch request

 @param name - message's name

 @returns where the message now is, or {@link NO_CLAIM}
 */
async function claimOrNothing({
  watch,
  name,
}: {
  readonly watch: WatchRequest;
  readonly name: string
},): Promise<string | typeof NO_CLAIM> {
  try {
    return await claimMessage({
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
  if ((typeof claim) === 'symbol') {
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
  return {
    claimed,
    async [Symbol.asyncDispose]() {
      await keepMessagesNotTaken({
        watch,
        held: claimed,
      },);
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
 Keeps one message a driver that is gone had claimed, and says on the feed where it went and why.

 A claim says that delivery had begun and never whether Pi took the message, and a note written for an earlier moment may no longer
 apply, so the message is kept and named rather than sent again on a guess.

 @param watch - watch request

 @param name - the message's name, as its claim folder names it

 @param claim - where the claim holds it
 */
async function keepOrphanedMessage({
  watch,
  name,
  claim,
}: {
  readonly watch: WatchRequest;
  readonly name: string;
  readonly claim: string
},): Promise<void> {
  /**
   Where the message was kept, when it could be.
   */
  const kept = await noteMessageNotTaken({
    watch,
    held: {
      name,
      claim,
    },
    keptAs: 'orphaned',
  },);
  watch.record
    .note({
      kind: 'problem',
      problem: (kept === NO_KEPT_NAME)
        ? `the driver that claimed ${name} is gone, so whether Pi took it is unknown; the message could not be kept`
        : `the driver that claimed ${name} is gone, so whether Pi took it is unknown; the message is kept as ${lastPartOf(kept,)}`,
    },);
}

/**
 Keeps and names every message a driver that is gone had claimed, so a message is either still to be delivered, delivered, or kept
 under a name a person reads with the reason it was not.

 The claim's folder names its owner, and the process folder says whether that owner is still there, so a driver that is slow is
 left alone and a driver that is gone has its claims laid down. A driver killed before this runs leaves claims that a later driver,
 or a person reading the inbox, still finds under the message's own name.

 @param watch - watch request

 @example
 ```ts
 await recoverClaimsOfGoneDrivers({ watch, },);
 ```
 */
export async function recoverClaimsOfGoneDrivers({
  watch,
}: {
  readonly watch: WatchRequest
},): Promise<void> {
  /**
   Every claim's owner folder.
   */
  const claimsFolder = join(
    watch.inbox,
    INBOX_CLAIMS_FOLDER,
  );
  /**
   The owners that have claimed a message, or nothing when none has.
   */
  const owners = await ownerFoldersIn({
    watch,
    claimsFolder,
  },);
  /* oxlint-disable eslint/no-await-in-loop -- one owner after another, each judged before its claims are touched; see
     doc/lint/sequential-awaits.md */
  for (const ownerName of owners) {
    /**
     Where that owner's claims are.
     */
    const ownerFolder = join(
      claimsFolder,
      ownerName,
    );
    /**
     What the entry is, or nothing when it has gone since the folder was listed.
     */
    const entry = await entryOrNothing(ownerFolder,);
    if (((typeof entry) === 'symbol')) {
      continue;
    }
    if (!entry.isDirectory()) {
      // A claim outside an owner's folder is not one this code makes; its own name is all that is left to keep it under.
      await keepOrphanedMessage({
        watch,
        name: ownerName,
        claim: ownerFolder,
      },);
      continue;
    }
    // A driver that is still there, or that cannot be shown to be gone, keeps its claims: it may be about to take them.
    if (!(await claimOwnerIsGone({
      processFolder: processFolderOf(watch,),
      ownerName,
    },))) {
      continue;
    }
    /**
     The messages that owner claimed.
     */
    const claimed = await readdir(ownerFolder,);
    for (const name of claimed) {
      await keepOrphanedMessage({
        watch,
        name,
        claim: join(
          ownerFolder,
          name,
        ),
      },);
    }
    await removeClaimOwnerFolder(ownerFolder,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
}

/**
 Lists the owners that have claimed a message, or answers nothing when no claim is there to read.

 @param watch - watch request

 @param claimsFolder - the folder that holds one owner's claims per entry

 @returns the owners' folder names, in order

 @example
 ```ts
 const owners = await ownerFoldersIn({ watch, claimsFolder: '/runs/a-1/inbox/.claims', },);
 ```
 */
async function ownerFoldersIn({
  watch,
  claimsFolder,
}: {
  readonly watch: WatchRequest;
  readonly claimsFolder: string
},): Promise<readonly string[]> {
  try {
    return (await readdir(claimsFolder,)).toSorted();
  }
  catch (error) {
    if (errorCodeOf(error,) === 'ENOENT') {
      return [];
    }
    watch.record
      .note({
        kind: 'problem',
        problem: `could not read the claims of an earlier driver in ${claimsFolder}: ${String(error,)}`,
      },);
    return [];
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
