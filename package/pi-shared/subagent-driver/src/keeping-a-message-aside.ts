/**
 Keeping a message a driver did not hand to Pi, under a name that says why, without ever replacing an earlier kept message or
 making the same publication twice.

 Publication names a claimed message `.failed-<name>[.<n>]` or `.orphaned-<name>[.<n>]`: `.failed-` when Pi refused it while it
 rested or never answered, `.orphaned-` when the driver that claimed it died before it knew whether Pi took it. The `<n>` counts up
 over names already taken, and a link fails when its name is taken, so nothing is replaced. A kept name that is one file with the
 claim is this message's own publication, made by a pass that was killed or is still going: it is finished here, never made twice.
 The folders and names a claim lives under are in `claims-of-an-inbox.ts`, and every state and step is written in
 `doc/states-of-a-claimed-message.md`.

 @module
 */

import {
  type BigIntStats,
  constants as fileConstants,
  type Stats,
} from 'node:fs';
import {
  copyFile,
  link as linkOnDisk,
  readdir,
  unlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  type HeldMessage,
  entryOrNothing,
  removeClaimFolderOf,
} from './claims-of-an-inbox.ts';
import { errorCodeOf, } from './error-code-of.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 The absence of a kept name: the message could not be kept aside at all.
 */
export const NO_KEPT_NAME: unique symbol = Symbol('message could not be kept');

/**
 The claim a keep was about has gone, because another pass finished it: the message is kept or delivered, and this pass has
 nothing left to publish.
 */
export const CLAIM_FINISHED: unique symbol = Symbol('another pass finished the claim');

/**
 A keep whose claim has gone before it could publish: another pass finished the claim first. It is an error for the keep,
 because this pass keeps nothing, and a fact for its caller, which names the message rather than losing it.
 */
export class ClaimFinishedError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ClaimFinishedError';
}


/**
 What is at a path, as the filesystem reports it: the ordinary and the big-integer report both say when two names are one file.
 */
type WhatIsAtAPath = Stats | BigIntStats;

/**
 Tells whether two names stand for one file, which is how a publication another pass already made is told from a kept message of
 some other name.

 @param one - what one path is

 @param another - what the other path is

 @returns whether the two describe one file

 @example
 ```ts
 const same = oneFile({ one: infoOfKept, another: infoOfClaim, },);
 ```
 */
function oneFile({
  one,
  another,
}: {
  readonly one: WhatIsAtAPath;
  readonly another: WhatIsAtAPath
},): boolean {
  return (one.dev === another.dev) && (one.ino === another.ino);
}

/**
 Removes a claim's own name, and the folders that held it once each is empty. A claim another pass has already removed is the
 work of that pass done: nothing here fails over it.

 @param claim - where the message was
 */
async function removeClaimName(claim: string,): Promise<void> {
  try {
    await unlink(claim,);
  }
  catch (error) {
    // The other pass that published this message removed the claim's name first; what is left to do is the same.
    if (errorCodeOf(error,) !== 'ENOENT') {
      throw error;
    }
  }
  await removeClaimFolderOf(claim,);
}

/**
 Publishes a kept message under one name, without ever replacing a file already there: a link fails when its name is taken, so the
 caller tries the next number. A kept name that is one file with the claim is this message's own publication, made by a pass that
 was killed or that is still going: it is finished here and never made twice.

 @param claim - where the message now is

 @param claimInfo - what the claim is

 @param target - the name this attempt would keep it under

 @param link - the hard link a filesystem gives, which a test replaces to make one fail

 @returns what this attempt achieved, so the caller knows whether to try another name
 */
async function publishKept({
  claim,
  claimInfo,
  target,
  link,
}: {
  readonly claim: string;
  readonly claimInfo: WhatIsAtAPath;
  readonly target: string;
  readonly link: (
    existing: string,
    created: string,
  ) => Promise<void>
},): Promise<'published' | 'name-taken' | 'claim-finished'> {
  /**
   What already stands at the kept name, if anything does.
   */
  const standing = await entryOrNothing(target,);
  if ((typeof standing) !== 'symbol') {
    if (oneFile({
      one: standing,
      another: claimInfo,
    },)) {
      // The kept name and the claim are one file, so an earlier pass published this message and was killed before it removed the
      // claim, or is still going: finishing the removal makes one result, and no second copy is made.
      await removeClaimName(claim,);
      return 'published';
    }
    return 'name-taken';
  }
  try {
    await link(
      claim,
      target,
    );
    await removeClaimName(claim,);
    return 'published';
  }
  catch (error) {
    /**
     What the operating system said about this attempt.
     */
    const code = errorCodeOf(error,);
    if (code === 'EEXIST') {
      // The name was taken between the read above and the link. It is this message's own publication only when the two names are
      // one file; otherwise the next number is tried.
      /**
       What stands at the kept name now.
       */
      const takenBy = await entryOrNothing(target,);
      if (
        ((typeof takenBy) !== 'symbol')
          && oneFile({
            one: takenBy,
            another: claimInfo,
          },)
      ) {
        await removeClaimName(claim,);
        return 'published';
      }
      return 'name-taken';
    }
    if (code === 'ENOENT') {
      // The claim went between the read above and the link: another pass finished it.
      return 'claim-finished';
    }
    if ((code === 'EPERM') || (code === 'EACCES')) {
      /**
       What the claim holds, which a copy reproduces only when it is an ordinary file.
       */
      const info = await entryOrNothing(claim,);
      if ((typeof info) === 'symbol') {
        return 'claim-finished';
      }
      if (!info.isFile()) {
        // A folder is not a file a copy can hold; the message stays under its claim, where its own name reads it, and is named.
        throw error;
      }
      // A filesystem without hard links still keeps the no-replacement promise: a copy that creates its name exclusively fails
      // when something already stands there, exactly as the link does.
      try {
        await copyFile(
          claim,
          target,
          fileConstants.COPYFILE_EXCL,
        );
      }
      catch (copyError) {
        if (errorCodeOf(copyError,) !== 'EEXIST') {
          throw copyError;
        }
        /**
         What the claim is now, since a copy is a file of its own and cannot be told from the claim by which file it is.
         */
        const afterCopy = await entryOrNothing(claim,);
        return ((typeof afterCopy) === 'symbol') ? 'claim-finished' : 'name-taken';
      }
      await removeClaimName(claim,);
      return 'published';
    }
    throw error;
  }
}

/**
 Keeps a message that did not reach Pi, so it stays readable and is never sent again. Publication never replaces an earlier kept
 message: a link fails when its name is taken, so the next number is tried instead. A kept name that is one file with the claim is
 this message's own publication, made by a pass that was killed or is still going: it is finished here, not made twice, and the
 name it was made under is the one answered.

 @param watch - watch request

 @param name - message's name

 @param claim - where the message now is

 @param keptAs - the word the kept name carries, which says why the message was not taken

 @param link - the hard link a filesystem gives, which a test replaces to make one fail

 @returns the name the message was kept under

 @throws {@link ClaimFinishedError} when the claim has gone, because another pass finished it

 @example
 ```ts
 const kept = await keepMessage({ watch, name: 'm', claim, keptAs: 'failed', },);
 ```
 */
export async function keepMessage({
  watch,
  name,
  claim,
  keptAs,
  link = linkOnDisk,
}: {
  readonly watch: WatchRequest;
  readonly name: string;
  readonly claim: string;
  readonly keptAs: 'failed' | 'orphaned';
  readonly link?: (
    existing: string,
    created: string,
  ) => Promise<void>
},): Promise<string> {
  /**
   What the claim is, which tells a kept name this message already stands under from one another message holds.
   */
  const claimInfo = await entryOrNothing(claim,);
  if ((typeof claimInfo) === 'symbol') {
    throw new ClaimFinishedError(`the claim on ${name} has gone: another pass finished it`,);
  }
  /**
   The most numbered names to try before giving up: there are always fewer kept messages than names in the inbox.
   */
  const attempts = (await readdir(watch.inbox,)).length + 2;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    /**
     The name this attempt would keep the message under.
     */
    const target = join(
      watch.inbox,
      (attempt === 1) ? `.${keptAs}-${name}` : `.${keptAs}-${name}.${String(attempt,)}`,
    );
    /**
     What this attempt achieved.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one name after another until one is free, and a kept message must never replace another; see doc/lint/sequential-awaits.md
    const outcome = await publishKept({
      claim,
      claimInfo,
      target,
      link,
    },);
    if (outcome === 'published') {
      return target;
    }
    if (outcome === 'claim-finished') {
      throw new ClaimFinishedError(`the claim on ${name} went while it was being kept: another pass finished it`,);
    }
  }
  throw new Error(`every name for a kept message of ${name} is taken`,);
}

/**
 Keeps one message, or answers which of the two ways it was not kept: another pass finished the claim, or the message could not be
 kept at all.

 @param watch - watch request

 @param held - the message and where its claim is

 @param keptAs - the word the kept name carries

 @returns where the message was kept, {@link CLAIM_FINISHED} when another pass finished the claim, or {@link NO_KEPT_NAME}
 */
async function keepOrNothing({
  watch,
  held,
  keptAs,
}: {
  readonly watch: WatchRequest;
  readonly held: HeldMessage;
  readonly keptAs: 'failed' | 'orphaned'
},): Promise<string | typeof CLAIM_FINISHED | typeof NO_KEPT_NAME> {
  try {
    return await keepMessage({
      watch,
      name: held.name,
      claim: held.claim,
      keptAs,
    },);
  }
  catch (error) {
    if (error instanceof ClaimFinishedError) {
      // The claim went while this pass held it: another pass finished it, and nothing here is left to keep.
      return CLAIM_FINISHED;
    }
    // A claim that is already gone was kept or delivered by another driver, and nothing here is left to keep.
    if (errorCodeOf(error,) !== 'ENOENT') {
      watch.record
        .note({
          kind: 'problem',
          problem: `could not keep ${held.name}: ${String(error,)}`,
        },);
    }
    return NO_KEPT_NAME;
  }
}

/**
 Keeps a message Pi never took, and names it in the summary. A message whose file cannot be kept is still named: the summary is
 what a reader of the run has, and a message that is neither with Pi nor under its name must not read as delivered. A claim another
 pass finished first is named too: the message is kept or delivered there, and never lost here.

 @param watch - watch request

 @param held - the message and where its claim is

 @param keptAs - the word the kept name carries, which says why the message was not taken

 @returns where the message was kept, {@link CLAIM_FINISHED} when another pass finished the claim, or {@link NO_KEPT_NAME}

 @example
 ```ts
 await noteMessageNotTaken({ watch, held, keptAs: 'failed', },);
 ```
 */
export async function noteMessageNotTaken({
  watch,
  held,
  keptAs,
}: {
  readonly watch: WatchRequest;
  readonly held: HeldMessage;
  readonly keptAs: 'failed' | 'orphaned'
},): Promise<string | typeof CLAIM_FINISHED | typeof NO_KEPT_NAME> {
  /**
   Where the message was kept, or the sentinel that says it could not be kept at all.
   */
  const kept = await keepOrNothing({
    watch,
    held,
    keptAs,
  },);
  watch.record
    .noteMessageNotTaken(held.name,);
  return kept;
}
