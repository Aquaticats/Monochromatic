/**
 Whether a host path is there.

 The answer distinguishes "not there" from "could not be read": only the
 kernel's own `ENOENT` and `ENOTDIR` answer absent, so an unreadable directory
 reaches the caller as the failure it is instead of being taken for a missing
 file.

 @module
 */

import { stat, } from 'node:fs/promises';

/**
 Reports whether something is at `path`.

 @param path - host path to look at

 @returns whether the path exists

 @throws when the path cannot be examined for any reason other than absence

 @example
 ```ts
 await fileExists('/etc/hostname'); // true
 ```
 */
export async function fileExists(path: string,): Promise<boolean> {
  try {
    await stat(path,);
    return true;
  }
  catch (error) {
    if ((Error.isError(error,)) && ('code' in error) && ((error.code === 'ENOENT') || (error.code === 'ENOTDIR'))) {
      return false;
    }
    throw error;
  }
}
