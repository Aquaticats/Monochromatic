import { randomUUID, } from 'node:crypto';
import {
  mkdir,
  rename,
  rm,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

import { acquireManifestLock, } from './staleness-manifest-lock.ts';
import {
  manifestTempFilePath,
  removeOrphanedManifestTempFiles,
} from './staleness-manifest-temp-orphan.ts';
import {
  readManifestFromDisk,
  serializeManifest,
  type PersistableStalenessManifest,
} from './staleness-manifest-parse.ts';
import {
  MANIFEST_VERSION,
  type StalenessManifest,
} from './staleness-types.ts';
import {
  fsyncDirectory,
  writeTempFileDurably,
  type AtomicTempFileWriter,
} from './write-atomic.ts';

export { readManifestFromDisk, } from './staleness-manifest-parse.ts';

//region Atomic write and inter-process merge

/**
 Writes and fsyncs manifest temp file before rename, serializing with
 {@link serializeManifest} and durably writing via {@link writeTempFileDurably}.
 
 @param tempPath - Same-directory temp path.
 
 @param manifest - Manifest to serialize.
 
 @param tempFileWriter - Writer that creates the temp file; fault-injection seam.
 
 @mutates manifest - `JSON.stringify` may invoke hooks on manifest entries.
 
 @example
 ```ts
 await writeManifestTempFile({ tempPath, manifest });
 ```
 */
async function writeManifestTempFile(
  {
    tempPath,
    manifest,
    tempFileWriter,
  }: {
    manifest: PersistableStalenessManifest;
    readonly tempFileWriter: AtomicTempFileWriter;
    readonly tempPath: string;
  },
): Promise<void> {
  await tempFileWriter({
    tempPath,
    content: serializeManifest(manifest,),
  },);
}

/**
 Writes manifest through same-directory temp file via {@link writeManifestTempFile}
 and atomic rename, fsyncing the directory afterward with {@link fsyncDirectory}.
 
 @param manifestPath - Absolute manifest path.
 
 @param manifest - Manifest to persist.
 
 @param tempFileWriter - Writer that creates the temp file; fault-injection seam.
 
 @mutates manifest - `JSON.stringify` may invoke hooks on manifest entries.
 
 @example
 ```ts
 await writeManifestAtomically({ manifestPath, manifest, tempFileWriter });
 ```
 */
async function writeManifestAtomically(
  {
    manifestPath,
    manifest,
    tempFileWriter,
  }: {
    manifest: PersistableStalenessManifest;
    readonly manifestPath: string;
    readonly tempFileWriter: AtomicTempFileWriter;
  },
): Promise<void> {
  /**
   Directory containing manifest and same-directory temp files.
   */
  const manifestDirectory = dirname(manifestPath,);
  await mkdir(
    manifestDirectory,
    { recursive: true, },
  );
  /**
   Same-directory temp path used for atomic replacement.
   */
  const tempPath = manifestTempFilePath({
    manifestPath,
    processId: process.pid,
    uniqueToken: randomUUID(),
  },);
  /**
   Cleanup handle for temp file when write or rename fails in this process.
   A process that dies before disposal leaves the file for
   {@link removeOrphanedManifestTempFiles} to remove on a later locked write.
   */
  await using _tempCleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        tempPath,
        { force: true, },
      );
    },
  };
  await writeManifestTempFile({
    tempPath,
    manifest,
    tempFileWriter,
  },);
  await rename(
    tempPath,
    manifestPath,
  );
  await fsyncDirectory(manifestDirectory,);
}

/**
 Merges cached entries with latest disk state under a lock from
 {@link acquireManifestLock}, removing temp files orphaned by dead writers via
 {@link removeOrphanedManifestTempFiles}, reading the existing manifest via
 {@link readManifestFromDisk}, then writes atomically with
 {@link writeManifestAtomically}.
 
 @param manifestPath - Absolute manifest path.
 
 @param manifest - Cached manifest changes to persist.
 
 @param tempFileWriter - Optional writer that creates the temp file; fault-injection seam for tests.
 
 @returns Merged manifest written to disk.
 
 @example
 ```ts
 const merged = await writeMergedManifest({ manifestPath, manifest });
 ```

 @internal
 */
export async function writeMergedManifest(
  {
    manifestPath,
    manifest,
    tempFileWriter = writeTempFileDurably,
  }: {
    readonly manifest: Readonly<PersistableStalenessManifest>;
    readonly manifestPath: string;
    readonly tempFileWriter?: AtomicTempFileWriter;
  },
): Promise<StalenessManifest> {
  /**
   Lock handle released after merged manifest has been written.
   */
  await using _lock = await acquireManifestLock(manifestPath,);
  await removeOrphanedManifestTempFiles(manifestPath,);
  /**
   Latest manifest state from disk while lock is held.
   */
  const existingManifest = await readManifestFromDisk(manifestPath,);
  /**
   Manifest preserving entries from disk plus this process's changes.
   */
  const mergedManifest: StalenessManifest = {
    version: MANIFEST_VERSION,
    entries: {
      ...existingManifest.entries,
      ...manifest.entries,
    },
  };
  await writeManifestAtomically({
    manifestPath,
    manifest: mergedManifest,
    tempFileWriter,
  },);
  return mergedManifest;
}

//endregion Atomic write and inter-process merge
