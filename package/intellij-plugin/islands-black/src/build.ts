import type { Dirent, } from 'node:fs';
import {
  readFile,
  readdir,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  relative,
  sep,
} from 'node:path';

import { ZipWriter, } from '@monochromatic-dev/zip-writer-core/ts';

/**
 Output plugin archive path relative to package root.
 
 @example
 ```ts
 await writeFile(JAR_PATH, zipWriter.build(),);
 ```
 */
const JAR_PATH = 'islands-black.jar';

/**
 Plugin directories included in installable archive.
 
 @example
 ```ts
 const entries = (await Promise.all(ARCHIVE_ROOTS.map(listRoot,),)).flat();
 ```
 */
const ARCHIVE_ROOTS = [
  'META-INF',
  'theme',
  'scheme',
] as const;

/**
 File queued for inclusion in plugin archive.
 
 @example
 ```ts
 const entry: ArchiveEntry = {
   archivePath: 'META-INF/plugin.xml',
   diskPath: 'META-INF/plugin.xml',
 };
 ```
 */
type ArchiveEntry = Readonly<{
  archivePath: string;
  diskPath: string;
}>;

/**
 File content ready to copy into plugin archive.
 
 @example
 ```ts
 const entryContent: ArchiveEntryContent = {
   archivePath: 'META-INF/plugin.xml',
   content: await readFile('META-INF/plugin.xml',),
 };
 ```
 */
type ArchiveEntryContent = Readonly<{
  archivePath: string;
  content: Uint8Array;
}>;

/**
 Sort directory entries by name for deterministic archive entry order.
 
 @param left - Directory entry compared before right entry
 
 @param right - Directory entry compared after left entry
 
 @returns Sort order for left relative to right
 
 @example
 ```ts
 compareDirentNames({ left, right, });
 ```
 */
function compareDirentNames(
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

/**
 Convert platform-native package path to ZIP-standard forward-slash path.
 
 @param diskPath - Package-relative path needing archive normalization
 
 @returns Archive entry path using forward slash separators
 
 @example
 ```ts
 toArchivePath({ diskPath: join('META-INF', 'plugin.xml',), });
 ```
 */
function toArchivePath(
  { diskPath, }: Readonly<{ diskPath: string; }>,
): string {
  return relative(
    '.',
    diskPath,
  )
    .split(sep,)
    .join('/',);
}

/**
 Recursively list regular files under root directory for archive inclusion.
 
 @param root - Directory path whose regular files enter archive
 
 @returns Archive entries sorted by package-relative path
 
 @throws When input tree contains unsupported special files
 
 @example
 ```ts
 const entries = await listArchiveEntries({ root: 'META-INF', },);
 ```
 */
async function listArchiveEntries(
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

/**
 Archive entries for all plugin roots in deterministic root order.
 */
const archiveEntries = (await Promise.all(
  ARCHIVE_ROOTS.map(function archiveEntriesForRoot(root: string,): Promise<readonly ArchiveEntry[]> {
    return listArchiveEntries({ root, },);
  },),
)).flat();

/**
 ZIP writer used to create installable JetBrains plugin JAR.
 */
const zipWriter = new ZipWriter();

/**
 File contents read concurrently before insertion-order-preserving archive writes.
 */
const archiveEntryContents = await Promise.all(
  archiveEntries.map(async function readArchiveEntryContent(
    archiveEntry: ArchiveEntry,
  ): Promise<ArchiveEntryContent> {
    /**
     File bytes copied verbatim into archive entry.
     */
    const content = await readFile(archiveEntry.diskPath,);
    return {
      archivePath: archiveEntry.archivePath,
      content,
    };
  },),
);

for (const archiveEntryContent of archiveEntryContents) {
  zipWriter.add(
    archiveEntryContent.archivePath,
    archiveEntryContent.content,
  );
}

await writeFile(
  JAR_PATH,
  zipWriter.build(),
);

console.info(`Built ${JAR_PATH} with ${archiveEntries.length} files.`,);
