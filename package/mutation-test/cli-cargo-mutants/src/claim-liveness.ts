/**
 Whether the owner a claim file names is still alive, by the rules of
 `package/git-policy/cli/src/native/single_owner_claim.rs` and `pid_namespace.rs`.
 */
import { stat } from 'node:fs/promises';

import type { ClaimRecord } from './claim-record.ts';
import { CargoMutantsToolingError } from './error.ts';
import {
  namespaceUnknown,
  readPidNamespace,
  readProcessBirth,
} from './process-identity.ts';

/**
 How long an owner of another PID namespace counts as alive, measured from when its claim file was written: one hour
 (`pid_namespace.rs:19`). A command holds a claim for seconds to hours; a crashed foreign owner stops blocking after this.
 */
export const foreignOwnerPatienceMs = 3_600_000;

/**
 Stands for an item no live process owns.
 */
export const noLiveOwner: unique symbol = Symbol('live-owner.absent');

/**
 Tell whether a claim file is older than the patience of owners in another PID namespace (`pid_namespace.rs:89`).

 @param ageMs - the file's age in milliseconds

 @returns true when the age is more than {@link foreignOwnerPatienceMs}; exactly the patience still counts as alive

 @example
 ```ts
 ageOutlivesPatience(3_600_001);
 ```
 */
export function ageOutlivesPatience(ageMs: number): boolean {
  return ageMs > foreignOwnerPatienceMs;
}

/**
 Tell whether the claim file of an owner in another PID namespace has outlived the patience (`pid_namespace.rs:70`).

 @param path - the claim file

 @returns false when the file cannot be measured, because what cannot be proved dead is never retired

 @example
 ```ts
 await foreignRecordHasExpired('/d/item.owner.1');
 ```
 */
async function foreignRecordHasExpired(path: string): Promise<boolean> {
  try {
    /**
     The file's status.
     */
    const status = await stat(path);
    return ageOutlivesPatience(Date.now() - status.mtimeMs);
  }
  catch (error) {
    console.error(`The age of ${path} cannot be measured, so its owner in another PID namespace counts as alive: ${String(error)}`);
    return false;
  }
}

/**
 The process id of the owner a record names when that process is alive (`single_owner_claim.rs:211`).

 @param record - what the claim file says

 @param path - the claim file, whose age judges an owner in another PID namespace

 @returns the owner's process id, or {@link noLiveOwner} when the claim is released or its owner is gone

 @throws CargoMutantsToolingError when whether the owner runs cannot be decided

 @example
 ```ts
 await liveOwnerPid({ record, path: '/d/item.owner.1' });
 ```
 */
export async function liveOwnerPid({
  record,
  path,
}: {
  readonly record: ClaimRecord;
  readonly path: string;
}): Promise<number | typeof noLiveOwner> {
  if (record.kind === 'released')
    return noLiveOwner;
  /**
   The namespace this process counts ids in.
   */
  const here = await readPidNamespace({ pid: process.pid });
  // Another namespace cannot be probed (`pid_namespace.rs:53`): the owner is alive until its claim file is old.
  if ((record.namespace !== undefined) && (here !== namespaceUnknown)
    && (record.namespace !== here))
    return (await foreignRecordHasExpired(path)) ? noLiveOwner : record.pid;
  try {
    /**
     The birth of whatever process has the id now.
     */
    const current = await readProcessBirth({ pid: record.pid });
    return current === record.birth ? record.pid : noLiveOwner;
  }
  catch (error) {
    throw new CargoMutantsToolingError(`${path} names process ${String(record.pid)}, and whether it still runs cannot be decided (${String(error)}); nothing was changed. If it does not run, remove the file and run the command again.`);
  }
}
