/**
 What the operating system answered about a path, in the three answers every source and every kept file has: it is there, it is not
 there, or it could not be read.

 Only one answer is "not there": the operating system said there is no such file, *and* nothing above it denied the way. Everything
 else is "could not": a parent that denies traversal, a file that denies reading, a path that is not a directory where it should be,
 a name too long, a link that loops, a failure this code does not know, or no answer at all. That distinction is the one this package
 gets wrong when it is left to each reader, so it is made here once.

 @module
 */

import { lstat, } from 'node:fs/promises';
import { dirname, } from 'node:path';

//region Constants

/**
 The answer a path that is not there gets.
 */
const NOT_THERE = 'ENOENT';

/**
 The answers that say something above the last name refused the way, or that the name is not what the path needs.
 */
const DENIED_OR_WRONG_TYPE: ReadonlySet<string> = new Set([
  'EACCES',
  'EPERM',
  'ELOOP',
  'ENOTDIR',
  'EISDIR',
  'ENAMETOOLONG',
  'EOVERFLOW',
  'EIO',
]);

//endregion Constants

//region The three answers

/**
 What one path is: there, not there, or something this code could not read.
 */
export type Presence =
  | {
    readonly present: true;

    /**
     Whether the path names an ordinary file, which is what a source or a kept file has to be.
     */
    readonly ordinaryFile: boolean
  }
    | { readonly absent: true }
    | {
      readonly couldNot: true;

    /**
     What the operating system answered, in words that name no value.
     */
      readonly cause: string
    };

/**
 Says what the operating system answered about a path, from the failure its stat gave.

 Only "no such file" is not-there, and only when every parent on the way could be asked; anything else is could-not.

 @param path - the path that was asked about

 @param error - what the stat threw

 @returns the answer

 @example
 ```ts
 const presence = await presenceFromFailure({ path: '/a/auth.json', error, },);
 ```
 */
export async function presenceFromFailure({
  path,
  error,
}: {
  readonly path: string;
  readonly error: unknown;
},): Promise<Presence> {
  /**
   The code the operating system gave, when it gave one.
   */
  const code = codeOf({ error, },);
  if (code !== NOT_THERE) {
    return {
      couldNot: true,
      cause: (code === '')
        ? `asking for ${path} gave no answer this code knows`
        : `asking for ${path} was refused: ${code}`,
    };
  }
  // "No such file" says nothing about *which* name is missing: a parent that is not there is a path that cannot exist, and a parent
  // that denies the way is a path this code cannot read. So the parents are asked, nearest first.
  /**
   The folders above the path, nearest first, down to the root.
   */
  const parents = parentFolders({ path, },);
  /* oxlint-disable eslint/no-await-in-loop -- one folder at a time, nearest first, and the first answer ends the walk. */
  for (const folder of parents) {
    try {
      /**
       What that folder is.
       */
      const status = await lstat(folder,);
      // The nearest folder that is there answers for everything above it.
      return status.isDirectory()
        ? { absent: true, }
        : {
          couldNot: true,
          cause: `${folder} is not a folder, so ${path} cannot be read`,
        };
    }
    catch (parentError) {
      /**
       What the folder above answered.
       */
      const parentCode = codeOf({ error: parentError, },);
      if (parentCode === NOT_THERE) {
        continue;
      }
      return {
        couldNot: true,
        cause: (parentCode === '')
          ? `asking for ${folder}, above ${path}, gave no answer this code knows`
          : `${folder}, above ${path}, could not be read: ${parentCode}`,
      };
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return { absent: true, };
}

/**
 Reads the code a failure carries.

 @param error - what was thrown

 @returns the code, or empty text when there is none

 @example
 ```ts
 const code = codeOf({ error, },);
 ```
 */
function codeOf({ error, }: { readonly error: unknown },): string {
  if ((!Error.isError(error)) || (!('code' in error))) {
    return '';
  }
  return ((typeof error.code) === 'string') ? error.code : '';
}

/**
 The folders above a path, nearest first.

 @param path - the path

 @returns the folders, without the root repeated
 */
function parentFolders({ path, }: { readonly path: string },): readonly string[] {
  /**
   The folders found.
   */
  const folders: string[] = [];
  /**
   The folder being looked at.
   */
  let folder = dirname(path,);
  while ((folder !== '/') && (folder !== '.')) {
    folders.push(folder,);
    folder = dirname(folder,);
  }
  return folders;
}

/**
 Says whether a failure's code is one of those that deny the way or name the wrong type.

 @param code - the code

 @returns whether it is a could-not code

 @example
 ```ts
 const denied = isDeniedOrWrongType({ code: 'EACCES', },);
 ```
 */
export function isDeniedOrWrongType({ code, }: { readonly code: string },): boolean {
  return DENIED_OR_WRONG_TYPE.has(code,);
}

//endregion The three answers

//region Asking the file system

/**
 Asks the file system what a path is.

 @param path - the path to ask about

 @returns the three answers, with whether it names an ordinary file

 @example
 ```ts
 const presence = await presenceOfPath({ path: '/repo/.env.local.json', },);
 ```
 */
export async function presenceOfPath({ path, }: { readonly path: string },): Promise<Presence> {
  try {
    /**
     What the path is, without following a link.
     */
    const status = await lstat(path,);
    return {
      present: true,
      ordinaryFile: status.isFile(),
    };
  }
  catch (error) {
    return await presenceFromFailure({
      path,
      error,
    },);
  }
}

//endregion Asking the file system
