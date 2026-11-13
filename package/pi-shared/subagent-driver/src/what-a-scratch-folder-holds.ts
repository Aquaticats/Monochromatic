/**
 Reading what a scratch folder holds, one entry at a time: the names inside a folder, what one entry is without following a
 symbolic link, and the words a failure gives.

 Every read answers with what it found or with the reason it found nothing, so a caller that meets an entry it cannot read names that
 entry and goes on to the next rather than ending the walk.

 @module
 */

import type { Stats, } from 'node:fs';
import {
  lstat,
  readdir,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';

/**
 Says why one entry could not be read or removed, in the words the operating system gave.

 @param error - what was caught

 @returns the reason, which names the path and the failure

 @example
 ```ts
 const reason = reasonOfFailure({ error, },);
 ```
 */
export function reasonOfFailure({ error, }: { readonly error: unknown },): string {
  if (Error.isError(error,) && (error.message !== '')) {
    return error.message;
  }
  return String(error,);
}

/**
 Reads the names inside one folder, or why it could not be read and the code that failure carries.

 @param folder - the folder to read

 @returns the names, or the reason and code there are none to read

 @example
 ```ts
 const read = await namesIn('/home/a/temp/agent/a-scratch');
 ```
 */
export async function namesIn(folder: string,): Promise<{
  readonly names?: readonly string[];
  readonly reason?: string;
  readonly code?: string
}> {
  try {
    return { names: await readdir(folder,), };
  }
  catch (error) {
    return {
      reason: reasonOfFailure({ error, },),
      code: errorCodeOf(error,),
    };
  }
}

/**
 Reads what one entry is, without following a symbolic link, or why it could not be read.

 @param path - the entry to read

 @returns what it is, or the reason and code it could not be read

 @example
 ```ts
 const read = await entryIn('/home/a/temp/agent/a-scratch/report.md');
 ```
 */
export async function entryIn(path: string,): Promise<{
  readonly info?: Stats;
  readonly reason?: string;
  readonly code?: string
}> {
  try {
    return { info: await lstat(path,), };
  }
  catch (error) {
    return {
      reason: reasonOfFailure({ error, },),
      code: errorCodeOf(error,),
    };
  }
}
