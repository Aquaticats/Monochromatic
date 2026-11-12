/**
 The entry of another command of this package: its TypeScript source when the package is run from a checkout, and its built file
 when the package is installed. The two names differ only by extension, so one lookup serves both.

 @module
 */

import { access, } from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The extension of a TypeScript source entry, which Node runs by stripping the types.
 */
const SOURCE_EXTENSION = '.ts';

/**
 The extension of a built entry.
 */
const BUILT_EXTENSION = '.mjs';

/**
 Says whether a path is there, without reading it.

 @param path - the path to look for

 @returns whether it is there

 @example
 ```ts
 const there = await isThere('/pkg/src/drive-pi-subagent.ts',);
 ```
 */
async function isThere(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    // A path that is not there is exactly what the caller asked about.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Reads the entry of another command of this package.

 @param folder - the folder this command's own entry is in

 @param command - the command's name, without an extension

 @returns the path of its entry

 @throws Error when neither the source nor the built entry is there, which means the command was asked for by a name it does
 not have

 @example
 ```ts
 const launcher = await siblingCommandEntry({ folder: import.meta.dirname, command: 'start-pi-run-in-a-worktree', },);
 ```
 */
export async function siblingCommandEntry({
  folder,
  command,
}: {
  readonly folder: string;
  readonly command: string;
}): Promise<string> {
  /**
   The command's source entry.
   */
  const source = join(
    folder,
    `${command}${SOURCE_EXTENSION}`,
  );
  if (await isThere(source,)) {
    return source;
  }
  /**
   The command's built entry.
   */
  const built = join(
    folder,
    `${command}${BUILT_EXTENSION}`,
  );
  if (await isThere(built,)) {
    return built;
  }
  throw new Error(`this package has no command named ${command}: neither ${source} nor ${built} is there`,);
}
