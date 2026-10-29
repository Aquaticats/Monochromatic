/**
 Filesystem probes shared by the toolchain modules that treat an unusable path as "not there" while still
 surfacing programming errors. Node builtins only.
 */

import { access, } from 'node:fs/promises';

/**
 Whether a caught value is an operating-system file error: an `Error` that carries a string `code` such as
 `ENOENT` or `EACCES`.

 Callers use it to swallow only the failures that mean "this path is unusable" and rethrow everything else.

 @param error - Caught value to test

 @returns `true` when the value is an `Error` with a string `code`

 @example
 ```ts
 try {
   statSync('/missing',);
 } catch (error) {
   if (!isErrnoException(error,))
     throw error;
 }
 ```
 */
export function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return Error.isError(error,)
    && ('code' in error)
    && ((typeof error.code) === 'string');
}

/**
 Whether a path exists and is reachable by the current user.

 @param path - Absolute path to test

 @returns `true` when the path resolves; `false` for a missing or unreachable path

 @example
 ```ts
 await pathExists('/usr/bin/env',); // true
 ```
 */
export async function pathExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  } catch (error) {
    if (!isErrnoException(error,))
      throw error;
    return false;
  }
}
