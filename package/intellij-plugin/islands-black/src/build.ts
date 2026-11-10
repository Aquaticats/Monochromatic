import {
  readFile,
  writeFile,
} from 'node:fs/promises';

import { ZipWriter, } from '@monochromatic-dev/zip-writer-core/ts';

import {
  type ArchiveEntry,
  listArchiveEntries,
} from './archive-entries.ts';
import { ARCHIVE_MODIFIED_AT, } from './archive-timestamp.ts';

//region Archive constants: output path and the directories an install needs

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

//endregion Archive constants

//region Archive entry shapes: the listing helpers return entries, the writer takes bytes

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

//endregion Archive entry shapes

//region Build flow: list every bundled file, read it, then write one archive

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

 Every member carries the archive's fixed instant, so two builds of the same
 inputs produce the same bytes and a rebuild never dirties the committed JAR.
 */
const zipWriter = new ZipWriter({ modifiedAt: ARCHIVE_MODIFIED_AT, },);

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

//endregion Build flow
