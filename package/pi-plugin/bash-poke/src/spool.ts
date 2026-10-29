/**
 Complete-output spooling below the OS temp directory.

 @module
 */

import { once, } from 'node:events';
import {
  createWriteStream,
  type WriteStream,
} from 'node:fs';
import {
  chmod,
  mkdir,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  SPOOL_DIR_MODE,
  SPOOL_DIR_NAME,
  SPOOL_FILE_EXTENSION,
  SPOOL_FILE_MODE,
} from './constants.ts';
import { bashPokeLogger, } from './logger.ts';

//region Types

/**
 Open spool file a job writes its complete output into.
 */
type SpoolWriter = {
  /**
   Absolute path named in the poke, absent when spooling is unavailable.
   */
  readonly path?: string;

  /**
   Appends text, reporting whether the stream still accepts writes promptly.
   */
  readonly write: (text: string) => boolean;

  /**
   Registers one callback for when buffered writes have reached disk.
   */
  readonly onceDrain: (resume: () => void) => void;

  /**
   Flushes and releases the file handle.
   */
  readonly close: () => Promise<void>;
};

//endregion Types

//region Directory

/**
 Resolves the spool directory below a temp root and enforces owner-only access.
 
 @param tmp - temp root, injectable so tests use a disposable directory
 
 @returns absolute spool directory path
 
 @throws when the directory cannot be created or its mode cannot be tightened
 
 @example
 ```ts
 await spoolDirectory({ tmp: '/tmp', },);
 ```
 */
async function spoolDirectory(
  { tmp = tmpdir(), }: { readonly tmp?: string; },
): Promise<string> {
  /**
   Directory holding complete captured output for this package.
   */
  const dir = join(
    tmp,
    SPOOL_DIR_NAME,
  );
  await mkdir(
    dir,
    {
      recursive: true,
      mode: SPOOL_DIR_MODE,
    },
  );
  // mkdir applies its mode only when it creates the directory, so one left
  // behind by an earlier umask is tightened explicitly; captured output can hold
  // credentials a command printed.
  await chmod(
    dir,
    SPOOL_DIR_MODE,
  );
  return dir;
}

//endregion Directory

//region Discarding writer

/**
 Writer used when no spool file could be opened.
 
 Spooling is an enhancement over the poke, so its failure must not stop the job
 or the poke. Callers therefore always receive a writer and learn whether
 spooling happened by reading the absent path.
 */
const discardingSpoolWriter: SpoolWriter = {
  write(): boolean {
    return true;
  },
  onceDrain(): void {
    tagged({
      tag: 'discardingSpoolWriter.onceDrain',
      l: bashPokeLogger,
    }, )
      .debug('discarding spool writer never buffers, so it never drains', );
  },
  close(): Promise<void> {
    tagged({
      tag: 'discardingSpoolWriter.close',
      l: bashPokeLogger,
    }, )
      .debug('discarding spool writer closed', );
    return Promise.resolve();
  },
};

//endregion Discarding writer

//region Writing

/**
 Opens the spool file for one job, or falls back to discarding complete output.
 
 @param tmp - temp root, injectable so tests use a disposable directory
 
 @param jobId - identifier naming the spool file
 
 @returns writer for complete output, whose path is absent when spooling failed
 
 @example
 ```ts
 const writer = await openSpoolWriter({ tmp: '/tmp', jobId: 'job-1', },);
 await writer.close();
 ```
 */
async function openSpoolWriter(
  {
    tmp = tmpdir(),
    jobId,
  }: {
    readonly tmp?: string;
    readonly jobId: string;
  },
): Promise<SpoolWriter> {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: openSpoolWriter.name,
    l: bashPokeLogger,
  }, );
  try {
    /**
     Directory ensured per job so a temp cleanup between jobs cannot break it.
     */
    const dir = await spoolDirectory({ tmp, }, );

    /**
     File carrying this job's complete output.
     */
    const path = join(
      dir,
      `${jobId}${SPOOL_FILE_EXTENSION}`,
    );

    /**
     Append stream, owner-only because captured output can hold secrets.
     */
    const stream: WriteStream = createWriteStream(
      path,
      {
        encoding: 'utf8',
        mode: SPOOL_FILE_MODE,
      },
    );
    // A stream error has no awaiting caller to reject, so it is logged here
    // instead of surfacing as an unhandled 'error' event that would kill Pi.
    stream.on(
      'error',
      function onSpoolStreamError(error: Error, ): void {
        l.warn(`spool stream failed for ${path}: ${caughtValueText(error, )}`, );
      },
    );
    l.debug(`spooling job ${jobId} to ${path}`, );

    return {
      path,
      write(text: string, ): boolean {
        return stream.write(text, );
      },
      onceDrain(resume: () => void, ): void {
        stream.once(
          'drain',
          resume,
        );
      },
      async close(): Promise<void> {
        stream.end();
        await once(
          stream,
          'close',
        );
      },
    };
  }
  catch (error: unknown) {
    l.warn(`spool open failed for job ${jobId}: ${caughtValueText(error, )}`, );
    return discardingSpoolWriter;
  }
}

//endregion Writing

export {
  discardingSpoolWriter,
  openSpoolWriter,
  spoolDirectory,
};

export type { SpoolWriter, };
