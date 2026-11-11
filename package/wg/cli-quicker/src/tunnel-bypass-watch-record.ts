//! The watcher record a lifecycle reads,
//! and the one state it must not refuse the whole lifecycle over:
//! a fragment left by a build that wrote the record in place.

import { rename, } from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { PROCESS_ABSENT, } from './linux-process-identity.ts';
import {
  readWatcherRecord,
  type WatcherProcessIdentity,
  type WatcherRecord,
} from './tunnel-bypass-watcher-sidecar.ts';

/**
 Module logger for the record and the lifecycle that reads it.
 */
const l = tagged({ tag: 'tunnel-bypass-watch-record', },);

/**
 Reads the watcher record of a state path,
 and moves aside a record that holds no identity so the lifecycle can go on.
 
 A record left half written by a build that wrote it in place is a fragment of a watcher that
 was killed while writing,
 so no watcher of it runs,
 and refusing the whole `down` over the fragment left the state, the rules and the routes for ever.
 The fragment is kept, beside the record, under a name that says what it is,
 because it is the only trace of what the earlier build was doing.

 @param statePath - Persisted bypass state path.

 @param read - Reader of the record,
 replaceable in tests.

 @returns The identity, or absence.
 
 @example
 ```ts
 await identityOrNone({ statePath });
 ```
 */
export async function identityOrNone(
  {
    statePath,
    read = readWatcherRecord,
  }: {
    readonly statePath: string;
    readonly read?: typeof readWatcherRecord;
  },
): Promise<WatcherProcessIdentity | typeof PROCESS_ABSENT> {
  /**
   What the record says.
   */
  const record: WatcherRecord = await read({ statePath, },);
  if (record.kind === 'absent')
    return PROCESS_ABSENT;
  if (record.kind === 'identity')
    return record.identity;
  /**
   Where the unreadable record is kept.
   */
  const kept = `${record.path}.unreadable-${String(Date.now(),)}`;
  await rename(record.path, kept,);
  l.warn(
    `${record.path} held no watcher identity (${record.reason}), so it was moved to ${kept} and no watcher is signalled; `
      + 'a route watcher started by an earlier build may still run, and `pgrep --full bypass-watch.mjs` finds it',
  );
  return PROCESS_ABSENT;
}
