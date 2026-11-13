/**
 Deleting what a run left in its scratch folder, one entry at a time, and counting what went and what stayed.

 Nothing here decides whether a folder may be emptied; that decision is in `emptying-a-scratch-folder.ts`. This is the walk itself:
 a symbolic link is deleted as a link rather than followed, an entry on another device or at a mount point is left where it is, and
 an entry that cannot be read or removed is named with what the operating system said.

 @module
 */

import {
  rmdir,
  unlink,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';
import { KEEP_FOLDER, } from './scratch-folder-of-a-run.ts';
import {
  entryIn,
  namesIn,
  reasonOfFailure,
} from './what-a-scratch-folder-holds.ts';

/**
 One entry of the folder that was left where it is, and why.
 */
export type UnremovedEntry = {
  /**
   The entry's path.
   */
  readonly path: string;

  /**
   Why it was not deleted, in words a reader of the run's end needs.
   */
  readonly reason: string;
};

/**
 What one deletion counts as it goes.
 */
export type ScratchRemovalTally = {
  /**
   The sizes of the files removed, as their metadata read them.
   */
  removedBytes: number;

  /**
   How many files and symbolic links were removed.
   */
  removedFiles: number;

  /**
   How many folders were removed.
   */
  removedFolders: number;

  /**
   The sizes of the ordinary files `keep/` holds.
   */
  keepBytes: number;

  /**
   How many ordinary files `keep/` holds.
   */
  keepFiles: number;

  /**
   The entries left where they are.
   */
  leftBehind: UnremovedEntry[];
};

/**
 Reads a tally with nothing counted yet.

 @returns the tally, which the walk fills as it goes

 @example
 ```ts
 const tally = emptyTally();
 ```
 */
export function emptyTally(): ScratchRemovalTally {
  return {
    removedBytes: 0,
    removedFiles: 0,
    removedFolders: 0,
    keepBytes: 0,
    keepFiles: 0,
    leftBehind: [],
  };
}

/**
 Counts the files one kept folder holds, which is what the line reports `keep/` holds.

 @param folder - the folder to count

 @param tally - the counts to add to

 @example
 ```ts
 await countInto({ folder: '/home/a/temp/agent/a-scratch/keep', tally, },);
 ```
 */
async function countInto({
  folder,
  tally,
}: {
  readonly folder: string;
  readonly tally: ScratchRemovalTally;
},): Promise<void> {
  /**
   The folders still to count, from the one this walk starts at.
   */
  const pending: string[] = [folder,];
  /* oxlint-disable eslint/no-await-in-loop -- one folder at a time, so a kept folder of many entries holds one descriptor at a
     time and every refusal names the entry it belongs to; see doc/lint/sequential-awaits.md. */
  for (let next = 0; next < pending.length;) {
    /**
     The folder this pass counts.
     */
    const current = pending[next];
    next += 1;
    if (current === undefined) {
      continue;
    }
    /**
     What reading that folder answered.
     */
    const read = await namesIn(current,);
    if (read.names === undefined) {
      tally.leftBehind
        .push({
          path: current,
          reason: read.reason ?? 'it could not be read',
        },);
      continue;
    }
    for (const name of read.names) {
      /**
       The entry's path.
       */
      const path = join(
        current,
        name,
      );
      /**
       What that entry is.
       */
      const entry = await entryIn(path,);
      if (entry.info === undefined) {
        tally.leftBehind
          .push({
            path,
            reason: entry.reason ?? 'it could not be read',
          },);
        continue;
      }
      if (entry.info
        .isDirectory()) {
        pending.push(path,);
        continue;
      }
      if (entry.info
        .isFile()) {
        tally.keepBytes += entry.info
          .size;
        tally.keepFiles += 1;
      }
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}

/**
 Keeps the entry that carries the name of the kept subfolder, counting what it holds when it is a folder and naming it when it is
 not.

 @param path - the entry carrying that name

 @param tally - the counts to add to

 @example
 ```ts
 await keepOf({ path: '/home/a/temp/agent/a-scratch/keep', tally, },);
 ```
 */
async function keepOf({
  path,
  tally,
}: {
  readonly path: string;
  readonly tally: ScratchRemovalTally;
},): Promise<void> {
  /**
   What that entry is.
   */
  const entry = await entryIn(path,);
  if (entry.info === undefined) {
    tally.leftBehind
      .push({
        path,
        reason: entry.reason ?? 'it could not be read',
      },);
    return;
  }
  if (entry.info
    .isDirectory()) {
    await countInto({
      folder: path,
      tally,
    },);
    return;
  }
  // A link, or a file, that carries the name of the kept subfolder stays where it is rather than being followed or deleted: the
  // report a reader is told to look for must not be replaced by whatever this entry points at.
  tally.leftBehind
    .push({
      path,
      reason: `it carries the name of the subfolder a run keeps (${KEEP_FOLDER}), but it is not a folder, so it was left as it is`,
    },);
}

/**
 Says why one entry of a scratch folder is left where it is, or nothing when it may be deleted.

 Two entries are another filesystem's: one at a mount point, whether this machine's mount list names it or the kernel reports it
 through another device, and one on another device than the folder's own. Emptying either would delete files this run did not make,
 so they stay and the line says why.

 @param path - the entry's path, for the reason

 @param mountPoints - every mount point of the machine

 @param device - the device the entry is on, as its metadata reads it

 @param referenceDevice - the device the folder being emptied is on

 @returns the reason it stays, or nothing when it may be deleted

 @example
 ```ts
 const left = leftWhereItIsIn({ path: '/home/a/temp/agent/a-scratch/mnt', mountPoints: new Set(['/home/a/temp/agent/a-scratch/mnt']), device: 1, referenceDevice: 1, },);
 ```
 */
export function leftWhereItIsIn({
  path,
  mountPoints,
  device,
  referenceDevice,
}: {
  readonly path: string;
  readonly mountPoints: ReadonlySet<string>;
  readonly device: number;
  readonly referenceDevice: number;
},): { readonly reason?: string } {
  if (mountPoints.has(resolve(path,),)) {
    return { reason: 'it is a mount point, so emptying it would cross into a filesystem this run did not make', };
  }
  if (device !== referenceDevice) {
    return { reason: `it is on another filesystem (device ${String(device,)}), so it was left as it is`, };
  }
  return {};
}

/**
 Deletes what one folder holds, keeping its `keep/` subfolder, and counts what it removed and what it left.

 @param folder - the folder to empty

 @param referenceDevice - the device the folder is on, which an entry on another device is not

 @param mountPoints - every mount point of the machine, which an entry at one of them is left for

 @param tally - the counts to fill

 @example
 ```ts
 await removeInto({ folder: '/home/a/temp/agent/a-scratch', referenceDevice: 1, mountPoints: new Set<string>(), tally, },);
 ```
 */
export async function removeInto({
  folder,
  referenceDevice,
  mountPoints,
  tally,
}: {
  readonly folder: string;
  readonly referenceDevice: number;
  readonly mountPoints: ReadonlySet<string>;
  readonly tally: ScratchRemovalTally;
},): Promise<void> {
  /**
   The folders to remove once everything inside them is gone, in the order they were met, so the last one met is removed first.
   */
  const foldersToRemove: string[] = [];
  /**
   The folders still to empty, from the one this walk starts at.
   */
  const pending: string[] = [folder,];
  /* oxlint-disable eslint/no-await-in-loop -- one entry at a time, at the lowest disk priority a process can ask for, so a
     deletion of tens of GiB neither starves another process's reads nor opens a descriptor for every file at once; see
     doc/lint/sequential-awaits.md. */
  for (let next = 0; next < pending.length;) {
    /**
     The folder this pass empties.
     */
    const current = pending[next];
    next += 1;
    if (current === undefined) {
      continue;
    }
    /**
     What reading that folder answered.
     */
    const read = await namesIn(current,);
    if (read.names === undefined) {
      tally.leftBehind
        .push({
          path: current,
          reason: read.reason ?? 'it could not be read',
        },);
      continue;
    }
    for (const name of read.names) {
      /**
       The entry's path.
       */
      const path = join(
        current,
        name,
      );
      // The kept subfolder is the one entry of the folder at the top of this walk that stays, and one that carries its name
      // without being a folder stays too.
      if ((current === folder) && (name === KEEP_FOLDER)) {
        await keepOf({
          path,
          tally,
        },);
        continue;
      }
      /**
       What that entry is, without following a symbolic link.
       */
      const entry = await entryIn(path,);
      if (entry.info === undefined) {
        tally.leftBehind
          .push({
            path,
            reason: entry.reason ?? 'it could not be read',
          },);
        continue;
      }
      /**
       Why that entry is another filesystem's, when it is.
       */
      const left = leftWhereItIsIn({
        path,
        mountPoints,
        device: entry.info
          .dev,
        referenceDevice,
      },);
      if (left.reason !== undefined) {
        tally.leftBehind
          .push({
            path,
            reason: left.reason,
          },);
        continue;
      }
      if (entry.info
        .isDirectory()) {
        foldersToRemove.push(path,);
        pending.push(path,);
        continue;
      }
      // A symbolic link is removed as a link, never followed: what it points at is outside this folder, whether it names a file
      // in the worktree or the root of a filesystem.
      try {
        await unlink(path,);
        tally.removedFiles += 1;
        tally.removedBytes += entry.info
          .size;
      }
      catch (error) {
        tally.leftBehind
          .push({
            path,
            reason: reasonOfFailure({ error, },),
          },);
      }
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /* oxlint-disable eslint/no-await-in-loop -- one folder at a time, deepest first, so a folder that holds an entry this cannot
     remove is named rather than its parent being reported for it; see doc/lint/sequential-awaits.md. */
  for (const path of foldersToRemove.toReversed()) {
    try {
      await rmdir(path,);
      tally.removedFolders += 1;
    }
    catch (error) {
      tally.leftBehind
        .push({
          path,
          reason: reasonOfFailure({ error, },),
        },);
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
