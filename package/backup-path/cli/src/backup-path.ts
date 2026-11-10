/**
 Copies one path into a timestamped folder under a backup root.

 Kept apart from the CLI entry so the copy contract (which `fs.cp` options
 reject an existing destination) is exercised without launching a program.
 */

import {
  cp,
  mkdir,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

//region Folder naming
/**
 Names the backup folder for one moment.
 
 The ISO 8601 text has its colons removed, because a colon cannot appear in a
 Windows path segment and this name is one path segment.
 
 @param now - Moment that names the folder
 
 @returns ISO 8601 timestamp usable as a single path segment
 
 @example
 ```ts
 backupFolderName({ now: new Date('2026-05-14T06:17:11.030Z'), },);
 // '2026-05-14T061711.030Z'
 ```
 */
export function backupFolderName(
  {
    now,
  }: {
    readonly now: Date;
  },
): string {
  return now.toISOString()
    .replaceAll(
    ':',
    '',
    );
}
//endregion Folder naming

//region Copying
/**
 Copies `sourcePath` to `destination`, preserving its timestamps and refusing to
 replace anything already there.
 
 `force` defaults to `true` in `fs.cp`, which leaves `errorOnExist` inert: with
 only `errorOnExist: true` a destination collision replaces the copy silently.
 `force: false` is what makes the rejection real, so a backup can never replace
 an earlier one.
 
 @param sourcePath - File or directory to copy
 
 @param destination - Path the copy is written to, which must not exist
 
 @throws NodeJS.ErrnoException with code `ERR_FS_CP_EEXIST` when `destination` already exists, which is then left untouched
 
 @example
 ```ts
 await copyIntoBackup({
   sourcePath: 'notes.md',
   destination: 'bak/2026-05-14T061711.030Z/notes.md',
 },);
 ```
 */
export async function copyIntoBackup(
  {
    sourcePath,
    destination,
  }: {
    readonly sourcePath: string;
    readonly destination: string;
  },
): Promise<void> {
  await cp(
    sourcePath,
    destination,
    {
      recursive: true,
      force: false,
      errorOnExist: true,
      preserveTimestamps: true,
    },
  );
}

/**
 Backs up one path inside `<backupRoot>/<timestamp>/`, under its own base name.
 
 @param sourcePath - File or directory to copy
 
 @param backupRoot - Existing folder the timestamped folder is created in
 
 @param now - Moment that names the timestamped folder; defaults to the current time
 
 @returns Path of the copy inside the timestamped folder
 
 @throws NodeJS.ErrnoException with code `ENOENT` when `backupRoot` does not exist, because the timestamped folder is created without its parents
 
 @throws NodeJS.ErrnoException with code `ERR_FS_CP_EEXIST` when the copy destination already exists, which is then left untouched
 
 @example
 ```ts
 const destination = await backupPath({
   sourcePath: 'notes.md',
   backupRoot: 'bak',
 },);
 ```
 */
export async function backupPath(
  {
    sourcePath,
    backupRoot,
    now = new Date(),
  }: {
    readonly sourcePath: string;
    readonly backupRoot: string;
    readonly now?: Date;
  },
): Promise<string> {
  /**
   Folder holding this backup, named after the moment it was taken.
   */
  const destinationDirectory = join(
    backupRoot,
    backupFolderName({ now, },),
  );
  await mkdir(destinationDirectory,);
  /**
   Path of the copy inside that folder.
   */
  const destination = join(
    destinationDirectory,
    basename(sourcePath,),
  );
  await copyIntoBackup({
    sourcePath,
    destination,
  },);
  return destination;
}
//endregion Copying
