/**
 Probe a destination directory and create one sink-owned JSONL file on demand. @module
 */
import { randomUUID, } from 'node:crypto';
import {
  appendFile,
  mkdir,
  open,
  readFile,
  rm,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { DAY_KEY_LENGTH, } from './file-retention-name.ts';

/**
 Owner-only file mode for sink-owned files and probes.
 */
const LOG_FILE_CREATE_MODE = 0o600;

/**
 Owner-only directory mode for destination and day directories.
 */
const LOG_DIRECTORY_CREATE_MODE = 0o700;

//region Directory probes separate writability proof from record ownership.
/**
 A filesystem round trip did not retain the sink's probe record.
 */
class FileProbeMismatchError extends Error {
  /**
   @param probePath - affected probe file whose read did not match
   */
  constructor(probePath: string,) {
    super(`File sink probe did not round-trip in ${JSON.stringify(probePath,)}`,);
    this.name = 'FileProbeMismatchError';
  }
}

/**
 Prove the destination directory round-trips data, using a probe file removed before return.

 Verification must prove writability without creating the sink's log file,
 so zero-record processes leave zero files behind.
 The probe file carries a `.probe.tmp` suffix,
 the retention sweep expires that name family too,
 and a crashed probe never looks like a log record.

 @param directory - selected destination, kept literal for explicit callers

 @throws FileProbeMismatchError when the written probe does not round-trip.

 @example
 ```ts
 await probeLogDirectory('/fixture-logs');
 ```
 */
export async function probeLogDirectory(directory: string,): Promise<void> {
  await mkdir(
    directory,
    {
      recursive: true,
      mode: LOG_DIRECTORY_CREATE_MODE,
    },
  );
  /**
   Portable timestamp component shared with sink file names.
   */
  const timestamp = new Date().toISOString()
    .replaceAll(
    ':',
    '-',
    );
  /**
   Probe path outside the log name family,
   so a crash between create and remove leaves sweepable residue, never a fake record.
   */
  const probePath = join(
    directory,
    `${timestamp}.${randomUUID()}.probe.tmp`,
  );
  /**
   Exact probe record required to round-trip before the directory is published.
   */
  const testData = `{"test":true,"timestamp":${Date.now()}}\n`;
  await appendFile(
    probePath,
    testData,
    {
      flag: 'ax',
      mode: LOG_FILE_CREATE_MODE,
    },
  );
  /**
   Probe file removed on every exit path, including a mismatch throw.
   */
  await using _probeTemp = {
    [Symbol.asyncDispose]: async function removeProbeTemp(): Promise<void> {
      await rm(
        probePath,
        {
          force: true,
        },
      );
    },
  };
  /**
   Read-back must preserve the complete probe rather than a matching substring.
   */
  const content = await readFile(
    probePath,
    'utf8',
  );
  if (content !== testData)
    throw new FileProbeMismatchError(probePath,);
}

/**
 Choose the sink's JSONL file path under the day directory of its first record.

 One process keeps one file,
 the incident-evidence shape `doc/troubleshooting/` cites.
 The day directory (`YYYY-MM-DD`,
 UTC date of the file timestamp) keeps retention sweeps and
 human navigation on bounded directory sizes.
 Each call names a fresh file,
 so one sink resolves its path before queueing records and
 creates it only once its first append runs.

 @param directory - verified destination directory, kept literal for explicit callers

 @returns Fresh path one sink owns exclusively.

 @example
 ```ts
 const filePath = newLogFilePath('/fixture-logs');
 ```
 */
export function newLogFilePath(directory: string,): string {
  /**
   Portable timestamp component for the exclusively created file,
   whose date prefix also names the day directory.
   */
  const timestamp = new Date().toISOString()
    .replaceAll(
    ':',
    '-',
    );
  /**
   Day directory of this file's first record,
   named by the UTC date prefix of its timestamp.
   */
  const dayDirectory = join(
    directory,
    timestamp.slice(
      0,
      DAY_KEY_LENGTH,
    ),
  );
  return join(
    dayDirectory,
    `${timestamp}.${randomUUID()}.log.jsonl`,
  );
}

/**
 Create the sink's JSONL file exclusively at a previously chosen path.

 The file is created empty and exclusively (`ax`),
 so ownership comes from creation itself and no bogus probe record is written,
 since the destination directory already round-tripped in {@link probeLogDirectory}.
 Creation runs once,
 before the first append,
 so a zero-record process leaves zero files.

 @param filePath - path from {@link newLogFilePath}, owned exclusively by one sink

 @example
 ```ts
 await createLogFile('/fixture-logs/2026-10-22/2026-10-22T08-00-00.000Z.….log.jsonl');
 ```
 */
export async function createLogFile(filePath: string,): Promise<void> {
  /**
   Day directory of the file,
   created before its first record lands.
   */
  const dayDirectory = dirname(filePath,);
  await mkdir(
    dayDirectory,
    {
      recursive: true,
      mode: LOG_DIRECTORY_CREATE_MODE,
    },
  );
  /**
   Exclusive empty creation publishes ownership without a probe record.
   */
  const handle = await open(
    filePath,
    'ax',
    LOG_FILE_CREATE_MODE,
  );
  await handle.close();
}

/**
 Create the sink's JSONL file under the day directory of its first record.

 {@link newLogFilePath} chooses the path and {@link createLogFile} owns it;
 this pairing keeps the one-call seam for callers that need both together.

 @param directory - verified destination directory, kept literal for explicit callers

 @returns Path owned exclusively by this sink.

 @example
 ```ts
 const filePath = await openLogFile('/fixture-logs');
 ```
 */
export async function openLogFile(directory: string,): Promise<string> {
  /**
   Fresh path this call creates and returns.
   */
  const filePath = newLogFilePath(directory,);
  await createLogFile(filePath,);
  return filePath;
}
//endregion Directory probes separate writability proof from record ownership.
