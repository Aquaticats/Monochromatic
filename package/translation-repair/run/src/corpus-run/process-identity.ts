import {
  readFile,
  readlink,
} from 'node:fs/promises';
import { hostname, } from 'node:os';

import {
  allInInputOrder,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Process identity
// WHICH PROCESS A PROCESS ID NAMED, not merely whether the id is in use (ledger
// A16). A pass killed at its hard cap leaves its lock behind, and the id the
// lock records is free for the kernel to hand to any later process; after a
// reboot the ids start again from the bottom. A lock judged by id alone then
// reads as held by whatever unrelated process now carries that id, and every
// later pass in the directory refuses, saying its holder is alive.
//
// A process is the id plus when it started. Linux says both: the boot it
// belongs to (`/proc/sys/kernel/random/boot_id`), the process-id namespace the
// id is counted in (`/proc/self/ns/pid`), and its start time in clock ticks
// since that boot (field twenty-two of `/proc/<pid>/stat`). Where `/proc` is
// absent the reads fail and the lock falls back to the id alone, saying so.

/**
 Logger every identity line goes through.
 */
const identityLog = contextRoot({ tag: 'process-identity', },);

/**
 Position of the start time among the fields after the command name in
 `/proc/<pid>/stat`: field twenty-two counted from one, less the two fields
 (the process id and the parenthesised command) that come before the split.
 */
const START_TICKS_FIELD = 19;

/**
 Where this machine and this process's id namespace stand.

 @example
 ```ts
 const here: HostIdentity = { host: 'tabby', bootId: 'b0a7…', pidNamespace: 'pid:[4026531836]', };
 ```
 */
export type HostIdentity = Readonly<{
  /**
   Machine name, so a lock read on another machine is never judged by this
   machine's process table.
   */
  host: string;

  /**
   This boot, which a reboot changes: no process outlives it.
   */
  bootId: string;

  /**
   Namespace process ids are counted in; a container counts its own.
   */
  pidNamespace: string;
}>;

/**
 What reading this machine's identity came to.

 @example
 ```ts
 const read: HostRead = { kind: 'unread', };
 ```
 */
export type HostRead =
  | Readonly<{
    /**
     All three were read.
     */
    kind: 'read';

    /**
     What was read.
     */
    here: HostIdentity;
  }>
    | Readonly<{
    /**
     `/proc` did not answer, so locks here are judged by process id alone.
     */
      kind: 'unread';
    }>;

/**
 What reading one process's start time came to.

 @example
 ```ts
 const read: StartTicksRead = { kind: 'read', startTicks: '275883698', };
 ```
 */
export type StartTicksRead =
  | Readonly<{
    /**
     The start time was read.
     */
    kind: 'read';

    /**
     Clock ticks from boot to the process's start, as the kernel prints them.
     */
    startTicks: string;
  }>
    | Readonly<{
    /**
     Nothing was read: no such entry, no `/proc`, or an entry the reader may
     not see (a `hidepid` mount hides other users' processes). The id alone
     then has to answer.
     */
      kind: 'unread';
    }>;

/**
 Reads this machine's name, boot and process-id namespace.

 @returns The three, or that `/proc` did not answer

 @example
 ```ts
 const read = await hostIdentity();
 ```
 */
export async function hostIdentity(): Promise<HostRead> {
  try {
    /**
     This boot and this namespace, read together.
     */
    const [
      bootText,
      pidNamespace,
    ] = await allInInputOrder({
      members: [
        readFile(
          '/proc/sys/kernel/random/boot_id',
          'utf8',
        ),
        readlink('/proc/self/ns/pid',),
      ],
    },);
    return {
      kind: 'read',
      here: {
        host: hostname(),
        bootId: bootText.trim(),
        pidNamespace,
      },
    };
  }
  catch (error) {
    identityLog.warn(
      `no process identity on this system (${refusalText({ error, },)}); a runs lock is judged by process id alone`,
    );
    return { kind: 'unread', };
  }
}

/**
 Reads the start time out of one `/proc/<pid>/stat` line.

 The command name sits in parentheses and may itself hold spaces or a closing
 parenthesis, so the fields are read after the LAST one.

 @param stat - the line the kernel prints for one process

 @returns Its start time, or that the line holds none

 @example
 ```ts
 const started = startTicksOfStat({ stat: '1 (init) S 0 1 1 0 -1 4194560 1 2 3 4 5 6 7 8 20 0 1 0 275883698', },);
 ```
 */
export function startTicksOfStat({ stat, }: { readonly stat: string; },): StartTicksRead {
  /**
   Start time among the fields after the command name.
   */
  const startTicks = stat
    .slice(stat.lastIndexOf(')',) + 1,)
    .trim()
    .split(' ',)[START_TICKS_FIELD];
  return ((startTicks === undefined) || (startTicks === ''))
    ? { kind: 'unread', }
    : {
      kind: 'read',
      startTicks,
    };
}

/**
 Reads when one process started, in clock ticks since boot, out of the line
 the kernel prints for it.

 @param pid - process to read

 @returns Its start time, or that none was read

 @example
 ```ts
 const started = await startTicksOf({ pid: process.pid, },);
 ```
 */
export async function startTicksOf(
  { pid, }: { readonly pid: number; },
): Promise<StartTicksRead> {
  try {
    /**
     The process's status line.
     */
    const stat = await readFile(
      `/proc/${String(pid,)}/stat`,
      'utf8',
    );
    return startTicksOfStat({ stat, },);
  }
  catch (error) {
    // Absent or hidden alike: the caller falls back to the id, which tells a
    // gone process from a hidden one.
    identityLog.debug(`process ${String(pid,)} start time unread (${refusalText({ error, },)})`,);
    return { kind: 'unread', };
  }
}

//endregion Process identity
