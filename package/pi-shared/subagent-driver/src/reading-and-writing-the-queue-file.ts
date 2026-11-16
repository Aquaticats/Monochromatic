/**
 The file a queue of runs is: one JSON array of entries, read whole and written whole through a file beside it that is renamed over it,
 so a reader never meets half a queue.

 Every write here is made by a caller that holds the queue's lock (`queue-of-runs.ts`): this module knows the text and the shape of the
 file, and nothing about which entries its caller decided to keep.

 @module
 */

import {
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import { aPathWrittenBeside, } from './queue-lock.ts';

/**
 One entry of a queue, as the queue file holds it: a JSON object named by `name`.

 The fields a run or a review needs are read by `entry-of-a-queue.ts`, which refuses an entry it cannot take rather than dropping a
 field; this module only knows that every entry is an object with a name.
 */
export type QueueRecord = Readonly<Record<string, unknown> & { readonly name: string }>;

/**
 A queue file that cannot be read as a queue.
 */
export class QueueFileError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'QueueFileError';
}

/**
 Says whether a value is one entry of a queue: a JSON object named by a non-empty `name`.

 @param value - the value, of any shape

 @returns whether it is one entry of a queue

 @example
 ```ts
 const isOne = isQueueRecord({ name: 'a' });
 ```
 */
export function isQueueRecord(value: unknown,): value is QueueRecord {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('name' in value)
    && ((typeof value.name) === 'string')
    && (value.name !== '');
}

/**
 Reads a queue file's entries.

 @param queueFile - the queue's path

 @returns the entries, in the order the file holds them

 @throws QueueFileError when the file cannot be read, holds no JSON array, or holds an entry that is no JSON object with a name

 @example
 ```ts
 const entries = await readQueueOfRuns({ queueFile: '/runs/queue.json' });
 ```
 */
export async function readQueueOfRuns({
  queueFile,
}: {
  readonly queueFile: string;
}): Promise<readonly QueueRecord[]> {
  /**
   The file's text.
   */
  const shown = await textOfQueue({ queueFile, },);
  return recordsInQueue({
    parsed: parseQueue({
      shown,
      queueFile
    },),
    queueFile,
  },);
}

/**
 Reads a queue file's text.

 @param queueFile - the queue's path

 @returns what the file holds

 @throws QueueFileError when the file cannot be read

 @example
 ```ts
 const shown = await textOfQueue({ queueFile: '/runs/queue.json' });
 ```
 */
async function textOfQueue({ queueFile, }: { readonly queueFile: string },): Promise<string> {
  try {
    return await readFile(
      queueFile,
      'utf8',
    );
  }
  catch (error) {
    throw new QueueFileError(`the queue at ${queueFile} cannot be read: ${String(error,)}`,);
  }
}

/**
 Reads a queue file's text as JSON.

 @param shown - the file's text

 @param queueFile - the queue's path, for the message

 @returns what the text holds, as anything

 @throws QueueFileError when the text is no JSON

 @example
 ```ts
 const parsed = parseQueue({ shown: '[]', queueFile: '/runs/queue.json' });
 ```
 */
function parseQueue({
  shown,
  queueFile,
}: {
  readonly shown: string;
  readonly queueFile: string;
}): unknown {
  try {
    /**
     What the text parsed to.
     */
    const parsed: unknown = JSON.parse(shown,);
    return parsed;
  }
  catch (error) {
    throw new QueueFileError(`the queue at ${queueFile} holds no JSON: ${String(error,)}`,);
  }
}

/**
 Reads the entries a parsed queue file holds.

 @param parsed - what the file's text parsed to

 @param queueFile - the queue's path, for the message

 @returns the entries, in the order the file holds them

 @throws QueueFileError when it is no JSON array, or holds an entry that is no object with a string name

 @example
 ```ts
 const entries = recordsInQueue({ parsed: [], queueFile: '/runs/queue.json' });
 ```
 */
function recordsInQueue({
  parsed,
  queueFile,
}: {
  readonly parsed: unknown;
  readonly queueFile: string;
}): readonly QueueRecord[] {
  if (!Array.isArray(parsed,)) {
    throw new QueueFileError(`${queueFile} holds no JSON array of entries, so nothing can be started from it`,);
  }
  return parsed.map(function asARecord(
    entry: unknown,
    index: number,
  ) {
    if (!isQueueRecord(entry,)) {
      throw new QueueFileError(
        `${queueFile}: entry ${String(index + 1,)} is no JSON object naming a run, so it cannot be started`,
      );
    }
    return entry;
  },);
}

/**
 Writes a queue's entries as the file's text.

 @param entries - the entries, in the order they are to start

 @returns the text to write

 @example
 ```ts
 const text = writtenQueueOfRuns({ entries: [{ name: 'a' }] });
 ```
 */
export function writtenQueueOfRuns({ entries }: { readonly entries: readonly QueueRecord[] },): string {
  /**
   One line per entry, which keeps a queue's diff readable.
   */
  const lines = entries.map(function asALine(entry,) {
    return `  ${JSON.stringify(entry,)}`;
  },);
  return `[\n${lines.join(',\n',)}\n]\n`;
}

/**
 Writes a queue file whole, through a file beside it that is renamed over it, and writes nothing else: callers hold the queue's lock.

 @param queueFile - the queue's path

 @param entries - the entries, in the order they are to start

 @throws QueueFileError when the file cannot be written, which leaves the queue as it was

 @example
 ```ts
 await writeTheQueueWhole({ queueFile: '/runs/queue.json', entries: [] });
 ```
 */
export async function writeTheQueueWhole({
  queueFile,
  entries,
}: {
  readonly queueFile: string;
  readonly entries: readonly QueueRecord[];
},): Promise<void> {
  /**
   The file written beside the queue and renamed over it.
   */
  const beside = aPathWrittenBeside({ path: queueFile, },);
  try {
    await writeFile(
      beside,
      writtenQueueOfRuns({ entries, },),
    );
    await rename(
      beside,
      queueFile,
    );
  }
  catch (error) {
    throw new QueueFileError(`the queue at ${queueFile} cannot be written: ${String(error,)}`,);
  }
}
