/**
 Reads the record of what a review was given, from its path, without ever waiting for it.

 The path may be a named pipe with no writer, which a plain read waits on forever; it may be a directory, a socket or a file of any
 size; and it may grow while it is read. So it is opened without blocking, asked of its own handle whether it is a regular file, and
 read up to the largest record this package writes. A pipe, a directory or a socket is answered as nothing readable before a byte is
 read, and a record longer than that bound is answered the same way, which is what the project's rule that nothing waits forever asks
 and what its answer for a record that cannot be read says.

 A record that cannot be read is this review's answer alone: the caller then judges the next review, so one review's record never
 discards another's result.

 @module
 */

import { constants, } from 'node:fs';
import { open, } from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import {
  type ReviewReading,
  ReviewReadingError,
  readingIn,
} from './material-of-a-review.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The largest record this package writes, beyond which nothing readable is kept there: the largest of the 358 records under
 `$HOME/temp/agent/change-reviews`, measured 2026-11-16, is 802 bytes, and a record of this size is not one this package wrote. The
 read stops here, so a record that grows while it is read cannot hold the save.
 */
const THE_LARGEST_RECORD_A_REVIEW_GETS_BYTES = 65_536;

/**
 What a review's record turned out to be.
 */
type KeptRecord =
  | {
    /**
     The record's text.
     */
    readonly kind: 'text';

    /**
     What the record says.
     */
    readonly text: string;
  }
    | {
      /**
       Nothing readable is kept at the record's path.
       */
      readonly kind: 'unreadable';

      /**
       Why nothing is read, in words a reader of the answer can act on.
       */
      readonly why: string;
    };

/**
 Reads what a review recorded it read, or why nothing readable is kept at its path.

 @param file - the file that records what the review was given

 @returns the reading, or why nothing readable is kept there

 @example
 ```ts
 const read = await readTheRecordOfAReview({ file: '/reviews/a/what-the-review-read.json', },);
 ```
 */
export async function readTheRecordOfAReview({ file, }: { readonly file: string },): Promise<{
  readonly reading?: ReviewReading;
  readonly readingProblem?: string;
}> {
  try {
    /**
     The record, as its path gave it.
     */
    const record = await recordTextOf({ file, },);
    if (record.kind === 'unreadable') {
      return { readingProblem: record.why, };
    }
    return {
      reading: readingIn({
        shown: record.text,
        file,
      },),
    };
  }
  catch (error) {
    // A record that is not there is the plain answer; anything else names its code, so a reader knows what to look at.
    /**
     The code the failure carries, when it carries one.
     */
    const code = errorCodeOf(error,);
    if (code === MISSING_PATH_CODE) {
      return {};
    }
    return {
      readingProblem: (code === '') ? String(error,) : code,
    };
  }
}

/**
 Reads a review's record from its path without ever waiting for it.

 @param file - the file that records what the review was given

 @returns the record's text, or why nothing readable is kept there

 @example
 ```ts
 const record = await recordTextOf({ file: '/reviews/a/what-the-review-read.json', },);
 ```
 */
async function recordTextOf({ file, }: { readonly file: string },): Promise<KeptRecord> {
  /**
   The record's path, opened for reading and without waiting for a writer of it.
   */
  const handle = await open(
    file,
    constants.O_RDONLY | constants.O_NONBLOCK,
  );
  /**
   The handle, closed when this scope ends, so no failure leaves it open.
   */
  await using opened = {
    async [Symbol.asyncDispose](): Promise<void> {
      await handle.close();
    },
  };
  /**
   What the opened path is, asked of the handle so that a path swapped between opening and asking cannot be read as another kind.
   */
  const question = await handle.stat();
  if (!question.isFile()) {
    return {
      kind: 'unreadable',
      why: 'its path is no regular file',
    };
  }
  /**
   The bytes read so far, with room for one more than the largest record, so a longer one is seen as longer.
   */
  const buffer = Buffer.alloc(THE_LARGEST_RECORD_A_REVIEW_GETS_BYTES + 1,);
  /**
   How many bytes of it are read.
   */
  const total = await (async function readUpToTheLargestRecord(): Promise<number> {
    /**
     How many bytes of the record are read so far.
     */
    let read = 0;
    while (read < buffer.length) {
    /**
     What one read of the handle answered.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one read after another of the same handle, each appending to what the last answered, because one read may answer fewer bytes than the file holds; see doc/lint/sequential-awaits.md
      const { bytesRead, } = await handle.read(
        buffer,
        read,
        buffer.length - read,
        read,
      );
      if (bytesRead === 0) {
        return read;
      }
      read += bytesRead;
    }
    return read;
  })();
  if (total > THE_LARGEST_RECORD_A_REVIEW_GETS_BYTES) {
    return {
      kind: 'unreadable',
      why: `the record is larger than the ${String(THE_LARGEST_RECORD_A_REVIEW_GETS_BYTES,)} bytes this package writes`,
    };
  }
  return {
    kind: 'text',
    text: buffer.subarray(
      0,
      total,
    )
      .toString('utf8',),
  };
}
