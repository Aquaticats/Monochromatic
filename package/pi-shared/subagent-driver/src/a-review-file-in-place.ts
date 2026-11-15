/**
 Puts a review's own text in place as its `review.md`, whole or not at all, by writing it under a name of its own and linking that
 name in with a step that cannot replace the file.

 A review's own file is read by a person and may be written by a second command at the same time, and a process can be killed at any
 instruction. A write straight to `review.md` leaves half a review for a reader to take as the whole, and a second writer's
 truncating write loses what the first wrote. So the text goes to a temporary name inside the review's own folder (the same
 filesystem, so the link below is atomic), and `link` puts it at `review.md`: `link` fails when that name is taken, where `rename`
 would replace what is there.

 The first writer wins: it links its whole text in and removes its temporary name. A second writer's link fails with the code a
 taken name carries, so it removes its own temporary name and answers that the review is already there, holding another review's
 text and none of its own. A writer killed before its link leaves a dot name that is never a review, so a retry writes the whole
 text and links it in.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  link,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the operating system gives a name that is already taken.
 */
const NAME_IS_TAKEN_CODE = 'EEXIST';

/**
 How one review's text is written to the name it is first put under, which a test replaces to fail after the first bytes.
 */
export type ReviewFileWrite = (request: {
  readonly file: string;
  readonly text: string;
}) => Promise<void>;

/**
 What putting a review's text in place did.
 */
export type ReviewPut =
  | {
    /**
     The text was written under its own name and linked in as the review, which held nothing before it.
     */
    readonly kind: 'in-place';
  }
    | {
    /**
     Another writer's review was already there, so this text was not put in place and the review holds that one.
     */
      readonly kind: 'already-there';
    };

/**
 Writes a review's text as its own file, whole or not at all, and says whether the review already held another text.

 @param folder - the review's own folder, which the temporary name is written in and the review file lives in

 @param reviewFile - the file the review is read from, which this never replaces

 @param text - the review's whole text, with the note that says where it came from

 @param write - how the text is written to its first name, which a test replaces to fail partway through

 @returns whether the text was put in place or a review was already there

 @throws Error when the text could not be written, or the name could not be linked in for a reason other than being taken

 @example
 ```ts
 const put = await putAReviewInPlace({ folder: '/reviews/a', reviewFile: '/reviews/a/review.md', text: 'a review\n', },);
 ```
 */
export async function putAReviewInPlace({
  folder,
  reviewFile,
  text,
  write = writeWholeText,
}: {
  readonly folder: string;
  readonly reviewFile: string;
  readonly text: string;
  readonly write?: ReviewFileWrite;
}): Promise<ReviewPut> {
  /**
   The name the text is written under first, which is a dot name no reader looks for and no two writers share.
   */
  const temporaryFile = join(
    folder,
    `.review.md-${String(process.pid,)}-${randomUUID()}.tmp`,
  );
  try {
    await write({
      file: temporaryFile,
      text,
    },);
  }
  catch (error) {
    // Nothing stands as a review: the half-written name is removed so no reader and no retry has to think about it.
    await removeWhatWasWritten({ file: temporaryFile, },);
    throw error;
  }
  try {
    await link(
      temporaryFile,
      reviewFile,
    );
  }
  catch (error) {
    await removeWhatWasWritten({ file: temporaryFile, },);
    if (errorCodeOf(error,) === NAME_IS_TAKEN_CODE) {
      return { kind: 'already-there', };
    }
    throw error;
  }
  await removeWhatWasWritten({ file: temporaryFile, },);
  return { kind: 'in-place', };
}

/**
 Writes a text to its first name, replacing nothing, because the name is one no two writers share.

 @param file - the name the text is written under

 @param text - the whole text

 @throws Error when the text could not be written

 @example
 ```ts
 await writeWholeText({ file: '/reviews/a/.review.md-1-a.tmp', text: 'a review\n', },);
 ```
 */
async function writeWholeText({
  file,
  text,
}: {
  readonly file: string;
  readonly text: string;
}): Promise<void> {
  await writeFile(
    file,
    text,
    {
      encoding: 'utf8',
      flag: 'wx',
    },
  );
}

/**
 Removes a name this module wrote, forgiving a name that is already gone.

 A name that is already gone is the state this leaves, so `force` is what this needs: removing nothing is not a failure, and any
 other failure to remove is thrown, because a review folder that cannot be cleaned is a fact its caller should see.

 @param file - the name to remove

 @example
 ```ts
 await removeWhatWasWritten({ file: '/reviews/a/.review.md-1-a.tmp', },);
 ```
 */
async function removeWhatWasWritten({ file, }: { readonly file: string },): Promise<void> {
  await rm(
    file,
    { force: true, },
  );
}
