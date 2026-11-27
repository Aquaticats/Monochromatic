/**
 The files of a source tree, each one visited once and never read through a link.

 A source folder of this repository holds a `uses/` folder whose entries are links to the folders its code uses
 (`AGENTS.md` rules `LYD`, `LYL` and `LYF`), so a read of the tree that follows a link reaches the same module under one
 more path for every chain of `uses/` folders above it. Measured 2026-11-27 for issue 6580: a recursive read of this
 package's `src/native` returned 165,978 entries and 163,498 `.rs` names where the tree holds 4,234 entries and 3,905
 Rust sources, and joining that much text ended the planted-controls unit test with
 `RangeError: Invalid string length`.

 So a script that reads sources walks them through this one walk: an entry that is a link, a socket, a fifo or a device
 is not a source file, and a link is never descended.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Every regular file below one folder, each once.

 A folder is entered once, and only where it is a real folder of the tree: a link is listed as nothing and never
 descended, so a tree whose links make a loop still ends, and a link to a file is not a second reading of that file.

 The walk recurses over the folders of the tree rather than over a caller's input, and each folder's own files are
 found before the folders below it are entered, so the folders of one level are read at once.

 @param root - absolute path of the folder to walk

 @returns absolute paths of the files under it, in code unit order

 @throws Error when a folder of the tree cannot be read

 @example
 ```ts
 const files = await sourceTreeFiles({ root: '/repo/package/git-policy/cli/src/native' });
 files.filter(function isRust(file) {
   return file.endsWith('.rs');
 });
 ```
 */
export async function sourceTreeFiles({ root, }: {
  readonly root: string;
},): Promise<readonly string[]> {
  return (await filesUnder({ folder: root, },)).toSorted();
}

/**
 Every regular file below one folder, in the order the folders were read.

 @param folder - absolute path of the folder to walk

 @returns absolute paths of the files under it

 @throws Error when a folder of the tree cannot be read

 @example
 ```ts
 const files = await filesUnder({ folder: '/repo/package/git-policy/cli/src/native' });
 ```
 */
async function filesUnder({ folder, }: {
  readonly folder: string;
},): Promise<readonly string[]> {
  /**
   Names and own types of what the folder holds.
   */
  const entries = await readdir(
    folder,
    { withFileTypes: true, },
  );
  /**
   Files the folder holds itself, which are read before anything under it is entered.
   */
  const files: string[] = [];
  /**
   Folders under this one, each a real folder and never a link.
   */
  const deeper: string[] = [];
  for (const entry of entries) {
    /**
     Path of this entry, which a caller reads when it is a file.
     */
    const path = join(
      folder,
      entry.name,
    );
    if (entry.isDirectory())
      deeper.push(path,);
    else if (entry.isFile())
      files.push(path,);
  }
  /**
   Files under each folder below this one, read in parallel.
   */
  const nested = await Promise.all(deeper.map(async function underIt(path) {
    return await filesUnder({ folder: path, },);
  },),);
  return [
    ...files,
    ...nested.flat(),
  ];
}
