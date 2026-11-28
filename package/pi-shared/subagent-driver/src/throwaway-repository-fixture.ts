/**
 A throwaway repository, its worktree, and the folders a test of the main-checkout guard reads: a home folder of the test's own, the
 repository's main checkout, one worktree of it made the way Git makes one, a sibling folder whose name begins with the main
 checkout's own, and a run's scratch folder.

 Nothing here is Git: the two files a worktree marks itself with are written as Git writes them, which is all this package reads to
 find the repository a command was started in (`src/checkout-of-a-command.ts`). The shapes are a capture of a real worktree's own
 `.git` file, `gitdir: <repository>/.git/worktrees/<name>`, and of its `commondir` file, `../..`.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 The name of the file each copy of this repository holds, which a test edits.
 */
export const A_FILE_NAME = 'a-file.ts';

/**
 The text the main checkout's copy holds, which a refused edit never replaces.
 */
export const MAIN_CHECKOUT_TEXT = 'the main checkout copy\n';

/**
 The text the worktree's copy holds, which an allowed edit replaces.
 */
export const WORKTREE_TEXT = 'the worktree copy\n';

/**
 The name of the one worktree a test's repository holds.
 */
const WORKTREE_NAME = 'pi-a';

/**
 A folder a test's repository is made under, named for what it holds.
 */
const HOME_FOLDER_PREFIX = 'pi-run-main-checkout-';

/**
 A throwaway repository and the folders around it.
 */
export type ThrowawayRepository = {
  /**
   The home folder of the test's own, which the repository lies under.
   */
  readonly home: string;

  /**
   The repository's main checkout, which a run may read and may not change.
   */
  readonly repository: string;

  /**
   The one worktree of the repository, which a run works in and may change.
   */
  readonly worktree: string;

  /**
   The folder holding every worktree of the repository.
   */
  readonly worktreesFolder: string;

  /**
   A folder whose name begins with the main checkout's own and holds a file of the same name.
   */
  readonly sibling: string;

  /**
   A run's scratch folder, which a run writes everything outside its worktree into.
   */
  readonly scratchFolder: string;

  /**
   A folder every process of a run would be read from, which holds none unless a test writes one.
   */
  readonly processFolder: string;

  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a throwaway repository with one worktree of it.

 @returns the folders, and a disposer that removes the whole home folder

 @example
 ```ts
 await using repository = await makeThrowawayRepository();
 ```
 */
export async function makeThrowawayRepository(): Promise<ThrowawayRepository> {
  /**
   The home folder the test owns, under the harness's scratch root rather than the system's temporary folder.
   */
  const home = await mkdtemp(join(
    scratchRoot(),
    HOME_FOLDER_PREFIX,
  ),);
  /**
   The repository's main checkout, which every other folder is read relative to.
   */
  const repository = join(
    home,
    'Monochromatic',
  );
  /**
   The folder holding every worktree of the repository.
   */
  const worktreesFolder = join(
    repository,
    '.claude',
    'worktrees',
  );
  /**
   The one worktree of the repository.
   */
  const worktree = join(
    worktreesFolder,
    WORKTREE_NAME,
  );
  /**
   The Git directory the worktree's marker file names.
   */
  const worktreeGitDirectory = join(
    repository,
    '.git',
    'worktrees',
    WORKTREE_NAME,
  );
  /**
   A folder whose name begins with the main checkout's own, which a guard that compared the characters of two paths would refuse by
   mistake.
   */
  const sibling = join(
    home,
    'Monochromatic-sibling',
  );
  /**
   A run's scratch folder.
   */
  const scratchFolder = join(
    home,
    'temp',
    'agent',
    'a-scratch',
  );
  /**
   The folder a run's process table is read from, which holds nothing.
   */
  const processFolder = join(
    home,
    'processes',
  );
  await Promise.all([
    mkdir(
      worktreeGitDirectory,
      { recursive: true, },
    ),
    mkdir(
      join(
        repository,
        'package',
      ),
      { recursive: true, },
    ),
    mkdir(
      join(
        worktree,
        'package',
      ),
      { recursive: true, },
    ),
    mkdir(
      join(
        sibling,
        'package',
      ),
      { recursive: true, },
    ),
    mkdir(
      scratchFolder,
      { recursive: true, },
    ),
    mkdir(
      processFolder,
      { recursive: true, },
    ),
  ],);
  await Promise.all([
    writeFile(
      join(
        repository,
        'package',
        A_FILE_NAME,
      ),
      MAIN_CHECKOUT_TEXT,
    ),
    writeFile(
      join(
        worktree,
        'package',
        A_FILE_NAME,
      ),
      WORKTREE_TEXT,
    ),
    writeFile(
      join(
        sibling,
        'package',
        A_FILE_NAME,
      ),
      MAIN_CHECKOUT_TEXT,
    ),
    // The two files Git writes into a worktree, in the shapes a capture of this repository's own worktree holds.
    writeFile(
      join(
        worktree,
        '.git',
      ),
      `gitdir: ${worktreeGitDirectory}\n`,
    ),
    writeFile(
      join(
        worktreeGitDirectory,
        'commondir',
      ),
      '../..\n',
    ),
  ],);
  return {
    home,
    repository,
    worktree,
    worktreesFolder,
    sibling,
    scratchFolder,
    processFolder,
    async [Symbol.asyncDispose]() {
      await rm(
        home,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}
