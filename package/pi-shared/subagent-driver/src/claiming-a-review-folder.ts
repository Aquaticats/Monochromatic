/**
 Claims a review's folder for one start, so two branches of one change name never write over each other's review, and so a folder
 left by a start that was killed can be taken again.

 The claim is an owner file created by linking a written record into its place: a link makes the file appear with the record
 complete, or fails because another start made it, so no start can read a half-written record. Only one start holds the folder, and
 a second start says whose it is and whether that start still runs: a folder whose owner has ended is taken again, a folder a live
 start holds is refused, and two branches of one change name are named in the refusal.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  link,
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import { readFileOrNothing, } from './finished-worktrees.ts';
import {
  type ProcessIdentity,
  identityStillRuns,
} from './liveness-of-a-process.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 The code the operating system gives a path that another start already made.
 */
const EXISTS_CODE = 'EEXIST';

/**
 What a start recorded when it took a review's folder: the branch and the commit it writes the review for, and the process that
 took it, so a second start can say whose the folder is and whether that start still runs.
 */
export type ReviewFolderOwner = {
  /**
   The branch whose change the folder's inputs are written for, when the caller named one.
   */
  readonly branch?: string;

  /**
   The commit under review.
   */
  readonly commit: string;

  /**
   What changed, each as `git diff` takes a range.
   */
  readonly ranges: readonly string[];

  /**
   The folder whose changes are reviewed.
   */
  readonly source: string;

  /**
   When the folder was taken, as an ISO 8601 moment.
   */
  readonly claimedAt: string;

  /**
   The start that took it, as the kernel names it.
   */
  readonly process: ProcessIdentity;
};

/**
 A review folder another start already holds, which is refused so two branches of one change name never write over each other's
 review.
 */
export class ReviewFolderTakenError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewFolderTakenError';
}

/**
 Where a test holds a start that is taking a dead start's place, so two starts can be made to look at one dead record at the
 same moment. It is a seam: no command passes it.
 */
export type ReviewFolderClaimSeams = {
  /**
   Awaited after the start has read a dead owner's record and before it removes that record.
   */
  readonly afterLookingAtADeadOwner?: () => Promise<void>;
};

/**
 What a review folder holds of its owner: the record a start left, or no record that can be read.
 */
type OwnerOfAReview =
  | {
    /**
     A record this start can read.
     */
    readonly kind: 'an-owner';

    /**
     The record itself.
     */
    readonly owner: ReviewFolderOwner;
  }
    | {
      /**
       The folder holds no record, which a start takes as a folder it may take itself.
       */
      readonly kind: 'no-owner';
    };

/**
 The answer for a folder that holds no record this module can read.
 */
const NO_OWNER: OwnerOfAReview = { kind: 'no-owner', };

/**
 Whether two branches, each known, write one change name: the case a start refuses and names both for.

 @param holderBranch - the branch the folder's record names

 @param branch - the branch this start writes the review for

 @returns whether both are known and differ

 @example
 ```ts
 const differ = namesTwoBranches({ holderBranch: 'fix/a', branch: 'feat/a', },);
 ```
 */
function namesTwoBranches({
  holderBranch,
  branch,
}: {
  readonly holderBranch?: string;
  readonly branch?: string;
},): boolean {
  if ((holderBranch === undefined) || (branch === undefined)) {
    return false;
  }
  return holderBranch !== branch;
}

/**
 Reads the owner record a review folder holds, when it holds one that can be read.

 @param ownerFile - the file the record is kept in

 @returns the record, or no record

 @example
 ```ts
 const owner = await ownerIn({ ownerFile: '/reviews/a/the-owner-of-this-review.json', },);
 ```
 */
async function ownerIn({ ownerFile, }: { readonly ownerFile: string },): Promise<OwnerOfAReview> {
  /**
   What the file holds.
   */
  const shown = (await readFileOrNothing(ownerFile,)).trim();
  if (shown === '') {
    return NO_OWNER;
  }
  return ownerRecordIn({ shown, },);
}

/**
 Reads an owner record from the text it was written as.

 An owner file that cannot be read as a record is treated as no record: only this module writes that file, and it writes it whole
 before the file appears, so a record that cannot be read is one this start may take the folder from.

 @param shown - the file's text

 @returns the record, or no record

 @example
 ```ts
 const owner = ownerRecordIn({ shown: '{"commit":"abc","ranges":[],"source":"a","claimedAt":"now","process":{"pid":1,"bootId":"b","startTicks":"1"}}', },);
 ```
 */
function ownerRecordIn({ shown, }: { readonly shown: string },): OwnerOfAReview {
  /**
   The record, of any shape.
   */
  const kept: unknown = JSON.parse(shown,);
  if (!isRecord(kept,)) {
    return NO_OWNER;
  }
  /**
   What the record says of the start that took the folder.
   */
  const { process, } = kept;
  if (!isRecord(process,)) {
    return NO_OWNER;
  }
  if ((typeof process.pid) !== 'number') {
    return NO_OWNER;
  }
  if ((typeof process.bootId) !== 'string') {
    return NO_OWNER;
  }
  if ((typeof process.startTicks) !== 'string') {
    return NO_OWNER;
  }
  if ((typeof kept.commit) !== 'string') {
    return NO_OWNER;
  }
  if ((typeof kept.source) !== 'string') {
    return NO_OWNER;
  }
  if ((typeof kept.claimedAt) !== 'string') {
    return NO_OWNER;
  }
  if ((kept.branch !== undefined) && ((typeof kept.branch) !== 'string')) {
    return NO_OWNER;
  }
  if (!Array.isArray(kept.ranges)) {
    return NO_OWNER;
  }
  /**
   The ranges the record kept, which a start wrote as text.
   */
  const ranges = kept.ranges
    .filter(function isARange(range,): range is string {
      return (typeof range) === 'string';
    },);
  if (ranges.length
    !== kept.ranges
      .length) {
    return NO_OWNER;
  }
  return {
    kind: 'an-owner',
    owner: {
      ...((kept.branch === undefined) ? {} : { branch: kept.branch, }),
      commit: kept.commit,
      ranges,
      source: kept.source,
      claimedAt: kept.claimedAt,
      process: {
        pid: process.pid,
        bootId: process.bootId,
        startTicks: process.startTicks,
      },
    },
  };
}

/**
 What making the owner file by linking it into place answered.
 */
type OwnerFileOutcome =
  | {
    /**
     This start made the owner file.
     */
    readonly kind: 'made';
  }
    | {
      /**
       Another start had already made the owner file.
       */
      readonly kind: 'another-start-has-it';
    }
    | {
      /**
       The link failed for a reason that is not a second owner.
       */
      readonly kind: 'the-link-failed';

      /**
       What the link gave.
       */
      readonly error: unknown;
    };

/**
 Links a written record into the owner file's place, which only one start can do.

 @param temporary - the file the record was written to

 @param ownerFile - the owner file's path

 @returns whether this start made the owner file, or the reason the link did not

 @example
 ```ts
 const outcome = await linkTheOwnerFile({ temporary, ownerFile },);
 ```
 */
async function linkTheOwnerFile({
  temporary,
  ownerFile,
}: {
  readonly temporary: string;
  readonly ownerFile: string;
},): Promise<OwnerFileOutcome> {
  try {
    await link(
      temporary,
      ownerFile,
    );
    return { kind: 'made', };
  }
  catch (error) {
    if (errorCodeOf(error,) === EXISTS_CODE) {
      return { kind: 'another-start-has-it', };
    }
    return {
      kind: 'the-link-failed',
      error,
    };
  }
}

/**
 Creates the owner file with the record in it, only when no owner file is there.

 The record is written whole to a file of its own and then linked into place: a link either makes the owner file appear with the
 record complete or fails because another start made it, so no start can read a half-written record.

 @param ownerFile - the file the record is kept in

 @param owner - the record to keep

 @returns whether this start made the file

 @example
 ```ts
 const taken = await takeTheOwnerFile({ ownerFile, owner },);
 ```
 */
async function takeTheOwnerFile({
  ownerFile,
  owner,
}: {
  readonly ownerFile: string;
  readonly owner: ReviewFolderOwner;
},): Promise<boolean> {
  /**
   A file of this start's own, beside the owner file, which no other start writes.
   */
  const temporary = `${ownerFile}.${String(process.pid,)}.${randomUUID()}.tmp`;
  /**
   The record as it is written.
   */
  const record = JSON.stringify(
    owner,
    undefined,
    2,
  );
  await writeFile(
    temporary,
    `${record}\n`,
    'utf8',
  );
  /**
   What linking the record into the owner file's place answered.
   */
  const outcome = await linkTheOwnerFile({
    temporary,
    ownerFile,
  },);
  await rm(
    temporary,
    { force: true, },
  );
  if (outcome.kind === 'made') {
    return true;
  }
  if (outcome.kind === 'another-start-has-it') {
    return false;
  }
  throw outcome.error;
}

/**
 Takes a review's folder for one start, or refuses when another start holds it.

 @param reviewFolder - the review's own folder

 @param ownerFile - the file the owner record is kept in, inside that folder

 @param branch - the branch whose change this start writes the review for, when it named one

 @param owner - the record to keep when this start takes the folder

 @param seams - a place a test holds a start at, which no command passes

 @throws ReviewFolderTakenError when another start holds the folder, naming both branches when both are known

 @example
 ```ts
 await takeReviewFolder({ reviewFolder, ownerFile, branch, owner },);
 ```
 */
export async function takeReviewFolder({
  reviewFolder,
  ownerFile,
  branch,
  owner,
  seams,
}: {
  readonly reviewFolder: string;
  readonly ownerFile: string;
  readonly branch?: string;
  readonly owner: ReviewFolderOwner;
  readonly seams?: ReviewFolderClaimSeams;
},): Promise<void> {
  await mkdir(
    reviewFolder,
    { recursive: true, },
  );
  if (await takeTheOwnerFile({
    ownerFile,
    owner,
  },)) {
    return;
  }
  /**
   What the folder records of the start that holds it.
   */
  const held = await ownerIn({ ownerFile, },);
  /**
   The record itself, when the folder holds one.
   */
  const heldOwner = (held.kind === 'an-owner') ? held.owner : undefined;
  /**
   The branch that record names, when it names one.
   */
  const holderBranch = heldOwner?.branch;
  /**
   Whether that start still runs.
   */
  const holderRuns = (heldOwner === undefined) ? false : await identityStillRuns({ identity: heldOwner.process, },);
  if (holderRuns || namesTwoBranches({
    ...((holderBranch === undefined) ? {} : { holderBranch, }),
    ...((branch === undefined) ? {} : { branch, }),
  },)) {
    throw takenFolderError({
      reviewFolder,
      ...((heldOwner === undefined) ? {} : { holder: heldOwner, }),
      ...((branch === undefined) ? {} : { branch, }),
      holderRuns,
    },);
  }
  // The holder has ended and wrote this same change: take its record away and try once more. Only one start can make the new
  // record, so two starts of one branch cannot both write the folder, and a start killed before it wrote its brief blocks nobody
  // for good.
  await seams?.afterLookingAtADeadOwner?.();
  await rm(
    ownerFile,
    { force: true, },
  );
  if (await takeTheOwnerFile({
    ownerFile,
    owner,
  },)) {
    return;
  }
  /**
   What the folder records of the start that took it between the two tries.
   */
  const second = await ownerIn({ ownerFile, },);
  /**
   The record of that start, when the folder holds one.
   */
  const secondOwner = (second.kind === 'an-owner') ? second.owner : undefined;
  throw takenFolderError({
    reviewFolder,
    ...((secondOwner === undefined) ? {} : { holder: secondOwner, }),
    ...((branch === undefined) ? {} : { branch, }),
    holderRuns: (secondOwner === undefined) ? false : await identityStillRuns({ identity: secondOwner.process, },),
  },);
}

/**
 Reads the refusal of a review folder another start holds, naming both branches when both are known.

 @param reviewFolder - the review's own folder

 @param holder - the record of the start that holds it, when one could be read

 @param branch - the branch this start writes the review for, when it named one

 @param holderRuns - whether the holder still runs

 @returns the error to throw

 @example
 ```ts
 const error = takenFolderError({ reviewFolder: '/reviews/a', holder, branch: 'feat/a', holderRuns: true },);
 ```
 */
function takenFolderError({
  reviewFolder,
  holder,
  branch,
  holderRuns,
}: {
  readonly reviewFolder: string;
  readonly holder?: ReviewFolderOwner;
  readonly branch?: string;
  readonly holderRuns: boolean;
},): ReviewFolderTakenError {
  /**
   The branch the holder writes the review for, when it named one.
   */
  const holderBranch = holder?.branch;
  if (namesTwoBranches({
    ...((holderBranch === undefined) ? {} : { holderBranch, }),
    ...((branch === undefined) ? {} : { branch, }),
  },)) {
    return new ReviewFolderTakenError(
      `the review folder ${reviewFolder} is held for ${String(holderBranch,)}, and this start is for ${String(branch,)}: two branches of one change name cannot share one review folder; ask for them in separate batches, or remove that folder once its start has ended`,
    );
  }
  if (holderRuns) {
    return new ReviewFolderTakenError(
      `the review folder ${reviewFolder} is held by a start${(holderBranch === undefined) ? '' : ` of ${holderBranch}`} that is still running; start this branch again when that start has ended`,
    );
  }
  return new ReviewFolderTakenError(
    `the review folder ${reviewFolder} is held by another start that could not be taken over; remove the folder and start the branch again`,
  );
}
