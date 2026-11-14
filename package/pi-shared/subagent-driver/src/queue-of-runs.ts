/**
 The queue of Pi runs waiting for a free place: one JSON array of entries, in the order they are to start, and the four moves a
 coordinator makes on it.

 The file is read and written whole, and every write goes to a file beside the queue and is renamed over it, so a reader never
 meets half a queue. Every change reads the queue, changes it and writes it while the queue's lock is held (`queue-lock.ts`), so two
 writers can never both read before either writes and drop one another's entry: that lost update was reproduced with two processes
 against the queue of 2026-11-14, 47 rounds of 200 losing an entry while both commands exited 0. The lock is one file beside the
 queue, taken by linking a file that names its holder into the lock's place, and it is released whatever the change answered.

 @module
 */

import {
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import {
  aPathWrittenBeside,
  acquireTheQueueLock,
} from './queue-lock.ts';

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
 Writes a queue file whole, through a file beside it that is renamed over it, and writes nothing else: callers hold the queue's lock.

 @param queueFile - the queue's path

 @param entries - the entries, in the order they are to start

 @throws QueueFileError when the file cannot be written, which leaves the queue as it was

 @example
 ```ts
 await writeTheQueueWhole({ queueFile: '/runs/queue.json', entries: [] });
 ```
 */
async function writeTheQueueWhole({
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

/**
 One change to make to a queue while it is held: the entries after the change, and what the caller is answered.
 */
export type ChangeOfAQueue<Answer> = (entries: readonly QueueRecord[]) => {
  /**
   The entries the queue holds after the change.
   */
  readonly entries: readonly QueueRecord[];

  /**
   What the caller is answered.
   */
  readonly answer: Answer;
};

/**
 Changes a queue while no other writer can: the read, the change and the write happen under the queue's lock.

 The change is asked for the entries the queue holds at that moment, so no entry another session queued meanwhile is written away.
 A change that answers the very array it was given writes nothing.

 @param queueFile - the queue's path

 @param change - the change to make, asked for the entries the queue holds under the lock

 @param now - the clock the lock's wait and age are read from

 @param wait - the pause between two attempts to take the lock

 @param waitMilliseconds - how long to keep asking for the lock before refusing

 @returns what the change answered

 @throws QueueFileError when the queue cannot be read or written, which leaves it as it was

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @example
 ```ts
 const answer = await changeTheQueueOfRuns({ queueFile: '/runs/queue.json', change: (entries) => ({ entries, answer: entries.length }) });
 ```
 */
export async function changeTheQueueOfRuns<Answer>({
  queueFile,
  change,
  now,
  wait,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly change: ChangeOfAQueue<Answer>;
  readonly now?: () => number;
  readonly wait?: (milliseconds: number) => Promise<unknown>;
  readonly waitMilliseconds?: number;
},): Promise<Answer> {
  /**
   The queue's lock, given back when this block ends whether the change answered or threw.
   */
  await using lock = await acquireTheQueueLock({
    queueFile,
    ...(now === undefined ? {} : { now, }),
    ...(wait === undefined ? {} : { wait, }),
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
  },);
  /**
   The queue as it stands under the lock.
   */
  const entries = await readQueueOfRuns({ queueFile, },);
  /**
   What the change made of it.
   */
  const changed = change(entries,);
  if (changed.entries !== entries) {
    await writeTheQueueWhole({
      queueFile,
      entries: changed.entries,
    },);
  }
  return changed.answer;
}

/**
 Writes a queue file whole, holding the queue's lock, so no change of another writer is written away.

 @param queueFile - the queue's path

 @param entries - the entries, in the order they are to start

 @param now - the clock the lock's wait reads

 @param wait - the pause between attempts to take the lock

 @param waitMilliseconds - how long to keep asking for the lock before refusing

 @throws QueueFileError when the file cannot be written, which leaves the queue as it was

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @example
 ```ts
 await writeQueueOfRuns({ queueFile: '/runs/queue.json', entries: [] });
 ```
 */
export async function writeQueueOfRuns({
  queueFile,
  entries,
  now,
  wait,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly entries: readonly QueueRecord[];
  readonly now?: () => number;
  readonly wait?: (milliseconds: number) => Promise<unknown>;
  readonly waitMilliseconds?: number;
},): Promise<void> {
  // A whole-file write reads nothing, so it makes a queue file that is not there yet, which is how every caller's queue starts.
  /**
   The queue's lock, given back when this block ends whether the write answered or threw.
   */
  await using lock = await acquireTheQueueLock({
    queueFile,
    ...(now === undefined ? {} : { now, }),
    ...(wait === undefined ? {} : { wait, }),
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
  },);
  await writeTheQueueWhole({
    queueFile,
    entries,
  },);
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
  return await changeTheQueueOfRuns({
    queueFile,
    change: function takeThemOut(queue,) {
      /**
       The entries of that name.
       */
      const taken = queue.filter(function isNamed(entry,) {
        return entry.name === name;
      },);
      if (taken.length === 0) {
        return {
          entries: queue,
          answer: {
            taken,
            entries: queue,
          },
        };
      }
      /**
       The queue without them.
       */
      const kept = queue.filter(function isNotNamed(entry,) {
        return entry.name !== name;
      },);
      return {
        entries: kept,
        answer: {
          taken,
          entries: kept,
        },
      };
    },
  },);
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
  return await changeTheQueueOfRuns({
    queueFile,
    change: function putTheEntryIn(queue,) {
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
      return {
        entries: next,
        answer: {
          position: at,
          count: next.length,
        },
      };
    },
  },);
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
  return await changeTheQueueOfRuns<{
    readonly from?: number;
    readonly to?: number;
    readonly count: number;
  }>({
    queueFile,
    change: function moveTheEntry(queue,) {
      /**
       Where the entry stands, or minus one when the queue holds none of that name.
       */
      const from = queue.findIndex(function isNamed(entry,) {
        return entry.name === name;
      },);
      if (from === (-1)) {
        return {
          entries: queue,
          answer: { count: queue.length, },
        };
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
      return {
        entries: next,
        answer: {
          from,
          to,
          count: next.length,
        },
      };
    },
  },);
}
