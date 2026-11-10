/**
 Listing of one directory inside the confinement root.
 
 The Electron main process answers the renderer's list-directory channel
 through `listDirectory`, so the confinement guard and the read that follows it
 live together here; nothing else in the main process reads a directory.
 
 @example
 ```ts
 await listDirectory({ path: '/home/docs', rootDirectory: '/home' });
 ```
 
 @packageDocumentation
 */

import type { Dirent, } from 'node:fs';
import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  BridgeEntryKind,
  BridgeFileEntry,
} from './bridge-types.js';
import { confinePath, } from './confinement.js';
import { sortBridgeEntries, } from './listing-sort.js';

/**
 Classifies one `readdir` dirent probe into a bridge entry kind, checking
 symlinks first because a symlink also answers the directory probe on some
 platforms.
 
 @param directory - Whether the dirent answered the directory probe.
 
 @param symlink - Whether the dirent answered the symlink probe.
 
 @returns Bridge entry kind.
 
 @example
 ```ts
 kindOfDirent({ directory: true, symlink: false });
 ```
 */
function kindOfDirent(
  {
    directory,
    symlink,
  }: {
    readonly directory: boolean;
    readonly symlink: boolean;
  },
): BridgeEntryKind {
  if (symlink)
    return 'symlink';

  if (directory)
    return 'directory';

  return 'file';
}

/**
 Lists one directory into sorted bridge entries.
 
 @param path - Renderer-supplied directory path.
 
 @param rootDirectory - Configured root every listing request must stay under.
 
 @returns Sorted entries of the directory.
 
 @throws ListingOutsideRootError when the request escapes the root, lexically or through a symbolic link.
 
 @example
 ```ts
 await listDirectory({ path: '/home', rootDirectory: '/home' });
 ```
 */
export async function listDirectory(
  {
    path,
    rootDirectory,
  }: {
    readonly path: string;
    readonly rootDirectory: string;
  },
): Promise<readonly BridgeFileEntry[]> {
  /**
   Confined absolute directory path.
   */
  const confined = await confinePath({
    path,
    rootDirectory,
  },);

  /**
   Raw directory entries with type information, without following symlinks.
   */
  const dirents = await readdir(
    confined,
    { withFileTypes: true, },
  );

  return sortBridgeEntries({
    entries: dirents.map(function toBridgeEntry(
      dirent: ForeignBorrowed<Dirent>,
    ): BridgeFileEntry {
      return {
        kind: kindOfDirent({
          directory: dirent.isDirectory(),
          symlink: dirent.isSymbolicLink(),
        },),
        name: dirent.name,
        path: join(
          confined,
          dirent.name,
        ),
      };
    },),
  },);
}
