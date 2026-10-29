import { randomUUID, } from 'node:crypto';
import {
  link,
  mkdir,
  rm,
  stat,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

import { caughtErrorHasCode, } from './error.ts';
import {
  type AtomicTempFileWriter,
  fsyncDirectory,
  writeTempFileDurably,
} from './write-atomic.ts';

//region Atomic create-if-absent constants and types

/**
 Suffix appended to same-directory temp files used for no-clobber creation.
 */
const TEMP_FILE_SUFFIX = '.tmp';

/**
 Sentinel returned when final destination already exists at create time.
 */
export const FILE_ALREADY_EXISTS: unique symbol = Symbol('file-enforcer/io/write-if-absent-atomic: file already exists',);

/**
 Result from atomic create-if-absent destination writes.
 */
export type WriteFileIfAbsentAtomicallyResult = number | typeof FILE_ALREADY_EXISTS;

//endregion Atomic create-if-absent constants and types

//region Destination existence helpers

/**
 Returns whether caught error means final destination already existed.
 
 @param error - Unknown caught value from hard-link creation.
 
 @returns Whether no-clobber creation should skip instead of replacing.
 
 @example
 ```ts
 const exists = destinationAlreadyExists(error);
 ```
 */
function destinationAlreadyExists(error: unknown,): boolean {
  return caughtErrorHasCode({
    error,
    code: 'EEXIST',
  },);
}

//endregion Destination existence helpers

//region Atomic create-if-absent destination writes

/**
 Writes file content through same-directory temp file (via {@link writeTempFileDurably}
 by default) and creates destination only if no directory entry exists at final path,
 fsyncing the directory afterward with {@link fsyncDirectory}.
 
 The temp file is fsynced before a hard link creates the destination name.
 Hard-link creation fails with `EEXIST`, detected via {@link destinationAlreadyExists},
 when another process or a dangling symlink already owns the final path,
 preserving that entry instead of replacing it.
 
 @param filePath - Destination file path to create.
 
 @param content - Destination content to persist.
 
 @param tempFileWriter - Optional {@link AtomicTempFileWriter} seam for fault-injection tests.
 
 @returns Destination mtime in whole milliseconds, or {@link FILE_ALREADY_EXISTS}.
 
 @throws Non-existence filesystem failures from temp writing, linking, cleanup, or stat.
 
 @example
 ```ts
 const result = await writeFileIfAbsentAtomically({ filePath: './defaults.json', content: '{}' });
 ```
 */
export async function writeFileIfAbsentAtomically(
  {
    filePath,
    content,
    tempFileWriter = writeTempFileDurably,
  }: {
    readonly content: string;
    readonly filePath: string;
    readonly tempFileWriter?: AtomicTempFileWriter;
  },
): Promise<WriteFileIfAbsentAtomicallyResult> {
  /**
   Directory containing final destination and same-directory temp file.
   */
  const destinationDirectory = dirname(filePath,);
  await mkdir(
    destinationDirectory,
    { recursive: true, },
  );
  /**
   Same-directory temp path used as durable source for final hard link.
   */
  const tempPath = `${filePath}.${String(process.pid,)}.${randomUUID()}${TEMP_FILE_SUFFIX}`;
  /**
   Cleanup handle for temp file when write, link, or cleanup fsync fails.
   */
  await using _tempCleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        tempPath,
        { force: true, },
      );
    },
  };
  await tempFileWriter({
    tempPath,
    content,
  },);
  try {
    await link(
      tempPath,
      filePath,
    );
  }
  catch (linkError: unknown) {
    if (destinationAlreadyExists(linkError,))
      return FILE_ALREADY_EXISTS;

    throw linkError;
  }
  await rm(
    tempPath,
    { force: true, },
  );
  await fsyncDirectory(destinationDirectory,);
  /**
   Actual post-link mtime used by watch echo suppression.
   */
  const destinationStat = await stat(filePath,);
  return Math.floor(destinationStat.mtimeMs,);
}

//endregion Atomic create-if-absent destination writes
