/**
 The record a start leaves in a review's folder: whose the folder is, what the review is for, and the review unit whose life the
 claim follows.

 The record names the unit the review runs as, so a taker can ask the manager whether that review still runs after the command that
 wrote the record has ended. A record written before this package named units is read as if it named this folder's unit.

 @module
 */

import { readFileOrNothing, } from './finished-worktrees.ts';
import type { ProcessIdentity, } from './liveness-of-a-process.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 What a start recorded when it took a review's folder: the branch and the commit it writes the review for, the review unit whose
 life the claim follows, this claim's own name, and the process that took it.
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
   The review unit this folder's review runs as: a folder's owner is whatever lives as long as the review does, which is that unit
   and not the command that wrote this record.
   */
  readonly unit: string;

  /**
   This claim's own name, which a takeover names so that it removes this record and no other.
   */
  readonly claim: string;

  /**
   The start that took it, as the kernel names it.
   */
  readonly process: ProcessIdentity;
};

/**
 What a review folder holds of its owner: the record a start left, or no record that can be read.
 */
export type OwnerOfAReview =
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
export const NO_OWNER: OwnerOfAReview = { kind: 'no-owner', };

/**
 Reads the owner record a review folder holds, when it holds one that can be read.

 @param ownerFile - the file the record is kept in

 @param unit - the review unit this folder's review runs as, which a record written before units were named is read as naming

 @returns the record, or no record

 @example
 ```ts
 const owner = await ownerIn({ ownerFile: '/reviews/a/the-owner-of-this-review.json', unit: 'pi-review-a.service', },);
 ```
 */
export async function ownerIn({
  ownerFile,
  unit,
}: {
  readonly ownerFile: string;
  readonly unit: string;
},): Promise<OwnerOfAReview> {
  /**
   What the file holds.
   */
  const shown = (await readFileOrNothing(ownerFile,)).trim();
  if (shown === '') {
    return NO_OWNER;
  }
  return ownerRecordIn({
    shown,
    unit,
  },);
}

/**
 Reads an owner record from the text it was written as.

 An owner file that cannot be read as a record is treated as no record: only this package writes that file, and it writes it whole
 before the file appears, so a record that cannot be read is one this start may take the folder from.

 @param shown - the file's text

 @param unit - the review unit this folder's review runs as

 @returns the record, or no record

 @example
 ```ts
 const owner = ownerRecordIn({ shown: '{"commit":"abc","unit":"pi-review-a.service","claim":"a","ranges":[],"source":"a","claimedAt":"now","process":{"pid":1,"bootId":"b","startTicks":"1"}}', unit: 'pi-review-a.service', },);
 ```
 */
function ownerRecordIn({
  shown,
  unit,
}: {
  readonly shown: string;
  readonly unit: string;
},): OwnerOfAReview {
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
  if ((kept.unit !== undefined) && ((typeof kept.unit) !== 'string')) {
    return NO_OWNER;
  }
  if ((kept.claim !== undefined) && ((typeof kept.claim) !== 'string')) {
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
  /**
   The identity of the start that took the folder, which is read once here.
   */
  const identity: ProcessIdentity = {
    pid: process.pid,
    bootId: process.bootId,
    startTicks: process.startTicks,
  };
  return {
    kind: 'an-owner',
    owner: {
      ...((kept.branch === undefined) ? {} : { branch: kept.branch, }),
      commit: kept.commit,
      ranges,
      source: kept.source,
      claimedAt: kept.claimedAt,
      unit: ((typeof kept.unit) === 'string') ? kept.unit : unit,
      claim: ((typeof kept.claim) === 'string') && (kept.claim !== '')
        ? kept.claim
        : claimOfALegacyRecord({
          claimedAt: kept.claimedAt,
          process: identity,
        },),
      process: identity,
    },
  };
}

/**
 Reads a claim name for a record written before claims were named, so that a takeover still has one exact record to name.

 @param claimedAt - when the record says the folder was taken

 @param process - the start the record names

 @returns a name no other record of this folder is likely to share

 @example
 ```ts
 const claim = claimOfALegacyRecord({ claimedAt: '2026-01-01T00:00:00.000Z', process: { pid: 9, bootId: 'b', startTicks: '1' }, },);
 ```
 */
function claimOfALegacyRecord({
  claimedAt,
  process,
}: {
  readonly claimedAt: string;
  readonly process: ProcessIdentity;
},): string {
  return `${process.startTicks}-${String(process.pid,)}-${claimedAt}`;
}
