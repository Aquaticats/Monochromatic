/**
 The file system a run's unit gets: the checkout its launcher works in is read-only, and the folders the run must write stay
 writable.

 Why: on 2026-11-14 two runs wrote into the main checkout, one of them for fifty minutes, because a brief that names a
 main-checkout path to read does not stop the same path from being written, and nothing in the unit refused it. A run's own
 worktree is copied from the checkout, so a run that edits the checkout edits the copy every later worktree is made from.

 What stays writable is Git's own answer about the checkout the launcher works in, and the folders this package knows a run
 needs beside it: the run's worktree, the Git directory its commits, fetches and pushes write, the Git directory every worktree of
 the repository shares, the checkout's `.cache` (the folder this repository's tools install their launchers and guards into), and
 the run's own folder. A Git folder that holds the checkout is refused rather than granted, and so is a checkout Git cannot
 describe. Everything else
 under the checkout is refused by the operating system, which is what the agent sees.

 Read-only folders are systemd's `ReadOnlyPaths=`; the folders that must stay writable are its `ReadWritePaths=`, which
 systemd applies inside the read-only ones. A path there that is not there stops the unit from starting, so the folders that may
 be absent are named apart and carry systemd's `-` prefix, which tells it to ignore a path it cannot find.

 The checkout's `.cache` is one of those tools' folders and cannot be one of those: a tool that installs its launcher or guard
 makes `.cache` itself, and under a read-only checkout that creation is refused, so a `-` prefix there lets the unit start and
 then refuses the very write it was meant to allow. The launcher makes the folder before it asks for the unit instead, and names it
 as a folder the unit must write.

 The run's worktree is not one of those: a run with no worktree has no working folder, and the driver would end at once with
 `spawn pi ENOENT` rather than work. So the launcher reads the worktree once more immediately before it asks for the unit, and
 refuses the start with words naming the worktree when it has gone meanwhile.

 @module
 */

import {
  access,
  mkdir,
  realpath,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { answerWithinABound, } from './an-answer-within-a-bound.ts';
import { gitFoldersOf, } from './checkout-of-a-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import type { GitRunner, } from './worktree-of-a-run.ts';
import { pathInAUnitProperty, } from './path-in-a-unit-property.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 How long reading one folder as the file system names it may take, in milliseconds.

 A bound rather than a measurement: every folder here is a checkout, a Git directory or a worktree on a mounted disk, which
 `realpath` answers for in microseconds, and the launcher asks before it makes any unit, so no unit's `RuntimeMaxSec` covers this
 wait. A filesystem that has stopped answering path resolution would otherwise hold the start for ever.
 */
export const CANONICAL_PATH_MILLISECONDS: number = 30_000;

/**
 Reads one folder as the file system names it, which a test replaces to hold a `realpath` that never answers.

 @param path - the folder to read

 @returns its canonical path

 @example
 ```ts
 const canonical = await readRealPath('/repo/.git');
 ```
 */
export type RealPathOf = (path: string) => Promise<string>;

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
 The folders a run's tools make inside the checkout, outside the run's own worktree and run folder, and which therefore have to
 exist before the unit is asked for.

 The checkout's `.cache` is where this repository's tools install their launchers, their exposure directories and their guard
 (`toolchain-bin`, `toolchain-launchers`, `<tool> -bin` folders and `toolchain.guard` all live inside it), and a tool makes the
 folder itself when a checkout has none. Measured on this machine: `pnpm install --offline --frozen-lockfile` in a worktree, the
 toolchain's own `list`, a package's type check and a guarded `git status` wrote nothing under the checkout, so this folder is
 the one a run's tools may still make there.

 @param checkoutFolder - the checkout the run's launcher works in

 @returns the folders, in the order a reader needs them

 @example
 ```ts
 const folders = foldersAToolMakesIn({ checkoutFolder: '/repo', });
 ```
 */
export function foldersAToolMakesIn({ checkoutFolder, }: { readonly checkoutFolder: string },): readonly string[] {
  return [join(
    checkoutFolder,
    CACHE_FOLDER_NAME,
  ),];
}

/**
 A folder a run's tools make that this launcher could not make before asking for the unit, so a tool that installs itself there
 would be refused by the run's own read-only checkout.
 */
export class FolderAToolMakesCannotBeMadeError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'FolderAToolMakesCannotBeMadeError';
}

/**
 Makes the folders a run's tools make inside the checkout, so the unit can be told they must be writable.

 @param checkoutFolder - the checkout the run's launcher works in

 @returns the folders, which are there when this answers

 @throws FolderAToolMakesCannotBeMadeError when one cannot be made, naming it and what that would cost a run

 @example
 ```ts
 const folders = await makeTheFoldersAToolMakesIn({ checkoutFolder: '/repo', });
 ```
 */
export async function makeTheFoldersAToolMakesIn({
  checkoutFolder,
}: {
  /**
   The checkout the run's launcher works in.
   */
  readonly checkoutFolder: string;
},): Promise<readonly string[]> {
  /**
   The folders a tool makes inside this checkout.
   */
  const folders = foldersAToolMakesIn({ checkoutFolder, },);
  await Promise.all(folders.map(async function makeOne(folder,) {
    await makeTheFolder({ folder, },);
  },),);
  return folders;
}

/**
 Makes one folder a run's tools write, naming it when it cannot be made.

 @param folder - the folder to make, and everything above it

 @throws FolderAToolMakesCannotBeMadeError when it cannot be made, naming it and what that would cost a run

 @example
 ```ts
 await makeTheFolder({ folder: '/repo/.cache', });
 ```
 */
async function makeTheFolder({ folder, }: { readonly folder: string },): Promise<void> {
  try {
    await mkdir(
      folder,
      { recursive: true, },
    );
  }
  catch (error) {
    throw new FolderAToolMakesCannotBeMadeError(
      `the folder ${folder} could not be made, and a run's tools install their launchers and guards there: with the checkout read-only, a tool that made it would be refused, so this start made it first. ${String(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Reads what one run's unit may and may not write.

 @param checkoutFolder - the checkout the run's launcher works in, which its run must not change

 @param worktree - where the run works, which is a worktree of that checkout or the checkout itself

 @param runFolder - where the driver writes this run

 @param worksInTheCheckoutItself - whether the run works in the checkout itself, in which case it gets no worktree of its own to write

 @returns the folders the unit cannot write and the folders it must write

 @throws GitFoldersUnreadableError when Git does not answer about the checkout's Git directories

 @throws GitFolderAboveTheCheckoutError when Git names a folder that holds the checkout, which a start must not make writable

 @throws ACanonicalPathThatDidNotAnswerError when one of those folders did not answer inside {@link CANONICAL_PATH_MILLISECONDS}

 @example
 ```ts
 const fileSystem = await fileSystemOfARun({ git, checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', worksInTheCheckoutItself: false },);
 ```
 */
export async function fileSystemOfARun({
  git,
  checkoutFolder,
  worktree,
  runFolder,
  worksInTheCheckoutItself,
  readRealPath = realpath,
  canonicalPathMilliseconds = CANONICAL_PATH_MILLISECONDS,
}: {
  /**
   How one git question is run, which Git's own answers about the checkout come through.
   */
  readonly git: GitRunner;

  /**
   How one folder is read as the file system names it, which a test replaces to hold a `realpath` that never answers.
   */
  readonly readRealPath?: RealPathOf;

  /**
   How long reading one folder that way may take, which a test shortens and no caller else names.
   */
  readonly canonicalPathMilliseconds?: number;

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
   The Git folders of the checkout, as Git itself names them.
   */
  const gitFolders = await gitFoldersOf({
    git,
    folder: checkoutFolder,
  },);
  /**
   The checkout and its Git folders as the file system names them, so that a path reached through a symbolic link and the same
   path written as its target compare equal.
   */
  const canonical = {
    checkout: await canonicalPathOf({
      path: checkoutFolder,
      readRealPath,
      milliseconds: canonicalPathMilliseconds,
    },),
    gitDirectory: await canonicalPathOf({
      path: gitFolders.gitDirectory,
      readRealPath,
      milliseconds: canonicalPathMilliseconds,
    },),
    commonDirectory: await canonicalPathOf({
      path: gitFolders.commonDirectory,
      readRealPath,
      milliseconds: canonicalPathMilliseconds,
    },),
  };
  /**
   The Git folders this run's unit writes: a run with a worktree of its own commits there, so its own Git directory and the one
   every worktree shares must be writable wherever they lie. A run that works in the checkout itself has nothing of its own to
   commit: its worktree is the checkout, the checkout is read-only, and a writable Git directory would let it write `HEAD` and the
   refs of a checkout it cannot change otherwise, which is how a run that could not edit one file still moved `main`.
   */
  const gitFoldersTheRunWrites: readonly string[] = worksInTheCheckoutItself
    ? []
    : [
      canonical.gitDirectory,
      canonical.commonDirectory,
    ];
  /**
   The folders this run must write, in the order a reader needs them: where it works, where its Git data is, where this
   repository's tools install themselves, and where the driver writes it.
   */
  const wanted: readonly string[] = [
    ...(worksInTheCheckoutItself ? [] : [worktree,]),
    ...gitFoldersTheRunWrites,
    ...foldersAToolMakesIn({ checkoutFolder: canonical.checkout, },),
    runFolder,
  ];
  /**
   The Git folders as one set, which decides whether the checkout itself stays read-only.
   */
  const gitFoldersTheRunWritesSet = new Set(gitFoldersTheRunWrites,);
  return {
    // A folder that is a Git directory of the run is not read-only, even when it is the checkout: a bare repository holds the
    // run's commits and nothing a run could edit, so its read-only view would only block them. A run with no worktree of its own
    // writes no Git folder, so its checkout stays read-only and every ref in it stays where it was.
    foldersAUnitCannotWrite: [canonical.checkout,].filter(function isNoGitFolder(path,) {
      return !gitFoldersTheRunWritesSet.has(path,);
    },),
    foldersAUnitMustWrite: namedOnce(wanted,),
  };
}

/**
 Reads a folder as the file system names it, with every symbolic link resolved.

 @param path - the folder to read

 @returns the canonical path

 @throws ACanonicalPathThatDidNotAnswerError when the folder did not answer inside its bound, naming it and the bound

 @throws the error the read gives, such as a folder that is not there

 @example
 ```ts
 const canonical = await canonicalPathOf({ path: '/repo/.git', });
 ```
 */
async function canonicalPathOf({
  path,
  readRealPath,
  milliseconds,
}: {
  /**
   The folder to read.
   */
  readonly path: string;

  /**
   How one folder is read as the file system names it, which a test replaces to hold a `realpath` that never answers.
   */
  readonly readRealPath: RealPathOf;

  /**
   How long that read may take.
   */
  readonly milliseconds: number;
},): Promise<string> {
  try {
    return await answerWithinABound({
      call: readRealPath(path,),
      milliseconds,
      refusal: function readingTheFolderTookTooLong() {
        return new ACanonicalPathThatDidNotAnswerError(
          `reading ${path} as the file system names it took longer than ${String(milliseconds,)} ms, so this start stopped waiting for it and no unit was asked for. The request is still the kernel's until the filesystem answers, and nothing here waits on it any more; a checkout on a filesystem that answers again can be started then.`,
        );
      },
    },);
  }
  catch (error) {
    // A folder Git named that is not there is a checkout this start cannot describe: it is refused, never taken as written, and
    // a unit told to write a folder nobody can make is not started on a manager's bare status.
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    throw new AGitFolderThatIsNotThereError(
      `${path} is one of the folders Git named for this checkout, and it is not there, so a run's unit cannot be told which folders its commits write. Name a checkout Git can describe with --checkout-folder. ${String(error,)}`,
      { cause: error, },
    );
  }
}

/**
 A folder the file system did not name inside the bound, so a start has no folders to give its unit.

 @example
 ```ts
 throw new ACanonicalPathThatDidNotAnswerError('/repo took longer than 30000 ms to answer',);
 ```
 */
export class ACanonicalPathThatDidNotAnswerError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ACanonicalPathThatDidNotAnswerError';
}

/**
 A folder Git named that is not there, so a start cannot say which folders its run writes.

 @example
 ```ts
 throw new AGitFolderThatIsNotThereError('/repo/.git is not there',);
 ```
 */
export class AGitFolderThatIsNotThereError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'AGitFolderThatIsNotThereError';
}

/**
 Reads the folders a unit may write, each named once.

 @param folders - the folders a unit may write

 @returns the folders, each once, in the order they were given

 @example
 ```ts
 const folders = namedOnce(['/repo/.git', '/repo/.git', '/repo/.cache'],);
 ```
 */
function namedOnce(folders: readonly string[],): readonly string[] {
  return [...new Set(folders,),];
}

/**
 Writes what one run's unit may and may not write as the properties `systemd-run` reads.

 @param foldersAUnitCannotWrite - the folders the unit cannot write

 @param foldersAUnitMustWrite - the folders the unit must write

 @returns one `--property=` word per folder, the read-only ones first

 @throws PathThatIsNotAbsoluteError when a folder is not an absolute path, which systemd cannot take

 @example
 ```ts
 const words = fileSystemPropertiesOf({ foldersAUnitCannotWrite: ['/repo'], foldersAUnitMustWrite: ['/repo/.git', '/repo/.cache'] },);
 ```
 */
export function fileSystemPropertiesOf({
  foldersAUnitCannotWrite,
  foldersAUnitMustWrite,
}: FileSystemOfARun,): readonly string[] {
  return [
    ...foldersAUnitCannotWrite.map(function readOnlyWord(path,) {
      return `--property=ReadOnlyPaths=${pathInAUnitProperty({ path, },)}`;
    },),
    ...foldersAUnitMustWrite.map(function writableWord(path,) {
      return `--property=ReadWritePaths=${pathInAUnitProperty({ path, },)}`;
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
  throw new WorktreeGoneError(worktreeGoneWordsOf({
    worktree,
    runName,
    runFolder,
  },),);
}

/**
 Writes the words for a start whose worktree has gone before its unit was asked for, so no run happened.

 @param worktree - the worktree that has gone

 @param runName - the run's name, which names the worktree a launcher of another start of it would remove

 @param runFolder - the run's own folder, which holds no run

 @returns the words to print

 @example
 ```ts
 const words = worktreeGoneWordsOf({ worktree: '/repo/.claude/worktrees/pi-a', runName: 'a', runFolder: '/runs/a-1', });
 ```
 */
export function worktreeGoneWordsOf({
  worktree,
  runName,
  runFolder,
}: {
  /**
   The worktree that has gone.
   */
  readonly worktree: string;

  /**
   The run's name, which names the worktree a launcher of another start of it would remove.
   */
  readonly runName: string;

  /**
   The run's own folder, which holds no run.
   */
  readonly runFolder: string;
},): string {
  return `the worktree ${worktree} is gone, and this start read it a moment ago, so no unit was started and no run happened. Most likely a launcher of another start of the name ${runName} finished just now, and a launcher removes a finished worktree once its unit has ended. Its branch and every commit on it are in the Git directory the worktree named; what the worktree held that was never committed, including the files Git ignores, is not there any more. Start the run again; the run folder ${runFolder} holds this start's refusal and no run.`;
}
