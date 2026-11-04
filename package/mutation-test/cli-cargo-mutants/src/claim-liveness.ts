/**
 Whether the owner a claim file names is still alive, by the rules of
 `package/git-policy/cli/src/native/lock/single_owner_claim.rs`, except that no age ever decides.

 An owner is alive or gone by evidence about the process: its id with its start time and boot. In this PID namespace the
 process directory answers directly. For an owner of another namespace, or of one the record or this process does not name,
 the id may mean nothing here, so the processes this one can see are searched for the owner's id inside its own namespace
 (`src/foreign-process-scan.ts`), and what cannot be decided counts as alive: a live holder keeps its claim however long it
 runs, and a holder that cannot be shown dead is removed only by a person (the notice names the lock directory and the holder's process id). The one
 proof that needs no view into the namespace is the boot: a claim written before the current boot names nothing that runs.
 */
import type { ClaimRecord } from './claim-record.ts';
import { CargoMutantsToolingError } from './error.ts';
import { scanForOwner } from './foreign-process-scan.ts';
import {
  birthNotParsed,
  defaultProcRoot,
  type namespaceUnknown,
  parseBirth,
  readBootId,
  readPidNamespace,
  readProcessBirth,
} from './process-identity.ts';

/**
 Stands for an item no live process owns.
 */
export const noLiveOwner: unique symbol = Symbol('live-owner.absent');

/**
 Tell whether the owner of a record began in an earlier boot than this one.

 @param birth - the owner's birth text

 @returns true only when the text names a boot that is not the current one; a text of another spelling proves nothing

 @example
 ```ts
 await beganInEarlierBoot('linux:1@a-boot');
 ```
 */
async function beganInEarlierBoot(birth: string): Promise<boolean> {
  /**
   What the birth text says.
   */
  const parsed = parseBirth(birth);
  return (parsed !== birthNotParsed) && (parsed.boot !== await readBootId());
}

/**
 Tell whether a record and this process name the same PID namespace, which is the only case in which the record's process id
 is an id here.

 @param recorded - the namespace the record names, absent when it names none

 @param here - the namespace this process counts ids in

 @returns true only when both are named and equal

 @example
 ```ts
 sameNamespace({ recorded: 'pid:[1]', here: 'pid:[1]' });
 ```
 */
function sameNamespace({
  recorded,
  here,
}: {
  readonly recorded?: string;
  readonly here: string | typeof namespaceUnknown;
}): boolean {
  if (recorded === undefined)
    return false;
  return recorded === here;
}

/**
 Judge an owner whose process id may not be an id of this namespace.

 @param record - what the claim file says

 @param procRoot - the directory that shows the processes

 @returns the owner's process id while it may be alive, or {@link noLiveOwner} when evidence shows it ended

 @example
 ```ts
 await foreignOwnerPid({ record, procRoot: '/proc' });
 ```
 */
async function foreignOwnerPid({
  record,
  procRoot,
}: {
  readonly record: Extract<ClaimRecord, { kind: 'owned' }>;
  readonly procRoot: string;
}): Promise<number | typeof noLiveOwner> {
  if (await beganInEarlierBoot(record.birth))
    return noLiveOwner;
  return (await scanForOwner({
    record,
    procRoot,
  })) === 'gone' ? noLiveOwner : record.pid;
}

/**
 The process id of the owner a record names when that process may be alive.

 @param record - what the claim file says

 @param path - the claim file, named when the owner cannot be judged

 @param procRoot - the directory that shows the processes; a test gives a fake one

 @returns the owner's process id, or {@link noLiveOwner} when the claim is released or evidence shows its owner ended

 @throws CargoMutantsToolingError when whether an owner of this namespace runs cannot be decided

 @example
 ```ts
 await liveOwnerPid({ record, path: '/d/item.owner.1' });
 ```
 */
export async function liveOwnerPid({
  record,
  path,
  procRoot = defaultProcRoot,
}: {
  readonly record: ClaimRecord;
  readonly path: string;
  readonly procRoot?: string;
}): Promise<number | typeof noLiveOwner> {
  if (record.kind === 'released')
    return noLiveOwner;
  /**
   The namespace this process counts ids in.
   */
  const here = await readPidNamespace({
    pid: process.pid,
    procRoot,
  });
  // An id is an id of this namespace only when both namespaces are named and equal; with either unnamed, or different, the
  // id may mean nothing here, and a lookup of it here could name an unrelated process or none.
  if (!sameNamespace({
    ...(record.namespace === undefined ? {} : { recorded: record.namespace }),
    here,
  }))
    return await foreignOwnerPid({
      record,
      procRoot,
    });
  try {
    /**
     The birth of whatever process has the id now.
     */
    const current = await readProcessBirth({
      pid: record.pid,
      procRoot,
    });
    return current === record.birth ? record.pid : noLiveOwner;
  }
  catch (error) {
    throw new CargoMutantsToolingError(`${path} names process ${String(record.pid)}, and whether it still runs cannot be decided (${String(error)}); nothing was changed. If it does not run, remove the file and run the command again.`);
  }
}
