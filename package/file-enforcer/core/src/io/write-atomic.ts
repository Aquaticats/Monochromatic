import { randomUUID, } from 'node:crypto';
import {
  mkdir,
  open,
  rename,
  rm,
  stat,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

import { caughtErrorHasCode, } from './error.ts';

//region Atomic write constants and types

/**
 Suffix appended to same-directory temp files used for destination replacement.
 */
const TEMP_FILE_SUFFIX = '.tmp';

/**
 Error codes meaning directory fsync is unsupported by platform or filesystem.
 */
const UNSUPPORTED_DIRECTORY_FSYNC_ERROR_CODES = [
  'EACCES',
  'EINVAL',
  'EISDIR',
  'ENOSYS',
  'ENOTSUP',
  'EPERM',
] as const;

/**
 Writes content to same-directory temp file before atomic rename.
 */
export type AtomicTempFileWriter = (
  args: {
    /**
     Same-directory temp path that will be renamed into place after writing.
     */
    readonly tempPath: string;

    /**
     Destination content to persist.
     */
    readonly content: string;
  },
) => Promise<void>;

//endregion Atomic write constants and types

//region File descriptor durability helpers

/**
 Returns whether caught error means directory fsync is unsupported.
 
 @param error - Unknown caught value from directory open or fsync.
 
 @returns Whether directory fsync should degrade to best effort.
 
 @example
 ```ts
 const unsupported = directoryFsyncUnsupported(error);
 ```
 */
export function directoryFsyncUnsupported(error: unknown,): boolean {
  return UNSUPPORTED_DIRECTORY_FSYNC_ERROR_CODES.some(function errorCodeMatches(code,): boolean {
    return caughtErrorHasCode({
      error,
      code,
    },);
  },);
}

/**
 Writes and fsyncs destination temp file before rename.
 
 @param tempPath - Same-directory temp path.
 
 @param content - Destination content to write.
 
 @example
 ```ts
 await writeTempFileDurably({ tempPath, content });
 ```
 */
export async function writeTempFileDurably(
  {
    tempPath,
    content,
  }: {
    readonly content: string;
    readonly tempPath: string;
  },
): Promise<void> {
  /**
   Writable temp-file handle fsynced before rename.
   */
  await using tempFile = await open(
    tempPath,
    'w',
  );
  await tempFile.writeFile(content,);
  await tempFile.sync();
}

/**
 Fsyncs directory containing renamed destination to persist directory entry,
 degrading to best effort when {@link directoryFsyncUnsupported} says the
 platform or filesystem cannot fsync directories.
 
 @param directoryPath - Directory path to fsync.
 
 @example
 ```ts
 await fsyncDirectory('/tmp/output-directory');
 ```
 */
export async function fsyncDirectory(directoryPath: string,): Promise<void> {
  try {
    /**
     Directory handle fsynced after destination rename.
     */
    await using directory = await open(
      directoryPath,
      'r',
    );
    await directory.sync();
  }
  catch (directoryFsyncError: unknown) {
    if (directoryFsyncUnsupported(directoryFsyncError,))
      return;

    throw directoryFsyncError;
  }
}

//endregion File descriptor durability helpers

//region Atomic destination replacement

/**
 Writes file content through same-directory temp file (via {@link writeTempFileDurably}
 by default) and atomic rename, fsyncing the directory afterward with {@link fsyncDirectory}.
 
 @param filePath - Destination file path to replace.
 
 @param content - Destination content to persist.
 
 @param tempFileWriter - Optional {@link AtomicTempFileWriter} seam for fault-injection tests.
 
 @returns Destination mtime in whole milliseconds after rename.
 
 @example
 ```ts
 await writeFileAtomically({ filePath: './CLAUDE.md', content: '# Generated' });
 ```
 */
export async function writeFileAtomically(
  {
    filePath,
    content,
    tempFileWriter = writeTempFileDurably,
  }: {
    readonly content: string;
    readonly filePath: string;
    readonly tempFileWriter?: AtomicTempFileWriter;
  },
): Promise<number> {
  /**
   Directory containing final destination and same-directory temp file.
   */
  const destinationDirectory = dirname(filePath,);
  await mkdir(
    destinationDirectory,
    { recursive: true, },
  );
  /**
   Same-directory temp path used for atomic replacement.
   */
  const tempPath = `${filePath}.${String(process.pid,)}.${randomUUID()}${TEMP_FILE_SUFFIX}`;
  /**
   Cleanup handle for temp file when write or rename fails.
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
  await rename(
    tempPath,
    filePath,
  );
  await fsyncDirectory(destinationDirectory,);
  /**
   Actual post-rename mtime used by watch echo suppression.
   */
  const destinationStat = await stat(filePath,);
  return Math.floor(destinationStat.mtimeMs,);
}

//endregion Atomic destination replacement
