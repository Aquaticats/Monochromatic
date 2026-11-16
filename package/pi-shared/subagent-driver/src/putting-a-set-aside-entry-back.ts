/**
 Putting one entry a filling set aside back into the queue, at a position counted from the head, from the record its failed start left
 beside the runs.

 The entry goes into the queue before its record goes away, each one atomic step on one file: a kill between the two leaves both, and the
 next pass's settlement drops the record because the queue holds an entry of the record's identity. The entry goes back with the identity
 its record holds, so the settlement can tell it from an entry queued under the same name later.

 @module
 */

import {
  NO_QUEUE_ID,
  type QueueRecord,
  aFreshQueueId,
  queueIdOf,
} from './queue-of-runs.ts';
import {
  aPlaceFor,
  readQueueOfRuns,
  readTheEntryNamed,
  writeAnEntryOfAQueue,
} from './reading-and-writing-the-folder-a-queue-is.ts';
import {
  NO_SET_ASIDE_RECORD,
  removeSetAsideRecord,
  setAsideRecordOf,
} from './set-aside-entry.ts';

/**
 Puts a set-aside entry back into the queue, at a position counted from the head.

 @param queueFolder - the queue's folder

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param position - where it goes, counted from the head, zero being the head

 @returns the line to print

 @throws SetAsideRecordError when the record cannot be read or deleted, or the queue cannot be written

 @example
 ```ts
 const line = await putASetAsideEntryBack({ queueFolder: '/runs/q', runsFolder: '/runs', name: 'a', position: 0 });
 ```
 */
export async function putASetAsideEntryBack({
  queueFolder,
  runsFolder,
  name,
  position,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly name: string;
  readonly position: number;
},): Promise<{ readonly line: string }> {
  /**
   The record the entry would be put back from.
   */
  const record = await setAsideRecordOf({
    runsFolder,
    name,
  },);
  if (record === NO_SET_ASIDE_RECORD) {
    return { line: `no entry named ${name} is set aside beside the runs in ${runsFolder}; nothing changed`, };
  }
  /**
   The entry the queue holds under that name now, or that it holds none.
   */
  const queued = await readTheEntryNamed({
    queueFolder,
    name,
  },);
  if ((typeof queued) === 'object') {
    // The entry is still queued, which only a kill between a record and its entry leaving the queue leaves. The record is left for the
    // next pass's settlement, which decides with the whole state in front of it; removing it here is how an entry in flight was lost.
    return {
      line: `${name} is still in the queue, so it is not set aside: its record is from an interrupted set-aside, and the next pass's `
        + `settlement finishes or drops it (${record.reason}). Nothing changed.`,
    };
  }
  /**
   The queue as it stands, which the entry's place is counted in.
   */
  const entries = await readQueueOfRuns({ queueFolder, },);
  /**
   The entry as it goes back: every field its record holds, with the place the position gives it.
   */
  const entry: QueueRecord = {
    ...record.entry,
    order: aPlaceFor({
      entries,
      position,
    },)
      .order,
    queueId: (queueIdOf({ record: record.entry, },) === NO_QUEUE_ID)
      ? aFreshQueueId()
      : queueIdOf({ record: record.entry, },),
  };
  /**
   Whether the entry went back, or a name was queued under it while this ran.
   */
  const written = await writeAnEntryOfAQueue({
    queueFolder,
    entry,
    replace: false,
  },);
  if (!written) {
    return {
      line: `${name} is in the queue again, queued there by another command while this one ran, so it was not put back; its record is `
        + `left for the next pass's settlement (${record.reason}). Nothing changed.`,
    };
  }
  // The entry is in the queue, so its record is now the second copy and goes: a kill between the two leaves it to the settlement.
  await removeSetAsideRecord({
    runsFolder,
    name,
  },);
  return {
    line: `put ${name} back at position ${String(Math.min(
      position,
      entries.length,
    ),)} of ${String(entries.length + 1,)}; it was set aside at ${record.setAsideAt} for: ${record.reason}`,
  };
}
