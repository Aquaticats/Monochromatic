/**
 Reads one file this package's checks may read, without waiting on anything: a named pipe, a device, a folder or a file larger than
 the reader takes is refused by what the opened descriptor says, and the descriptor is opened first and looked at second, so a path
 replaced between a check and an open cannot slip a pipe past the reader.

 A record is read through the descriptor that was opened, never the path again: the path's own record and the path's text can be
 two different things when another command runs at the same time.

 @module
 */

import type { Stats, } from 'node:fs';
import {
  constants,
  open,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';

/**
 Mebibytes of one file this reader takes when its caller names no bound.

 Measured: the diff of the review of 2026-11-12T10:21:26Z was 58,970 bytes, its review file 16,552 bytes, and the 103-turn session
 of the same day 1,133,922 bytes; 64 MiB is about fifty-nine times the largest of those.
 */
const READ_LIMIT_MEBIBYTES = 64;

/**
 Bytes in one mebibyte.
 */
const BYTES_PER_MEBIBYTE = 1_048_576;

/**
 The most bytes one file may hold for this reader to read it, when its caller names no bound.
 */
export const READ_LIMIT_BYTES: number = READ_LIMIT_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 What one read answered.
 */
export type RegularFileReading =
  | {
    readonly kind: 'text';
    readonly text: string;
  }
    | {
      readonly kind: 'no file';
    }
    | {
      readonly kind: 'not a regular file';
      readonly reason: string;
    }
    | {
      readonly kind: 'too large';
      readonly reason: string;
    }
    | {
      readonly kind: 'unreadable';
      readonly reason: string;
    };

/**
 The flags a read opens with: read-only, and never waiting for a writer.

 `O_NONBLOCK` on a named pipe with no writer returns the descriptor at once instead of waiting for one, which is what makes a pipe at
 a review file's path a refusal rather than the wait every brief bans.
 */
const READ_ONLY_WITHOUT_WAITING: number = constants.O_RDONLY | constants.O_NONBLOCK;

/**
 Opens a path for reading, or says what is at it.

 @param path - the path to open

 @returns the open descriptor, or what is at the path instead

 @example
 ```ts
 const opened = await openedForReading('/reviews/a/review.md');
 ```
 */
async function openedForReading(path: string,): Promise<{
  readonly kind: 'a file';
  readonly handle: Awaited<ReturnType<typeof open>>
} | {
  readonly kind: 'no file'
} | {
  readonly kind: 'unreadable';
  readonly reason: string
}> {
  try {
    return {
      kind: 'a file',
      handle: await open(
        path,
        READ_ONLY_WITHOUT_WAITING,
      ),
    };
  }
  catch (error) {
    /**
     The code the operating system gave the failure.
     */
    const code = errorCodeOf(error,);
    // A path that is not there is a fact each caller turns into its own answer.
    if ((code === 'ENOENT') || (code === 'ENOTDIR')) {
      return { kind: 'no file', };
    }
    return {
      kind: 'unreadable',
      reason: `${path} cannot be read: ${(code === '') ? String(error,) : code}`,
    };
  }
}

/**
 Reads one file without ever waiting on a pipe, a device or a folder, and without holding more than its bound.

 @param path - the file to read

 @param limitBytes - the most bytes the file may hold

 @returns the file's text, or why it was not read

 @example
 ```ts
 const read = await readRegularFileAt({ path: '/reviews/a/review.md' });
 ```
 */
export async function readRegularFileAt({
  path,
  limitBytes = READ_LIMIT_BYTES,
}: {
  readonly path: string;
  readonly limitBytes?: number;
},): Promise<RegularFileReading> {
  /**
   What is at the path, or the open descriptor.
   */
  const opened = await openedForReading(path,);
  if (opened.kind === 'no file') {
    return { kind: 'no file', };
  }
  if (opened.kind === 'unreadable') {
    return opened;
  }
  /**
   The opened descriptor, closed when this reading ends.
   */
  await using handle = opened.handle;
  /**
   What the opened descriptor says about itself, which is read from the descriptor rather than the path: a path replaced after the
   open cannot change what this reading holds.
   */
  const details: Stats = await handle.stat();
  if (!details.isFile()) {
    return {
      kind: 'not a regular file',
      reason: `${path} is ${describeKindOf(details,)}, so nothing that reads it can wait for a writer`,
    };
  }
  if (details.size > limitBytes) {
    return {
      kind: 'too large',
      reason: `${path} holds ${String(details.size,)} bytes, more than the ${String(limitBytes,)} this reader takes`,
    };
  }
  try {
    return {
      kind: 'text',
      text: await handle.readFile({ encoding: 'utf8', },),
    };
  }
  catch (error) {
    return {
      kind: 'unreadable',
      reason: `${path} cannot be read: ${String(error,)}`,
    };
  }
}

/**
 Names what an opened descriptor is, for a message a reader can act on.

 @param details - the descriptor's own record

 @returns the words that name it

 @example
 ```ts
 const kind = describeKindOf(details);
 ```
 */
function describeKindOf(details: Stats,): string {
  if (details.isFIFO()) {
    return 'no regular file but a named pipe';
  }
  if (details.isDirectory()) {
    return 'no regular file but a folder';
  }
  if (details.isSocket()) {
    return 'no regular file but a socket';
  }
  if (details.isCharacterDevice()) {
    return 'no regular file but a device';
  }
  if (details.isBlockDevice()) {
    return 'no regular file but a block device';
  }
  return 'no regular file';
}
