/**
 Look for the owner of a claim among the processes this process can see, when its id cannot be read as one of this
 namespace's.

 A process id means something only inside the PID namespace it is counted in. A process of a namespace nested inside the one
 this process is in shows in the process directory under the id of the outer namespace, and its `status` file lists every id
 it has, outermost first, on the `NSpid` line: the last one is the id the owner wrote into its claim. That id, with the start
 time the claim records, names the owner exactly, whatever its place in the outer namespace is, and no age of any file is
 involved.

 What this process cannot see into (a sibling or an outer namespace) shows nothing at all, and nothing is not evidence of
 death. The scan therefore answers `gone` only when a process proves the id was reused, or when the namespace is seen and
 every process was readable and none has the id, and `unknown` otherwise.
 */
import {
  readdir,
  readFile,
  readlink,
} from 'node:fs/promises';
import { join } from 'node:path';

import type { OwnedRecord } from './claim-record.ts';
import { isMissing } from './fs-guards.ts';
import {
  birthNotParsed,
  parseBirth,
  processGone,
  startTicksFromStat,
} from './process-identity.ts';

/**
 What the scan found out about the owner: it is `alive`, it is `gone`, or nothing here can tell.
 */
export type ForeignStanding = 'alive' | 'gone' | 'unknown';

/**
 A process directory entry that vanished while it was read: it says nothing.
 */
type VanishedEntry = {
  /**
   Marks the entry as gone.
   */
  readonly kind: 'vanished';
};

/**
 A process directory entry that could not be read, so it might be the owner.
 */
type UnreadableEntry = {
  /**
   Marks the entry as unreadable.
   */
  readonly kind: 'unreadable';
};

/**
 A process that was read.
 */
type SeenEntry = {
  /**
   Marks the entry as read.
   */
  readonly kind: 'seen';
  /**
   Its PID namespace link, absent when the link cannot be read.
   */
  readonly namespace?: string;
  /**
   The id it has in its own namespace.
   */
  readonly innerPid: number;
  /**
   Its start time, read only for a process that has the owner's id; absent for a process that exited but is not yet reaped.
   */
  readonly startTicks?: string;
};

/**
 What one process directory entry showed.
 */
type Observation = VanishedEntry | UnreadableEntry | SeenEntry;

/**
 Stands for a status text that does not say the id the process has in its own namespace.
 */
export const noInnerPid: unique symbol = Symbol('status.no-inner-pid');

/**
 Stands for a namespace link that this process may not read.
 */
const linkUnreadable: unique symbol = Symbol('ns-pid.unreadable');

/**
 Take the id a process has in its own namespace from its `status` text.

 @param status - the text of the file

 @returns the last id of the `NSpid` line, or {@link noInnerPid} when the line is absent or malformed

 @example
 ```ts
 innerPidFromStatus('NSpid:\t4242\t77\n');
 ```
 */
export function innerPidFromStatus(status: string): number | typeof noInnerPid {
  /**
   The line, which older kernels do not write.
   */
  const line = status.split('\n')
    .find(function isNamespaceLine(candidate) {
      return candidate.startsWith('NSpid:');
    });
  if (line === undefined)
    return noInnerPid;
  /**
   The ids on the line, which the kernel separates by tabs.
   */
  const ids = line.slice('NSpid:'.length)
    .trim()
    .split('\t');
  /**
   The last one.
   */
  const last = Number(ids.at(-1));
  return (Number.isSafeInteger(last) && (last > 0)) ? last : noInnerPid;
}

/**
 Read the PID namespace link of a process directory entry.

 @param path - the link

 @returns the namespace text, or {@link linkUnreadable} when this process may not read it

 @throws Error when the entry is gone, so the caller can tell a vanished process from an unreadable one

 @example
 ```ts
 await readNamespaceLink('/proc/4242/ns/pid');
 ```
 */
async function readNamespaceLink(path: string): Promise<string | typeof linkUnreadable> {
  try {
    return await readlink(path);
  }
  catch (error) {
    if (isMissing(error))
      throw error;
    return linkUnreadable;
  }
}

/**
 Tell whether a directory entry name is a process id: whole digits, no leading zero.

 @param name - the entry name

 @returns true for the spelling of a positive whole number

 @example
 ```ts
 isProcessId('4242');
 ```
 */
function isProcessId(name: string): boolean {
  /**
   The number the name spells.
   */
  const number = Number(name);
  return Number.isSafeInteger(number) && (number > 0)
    && (String(number) === name);
}

/**
 Read one entry of the process directory.

 @param procRoot - the process directory

 @param entry - the name of the entry, a process id of this namespace

 @param ownerPid - the id the owner has in its own namespace

 @returns what the entry showed

 @example
 ```ts
 await observe({ procRoot: '/proc', entry: '4242', ownerPid: 77 });
 ```
 */
async function observe({
  procRoot,
  entry,
  ownerPid,
}: {
  readonly procRoot: string;
  readonly entry: string;
  readonly ownerPid: number;
}): Promise<Observation> {
  try {
    /**
     The status of the process.
     */
    const innerPid = innerPidFromStatus(await readFile(
      join(
        procRoot,
        entry,
        'status',
      ),
      'utf8',
    ));
    if (innerPid === noInnerPid)
      return { kind: 'unreadable' };
    /**
     Its namespace link, when this process may read it, as the properties it adds to what was seen.
     */
    const link = await readNamespaceLink(join(
      procRoot,
      entry,
      'ns',
      'pid',
    ));
    /**
     The namespace property, absent for a link that cannot be read.
     */
    const namespace = link === linkUnreadable ? {} : { namespace: link };
    if (innerPid !== ownerPid)
      return {
        kind: 'seen',
        innerPid,
        ...namespace,
      };
    /**
     The start time, which is read only for a process that has the owner's id.
     */
    const startTicks = startTicksFromStat(await readFile(
      join(
        procRoot,
        entry,
        'stat',
      ),
      'utf8',
    ));
    return {
      kind: 'seen',
      innerPid,
      ...namespace,
      ...(startTicks === processGone ? {} : { startTicks }),
    };
  }
  catch (error) {
    return isMissing(error) ? { kind: 'vanished' } : { kind: 'unreadable' };
  }
}

/**
 Judge the owner from what the entries showed.

 @param record - the owner's claim record

 @param startTicks - the owner's start time

 @param observations - what every entry showed

 @returns the standing

 @example
 ```ts
 judge({ record, startTicks: '123', observations: [] });
 ```
 */
function judge({
  record,
  startTicks,
  observations,
}: {
  readonly record: OwnedRecord;
  readonly startTicks: string;
  readonly observations: readonly Observation[];
}): ForeignStanding {
  /**
   The processes that were read.
   */
  const seen = observations.flatMap(function onlySeen(observation) {
    return observation.kind === 'seen' ? [observation] : [];
  });
  /**
   The processes that have the owner's id and could be in the owner's namespace: their link is the record's, or cannot be
   read, or the record names none.
   */
  const candidates = seen.filter(function hasOwnerId(observation) {
    return (observation.innerPid === record.pid)
      && ((observation.namespace === undefined) || (record.namespace === undefined)
        || (observation.namespace === record.namespace));
  });
  if (candidates.some(function sameStart(candidate) {
    return candidate.startTicks === startTicks;
  }))
    return 'alive';
  // The id in the owner's own namespace belongs to a process that started at another time, so the owner ended.
  if (candidates.some(function otherStart(candidate) {
    return (candidate.namespace === record.namespace) && (candidate.namespace !== undefined)
      && (candidate.startTicks !== undefined);
  }))
    return 'gone';
  // Every process was read and the owner's namespace was seen, so the owner would have shown if it ran.
  if ((record.namespace !== undefined)
    && observations.every(function readable(observation) {
      return observation.kind !== 'unreadable';
    })
    && seen.every(function linkRead(observation) {
      return observation.namespace !== undefined;
    })
    && seen.some(function inOwnerNamespace(observation) {
      return observation.namespace === record.namespace;
    }))
    return 'gone';
  return 'unknown';
}

/**
 Look for the owner of a claim among the processes of the process directory.

 @param record - the owner's claim record

 @param procRoot - the process directory

 @returns `alive` when a process with the owner's id and start time is there, `gone` when the processes prove it ended, and
 `unknown` when this process cannot tell, which the caller counts as alive

 @example
 ```ts
 await scanForOwner({ record, procRoot: '/proc' });
 ```
 */
export async function scanForOwner({
  record,
  procRoot,
}: {
  readonly record: OwnedRecord;
  readonly procRoot: string;
}): Promise<ForeignStanding> {
  /**
   The start time the record names.
   */
  const parsed = parseBirth(record.birth);
  if (parsed === birthNotParsed)
    return 'unknown';
  /**
   The ids of this namespace's processes.
   */
  const entries = (await readdir(procRoot)).filter(isProcessId);
  return judge({
    record,
    startTicks: parsed.startTicks,
    observations: await Promise.all(entries.map(function observeEntry(entry) {
      return observe({
        procRoot,
        entry,
        ownerPid: record.pid,
      });
    })),
  });
}
