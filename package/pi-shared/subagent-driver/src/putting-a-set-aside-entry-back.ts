/**
 Putting one entry a filling set aside back into the queue, at a position counted from the head, from the record its failed start left
 beside the runs.

 The entry goes into the queue before its record goes away, both inside one held step of the queue's lock: a kill between the two leaves
 both, and the next pass's settlement drops the record because the queue holds an entry of the record's identity. The entry goes back
 with the identity its record holds, so the settlement can tell it from an entry queued under the same name later.

 @module
 */

import {
  NO_SET_ASIDE_RECORD,
  removeSetAsideRecord,
  setAsideRecordOf,
} from './set-aside-entry.ts';
import {
  type QueueRecord,
  queueIdOf,
  withTheQueueLock,
  withoutTheClaim,
} from './queue-of-runs.ts';

/**
 Puts a set-aside entry back into the queue, at a position counted from the head.

 The entry goes into the queue before its record goes away, both inside one held step of the queue's lock. A kill between the two
 leaves both, and the filling's sweep drops the record because the queue holds the entry, so nothing is lost either way. An entry the
 queue still holds is left alone: its record is from an interrupted set-aside, and the sweep decides inside the same held step.

 @param queueFile - the queue's path

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param position - where it goes, counted from the head, `0` being the head

 @param waitMilliseconds - how long to keep asking for the queue's lock before refusing

 @returns the line to print

 @throws SetAsideRecordError when the record cannot be read or deleted, or the queue cannot be written

 @example
 ```ts
 const line = await putASetAsideEntryBack({ queueFile: '/runs/q.json', runsFolder: '/runs', name: 'a', position: 0 });
 ```
 */
export async function putASetAsideEntryBack({
  queueFile,
  runsFolder,
  name,
  position,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly runsFolder: string;
  readonly name: string;
  readonly position: number;
  readonly waitMilliseconds?: number;
},): Promise<{ readonly line: string }> {
  return await withTheQueueLock({
    queueFile,
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
    work: async function putItBack({
      entries,
      write,
    },) {
      /**
       The record the entry would be put back from, read inside this step.
       */
      const record = await setAsideRecordOf({
        runsFolder,
        name,
      },);
      if (record === NO_SET_ASIDE_RECORD) {
        return { line: `no entry named ${name} is set aside beside the runs in ${runsFolder}; nothing changed`, };
      }
      if (entries.some(function isNamed(entry,) {
        return entry.name === name;
      },)) {
        // The entry is still queued, which only a kill between a record and the claim it completes leaves. The record is left for the
        // next pass's settlement, which decides inside the same held step; removing it here is how an entry in flight was lost.
        return {
          line: `${name} is still in the queue, so it is not set aside: its record is from an interrupted set-aside, and the next `
            + `filling's settlement finishes or drops it (${record.reason}). Nothing changed.`,
        };
      }
      /**
       The entry as the record holds it, without any claim a record of an older shape may have carried.
       */
      const putBack = withoutTheClaim({ record: record.entry, },);
      // An entry of a name the queue no longer holds goes back with the identity its record holds, so the settlement can tell it from
      // an entry queued under the same name later. A record written before identities existed goes back as it is.
      /**
       Where the entry lands, which is the tail of a queue shorter than the position.
       */
      const at = Math.min(
        position,
        entries.length,
      );
      /**
       The queue with the entry in it.
       */
      const next = [
        ...entries.slice(
          0,
          at,
        ),
        putBack,
        ...entries.slice(at,),
      ];
      // The entry goes in before its record goes away: a kill between the two leaves both, and the settlement drops the record
      // because the queue holds an entry of the record's identity, so nothing is lost.
      await write(next,);
      await removeSetAsideRecord({
        runsFolder,
        name,
      },);
      return {
        line: `put ${name} back at position ${String(at,)} of ${String(next.length,)}; it was set aside at ${record.setAsideAt} for: ${record.reason}`,
      };
    },
  },);
}
