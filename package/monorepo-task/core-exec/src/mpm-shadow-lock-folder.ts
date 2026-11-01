/**
 Looking inside the `mpm-shadow` folder of the toolchain root: which version folders exist, whether an install finished, where
 its program is, and removing what a killed task left. The layout is the one `mpm-shadow install` keeps for an archive-route
 tool that this repository publishes (`archive_staging.rs`, `published_archive.rs`, `layout.rs`).

 Imports only `node:` modules, so it runs from source on a checkout without `node_modules`.

 @module
 */

import {
  lstat,
  readdir,
  rm,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { isNotFound, } from './mpm-shadow-lock-fetch.ts';
import {
  COMPLETE_MARKER,
  compareVersions,
  CURRENT_LINK_NAME,
  TOOL_DIRECTORY_NAME,
} from './mpm-shadow-lock-read.ts';

//region Finding the program in an install

/**
 How many folder levels below a version folder the program is searched for (`MAX_BIN_SEARCH_DEPTH` in `layout.rs`).
 */
export const SEARCH_DEPTH = 3;

/**
 The `mpm-shadow` program inside a folder: a regular file of that name in the folder itself or up to three folders below, the
 same search `mpm-shadow` makes (`layout.rs`, `find_bin_under`).

 @param directory - Folder to search

 @param depth - How many more levels to go down

 @returns Path of the file, or an empty string when there is none

 @example
 ```ts
 const program = await findProgramUnder({ directory: versionFolder, depth: 3, },);
 ```
 */
export async function findProgramUnder(
  {
    directory,
    depth,
  }: {
    readonly directory: string;
    readonly depth: number;
  },
): Promise<string> {
  /**
   The file straight in the folder.
   */
  const direct = join(
    directory,
    TOOL_DIRECTORY_NAME,
  );
  if (await stat(direct,)
    .then(
    function isFile(found,) {
      return found.isFile();
    },
    function isMissing(error: unknown,) {
      if (isNotFound(error,))
        return false;
      throw error;
    },
    ))
    return direct;
  if (depth === 0)
    return '';
  /**
   Subfolders, in name order.
   */
  const entries = await readdir(
    directory,
    { withFileTypes: true, },
  );
  /**
   Names of the subfolders.
   */
  const folders = entries.filter(function isFolder(entry,) {
    return entry.isDirectory();
  },)
    .map(function toName(entry,) {
      return entry.name;
    },)
    .toSorted();
  /**
   What each subfolder holds.
   */
  const found = await Promise.all(folders.map(function search(name,) {
    return findProgramUnder({
      directory: join(
        directory,
        name,
      ),
      depth: depth - 1,
    },);
  },),);
  return found.find(function hasProgram(path,) {
    return path !== '';
  },) ?? '';
}

/**
 Whether a version folder holds a finished install: the completion marker as a regular file (a link of that name does not
 count) and the program (`published_archive.rs`, `is_finished`).

 @param versionDirectory - A folder `<toolchain root>/mpm-shadow/<version>`

 @returns True when the install finished

 @example
 ```ts
 await isFinishedVersion(join(toolDirectory, '0.2.1',),); // true
 ```
 */
export async function isFinishedVersion(versionDirectory: string,): Promise<boolean> {
  /**
   Whether the marker is a regular file.
   */
  const marked = await lstat(join(
    versionDirectory,
    COMPLETE_MARKER,
  ),)
    .then(
    function isFile(found,) {
      return found.isFile();
    },
    function isMissing(error: unknown,) {
      if (isNotFound(error,))
        return false;
      throw error;
    },
    );
  return marked && (await findProgramUnder({
    directory: versionDirectory,
    depth: SEARCH_DEPTH,
  },) !== '');
}

/**
 The version folders of the tool, newest first: plain folders that are not hidden and not `current`.

 @param toolDirectory - `<toolchain root>/mpm-shadow`

 @returns Folder names; empty when the tool folder does not exist

 @example
 ```ts
 const names = await listVersionNames(join(toolchainRoot, TOOL_DIRECTORY_NAME,),); // ['0.2.2', '0.2.1']
 ```
 */
export async function listVersionNames(toolDirectory: string,): Promise<readonly string[]> {
  try {
    /**
     Everything in the tool folder.
     */
    const entries = await readdir(
      toolDirectory,
      { withFileTypes: true, },
    );
    return entries.filter(function isVersionFolder(entry,) {
      return entry.isDirectory() && (!entry.name
        .startsWith('.',))
        && (entry.name !== CURRENT_LINK_NAME);
    },)
      .map(function toName(entry,) {
        return entry.name;
      },)
      .toSorted(function newestFirst(
        left,
        right,
      ) {
        return compareVersions({
          left: right,
          right: left,
        },);
      },);
  }
  catch (error) {
    if (isNotFound(error,))
      return [];
    throw error;
  }
}

//endregion Finding the program in an install

//region Cleaning what a killed task left

/**
 Whether the process id in a hidden leftover's name belongs to a process that is gone.

 @param pid - Process id taken from the name

 @returns True when no such process exists; false when it runs, exists under another user, or the id is 0

 @example
 ```ts
 isGone(4242,);
 ```
 */
function isGone(pid: number,): boolean {
  if (pid === 0)
    return false;
  try {
    process.kill(
      pid,
      0,
    );
    return false;
  }
  catch (error) {
    return Error.isError(error,) && ('code' in error)
      && (error.code === 'ESRCH');
  }
}

/**
 Whether a name is `.<version>.<purpose>.<pid>` for one of the purposes this module and `mpm-shadow` give a hidden sibling.

 @param name - Entry name in the tool folder

 @returns True for a staging folder, a download or a moved-aside copy

 @example
 ```ts
 hasLeftoverPurpose('.0.2.1.staging.4242',); // true
 ```
 */
function hasLeftoverPurpose(name: string,): boolean {
  return [
    '.staging.',
    '.download.',
    '.replaced.',
  ].some(function namesPurpose(purpose,) {
    return name.includes(purpose,);
  },);
}

/**
 The process id at the end of a leftover's name.

 @param name - Entry name in the tool folder

 @returns The number after the last dot, or 0 when that is not a positive integer

 @example
 ```ts
 pidOf('.0.2.1.staging.4242',); // 4242
 ```
 */
function pidOf(name: string,): number {
  /**
   The text after the last dot as a number.
   */
  const pid = Number(name.slice(name.lastIndexOf('.',) + 1,),);
  return Number.isInteger(pid,) && (pid > 0) ? pid : 0;
}

/**
 Removes the hidden leftovers a killed task left in the tool folder: staging folders, downloads and moved-aside copies that
 were named after the id of a process that is gone (`.<version>.staging.<pid>`, `.<version>.download.<pid>`,
 `.<version>.replaced.<pid>`).

 @param toolDirectory - `<toolchain root>/mpm-shadow`

 @param log - Prints one note for the person running the task

 @example
 ```ts
 await removeStaleHidden({ toolDirectory, log, },);
 ```
 */
export async function removeStaleHidden(
  {
    toolDirectory,
    log,
  }: {
    readonly toolDirectory: string;
    readonly log: (line: string,) => void;
  },
): Promise<void> {
  /**
   Everything in the tool folder.
   */
  const entries = await readdir(
    toolDirectory,
    { withFileTypes: true, },
  );
  /**
   The hidden leftovers whose process is gone.
   */
  const stale = entries.map(function toName(entry,) {
    return entry.name;
  },)
    .filter(function isLeftoverOfAGoneProcess(name,) {
      return name.startsWith('.',) && hasLeftoverPurpose(name,)
        && isGone(pidOf(name,),);
    },);
  await Promise.all(stale.map(async function remove(name,) {
    await rm(
      join(
        toolDirectory,
        name,
      ),
      {
        recursive: true,
        force: true,
      },
    );
    log(`removed ${join(
      toolDirectory,
      name,
    )}, left by a task that was killed`,);
  },),);
}

//endregion Cleaning what a killed task left
