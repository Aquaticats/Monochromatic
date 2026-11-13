/**
 Emptying one run's scratch folder when the run ends: whether this folder is one this may touch at all, how much room the volume
 holds before and after, and what the deletion itself counted.

 The owner, asked when a run's scratch is deleted: "At run end, all but keep/." A folder is emptied whatever its run ended with, and
 a folder that is not directly under the scratch root or not named after that run is refused before the disk is read: the only
 folder this touches is the one the launcher made for the run whose name it is given.

 @module
 */

import type { Stats, } from 'node:fs';
import { statfs, } from 'node:fs/promises';
import { mountPointsNow, } from './mounts-of-this-machine.ts';
import { ownershipProblemNow, } from './owner-of-a-scratch-folder.ts';
import type { RunCommand, } from './run-command.ts';
import {
  type UnremovedEntry,
  emptyTally,
  removeInto,
} from './deleting-what-a-run-left-in-its-scratch-folder.ts';
import {
  entryIn,
  reasonOfFailure,
} from './what-a-scratch-folder-holds.ts';
import {
  type MadeScratchFolder,
  scratchFolderProblemIn,
} from './scratch-folder-of-a-run.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 What emptying one run's scratch folder answered.
 */
export type ScratchFolderEmptiness =
  | {
    /**
     The folder was left as it is, because it is not the one the launcher gives that run.
     */
    readonly emptied: 'refused';

    /**
     Why it was refused, in words.
     */
    readonly reason: string;
  }
    | {
    /**
     There was no folder to empty, which is no failure: a run that ended before its launcher made one leaves none.
     */
      readonly emptied: 'no-folder';
    }
    | {
    /**
     Everything this could reach was deleted, and what it could not reach is named.
     */
      readonly emptied: 'done';

    /**
     The sizes of the files removed, as each one's own metadata read them, which is not the room a copy-on-write filesystem
     gained.
     */
      readonly removedBytes: number;

    /**
     How many files and symbolic links were removed.
     */
      readonly removedFiles: number;

    /**
     How many folders were removed.
     */
      readonly removedFolders: number;

    /**
     The sizes of the ordinary files `keep/` holds, as their metadata reads them.
     */
      readonly keepBytes: number;

    /**
     How many ordinary files `keep/` holds.
     */
      readonly keepFiles: number;

    /**
     The entries left where they are, each with the reason.
     */
      readonly leftBehind: readonly UnremovedEntry[];

    /**
     The room the volume holding the folder had before the deletion, when it could be read.
     */
      readonly freeBytesBefore?: number;

    /**
     The room it had after, when it could be read.
     */
      readonly freeBytesAfter?: number;

    /**
     Why the room could not be read, when it could not be.
     */
      readonly freeSpaceProblem?: string;
    };

/**
 What emptying one folder needs.
 */
export type ScratchEmptyingRequest = {
  /**
   The folder to empty.
   */
  readonly folder: string;

  /**
   The root every run's scratch folder lives directly under.
   */
  readonly scratchRoot: string;

  /**
   When the proof that the run still owned the folder was taken, which an entry written since then is not deleted for. A caller that
   took no proof gets this call's own moment, to the end of that millisecond, so everything already in the folder is its to delete.
   */
  readonly notBefore?: number;

  /**
   The run the folder is said to belong to.
   */
  readonly runName: string;

  /**
   How the machine's mount points are read, which a test replaces with its own.
   */
  readonly mountsOf?: () => Promise<readonly string[]>;

  /**
   How the room of the volume holding the folder is read, which a test replaces with its own.
   */
  readonly freeBytesOf?: (folder: string) => Promise<number>;
};

/**
 Reads how much room the volume holding one folder has, in the bytes a new file may still take.

 @param folder - the folder whose volume is read

 @returns the room in bytes

 @example
 ```ts
 const free = await freeRoomNow('/home/a/temp/agent/a-scratch');
 ```
 */
async function freeRoomNow(folder: string,): Promise<number> {
  /**
   What the volume answered about its blocks.
   */
  const answered = await statfs(folder,);
  return answered.bavail * answered.bsize;
}

/**
 Reads the room of the volume holding one folder, or why it could not be read.

 @param folder - the folder whose volume is read

 @param freeBytesOf - how the room is read, which a test replaces

 @returns the room, or the reason there is none

 @example
 ```ts
 const room = await freeRoomIn({ folder: '/home/a/temp/agent/a-scratch', freeBytesOf: freeRoomNow, },);
 ```
 */
async function freeRoomIn({
  folder,
  freeBytesOf,
}: {
  readonly folder: string;
  readonly freeBytesOf: (folder: string) => Promise<number>;
},): Promise<{
  readonly freeBytes?: number;
  readonly problem?: string
}> {
  try {
    return { freeBytes: await freeBytesOf(folder,), };
  }
  catch (error) {
    // A volume that cannot answer about its room is no reason to leave a folder full: the deletion goes on and the line says
    // the room could not be read, so nobody reads a missing figure as no room gained.
    return { problem: reasonOfFailure({ error, },), };
  }
}

/**
 Empties a folder that is there and is a folder, which is everything past the check that it may be touched.

 @param request - the folder, the root it must sit under, and the run it belongs to

 @param atTheFolder - what the folder itself is

 @returns what it removed, what `keep/` holds, and what it left where it is

 @example
 ```ts
 const emptied = await emptyTheFolderThatIsThere({ request, atTheFolder, },);
 ```
 */
async function emptyTheFolderThatIsThere({
  request,
  atTheFolder,
}: {
  readonly request: ScratchEmptyingRequest;
  readonly atTheFolder: Stats;
},): Promise<ScratchFolderEmptiness> {
  /**
   Every mount point of this machine.
   */
  const mountPoints = new Set(await (request.mountsOf ?? mountPointsNow)(),);
  // A folder that is itself a mount point is another filesystem's file tree, which this run did not make, so nothing inside it
  // is this run's to delete.
  if (mountPoints.has(request.folder)) {
    return {
      emptied: 'refused',
      reason: 'it is a mount point, so emptying it would cross into a filesystem this run did not make',
    };
  }
  /**
   How the room of the volume holding the folder is read.
   */
  const freeBytesOf = request.freeBytesOf ?? freeRoomNow;
  /**
   The room before the deletion, which is one half of what the volume gained.
   */
  const before = await freeRoomIn({
    folder: request.folder,
    freeBytesOf,
  },);
  /**
   What this emptying counts as it goes.
   */
  const tally = emptyTally();
  await removeInto({
    folder: request.folder,
    referenceDevice: atTheFolder.dev,
    mountPoints,
    notBefore: request.notBefore ?? (Date.now() + 1),
    tally,
  },);
  /**
   The room after the deletion, which is the other half of what the volume gained.
   */
  const after = await freeRoomIn({
    folder: request.folder,
    freeBytesOf,
  },);
  /**
   Why the room could not be read, when it could not be.
   */
  const freeSpaceProblem = before.problem ?? after.problem;
  return {
    emptied: 'done',
    removedBytes: tally.removedBytes,
    removedFiles: tally.removedFiles,
    removedFolders: tally.removedFolders,
    keepBytes: tally.keepBytes,
    keepFiles: tally.keepFiles,
    leftBehind: tally.leftBehind,
    ...(before.freeBytes === undefined ? {} : { freeBytesBefore: before.freeBytes, }),
    ...(after.freeBytes === undefined ? {} : { freeBytesAfter: after.freeBytes, }),
    ...(freeSpaceProblem === undefined ? {} : { freeSpaceProblem, }),
  };
}

/**
 Empties one run's scratch folder, keeping its `keep/` subfolder.

 Nothing outside the folder is reached: a symbolic link is deleted as a link, an entry on another filesystem or at a mount point
 is left where it is, and a folder that is not directly under the scratch root or not named after the run is refused before the
 disk is read.

 @param request - the folder, the root it must sit under, the run it belongs to, and the seams a test replaces

 @returns what it removed, what `keep/` holds, and what it left where it is

 @example
 ```ts
 const emptied = await emptyTheFolderIn({ folder: '/home/a/temp/agent/a-scratch', scratchRoot: '/home/a/temp/agent', runName: 'a', },);
 ```
 */
export async function emptyTheFolderIn(request: ScratchEmptyingRequest,): Promise<ScratchFolderEmptiness> {
  /**
   Why this folder is not the one the launcher gives that run, when it is not.
   */
  const problem = scratchFolderProblemIn({
    folder: request.folder,
    scratchRoot: request.scratchRoot,
    runName: request.runName,
  },);
  if (problem.problem !== undefined) {
    return {
      emptied: 'refused',
      reason: problem.problem,
    };
  }
  /**
   What is at the folder's path, without following a symbolic link.
   */
  const atTheFolder = await entryIn(request.folder,);
  if (atTheFolder.info === undefined) {
    // A run that ended before its launcher made its folder leaves none, which is no failure and nothing to report.
    if (atTheFolder.code === MISSING_PATH_CODE) {
      return { emptied: 'no-folder', };
    }
    return {
      emptied: 'refused',
      reason: `it could not be looked at: ${atTheFolder.reason ?? 'the reason was not given'}`,
    };
  }
  // A symbolic link is refused rather than followed: emptying it would delete whatever it points at, which is not this folder.
  if (!atTheFolder.info
    .isDirectory()) {
    return {
      emptied: 'refused',
      reason: atTheFolder.info
        .isSymbolicLink()
        ? 'it is a symbolic link, and following it would delete whatever it points at'
        : 'it is not a folder',
    };
  }
  return await emptyTheFolderThatIsThere({
    request,
    atTheFolder: atTheFolder.info,
  },);
}

/**
 Empties the scratch folder the launcher made for one run, from the folder it reported making.

 @param made - the folder the launcher made, or found from an earlier run of the same name

 @param scratchRoot - the root it lives directly under

 @param runName - the run it belongs to

 @returns what it removed, what `keep/` holds, and what it left where it is

 @example
 ```ts
 const emptied = await emptyTheFolderOfRun({ made, scratchRoot: '/home/a/temp/agent', runName: 'a', },);
 ```
 */
export async function emptyTheFolderOfRun({
  made,
  scratchRoot,
  runName,
}: {
  readonly made: MadeScratchFolder;
  readonly scratchRoot: string;
  readonly runName: string;
},): Promise<ScratchFolderEmptiness> {
  return await emptyTheFolderIn({
    folder: made.folder,
    scratchRoot,
    runName,
  },);
}

/**
 What emptying one folder as the run that ended needs: the folder, and how to prove that run still owns it.
 */
export type OwnedScratchEmptyingRequest = ScratchEmptyingRequest & {
  /**
   How a command is run, which the proof asks the manager with.
   */
  readonly run: RunCommand;

  /**
   The folder holding one entry per process, `/proc` here.
   */
  readonly processFolder: string;

  /**
   The processes that asked for this emptying, which the proof does not read as later attempts: the launcher whose end step this is,
   which is alive until the emptying finishes. This process is always excluded as well.
   */
  readonly askingPids?: readonly number[];
};

/**
 Empties a run's scratch folder only once the run that ended is proven still to own it, and the proof is taken here, at the moment
 of the deletion.

 A folder is named by the run's name, and a name is free as soon as the old unit is collected, so a newer attempt of the same name can
 own the folder while the attempt before it is still ending. Proving it immediately before the walk is as late as this can be done:
 the readings the end step took on its way are old by then, and one taken here is not.

 @param request - the folder, the root it must sit under, the run it belongs to, how the manager is asked, and the seams a test
 replaces

 @returns what it removed, or why it refused the folder

 @example
 ```ts
 const emptied = await emptyTheFolderItsRunStillOwns({ folder: '/home/a/temp/agent/a-scratch', scratchRoot: '/home/a/temp/agent', runName: 'a', run: runCommandWithChildProcess, processFolder: '/proc', },);
 ```
 */
export async function emptyTheFolderItsRunStillOwns(request: OwnedScratchEmptyingRequest,): Promise<ScratchFolderEmptiness> {
  // The moment the proof is asked, which is the horizon the walk deletes up to: anything written into the folder after it may belong
  // to a later attempt of the same name and is left where it is.
  /**
   When this emptying was proven the ended run's own, as the walk counts from it.
   */
  const notBefore = Date.now();
  /**
   Why this folder is not provably the ended run's own, when it is not.
   */
  const ownership = await ownershipProblemNow({
    run: request.run,
    runName: request.runName,
    folder: request.folder,
    processFolder: request.processFolder,
    excludingPids: [
      process.pid,
      ...(request.askingPids ?? []),
    ],
  },);
  if (ownership.problem !== undefined) {
    return {
      emptied: 'refused',
      reason: ownership.problem,
    };
  }
  return await emptyTheFolderIn({
    ...request,
    notBefore,
  },);
}
