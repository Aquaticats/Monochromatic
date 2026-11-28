/**
 The guard of one run's Pi: which folders an `edit` or `write` call may not change, read from the worktree the run works in.

 A run works in a worktree, and its repository's main checkout is read-only for it: an edit left there blocks merges for everyone,
 and a build run there tests a tree that is not the run's (`doc/rules-every-pi-brief-carries.md`, "Where the run works"). The
 driver knows both folders, so it reads them here before Pi takes a prompt, and the extension it loads refuses the call before the
 tool runs (`src/main-checkout-guard-extension.ts`).

 A Pi the driver did not start never loads that extension, and a run started in no worktree is handed the word `off`
 (`src/main-checkout-guard-words.ts`): either way nothing is refused, so a person's own session behaves as it always did.

 @module
 */

import { join, } from 'node:path';
import {
  type CheckoutAndWorktree,
  CheckoutNotFoundError,
  checkoutAndWorktreeOf,
} from './checkout-of-a-command.ts';
import {
  isInside,
  resolveWithSymbolicLinks,
} from './path-with-symbolic-links-resolved.ts';

//region Constants

/**
 The path parts, below the main checkout, that hold every worktree of the repository: a path below them belongs to some run's own
 worktree and is not refused. `AGENTS.md` rule `WTU` names this folder for every worktree of this repository, and the launcher
 makes a run's worktree there (`src/start-pi-run-in-a-worktree.ts`).
 */
export const WORKTREES_FOLDER_PARTS: readonly string[] = [
  '.claude',
  'worktrees',
];

//endregion Constants

//region The guard

/**
 The folders one run's Pi may not change, or that the run has none.

 A path already resolved through the file system, so the decision reads no disk beyond the path it is given.
 */
export type MainCheckoutGuard =
  | {
    readonly armed: true;

    /**
     The repository's main checkout, which this run may read and never change.
     */
    readonly mainCheckout: string;

    /**
     The run's own worktree, whose copy of a path is the one the refusal names.
     */
    readonly worktree: string;

    /**
     The folder holding every worktree of the repository, whose paths are not refused.
     */
    readonly worktreesFolder: string;
  }
    | {
      readonly armed: false;

      /**
       Why this run has no main checkout to guard, in words a reader of the feed can act on.
       */
      readonly why: string;
    };

//endregion The guard

//region Reading the folders of a run

/**
 Reads the folders this run may not change from the folder Pi works in.

 The repository is found the way every command of this package finds it (`src/checkout-of-a-command.ts`), and both folders are then
 resolved through the file system, so a checkout reached through a symbolic link is the same folder as the one the link names.

 @param workingFolder - the folder this run's Pi works in, which is its worktree

 @returns the folders to guard, or why this run has none

 @throws when the folder that marks a checkout is there and cannot be read as one, which stops the run rather than start it
 unguarded

 @example
 ```ts
 const guard = await mainCheckoutGuardOfARun({ workingFolder: '/repo/.claude/worktrees/pi-a', });
 ```
 */
export async function mainCheckoutGuardOfARun({ workingFolder, }: {
  readonly workingFolder: string;
},): Promise<MainCheckoutGuard> {
  /**
   The repository this run works in, and the worktree it was started in when it was started in one.
   */
  const found = await repositoryOfARun({ workingFolder, },);
  if ('noRepository' in found) {
    return found.noRepository;
  }
  if (found.worktree === undefined) {
    return {
      armed: false,
      why: `the run was started in ${found.checkoutFolder}, which is the repository itself and no worktree of it, so nothing is refused`,
    };
  }
  /**
   The main checkout as the file system names it.
   */
  const mainCheckout = await resolveWithSymbolicLinks({ path: found.checkoutFolder, },);
  return {
    armed: true,
    mainCheckout,
    worktree: await resolveWithSymbolicLinks({ path: found.worktree, },),
    worktreesFolder: await resolveWithSymbolicLinks({
      path: join(
        mainCheckout,
        ...WORKTREES_FOLDER_PARTS,
      ),
    },),
  };
}

/**
 Finds the repository a run works in, or says that there is none.

 @param workingFolder - the folder this run's Pi works in

 @returns the repository and the worktree, or the guard of a run in no repository

 @throws when the folder that marks a checkout is there and cannot be read as one
 */
async function repositoryOfARun({ workingFolder, }: {
  readonly workingFolder: string;
},): Promise<CheckoutAndWorktree | { readonly noRepository: MainCheckoutGuard }> {
  try {
    return await checkoutAndWorktreeOf({ from: workingFolder, },);
  }
  catch (error) {
    // No folder from here up to the filesystem root is a checkout, so this run has no checkout it could change. A run started in
    // a scratch folder is one such run; a marker that is there and cannot be read is not, and stops the run.
    if (error instanceof CheckoutNotFoundError) {
      return {
        noRepository: {
          armed: false,
          why: `the run was started in ${workingFolder}, which is in no repository, so there is no main checkout to guard`,
        },
      };
    }
    throw error;
  }
}

/**
 Says whether a path of this run's own worktree, or of any worktree beside it, lies where the guard refuses nothing.

 @param guard - the guard of this run

 @param path - a path already resolved through the file system

 @returns whether this path is a worktree's own copy, which the run may change

 @example
 ```ts
 const allowed = isAWorktreesOwnPath({ guard, path: '/repo/.claude/worktrees/pi-a/x.ts', });
 ```
 */
export function isAWorktreesOwnPath({
  guard,
  path,
}: {
  readonly guard: Extract<MainCheckoutGuard, { readonly armed: true }>;
  readonly path: string;
},): boolean {
  /**
   Whether the path lies in a worktree beside this run's own.
   */
  const inAWorktreeBeside = isInside({
    folder: guard.worktreesFolder,
    path,
  },);
  /**
   Whether the path lies in this run's own worktree.
   */
  const inThisWorktree = isInside({
    folder: guard.worktree,
    path,
  },);
  return inAWorktreeBeside || inThisWorktree;
}

//endregion Reading the folders of a run
