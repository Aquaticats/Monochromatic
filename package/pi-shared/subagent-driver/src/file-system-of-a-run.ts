/**
 The file system a run's unit gets: the checkout its launcher works in is read-only, and the folders the run must write stay
 writable.

 Why: on 2026-11-14 two runs wrote into the main checkout, one of them for fifty minutes, because a brief that names a
 main-checkout path to read does not stop the same path from being written, and nothing in the unit refused it. A run's own
 worktree is copied from the checkout, so a run that edits the checkout edits the copy every later worktree is made from.

 What stays writable is read from the checkout the launcher works in and from Git's own record of it: the run's worktree, the Git
 directory its commits, fetches and pushes write, the Git directory every worktree of the repository shares, the checkout's
 `.cache` (the folder this repository's tools install their launchers and guards into), and the run's own folder. Everything else
 under the checkout is refused by the operating system, which is what the agent sees.

 Read-only folders are systemd's `ReadOnlyPaths=`; the folders that must stay writable are its `ReadWritePaths=`, which
 systemd applies inside the read-only ones. A path there that is not there stops the unit from starting, so the one folder that
 may be absent carries systemd's `-` prefix, which tells it to ignore a path it cannot find.

 @module
 */

import { join, } from 'node:path';
import {
  CHECKOUT_MARKER,
  CheckoutNotFoundError,
  gitFoldersOf,
} from './checkout-of-a-command.ts';

/**
 The prefix that tells systemd to ignore a path that is not there.
 */
const MISSING_PATH_IS_FINE: string = '-';

/**
 The folder of the checkout this repository's tools cache their binaries and guards in.
 */
const CACHE_FOLDER_NAME: string = '.cache';

/**
 What a run's unit may and may not write.

 @example
 ```ts
 const fileSystem = await fileSystemOfARun({ checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', worksInTheCheckoutItself: false },);
 ```
 */
export type FileSystemOfARun = {
  /**
   The folders a unit of this run cannot write, which the operating system refuses with a read-only file system.
   */
  readonly foldersAUnitCannotWrite: readonly string[];

  /**
   The folders a unit of this run must write, which systemd makes writable inside the read-only ones.
   */
  readonly foldersAUnitMustWrite: readonly string[];
};

/**
 Reads the Git folders a run's commits, fetches and pushes write, from the checkout its launcher works in.

 A checkout Git cannot read still gets its own `.git` named, with systemd's `-` prefix so a folder that is not there is ignored:
 a start is not refused for it, and the checkout stays read-only around whatever it does hold.

 @param checkoutFolder - the checkout the launcher works in

 @returns the Git directories, each named once

 @throws Error when the checkout's marker is there but unreadable, which is a failure this reads nothing about

 @example
 ```ts
 const folders = await gitFoldersToWrite({ checkoutFolder: '/repo', },);
 ```
 */
async function gitFoldersToWrite({ checkoutFolder }: { readonly checkoutFolder: string },): Promise<readonly string[]> {
  try {
    /**
     The Git directory of the checkout and the one every worktree of its repository shares.
     */
    const {
      gitDirectory,
      commonDirectory,
    } = await gitFoldersOf({ folder: checkoutFolder, },);
    return [
      gitDirectory,
      commonDirectory,
    ];
  }
  catch (error) {
    if (!(error instanceof CheckoutNotFoundError)) {
      throw error;
    }
    /**
     The marker itself, named so that a folder that is not there is ignored.
     */
    const marker = join(
      checkoutFolder,
      CHECKOUT_MARKER,
    );
    return [`${MISSING_PATH_IS_FINE}${marker}`,];
  }
}

/**
 Reads what one run's unit may and may not write.

 @param checkoutFolder - the checkout the run's launcher works in, which its run must not change

 @param worktree - where the run works, which is a worktree of that checkout or the checkout itself

 @param runFolder - where the driver writes this run

 @param worksInTheCheckoutItself - whether the run works in the checkout itself, in which case it gets no worktree of its own to write

 @returns the folders the unit cannot write and the folders it must write

 @throws CheckoutNotFoundError when the run's worktree names no Git directory, in which case nothing says where its commits go

 @example
 ```ts
 const fileSystem = await fileSystemOfARun({ checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', worksInTheCheckoutItself: false },);
 ```
 */
export async function fileSystemOfARun({
  checkoutFolder,
  worktree,
  runFolder,
  worksInTheCheckoutItself,
}: {
  /**
   The checkout the run's launcher works in, which its run must not change.
   */
  readonly checkoutFolder: string;

  /**
   Where the run works, which is a worktree of that checkout or the checkout itself.
   */
  readonly worktree: string;

  /**
   Where the driver writes this run.
   */
  readonly runFolder: string;

  /**
   Whether the run works in the checkout itself, in which case it gets no worktree of its own to write.
   */
  readonly worksInTheCheckoutItself: boolean;
},): Promise<FileSystemOfARun> {
  /**
   The folder this repository's tools cache their launchers and guards in, which a checkout with none ignores.
   */
  const cacheFolder = join(
    checkoutFolder,
    CACHE_FOLDER_NAME,
  );
  /**
   The folders this run must write, in the order a reader needs them: where it works, where its Git data is, where this
   repository's tools cache, and where the driver writes it.
   */
  const wanted: readonly string[] = [
    ...(worksInTheCheckoutItself ? [] : [worktree,]),
    ...await gitFoldersToWrite({ checkoutFolder, },),
    `${MISSING_PATH_IS_FINE}${cacheFolder}`,
    runFolder,
  ];
  return {
    foldersAUnitCannotWrite: [checkoutFolder,],
    foldersAUnitMustWrite: [...new Set(wanted,),].filter(function isNotTheReadOnlyFolder(path,) {
      return (path !== checkoutFolder) && (path !== `${MISSING_PATH_IS_FINE}${checkoutFolder}`);
    },),
  };
}

/**
 Writes what one run's unit may and may not write as the properties `systemd-run` reads.

 @param foldersAUnitCannotWrite - the folders the unit cannot write

 @param foldersAUnitMustWrite - the folders the unit must write

 @returns one `--property=` word per folder, the read-only ones first

 @example
 ```ts
 const words = fileSystemPropertiesOf({ foldersAUnitCannotWrite: ['/repo'], foldersAUnitMustWrite: ['/repo/.git', '-/repo/.cache'] },);
 ```
 */
export function fileSystemPropertiesOf({
  foldersAUnitCannotWrite,
  foldersAUnitMustWrite,
}: FileSystemOfARun,): readonly string[] {
  return [
    ...foldersAUnitCannotWrite.map(function readOnlyWord(path,) {
      return `--property=ReadOnlyPaths=${path}`;
    },),
    ...foldersAUnitMustWrite.map(function writableWord(path,) {
      return `--property=ReadWritePaths=${path}`;
    },),
  ];
}
