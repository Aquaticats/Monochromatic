/**
 Writes the three files one review's brief is made of into a folder of their own and gives that folder its name in one step, so a
 review folder holds one generation whole or holds none.

 A review folder is read by a reviewer and a person, and it may be written by a second command while the first is killed at any
 instruction. Writing its three files in place leaves an old brief beside a new diff and record when the last write fails or the
 process dies before it, and the review that follows the old brief is then judged against a range it was never given.

 So the three files are written into a dot-named folder beside the place they belong (the same filesystem, so the step below is
 atomic), and `rename` gives that folder the review's name. `rename` onto a folder that holds anything fails, where it would replace
 an empty one; a review folder that is already there is therefore never written into, replaced or removed, and the caller is told
 what it holds. The names this module writes are `brief.md`, `change.diff`, `what-the-review-read.json` and `reproductions/`; it
 writes and removes none other, so a file another writer keeps in a review folder (an owner record, say) is never its to touch.

 A writer killed at any step leaves a dot-named folder with some or all of the three files in it, which is never a review: no reader
 looks for a review there, and the next call finds no review folder and writes its own generation.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  mkdir,
  readdir,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  CHANGE_DIFF_FILE_NAME,
  REPRODUCTIONS_FOLDER_NAME,
  REVIEW_BRIEF_FILE_NAME,
  REVIEW_READING_FILE_NAME,
  reviewFolderOf,
} from './reviewer-of-a-change.ts';

/**
 The code the operating system gives a name that is already taken.
 */
const NAME_IS_TAKEN_CODES: ReadonlySet<string> = new Set<string>([
  'EEXIST',
  'ENOTEMPTY',
  'ENOTDIR',
  'EISDIR',
]);

/**
 The code the operating system gives a path that is not there.
 */
const PATH_IS_NOT_THERE_CODE = 'ENOENT';

/**
 The character a folder this module writes under before it is given its name begins with, so no reader takes it for a review.
 */
const A_NAME_NO_READER_LOOKS_FOR = '.';

/**
 The three files one review's brief is made of.
 */
export type ReviewGeneration = {
  /**
   The brief the reviewer reads.
   */
  readonly brief: string;

  /**
   The diff of the change under review.
   */
  readonly diff: string;

  /**
   The record of what the review was given.
   */
  readonly record: string;
};

/**
 How one file of a generation is written, which a test replaces to fail after the files before it.
 */
export type GenerationFileWrite = (request: {
  readonly file: string;
  readonly text: string;
}) => Promise<void>;

/**
 What writing one generation did.
 */
export type GenerationPut =
  | {
    /**
     The generation was written into a folder of its own and that folder was given the review's name.
     */
    readonly kind: 'in-place';
  }
    | {
    /**
     A review folder is already there, so nothing was written, replaced or removed; this is what it holds.
     */
      readonly kind: 'already-there';

    /**
     The entries the review folder holds, in the order the operating system lists them.
     */
      readonly held: readonly string[];
    };

/**
 Writes one review's generation whole or not at all.

 @param reviewsFolder - the folder that holds one folder per review

 @param review - the change the review reads, whose name the generation's folder is given

 @param generation - the three files, with what each says

 @param write - how each file is written, which a test replaces to fail partway through

 @returns whether the generation was put in place, or the review folder that is already there and what it holds

 @throws Error when the generation could not be written, or its folder could not be given the review's name for any reason other
 than the name being taken

 @example
 ```ts
 const put = await putAGenerationInPlace({ reviewsFolder: '/reviews', review: 'a', generation, },);
 ```
 */
export async function putAGenerationInPlace({
  reviewsFolder,
  review,
  generation,
  write = writeWholeFile,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
  readonly generation: ReviewGeneration;
  readonly write?: GenerationFileWrite;
}): Promise<GenerationPut> {
  /**
   The folder the review is read from, which this gives a name to and never writes into.
   */
  const reviewFolder = reviewFolderOf({
    reviewsFolder,
    review,
  },);
  /**
   What the review folder holds, which is nothing when it is not there.
   */
  const held = await entriesOrNothingIn(reviewFolder,);
  if (held.length > 0) {
    return {
      kind: 'already-there',
      held,
    };
  }
  await mkdir(
    reviewsFolder,
    { recursive: true, },
  );
  /**
   The name the generation is written under first, which no reader looks for and no two writers share.
   */
  const writingFolder = join(
    reviewsFolder,
    `${A_NAME_NO_READER_LOOKS_FOR}writing-${String(process.pid,)}-${randomUUID()}`,
  );
  try {
    await mkdir(writingFolder,);
    await write({
      file: join(
        writingFolder,
        REVIEW_BRIEF_FILE_NAME,
      ),
      text: generation.brief,
    },);
    await write({
      file: join(
        writingFolder,
        CHANGE_DIFF_FILE_NAME,
      ),
      text: generation.diff,
    },);
    await write({
      file: join(
        writingFolder,
        REVIEW_READING_FILE_NAME,
      ),
      text: generation.record,
    },);
    await mkdir(join(
      writingFolder,
      REPRODUCTIONS_FOLDER_NAME,
    ),);
    await rename(
      writingFolder,
      reviewFolder,
    );
  }
  catch (error) {
    // Nothing of this generation stands the review's name: what it wrote is removed, and the folder it wrote under is never a review.
    await removeWhatWasWritten({ folder: writingFolder, },);
    if (NAME_IS_TAKEN_CODES.has(errorCodeOf(error,),)) {
      return {
        kind: 'already-there',
        held: await entriesOrNothingIn(reviewFolder,),
      };
    }
    throw error;
  }
  return { kind: 'in-place', };
}

/**
 Writes one file of a generation, replacing nothing, because the folder it is written in is one no other writer has.

 @param file - the file to write

 @param text - what the file says

 @throws Error when the file could not be written

 @example
 ```ts
 await writeWholeFile({ file: '/reviews/.writing-1-a/brief.md', text: 'a brief\n', },);
 ```
 */
async function writeWholeFile({
  file,
  text,
}: {
  readonly file: string;
  readonly text: string;
}): Promise<void> {
  await writeFile(
    file,
    text,
    { encoding: 'utf8', },
  );
}

/**
 Reads what a folder holds, or nothing when it is not there.

 @param folder - the folder to list

 @returns its entries' names, and nothing at all for a folder that is not there

 @example
 ```ts
 const entries = await entriesOrNothingIn('/reviews/a');
 ```
 */
async function entriesOrNothingIn(folder: string,): Promise<readonly string[]> {
  try {
    return await readdir(folder,);
  }
  catch (error) {
    if (errorCodeOf(error,) === PATH_IS_NOT_THERE_CODE) {
      return [];
    }
    throw error;
  }
}

/**
 Removes a folder this module wrote, and forgives one that is already gone.

 @param folder - the folder to remove

 @example
 ```ts
 await removeWhatWasWritten({ folder: '/reviews/.writing-1-a', },);
 ```
 */
async function removeWhatWasWritten({ folder, }: { readonly folder: string },): Promise<void> {
  await rm(
    folder,
    {
      force: true,
      recursive: true,
    },
  );
}
