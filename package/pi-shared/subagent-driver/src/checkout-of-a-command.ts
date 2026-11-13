/**
 The checkout a command of this package was run in, found by walking up from the folder it was started in to the first folder a
 checkout marks, and then reading which repository that folder belongs to.

 Git itself cannot answer this here: this repository's Git wrapper refuses every command whose working folder is not the root of
 the repository (`require-root`), and a command of this package is run from the package's own folder, so asking git for the root
 from there fails. On 2026-11-11 that refusal ended every start of a batch: the launcher printed an uncaught error, and the batch
 kept only the last line of it. Walking up costs one read per folder and needs no git at all.

 A linked worktree marks itself with a file rather than a folder, and that file names the Git directory it was given, which is a
 folder inside the repository's own Git directory. Every command of this package therefore answers the repository, from the
 repository or from any of its worktrees: a worktree that answered itself would put a run's worktree inside a worktree, and a
 checkout default read in one place would name a different folder than the same default read in another.

 @module
 */

import {
  access,
  readFile,
  stat,
} from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The name that marks the folder a checkout starts at, which is a folder in the main checkout and a file in a worktree made from it.
 */
export const CHECKOUT_MARKER: string = '.git';

/**
 The prefix of the line a worktree's marker file holds, which names the Git directory that worktree was given.
 */
const GITDIR_PREFIX = 'gitdir:';

/**
 The file inside a worktree's Git directory that names the Git directory every worktree of the repository shares.
 */
const COMMONDIR_FILE_NAME = 'commondir';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 No folder above the one a command was started in is a checkout, so nothing says where a run's worktree belongs.
 */
export class CheckoutNotFoundError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'CheckoutNotFoundError';
}

/**
 Says whether a path is there, without reading it.

 @param path - the path to look for

 @returns whether it is there

 @example
 ```ts
 const there = await isThere('/repo/.git');
 ```
 */
async function isThere(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Finds the nearest folder that marks itself as a checkout, or the one above it, and so on to the filesystem root.

 Each step is one folder nearer the root, so this walks a path's length of folders and no more.

 @param folder - the folder being looked at

 @param from - the folder the command was started in, for the message

 @returns the nearest folder holding {@link CHECKOUT_MARKER}

 @throws CheckoutNotFoundError when no folder up to the filesystem root holds one

 @example
 ```ts
 const marked = await markedFolderOf({ folder: '/repo/package/x', from: '/repo/package/x' });
 ```
 */
async function markedFolderOf({
  folder,
  from,
}: {
  readonly folder: string;
  readonly from: string;
},): Promise<string> {
  if (await isThere(join(
    folder,
    CHECKOUT_MARKER,
  ),)) {
    return folder;
  }
  /**
   The folder one step nearer the filesystem root.
   */
  const above = dirname(folder,);
  if (above === folder) {
    throw new CheckoutNotFoundError(
      `no folder from ${from} up to ${folder} holds a ${CHECKOUT_MARKER}, so which checkout a run's worktree belongs in cannot be read. Run the launcher from inside the checkout, or name the checkout with --checkout-folder.`,
    );
  }
  return markedFolderOf({
    folder: above,
    from,
  },);
}

/**
 Reads the Git directory a worktree's marker file names.

 @param marker - the marker file

 @param markedFolder - the folder holding it, which a relative path in the file is read from

 @returns the Git directory's path

 @throws CheckoutNotFoundError when the file names no Git directory

 @example
 ```ts
 const gitDirectory = await gitDirectoryOf({ marker: '/repo/pi-a/.git', markedFolder: '/repo/pi-a', },);
 ```
 */
async function gitDirectoryOf({
  marker,
  markedFolder,
}: {
  readonly marker: string;
  readonly markedFolder: string;
},): Promise<string> {
  /**
   The line the marker file holds.
   */
  const named = (await readFile(
    marker,
    'utf8',
  )).split('\n',)
    .map(function withoutTrailingSpace(line,) {
      return line.trim();
    },)
    .find(function namesAGitDirectory(line,) {
      return line.startsWith(GITDIR_PREFIX,);
    },);
  if (named === undefined) {
    throw new CheckoutNotFoundError(
      `${marker} is a file that names no ${GITDIR_PREFIX} path, so which repository ${markedFolder} belongs to cannot be read. Name the checkout with --checkout-folder.`,
    );
  }
  return resolve(
    markedFolder,
    named.slice(GITDIR_PREFIX.length,)
      .trim(),
  );
}

/**
 Reads the Git directory every worktree of a repository shares.

 Git writes the shared directory's path into the worktree's own Git directory, and a worktree whose directory names none keeps it
 the way Git always has: two folders above the worktree's own.

 @param gitDirectory - the Git directory of the worktree, or of the repository itself

 @returns the shared Git directory's path

 @example
 ```ts
 const common = await commonDirectoryOf({ gitDirectory: '/repo/.git/worktrees/pi-a', },);
 ```
 */
async function commonDirectoryOf({ gitDirectory, }: { readonly gitDirectory: string },): Promise<string> {
  /**
   The file naming the shared Git directory, when Git wrote one.
   */
  const named = join(
    gitDirectory,
    COMMONDIR_FILE_NAME,
  );
  if (await isThere(named,)) {
    return resolve(
      gitDirectory,
      (await readFile(
        named,
        'utf8',
      )).trim(),
    );
  }
  return resolve(
    gitDirectory,
    '..',
    '..',
  );
}

/**
 Finds the repository the checkout a command was started in belongs to.

 @param from - the folder the command was started in

 @returns the repository's folder, which is the folder holding the Git directory every worktree of it shares

 @throws CheckoutNotFoundError when no folder from there up to the filesystem root is a checkout

 @example
 ```ts
 const repository = await checkoutFolderOf({ from: '/repo/.claude/worktrees/pi-a/package/x', },);
 ```
 */
export async function checkoutFolderOf({ from }: { readonly from: string },): Promise<string> {
  /**
   The nearest folder that marks itself as a checkout, which is the worktree when a command is run inside one.
   */
  const markedFolder = await markedFolderOf({
    folder: resolve(from,),
    from,
  },);
  /**
   The marker itself, which is a folder in a repository and a file in a linked worktree.
   */
  const marker = join(
    markedFolder,
    CHECKOUT_MARKER,
  );
  if ((await stat(marker,)).isDirectory()) {
    return markedFolder;
  }
  /**
   The Git directory this worktree was given, inside the repository's own.
   */
  const gitDirectory = await gitDirectoryOf({
    marker,
    markedFolder,
  },);
  return dirname(await commonDirectoryOf({ gitDirectory, },),);
}
