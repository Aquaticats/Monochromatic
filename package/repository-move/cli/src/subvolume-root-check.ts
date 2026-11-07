/**
 The test for a subvolume root: inode 256 on btrfs, or a marker file in the unit tests.

 @module
 */

import { stat, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  MISSING_PATH_CODE,
  pathExists,
} from './completion-records.ts';
import { SUBVOLUME_ROOT_INODE, } from './constants.ts';
import { errorCodeOf, } from './error-code-of.ts';

/**
 Tells whether a folder is a subvolume root: inode 256 on btrfs, or the marker file in tests.

 @param path - the folder
 
 @param marker - name of the marker file that stands for a subvolume in tests; empty on a real filesystem
 
 @returns true for a subvolume root

 @example
 ```ts
 import { isSubvolume, } from './inventory.ts';
 
 const result = await isSubvolume({ path, marker, },);
 ```
 */
export async function isSubvolume(
  {
    path,
    marker,
  }: {
    readonly path: string;
    readonly marker: string;
  },
): Promise<boolean> {
  if (marker !== '') {
    return await pathExists(join(
      path,
      marker,
    ),);
  }
  try {
    return (await stat(
      path,
      { bigint: true, },
    )).ino === SUBVOLUME_ROOT_INODE;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}
