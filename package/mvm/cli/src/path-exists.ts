/**
 Async filesystem existence helpers.
 
 @module
 */

import { access, } from 'node:fs/promises';

/**
 Error codes that mean nothing stands at the path.
 
 `ENOENT` covers a missing entry and `ENOTDIR` a component that is a file
 rather than a directory. Every other code (a directory the process may not
 search, an I/O error) says the probe could not answer, so it must not be read
 as "not there".
 */
const ABSENT_PATH_ERROR_CODES: ReadonlySet<string> = new Set([
  'ENOENT',
  'ENOTDIR',
],);

/**
 Checks whether an error says the probed path holds no entry.
 
 @param error - caught value from the probe
 
 @returns whether the path is absent rather than uninspectable
 */
function isAbsentPathError(error: unknown,): boolean {
  if (!(Error.isError(error,)))
    return false;
  if (!('code' in error))
    return false;
  if ((typeof error.code) !== 'string')
    return false;
  return ABSENT_PATH_ERROR_CODES.has(error.code,);
}

/**
 Checks whether a filesystem path exists without using sync Node APIs.
 
 Only an entry the kernel reports as missing answers `false`. A failure that
 leaves existence unknown, such as a directory this process may not search,
 reaches the caller.
 
 @param path - filesystem path to probe
 
 @returns whether the path exists and is accessible to this process
 
 @throws The probe's own failure when existence cannot be decided
 
 @example
 ```ts
 if (await pathExists('/var/lib/machines/example.qcow2')) {
   // cached image can be reused
 }
 ```
 */
export async function pathExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (isAbsentPathError(error,))
      return false;
    throw error;
  }
}
