/**
 What a later driver makes of the claims a driver that died left behind: whose they are, whether that driver is gone, and keeping
 each message under its own name with the reason it was not taken.

 A claim says that delivery had begun and never whether Pi took the message, and a note written for an earlier moment may no longer
 apply, so a recovered message is kept and named rather than sent again on a guess.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  INBOX_CLAIMS_FOLDER,
  NO_KEPT_NAME,
  claimOwnerIsGone,
  entryOrNothing,
  lastPartOf,
  noteMessageNotTaken,
  processFolderOf,
  removeClaimOwnerFolder,
  removeFolderIfEmpty,
} from './claims-of-an-inbox.ts';
import { errorCodeOf, } from './error-code-of.ts';
import type { WatchRequest, } from './watch-request.ts';

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
    /**
     The claims that owner left, each in a folder of its own, or a message the code claimed before it used claim folders.
     */
    const claimed = await readdir(ownerFolder,);
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
      /**
       The messages that claim folder holds, which is one unless a claim took a message that was itself a folder.
       */
      const inside = await readdir(path,);
      for (const name of inside) {
        await keepOrphanedMessage({
          watch,
          name,
          claim: join(
            path,
            name,
          ),
        },);
      }
      await removeFolderIfEmpty(path,);
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
