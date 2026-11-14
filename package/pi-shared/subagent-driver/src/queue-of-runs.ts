/**
 The queue of Pi runs waiting for a free place: one JSON array of entries, in the order they are to start, and the four moves a
 coordinator makes on it.

 The file is read and written whole, and every write goes to a file beside the queue and is renamed over it, so a reader never
 meets half a queue. A write reads the queue again at the moment of writing and changes only the entry it was asked about: between
 a caller's first read and its write another session may have queued an entry, and a list written from the older read would drop it.

 @module
 */

import {
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';

/**
 The end of the file a queue is written through, which sits beside the queue until it is renamed over it.
 */
const BEING_WRITTEN_SUFFIX = '.being-written';

/**
 One entry of a queue, as the queue file holds it: a JSON object named by `name`.

 The fields a run or a review needs are read by `entry-of-a-queue.ts`, which refuses an entry it cannot take rather than dropping a
 field; this module only knows that every entry is an object with a name.
 */
export type QueueRecord = Readonly<Record<string, unknown> & { readonly name: string }>;

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
 A queue file that cannot be read as a queue.
 */
export class QueueFileError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'QueueFileError';
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
 Writes a queue file, through a file beside it that is renamed over it.

 @param queueFile - the queue's path

 @param entries - the entries, in the order they are to start

 @throws QueueFileError when the file cannot be written, which leaves the queue as it was

 @example
 ```ts
 await writeQueueOfRuns({ queueFile: '/runs/queue.json', entries: [] });
 ```
 */
export async function writeQueueOfRuns({
  queueFile,
  entries,
}: {
  readonly queueFile: string;
  readonly entries: readonly QueueRecord[];
},): Promise<void> {
  /**
   The file written beside the queue and renamed over it.
   */
  const beside = `${queueFile}${BEING_WRITTEN_SUFFIX}-${String(process.pid,)}`;
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

/**
 Takes every entry of one name out of the queue, reading the queue again at the moment of writing.

 @param queueFile - the queue's path

 @param name - the entry's name

 @returns the entries taken out, and the entries the queue holds afterwards

 @throws QueueFileError when the queue cannot be read or written

 @example
 ```ts
 const { taken, entries } = await takeAQueueEntryOut({ queueFile: '/runs/queue.json', name: 'a' });
 ```
 */
export async function takeAQueueEntryOut({
  queueFile,
  name,
}: {
  readonly queueFile: string;
  readonly name: string;
},): Promise<{
  readonly taken: readonly QueueRecord[];
  readonly entries: readonly QueueRecord[];
}> {
  /**
   The queue as it stands now.
   */
  const queue = await readQueueOfRuns({ queueFile, },);
  /**
   The entries of that name.
   */
  const taken = queue.filter(function isNamed(entry,) {
    return entry.name === name;
  },);
  if (taken.length === 0) {
    return {
      taken,
      entries: queue,
    };
  }
  /**
   The queue without them.
   */
  const kept = queue.filter(function isNotNamed(entry,) {
    return entry.name !== name;
  },);
  await writeQueueOfRuns({
    queueFile,
    entries: kept,
  },);
  return {
    taken,
    entries: kept,
  };
}

/**
 Puts one entry into the queue at a position counted from the head.

 An entry of that name is refused rather than queued twice: two runs of one name collide in every folder, unit and worktree the
 name gives them, so the second would replace the first rather than run beside it.

 @param queueFile - the queue's path

 @param entry - the entry to queue

 @param position - where it goes, counted from the head, `0` being the head

 @returns where it landed, and how many entries the queue now holds

 @throws QueueFileError when the queue cannot be read or written, or already holds an entry of that name

 @example
 ```ts
 const { position, count } = await putAQueueEntryIn({ queueFile: '/runs/queue.json', entry: { name: 'a' }, position: 0 });
 ```
 */
export async function putAQueueEntryIn({
  queueFile,
  entry,
  position,
}: {
  readonly queueFile: string;
  readonly entry: QueueRecord;
  readonly position: number;
},): Promise<{
  readonly position: number;
  readonly count: number;
}> {
  /**
   The queue as it stands now.
   */
  const queue = await readQueueOfRuns({ queueFile, },);
  if (queue.some(function isNamed(queued,) {
    return queued.name === entry.name;
  },)) {
    throw new QueueFileError(`an entry named ${entry.name} is already in the queue at ${queueFile}`,);
  }
  /**
   Where the entry lands, which is the head of a queue shorter than the position.
   */
  const at = Math.min(
    position,
    queue.length,
  );
  /**
   The queue with it in.
   */
  const next = [
    ...queue.slice(
      0,
      at,
    ),
    entry,
    ...queue.slice(at,),
  ];
  await writeQueueOfRuns({
    queueFile,
    entries: next,
  },);
  return {
    position: at,
    count: next.length,
  };
}

/**
 Moves the entry of one name to a position counted from the head.

 @param queueFile - the queue's path

 @param name - the entry's name

 @param position - where it goes, counted from the head, `0` being the head

 @returns where it was and where it landed, or that the queue holds no entry of that name

 @throws QueueFileError when the queue cannot be read or written

 @example
 ```ts
 const moved = await moveAQueueEntryTo({ queueFile: '/runs/queue.json', name: 'a', position: 0 });
 ```
 */
export async function moveAQueueEntryTo({
  queueFile,
  name,
  position,
}: {
  readonly queueFile: string;
  readonly name: string;
  readonly position: number;
},): Promise<{
  readonly from?: number;
  readonly to?: number;
  readonly count: number;
}> {
  /**
   The queue as it stands now.
   */
  const queue = await readQueueOfRuns({ queueFile, },);
  /**
   Where the entry stands, or minus one when the queue holds none of that name.
   */
  const from = queue.findIndex(function isNamed(entry,) {
    return entry.name === name;
  },);
  if (from === (-1)) {
    return { count: queue.length, };
  }
  /**
   The entry itself, taken as the one-element list it is so that no absent index has to be read.
   */
  const moved = queue.slice(
    from,
    from + 1,
  );
  /**
   The entries without it.
   */
  const rest = queue.filter(function isAnother(entry,) {
    return entry.name !== name;
  },);
  /**
   Where it lands, which is the tail of a queue shorter than the position.
   */
  const to = Math.min(
    position,
    rest.length,
  );
  /**
   The queue with it moved.
   */
  const next = [
    ...rest.slice(
      0,
      to,
    ),
    ...moved,
    ...rest.slice(to,),
  ];
  await writeQueueOfRuns({
    queueFile,
    entries: next,
  },);
  return {
    from,
    to,
    count: next.length,
  };
}
