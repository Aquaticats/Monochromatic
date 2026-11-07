/**
 Tells whether a path exists, without throwing for a missing one.

 @module
 */

import { access, } from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';

/**
 Error code of a path that does not exist.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Tells whether a path exists.

 @param path - the path
 
 @returns true when it exists
 
 @throws Error when the path cannot be inspected for a reason other than absence

 @example
 ```ts
 import { pathExists, } from './path-exists.ts';
 
 const result = await pathExists(path,);
 ```
 */
export async function pathExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}
