/**
 Reads the names inside one folder, or answers none when the folder is not there.

 A test that asks about a folder a command may or may not have made needs this answer rather than a throw, and a folder that is there and
 cannot be read is still carried out as the failure it is.

 @internal

 @module
 */

import { readdir, } from 'node:fs/promises';

/**
 No name, as an empty list, for a folder that is not there.

 @param folder - the folder to read

 @returns the names inside it, or none when it is not there

 @throws the error a read gave, other than a folder that is not there

 @example
 ```ts
 const names = await namesInAFolderThatMayNotBeThere({ folder: '/runs/q/claimed' });
 ```
 */
export async function namesInAFolderThatMayNotBeThere({
  folder,
}: {
  readonly folder: string;
},): Promise<readonly {
  readonly name: string;
  readonly isFile: () => boolean
}[]> {
  try {
    return await readdir(
      folder,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    // A folder nothing has made yet holds no names, which is not a failure to read it.
    if (String(error,)
      .includes('ENOENT',)) {
      return [];
    }
    throw error;
  }
}
