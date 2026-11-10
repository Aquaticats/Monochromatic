import type { Dirent, } from 'node:fs';
import { readdir, } from 'node:fs/promises';
import {
  join,
  relative,
  sep,
} from 'node:path';

//region Archive entry shape: one file selected for the plugin archive

/**
 File queued for inclusion in plugin archive.

 `archivePath` is what the ZIP entry is named,
 `diskPath` is where its bytes are read from,
 so a test can pair the two without a filesystem.

 @example
 ```ts
 const entry: ArchiveEntry = {
   archivePath: 'META-INF/plugin.xml',
   diskPath: 'META-INF/plugin.xml',
 };
 ```
 */
export type ArchiveEntry = Readonly<{
  archivePath: string;
  diskPath: string;
}>;

//endregion Archive entry shape

//region Entry ordering: name order, so filesystem order never reaches the archive

/**
 Sort directory entries by name for deterministic archive entry order.

 `readdir` returns entries in whatever order the filesystem hands them over,
 and the committed JAR must not change when that order does.

 @param left - Directory entry compared before right entry

 @param right - Directory entry compared after left entry

 @returns Sort order for left relative to right

 @example
 ```ts
 compareDirentNames({ left, right, });
 ```
 */
export function compareDirentNames(
  {
    left,
    right,
  }: Readonly<{
    left: Readonly<Dirent>;
    right: Readonly<Dirent>;
  }>,
): number {
  if (left.name < right.name)
    return -1;
  if (left.name > right.name)
    return 1;
  return 0;
}

//endregion Entry ordering

//region Archive paths: forward slashes whatever separator the running platform uses

/**
 Convert platform-native package path to ZIP-standard forward-slash path.

 The ZIP format stores `/` separators,
 while `join` writes the running platform's separator,
 so a JAR built on Windows must not carry backslash entry names.

 @param diskPath - Package-relative path needing archive normalization

 @param pathSeparator - Separator `diskPath` uses; defaults to this platform's,
   and a test passes the Windows separator on a POSIX host to reach that branch

 @returns Archive entry path using forward slash separators

 @example
 ```ts
 toArchivePath({ diskPath: join('META-INF', 'plugin.xml',), });
 ```
 */
export function toArchivePath(
  {
    diskPath,
    pathSeparator = sep,
  }: Readonly<{
    diskPath: string;
    pathSeparator?: string;
  }>,
): string {
  return relative(
    '.',
    diskPath,
  )
    .split(pathSeparator,)
    .join('/',);
}

//endregion Archive paths

//region Recursive listing: regular files under one plugin root, depth first

/**
 Recursively list regular files under root directory for archive inclusion.

 Directories are descended, regular files are returned,
 and anything else is refused rather than silently dropped:
 a symbolic link or device node in the plugin tree would otherwise produce a JAR
 that installs and then misses a file the author believed it carried.

 @param root - Directory path whose regular files enter archive

 @returns Archive entries sorted by package-relative path

 @throws When input tree contains unsupported special files

 @example
 ```ts
 const entries = await listArchiveEntries({ root: 'META-INF', },);
 ```
 */
export async function listArchiveEntries(
  { root, }: Readonly<{ root: string; }>,
): Promise<readonly ArchiveEntry[]> {
  /**
   Directory entries found immediately under current root.
   */
  const directoryEntries = await readdir(
    root,
    { withFileTypes: true, },
  );
  /**
   Directory entries sorted before traversal to keep archive deterministic.
   */
  const sortedDirectoryEntries = directoryEntries.toSorted(function compareDirectoryEntries(
    left: Readonly<Dirent>,
    right: Readonly<Dirent>,
  ): number {
    return compareDirentNames({
      left,
      right,
    },);
  },);
  /**
   Per-entry archive fragments, flattened before returning.
   */
  const archiveEntryGroups = await Promise.all(
    sortedDirectoryEntries.map(function archiveEntriesForDirent(
      dirent: Readonly<Dirent>,
    ): Promise<readonly ArchiveEntry[]> {
      /**
       Package-relative disk path for current directory entry.
       */
      const diskPath = join(
        root,
        dirent.name,
      );
      if (dirent.isDirectory())
        return listArchiveEntries({ root: diskPath, },);
      if (dirent.isFile()) {
        return Promise.resolve([
          {
            archivePath: toArchivePath({ diskPath, },),
            diskPath,
          },
        ],);
      }
      throw new Error(`Unsupported plugin archive input: ${diskPath}`,);
    },),
  );
  return archiveEntryGroups.flat();
}

//endregion Recursive listing
