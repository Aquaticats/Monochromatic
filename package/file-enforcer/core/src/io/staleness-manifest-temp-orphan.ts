import {
  readdir,
  rm,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { l as parentLogger, } from '../logger.ts';
import { caughtErrorMessage, } from './staleness-manifest-error.ts';
import { processAppearsAlive, } from './staleness-manifest-lock-owner.ts';

//region Manifest temp file naming

/**
 Tagged logger for manifest temp file naming and orphan removal.
 */
const l = tagged({
  tag: 'stalenessManifestTempOrphan',
  l: parentLogger,
},);

/**
 Suffix appended to manifest path for atomic-write temp files.
 */
export const MANIFEST_TEMP_FILE_SUFFIX = '.tmp';

/**
 Separator between the manifest file name, the writer process id, and the unique token in a temp file name.
 */
const MANIFEST_TEMP_FILE_SEPARATOR = '.';

/**
 Number of separator-delimited segments between the manifest file name and the temp suffix:
 the writer process id and the unique token.
 */
const MANIFEST_TEMP_FILE_SEGMENT_COUNT = 2;

/**
 Sentinel for a directory entry that is not a temp file of the manifest.
 */
const NOT_A_MANIFEST_TEMP_FILE: unique symbol = Symbol('file-enforcer/io/staleness-manifest-temp-orphan: directory entry is not a manifest temp file',);

/**
 Process id embedded in a manifest temp file name, or {@link NOT_A_MANIFEST_TEMP_FILE}
 when the entry does not follow the naming scheme of {@link manifestTempFilePath}.
 */
type ManifestTempFileWriter = number | typeof NOT_A_MANIFEST_TEMP_FILE;

/**
 Builds the same-directory temp path a manifest writer uses for atomic replacement.
 The embedded process id lets {@link removeOrphanedManifestTempFiles} attribute
 a leftover file to the process that wrote it.

 @param manifestPath - Absolute manifest path.

 @param processId - Process id of the writer.

 @param uniqueToken - Per-write token keeping concurrent temp paths of one process apart.

 @returns Temp path of the form `<manifest>.<pid>.<token>.tmp`.

 @example
 ```ts
 const tempPath = manifestTempFilePath({ manifestPath, processId: process.pid, uniqueToken: randomUUID() });
 ```
 */
export function manifestTempFilePath(
  {
    manifestPath,
    processId,
    uniqueToken,
  }: {
    readonly manifestPath: string;
    readonly processId: number;
    readonly uniqueToken: string;
  },
): string {
  return [
    manifestPath,
    String(processId,),
    uniqueToken,
  ].join(MANIFEST_TEMP_FILE_SEPARATOR,)
  + MANIFEST_TEMP_FILE_SUFFIX;
}

/**
 Extracts the writer process id from a directory entry name.
 Accepts only `<manifest file name>.<decimal pid>.<token>.tmp`, so temp files of
 other destinations that share the manifest file name as a prefix are never claimed.

 @param entry - Directory entry name.

 @param manifestFileName - File name of the manifest (no directory part).

 @returns Writer process id, or {@link NOT_A_MANIFEST_TEMP_FILE} when the entry does not match.

 @example
 ```ts
 const writer = manifestTempFileWriter({ entry: 'm.json.42.abc.tmp', manifestFileName: 'm.json' });
 ```
 */
function manifestTempFileWriter(
  {
    entry,
    manifestFileName,
  }: {
    readonly entry: string;
    readonly manifestFileName: string;
  },
): ManifestTempFileWriter {
  /**
   Leading part every temp file of this manifest shares.
   */
  const prefix = `${manifestFileName}${MANIFEST_TEMP_FILE_SEPARATOR}`;
  if (!entry.startsWith(prefix,))
    return NOT_A_MANIFEST_TEMP_FILE;
  if (!entry.endsWith(MANIFEST_TEMP_FILE_SUFFIX,))
    return NOT_A_MANIFEST_TEMP_FILE;
  if (entry.length < prefix.length + MANIFEST_TEMP_FILE_SUFFIX.length)
    return NOT_A_MANIFEST_TEMP_FILE;

  /**
   Segments between prefix and suffix: process id and unique token.
   */
  const segments = entry
    .slice(
      prefix.length,
      entry.length - MANIFEST_TEMP_FILE_SUFFIX.length,
    )
    .split(MANIFEST_TEMP_FILE_SEPARATOR,);
  if (segments.length !== MANIFEST_TEMP_FILE_SEGMENT_COUNT)
    return NOT_A_MANIFEST_TEMP_FILE;

  /**
   Decimal process id text and unique token text.
   */
  const [
    processIdText = '',
    uniqueToken = '',
  ] = segments;
  /**
   Numeric process id; round-trips through text only for canonical decimal integers.
   */
  const processId = Number(processIdText,);
  if (String(processId,) !== processIdText)
    return NOT_A_MANIFEST_TEMP_FILE;
  if (!Number.isSafeInteger(processId,))
    return NOT_A_MANIFEST_TEMP_FILE;
  if (processId < 1)
    return NOT_A_MANIFEST_TEMP_FILE;
  if (uniqueToken === '')
    return NOT_A_MANIFEST_TEMP_FILE;

  return processId;
}

//endregion Manifest temp file naming

//region Orphan removal

/**
 Removes one orphaned temp file, logging a failure instead of throwing
 because a leftover file must never block persisting the manifest.

 @param tempPath - Absolute path of the orphaned temp file.

 @param processId - Dead writer process id, for the log message.

 @example
 ```ts
 await removeOrphanedTempFile({ tempPath, processId: 42 });
 ```
 */
async function removeOrphanedTempFile(
  {
    tempPath,
    processId,
  }: {
    readonly processId: number;
    readonly tempPath: string;
  },
): Promise<void> {
  /**
   Function-scoped logger tagged with the call site.
   */
  const rl = tagged({
    tag: removeOrphanedTempFile.name,
    l,
  },);
  try {
    await rm(
      tempPath,
      { force: true, },
    );
    rl.info(`removed orphaned manifest temp file ${tempPath} left by dead process ${String(processId,)}`,);
  }
  catch (removeError: unknown) {
    rl.warn(`could not remove orphaned manifest temp file ${tempPath}: ${caughtErrorMessage(removeError,)}`,);
  }
}

/**
 Removes temp files beside the manifest whose embedded writer process is not alive.

 Callers must hold the manifest lock. A temp file is created only inside the
 locked write, so a live process owning one is either mid-write (impossible
 while the caller holds the lock) or is a leftover of a process that reused
 the pid; both are kept, and only files of dead processes are removed.
 The lock is taken from a holder only when its owner pid is dead or its owner
 record stayed absent for the stale age, so a live writer never loses the
 lock while it still owns a temp file. A recycled pid keeps its orphan until
 that pid exits; liveness checks err toward keeping a file.
 Cost is one directory listing plus one liveness probe per matching entry.

 @param manifestPath - Absolute manifest path whose directory is scanned.

 @example
 ```ts
 await using _lock = await acquireManifestLock(manifestPath);
 await removeOrphanedManifestTempFiles(manifestPath);
 ```
 */
export async function removeOrphanedManifestTempFiles(manifestPath: string,): Promise<void> {
  /**
   Function-scoped logger tagged with the call site.
   */
  const rl = tagged({
    tag: removeOrphanedManifestTempFiles.name,
    l,
  },);
  /**
   Directory holding the manifest and its temp files.
   */
  const manifestDirectory = dirname(manifestPath,);
  /**
   File name of the manifest inside its directory.
   */
  const manifestFileName = basename(manifestPath,);

  /**
   Directory entries, or nothing when the directory cannot be listed.
   */
  const entries: string[] = [];
  try {
    entries.push(...await readdir(manifestDirectory,),);
  }
  catch (listError: unknown) {
    rl.warn(`could not list ${manifestDirectory} for orphaned manifest temp files: ${caughtErrorMessage(listError,)}`,);
    return;
  }

  await Promise.all(entries.map(async function removeIfOrphaned(entry,): Promise<void> {
    /**
     Writer process id embedded in the entry name.
     */
    const processId = manifestTempFileWriter({
      entry,
      manifestFileName,
    },);
    if (processId === NOT_A_MANIFEST_TEMP_FILE)
      return;
    if (processAppearsAlive(processId,)) {
      rl.debug(`kept manifest temp file ${entry}: process ${String(processId,)} appears alive`,);
      return;
    }

    await removeOrphanedTempFile({
      tempPath: join(
        manifestDirectory,
        entry,
      ),
      processId,
    },);
  },),);
}

//endregion Orphan removal
