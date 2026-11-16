/**
 The folders one change's reviews keep: the first is the folder named after the change, and a start that finds that name taken makes
 the next free name beside it, `<change>-2`, `<change>-3` and so on. A folder is made by an exclusive create and belongs to the
 start that made it for good: no start takes another's folder, judges whether its starter lives, or reuses its record. A folder a
 killed start left is a leftover with its record, and nothing on a start's path removes it.

 The generation is part of the review's name, so the run, its branch, its unit and its worktree all carry it: a second start of one
 change cannot share a run name or a unit with the first. A folder of another change whose name happens to be a generation name is
 told apart by the branch its record names; the starter reads records, not names alone.

 @module
 */

import type { Dirent, } from 'node:fs';
import {
  mkdir,
  readdir,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { isThere, } from './newest-finished-run-of-a-name.ts';
import {
  recordFileOfAReview,
  reviewFileOfAReview,
  reviewFolderOf,
} from './reviewer-of-a-change.ts';
import { recordIn, } from './the-record-of-a-review-folder.ts';

/**
 The code the operating system gives a path that already exists.
 */
const EXISTS_CODE = 'EEXIST';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The review name one generation of a change keeps.

 @param review - the change's name

 @param generation - which start of it this is, 0 for the first

 @returns the review name, which carries the generation after the first

 @example
 ```ts
 const name = reviewNameOf({ review: 'rewrite-the-gate', generation: 2, },);
 ```
 */
export function reviewNameOf({
  review,
  generation,
}: {
  readonly review: string;
  readonly generation: number;
},): string {
  return (generation === 0) ? review : `${review}-${String(generation,)}`;
}

/**
 The answer for a name that is not a generation of the change asked about: a symbol, so no number a folder's name yields is taken
 for it.
 */
export const NOT_A_GENERATION: unique symbol = Symbol('Generation name belongs to another change');

/**
 The answer for a branch whose folders wrote no review file: a symbol, so no review name is ever taken for it.
 */
export const NO_REVIEWED_FOLDER: unique symbol = Symbol('Folder of this branch wrote no review.md');

/**
 Reads which generation of a change a review name is, when it is one.

 A name is a generation name when it is the change's own name or the change's name, a hyphen, and a number with no leading zero.
 Any other name belongs to another change, which the record settles.

 @param review - the change's name

 @param name - the folder's name

 @returns the generation, or nothing when the name is not one of this change's

 @example
 ```ts
 const generation = generationInAName({ review: 'rewrite-the-gate', name: 'rewrite-the-gate-2', },);
 ```
 */
export function generationInAName({
  review,
  name,
}: {
  readonly review: string;
  readonly name: string;
},): number | typeof NOT_A_GENERATION {
  if (name === review) {
    return 0;
  }
  /**
   What follows the change's name and a hyphen.
   */
  const suffix = name.startsWith(`${review}-`,)
    ? name.slice(review.length + 1,)
    : undefined;
  if ((suffix === undefined) || (!isAPositiveNumberWithoutALeadingZero({ shown: suffix, },))) {
    return NOT_A_GENERATION;
  }
  /**
   The generation the suffix names, where the first start is the change's own name and the next is 2.
   */
  const generation = Number(suffix,);
  return (generation === 1) ? NOT_A_GENERATION : generation;
}

/**
 Says whether text is a number a generation is written as: digits with no leading zero.

 @param shown - the text

 @returns whether it is such a number

 @example
 ```ts
 const isOne = isAPositiveNumberWithoutALeadingZero({ shown: '12', },);
 ```
 */
function isAPositiveNumberWithoutALeadingZero({ shown, }: { readonly shown: string },): boolean {
  if ((shown === '') || (shown.startsWith('0'))) {
    return false;
  }
  for (const character of shown) {
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}

/**
 Takes the first free review folder for a change, by making it, and answers the name it took.

 The folder is made by `mkdir`, which only one start can do for one name: a start that finds the name taken does not judge the start
 that took it, and does not touch it; it tries the next generation. A folder made here is this start's for good.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the change's name

 @returns the name this start's folder carries, and its path

 @throws whatever `mkdir` throws for a reason other than an existing name

 @example
 ```ts
 const taken = await takeTheNextReviewFolder({ reviewsFolder: '/reviews', review: 'rewrite-the-gate', },);
 ```
 */
export async function takeTheNextReviewFolder({
  reviewsFolder,
  review,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
},): Promise<{
  readonly review: string;
  readonly folder: string
}> {
  /**
   The folder the first generation keeps, whose parent must exist before an exclusive create can be tried in it: a change name may
   hold a slash, so the parent may be more than the reviews folder.
   */
  const firstFolder = reviewFolderOf({
    reviewsFolder,
    review,
  },);
  await mkdir(
    dirname(firstFolder,),
    { recursive: true, },
  );
  /* oxlint-disable eslint/no-await-in-loop -- one name is tried, and if another start made it the next name is tried; there is no
     list to read first, because the names a start may take are unbounded. See doc/lint/sequential-awaits.md. */
  for (let generation = 0; ; generation = (generation === 0) ? 2 : (generation + 1)) {
    /**
     The review name of this generation.
     */
    const name = reviewNameOf({
      review,
      generation,
    },);
    /**
     The folder that name keeps.
     */
    const folder = reviewFolderOf({
      reviewsFolder,
      review: name,
    },);
    try {
      await mkdir(folder,);
      return {
        review: name,
        folder,
      };
    }
    catch (error) {
      if (errorCodeOf(error,) !== EXISTS_CODE) {
        throw error;
      }
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}

/**
 Lists a folder, or says it holds nothing when it is not there.

 @param folder - the folder to list

 @returns its entries, or none when the folder disappeared

 @example
 ```ts
 const entries = await entriesIn({ folder: '/reviews/a', },);
 ```
 */
async function entriesIn({ folder, }: { readonly folder: string },): Promise<readonly Dirent[]> {
  try {
    return await readdir(
      folder,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw error;
  }
}

/**
 Reads every folder below a root, as names relative to it, without recursion.

 A reviews folder that is not there has no folders below it, which is a first start rather than a failure; a folder that disappears
 between two listings is left out.

 @param root - the folder to walk

 @returns every folder below it, as relative names

 @example
 ```ts
 const folders = await theFoldersBelow({ root: '/reviews', },);
 ```
 */
async function theFoldersBelow({ root, }: { readonly root: string },): Promise<readonly string[]> {
  /**
   The folders found so far.
   */
  const folders: string[] = [];
  /**
   The folders still to list, each a name relative to the root.
   */
  const toList: string[] = [''];
  /* oxlint-disable eslint/no-await-in-loop -- one folder is listed, then the next, a stack at a time; each listing is a separate
     directory and none may be taken at once. See doc/lint/sequential-awaits.md. */
  while (toList.length > 0) {
    /**
     The folder to list, as a name relative to the root.
     */
    const relative = toList.pop() ?? '';
    /**
     What that folder holds, or nothing when it disappeared between two listings.
     */
    const entries = await entriesIn({ folder: join(
      root,
      relative,
    ), },);
    for (const entry of entries) {
      if (!entry.isDirectory()) {
        continue;
      }
      /**
       The folder's name, relative to the root.
       */
      const name = (relative === '') ? entry.name : `${relative}/${entry.name}`;
      folders.push(name,);
      toList.push(name,);
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return folders;
}

/**
 Finds the newest review of a branch among the folders of one change name, when one wrote its review file.

 A folder belongs to this branch only when its record names the branch: a folder of another change whose name is a generation name is
 left out. A generation whose record names the branch but which holds no review file is a leftover, and the search goes on to the
 generations older than it.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the change's name

 @param branch - the branch whose review is looked for

 @returns the review name that wrote its review file, or nothing

 @example
 ```ts
 const reviewed = await theReviewOfABranch({ reviewsFolder: '/reviews', review: 'rewrite-the-gate', branch: 'fix/rewrite-the-gate', },);
 ```
 */
export async function theReviewOfABranch({
  reviewsFolder,
  review,
  branch,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
  readonly branch: string;
},): Promise<string | typeof NO_REVIEWED_FOLDER> {
  /**
   The folders below the reviews folder, as names relative to it, which include nested change names.
   */
  const folders = await theFoldersBelow({ root: reviewsFolder, },);
  /**
   The names that are generations of this change, newest first.
   */
  const generations: {
    readonly name: string;
    readonly generation: number
  }[] = [];
  for (const name of folders) {
    /**
     Which generation of this change this folder's name is.
     */
    const generation = generationInAName({
      review,
      name,
    },);
    if ((typeof generation) !== 'number') {
      continue;
    }
    generations.push({
      name,
      generation,
    },);
  }
  generations.sort(function newestFirst(
    left,
    right,
  ) {
    return right.generation - left.generation;
  },);
  /* oxlint-disable eslint/no-await-in-loop -- one generation is read, then the next older, until one is this branch's review; the
     record of each is what settles whether it is. See doc/lint/sequential-awaits.md. */
  for (const { name, } of generations) {
    /**
     What the folder records of itself.
     */
    const record = await recordIn({
      recordFile: recordFileOfAReview({
        reviewsFolder,
        review: name,
      },),
    },);
    if (((typeof record) === 'symbol') || (record.branch !== branch)) {
      continue;
    }
    if (await isThere(reviewFileOfAReview({
      reviewsFolder,
      review: name,
    },),)) {
      return name;
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return NO_REVIEWED_FOLDER;
}
