/**
 What a later driver makes of the claims a driver that died left behind: whose they are, whether that driver is gone, and keeping
 each message under its own name with the reason it was not taken.

 A claim says that delivery had begun and never whether Pi took the message, and a note written for an earlier moment may no longer
 apply, so a recovered message is kept and named rather than sent again on a guess.

 Two passes can recover the same claim at once. Every read here answers nothing for a folder that has gone, every removal tolerates
 a folder another pass took away, and the keep recognises the kept name another pass already made, so either pass finishing the
 work, or both, leaves one result. `doc/states-of-a-claimed-message.md` writes the states and steps down.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  INBOX_CLAIMS_FOLDER,
  claimOwnerIsGone,
  entryOrNothing,
  lastPartOf,
  processFolderOf,
  removeClaimOwnerFolder,
  removeFolderIfEmpty,
} from './claims-of-an-inbox.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  CLAIM_FINISHED,
  NO_KEPT_NAME,
  noteMessageNotTaken,
} from './keeping-a-message-aside.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 Keeps one message a driver that is gone had claimed, and says on the feed where it went and why.

 A claim says that delivery had begun and never whether Pi took the message, and a note written for an earlier moment may no longer
 apply, so the message is kept and named rather than sent again on a guess. A claim another pass has already finished is no
 failure: the message is kept or delivered there, never lost here.

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
   Where the message was kept, or which of the two ways this pass did not keep it.
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
      problem: (kept === CLAIM_FINISHED)
        ? `the claim on ${name} was finished by another pass, so this pass kept nothing from it`
        : (kept === NO_KEPT_NAME)
          ? `the driver that claimed ${name} is gone, so whether Pi took it is unknown; the message could not be kept`
          : `the driver that claimed ${name} is gone, so whether Pi took it is unknown; the message is kept as ${lastPartOf(kept,)}`,
    },);
}

/**
 A place in the recovery at which a test may let another recovery run, so a control can put two passes across one read, one keep or
 one removal. Production passes none.

 - `before-listing-its-claims`: the owner is proved gone and its folder is about to be listed.
 - `before-reading-a-claim-folder`: a claim folder was listed and is about to be read.
 - `before-keeping-a-message`: a message in a claim folder is about to be kept.
 - `before-removing-a-folder`: a folder that held claims is about to be taken away.
 */
export type WhereARecoveryPauses =
  | 'before-listing-its-claims'
    | 'before-reading-a-claim-folder'
    | 'before-keeping-a-message'
    | 'before-removing-a-folder';

/**
 Keeps and names every message a driver that is gone had claimed, so a message is either still to be delivered, delivered, or kept
 under a name a person reads with the reason it was not.

 The claim's folder names its owner, and the process folder says whether that owner can still act, so a driver that is slow is left
 alone and a driver that is gone, including one the kernel reports dead and has not reaped, has its claims laid down. A driver
 killed before this runs leaves claims that a later driver, or a person reading the inbox, still finds under the message's own name.

 @param watch - watch request

 @param pause - where a test waits with another recovery running, which production leaves out

 @example
 ```ts
 await recoverClaimsOfGoneDrivers({ watch, },);
 ```
 */
export async function recoverClaimsOfGoneDrivers({
  watch,
  pause,
}: {
  readonly watch: WatchRequest;
  readonly pause?: (
    where: WhereARecoveryPauses,
    path: string,
  ) => Promise<void>
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
      // Not one of this code's claims: a folder it names is an owner's, and nothing here proves that an owner is gone.
      continue;
    }
    // A driver that is still there, or that cannot be shown to be gone, keeps its claims: it may be about to take them.
    if (!(await claimOwnerIsGone({
      processFolder: processFolderOf(watch,),
      ownerName,
    },))) {
      continue;
    }
    await pause?.(
      'before-listing-its-claims',
      ownerFolder,
    );
    /**
     The claims that owner left, each in a folder of its own, or a message the code claimed before it used claim folders, or nothing
     when another pass took the folder away while this one was judging it.
     */
    const claimed = await namesInFolder(ownerFolder,);
    for (const claimEntry of claimed) {
      /**
       Where that entry is.
       */
      const path = join(
        ownerFolder,
        claimEntry,
      );
      /**
       What it is, or nothing when it has gone since the folder was listed.
       */
      const info = await entryOrNothing(path,);
      if ((typeof info) === 'symbol') {
        continue;
      }
      if (!info.isDirectory()) {
        // A message claimed by the code before it used claim folders: the message itself, under its own name.
        await keepOrphanedMessage({
          watch,
          name: claimEntry,
          claim: path,
        },);
        continue;
      }
      await pause?.(
        'before-reading-a-claim-folder',
        path,
      );
      /**
       The messages that claim folder holds, which is one unless a claim took a message that was itself a folder, or nothing when
       another pass took the folder away while this one was reading its siblings.
       */
      const inside = await namesInFolder(path,);
      for (const name of inside) {
        /**
         Where that message is.
         */
        const claim = join(
          path,
          name,
        );
        await pause?.(
          'before-keeping-a-message',
          claim,
        );
        await keepOrphanedMessage({
          watch,
          name,
          claim,
        },);
      }
      await pause?.(
        'before-removing-a-folder',
        path,
      );
      await removeFolderIfEmpty(path,);
    }
    await pause?.(
      'before-removing-a-folder',
      ownerFolder,
    );
    await removeClaimOwnerFolder(ownerFolder,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
}

/**
 Lists a folder's entries, or answers nothing when the folder has gone: a folder another pass took away is that pass's work done,
 never a failure of this one.

 @param folder - the folder to list

 @returns its entries in order, or nothing when it has gone

 @example
 ```ts
 const names = await namesInFolder('/runs/a-1/inbox/.claims/4242-123456',);
 ```
 */
async function namesInFolder(folder: string,): Promise<readonly string[]> {
  try {
    return (await readdir(folder,)).toSorted();
  }
  catch (error) {
    if (errorCodeOf(error,) === 'ENOENT') {
      return [];
    }
    throw error;
  }
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
