/**
 The checkout a command of this package was run in, found by walking up from the folder it was started in to the first folder a
 checkout marks.

 Git itself cannot answer this here: this repository's Git wrapper refuses every command whose working folder is not the root of
 the repository (`require-root`), and a command of this package is run from the package's own folder, so asking git for the root
 from there fails. On 2026-11-11 that refusal ended every start of a batch: the launcher printed an uncaught error, and the batch
 kept only the last line of it. Walking up costs one read per folder and needs no git at all.

 @module
 */

import { access, } from 'node:fs/promises';
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
 Finds the checkout that marks a folder, or the one above it, and so on to the filesystem root.

 Each step is one folder nearer the root, so this walks a path's length of folders and no more.

 @param folder - the folder being looked at

 @param from - the folder the command was started in, for the message

 @returns the nearest folder holding {@link CHECKOUT_MARKER}

 @throws CheckoutNotFoundError when no folder up to the filesystem root holds one

 @example
 ```ts
 const checkout = await checkoutAt({ folder: '/repo/package/x', from: '/repo/package/x' });
 ```
 */
async function checkoutAt({
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
  return checkoutAt({
    folder: above,
    from,
  },);
}

/**
 Finds the checkout a command was started in.

 @param from - the folder the command was started in

 @returns the checkout's folder, which is the nearest folder holding {@link CHECKOUT_MARKER}

 @throws CheckoutNotFoundError when no folder from there up to the filesystem root is a checkout

 @example
 ```ts
 const checkout = await checkoutFolderOf({ from: '/repo/package/pi-shared/subagent-driver', },);
 ```
 */
export function checkoutFolderOf({ from }: { readonly from: string },): Promise<string> {
  return checkoutAt({
    folder: resolve(from,),
    from,
  },);
}
