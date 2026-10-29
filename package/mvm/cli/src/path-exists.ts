/**
 Async filesystem existence helpers.
 
 @module
 */

import { access, } from 'node:fs/promises';

/**
 Checks whether a filesystem path exists without using sync Node APIs.
 
 @param path - filesystem path to probe
 
 @returns whether the path exists and is accessible to this process
 
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
    if (!(Error.isError(error,)))
      throw error;

    return false;
  }
}
