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
 systemd applies inside the read-only ones. A path there that is not there stops the unit from starting, so the folders that may
 be absent are named apart and carry systemd's `-` prefix, which tells it to ignore a path it cannot find.

 The run's worktree is not one of those: a run with no worktree has no working folder, and the driver would end at once with
 `spawn pi ENOENT` rather than work. So the launcher reads the worktree once more immediately before it asks for the unit, and
 refuses the start with words naming the worktree when it has gone meanwhile.

 @module
 */

import { access, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  CHECKOUT_MARKER,
  CheckoutNotFoundError,
  gitFoldersOf,
} from './checkout-of-a-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { pathInAUnitProperty, } from './path-in-a-unit-property.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

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

  /**
   The folders a unit of this run must write when they are there, which systemd leaves out rather than refusing the unit.
   */
  readonly foldersAUnitMustWriteWhenTheyAreThere: readonly string[];
};

/**
 Reads the Git folders a run's commits, fetches and pushes write, from the checkout its launcher works in.

 A checkout Git cannot read still gets its own `.git` named among the folders that may be absent, so a start is not refused for
 it, and the checkout stays read-only around whatever it does hold.

 @param checkoutFolder - the checkout the launcher works in

 @returns the Git directories that are there, and the marker to name when the checkout holds none

 @throws Error when the checkout's marker is there but unreadable, which is a failure this reads nothing about

 @example
 ```ts
 const folders = await gitFoldersToWrite({ checkoutFolder: '/repo', },);
 ```
 */
async function gitFoldersToWrite({ checkoutFolder }: { readonly checkoutFolder: string },): Promise<{
  readonly folders: readonly string[];
  readonly whenTheyAreThere: readonly string[];
}> {
  try {
    /**
     The Git directory of the checkout and the one every worktree of its repository shares.
     */
    const {
      gitDirectory,
      commonDirectory,
    } = await gitFoldersOf({ folder: checkoutFolder, },);
    return {
      folders: [
        gitDirectory,
        commonDirectory,
      ],
      whenTheyAreThere: [],
    };
  }
  catch (error) {
    if (!(error instanceof CheckoutNotFoundError)) {
      throw error;
    }
    return {
      folders: [],
      whenTheyAreThere: [
        join(
          checkoutFolder,
          CHECKOUT_MARKER,
        ),
      ],
    };
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
   The Git folders of the checkout, which are there unless the checkout keeps no marker Git can read.
   */
  const git = await gitFoldersToWrite({ checkoutFolder, },);
  /**
   The folders this run must write, in the order a reader needs them: where it works, where its Git data is, and where the driver
   writes it.
   */
  const wanted: readonly string[] = [
    ...(worksInTheCheckoutItself ? [] : [worktree,]),
    ...git.folders,
    runFolder,
  ];
  /**
   The folders this run must write when they are there: the checkout's cache, which a checkout that never installed a tool does not
   have, and a marker for a checkout Git cannot read.
   */
  const wantedWhenTheyAreThere: readonly string[] = [
    cacheFolder,
    ...git.whenTheyAreThere,
  ];
  return {
    foldersAUnitCannotWrite: [checkoutFolder,],
    foldersAUnitMustWrite: namedOnce({
      folders: wanted,
      butNot: [checkoutFolder,],
    },),
    foldersAUnitMustWriteWhenTheyAreThere: namedOnce({
      folders: wantedWhenTheyAreThere,
      butNot: [checkoutFolder,],
    },),
  };
}

/**
 Reads the folders a unit may write, each named once and never the folder the unit cannot write at all.

 @param folders - the folders a unit may write

 @param butNot - the folders to leave out, which is the read-only one

 @returns the folders, each once, in the order they were given

 @example
 ```ts
 const folders = namedOnce({ folders: ['/repo/.git', '/repo/.git', '/repo'], butNot: ['/repo'], });
 ```
 */
function namedOnce({
  folders,
  butNot,
}: {
  readonly folders: readonly string[];
  readonly butNot: readonly string[];
},): readonly string[] {
  return [...new Set(folders,),].filter(function isNotLeftOut(folder,) {
    return !butNot.includes(folder,);
  },);
}

/**
 Writes what one run's unit may and may not write as the properties `systemd-run` reads.

 @param foldersAUnitCannotWrite - the folders the unit cannot write

 @param foldersAUnitMustWrite - the folders the unit must write

 @param foldersAUnitMustWriteWhenTheyAreThere - the folders the unit must write when they are there

 @returns one `--property=` word per folder, the read-only ones first

 @throws PathThatIsNotAbsoluteError when a folder is not an absolute path, which systemd cannot take

 @example
 ```ts
 const words = fileSystemPropertiesOf({ foldersAUnitCannotWrite: ['/repo'], foldersAUnitMustWrite: ['/repo/.git'], foldersAUnitMustWriteWhenTheyAreThere: ['/repo/.cache'] },);
 ```
 */
export function fileSystemPropertiesOf({
  foldersAUnitCannotWrite,
  foldersAUnitMustWrite,
  foldersAUnitMustWriteWhenTheyAreThere,
}: FileSystemOfARun,): readonly string[] {
  return [
    ...foldersAUnitCannotWrite.map(function readOnlyWord(path,) {
      return `--property=ReadOnlyPaths=${pathInAUnitProperty({ path, },)}`;
    },),
    ...foldersAUnitMustWrite.map(function writableWord(path,) {
      return `--property=ReadWritePaths=${pathInAUnitProperty({ path, },)}`;
    },),
    ...foldersAUnitMustWriteWhenTheyAreThere.map(function writableWhenThereWord(path,) {
      return `--property=ReadWritePaths=${pathInAUnitProperty({
        path,
        ignoredWhenMissing: true,
      },)}`;
    },),
  ];
}

/**
 A run's worktree that is gone when the launcher is about to ask for its unit, so the run has nowhere to work.

 @example
 ```ts
 throw new WorktreeGoneError("the worktree /repo/.claude/worktrees/pi-a is gone",);
 ```
 */
export class WorktreeGoneError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'WorktreeGoneError';
}

/**
 Says whether a folder is there.

 @param folder - the folder to look for

 @returns whether the folder is there

 @throws Error when the folder's own state cannot be read, which is no answer about whether it is there

 @example
 ```ts
 const there = await isThere({ folder: '/repo/.claude/worktrees/pi-a', });
 ```
 */
async function isThere({ folder, }: { readonly folder: string },): Promise<boolean> {
  try {
    await access(folder,);
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
 Refuses a start whose worktree has gone since the launcher read it, before any unit is asked for.

 Why: the launcher reads the worktree, waits for the turn cap, the node and the driver's entry, and only then asks the manager for
 the unit. A launcher of another start of the same name that has just ended removes a finished worktree in that window, and the
 unit would then fail in the manager's own words (`Failed to set up mount namespacing`, exit 226) because a `ReadWritePaths=`
 folder that is not there refuses the unit. Words that name the worktree say what happened and what to do instead.

 @param worktree - where the run works, which Git made for it

 @param runName - the run's name, which names the worktree a launcher of another start of it would remove

 @param runFolder - the run's own folder, which this start made and which holds no run

 @throws WorktreeGoneError when the worktree is not there

 @example
 ```ts
 await refuseUnlessTheWorktreeIsThere({ worktree: '/repo/.claude/worktrees/pi-a', runName: 'a', runFolder: '/runs/a-1', });
 ```
 */
export async function refuseUnlessTheWorktreeIsThere({
  worktree,
  runName,
  runFolder,
}: {
  /**
   Where the run works, which Git made for it.
   */
  readonly worktree: string;

  /**
   The run's name, which names the worktree a launcher of another start of it would remove.
   */
  readonly runName: string;

  /**
   The run's own folder, which this start made and which holds no run.
   */
  readonly runFolder: string;
},): Promise<void> {
  if (await isThere({ folder: worktree, },)) {
    return;
  }
  throw new WorktreeGoneError(
    `the worktree ${worktree} is gone, and this start read it a moment ago, so no unit was started and no run happened. Most likely a launcher of another start of the name ${runName} finished just now, and a launcher removes a finished worktree once its unit has ended. Nothing of that work is lost: its commits are in the Git directory the worktree named. Start the run again; the run folder ${runFolder} holds this start's refusal and no run.`,
  );
}
