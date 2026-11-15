/**
 The record a start leaves in the review folder it made: what the folder is for, which is the branch whose change it reads, the
 commit under review, the ranges, the reviewed source folder and the moment it was written.

 A folder belongs to the start that made it and is never taken over, so the record is not a claim: it says what a folder is, for a
 person reading a leftover and for the starter that finds the review of a branch among the folders of one change name.

 @module
 */

import { readFileOrNothing, } from './finished-worktrees.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 The record a start leaves in a review's folder.
 */
export type ReviewFolderRecord = {
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
   When the folder was made, as an ISO 8601 moment.
   */
  readonly claimedAt: string;
};

/**
 The answer a review folder gives when it holds no record this package can read: a symbol, so no folder's name or record is ever
 taken for it.
 */
export const NO_RECORD: unique symbol = Symbol('Review folder holds no readable record');

/**
 Reads the record a review folder holds, when it holds one that can be read.

 @param recordFile - the file the record is kept in

 @returns the record, or nothing when the folder holds none that can be read

 @example
 ```ts
 const record = await recordIn({ recordFile: '/reviews/a/the-record-of-this-review.json', },);
 ```
 */
export async function recordIn({
  recordFile,
}: {
  readonly recordFile: string;
},): Promise<ReviewFolderRecord | typeof NO_RECORD> {
  /**
   What the file holds.
   */
  const shown = (await readFileOrNothing(recordFile,)).trim();
  if (shown === '') {
    return NO_RECORD;
  }
  return recordOf({ shown, },);
}

/**
 Reads a record from the text it was written as.

 A file that cannot be read as a record is treated as no record: only this package writes it, and a folder whose record cannot be
 read is one the starter cannot attribute to a branch, so it is left as a leftover.

 @param shown - the file's text

 @returns the record, or nothing when the text is not one

 @example
 ```ts
 const record = recordOf({ shown: '{"branch":"fix/a","commit":"abc","ranges":[],"source":"a","claimedAt":"now"}', },);
 ```
 */
function recordOf({ shown, }: { readonly shown: string },): ReviewFolderRecord | typeof NO_RECORD {
  /**
   The record, of any shape.
   */
  const kept: unknown = JSON.parse(shown,);
  if (!isRecord(kept,)) {
    return NO_RECORD;
  }
  if ((typeof kept.commit) !== 'string') {
    return NO_RECORD;
  }
  if ((typeof kept.source) !== 'string') {
    return NO_RECORD;
  }
  if ((typeof kept.claimedAt) !== 'string') {
    return NO_RECORD;
  }
  if ((kept.branch !== undefined) && ((typeof kept.branch) !== 'string')) {
    return NO_RECORD;
  }
  if (!Array.isArray(kept.ranges)) {
    return NO_RECORD;
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
    return NO_RECORD;
  }
  return {
    ...((kept.branch === undefined) ? {} : { branch: kept.branch, }),
    commit: kept.commit,
    ranges,
    source: kept.source,
    claimedAt: kept.claimedAt,
  };
}

/**
 Writes a record as the text a review folder keeps.

 @param record - the record to write

 @returns the text, which ends with a newline

 @example
 ```ts
 const text = recordTextOf({ commit: 'abc', ranges: ['main...fix/a'], source: 'a', claimedAt: 'now', },);
 ```
 */
export function recordTextOf(record: ReviewFolderRecord,): string {
  return `${JSON.stringify(
    record,
    undefined,
    2,
  )}\n`;
}
