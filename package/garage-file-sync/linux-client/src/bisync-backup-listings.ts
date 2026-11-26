/**
 Tells whether bisync kept the backup listings that let `--recover` resume an aborted run.

 When bisync aborts with "Must run --resync to recover", it moves its listings aside as
 `<name>.path1.lst-old` and `<name>.path2.lst-old`. With `--recover` and both backups
 present, the next run resumes from them and no `--resync` is needed
 (doc/troubleshooting/rclone-bisync-aborts-on-a-name-the-local-disk-refuses.md); only
 without them does syncing stay stopped. The alert's advice depends on which case it is.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Start of the state file names bisync derives from the pair `garage:files/Plain` and the
 local folder.
 */
export const BISYNC_LISTING_PREFIX = 'garage_files_Plain..';

/**
 What the backup-listing check found.

 `problem` names why the folder could not be read; `exist` is then false, so the alert
 gives the safer advice.
 */
export type BackupListings = {
  readonly exist: boolean;
  readonly problem?: string;
};

/**
 Lists a folder's names, or says why it could not.

 @param folder - folder to list

 @returns the names, or no names and the reason
 */
async function namesIn(folder: string,): Promise<{
  readonly names: readonly string[];
  readonly problem?: string;
}> {
  try {
    return { names: await readdir(folder,), };
  } catch (error) {
    return {
      names: [],
      problem: `cannot list ${folder}: ${String(error,)}`,
    };
  }
}

/**
 Looks for both backup listings in bisync's working folder.

 @param home - home folder whose `.cache/rclone/bisync` bisync uses

 @returns whether both `.lst-old` backups of the Garage pair exist

 @example
 ```ts
 const { exist } = await backupListingsExist({ home: '/home/example', },);
 ```
 */
export async function backupListingsExist({ home, }: Readonly<{ home: string; }>,): Promise<BackupListings> {
  /**
   bisync's working folder under the home folder.
   */
  const folder = join(
    home,
    '.cache',
    'rclone',
    'bisync',
  );
  /**
   Names in the folder, or the reason they could not be listed.
   */
  const listed = await namesIn(folder,);
  if (listed.problem !== undefined) {
    return {
      exist: false,
      problem: listed.problem,
    };
  }
  /**
   State files of the Garage pair.
   */
  const pairNames = listed.names
    .filter(function ofThePair(name,) {
      return name.startsWith(BISYNC_LISTING_PREFIX,);
    },);
  return {
    exist: pairNames.some(function path1Backup(name,) {
      return name.endsWith('.path1.lst-old',);
    },) && pairNames.some(function path2Backup(name,) {
      return name.endsWith('.path2.lst-old',);
    },),
  };
}
