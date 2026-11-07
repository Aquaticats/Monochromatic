/**
 The run lock of the daily update: one run at a time, and no run waits for another.

 Lock-free is not possible for "one run at a time", so the lock is the smallest one there is: one file created with the
 exclusive flag, so creation succeeds for one process and fails for the others; the file names its holder, and a lock whose
 holder is gone (a killed run) is replaced once.

 @module
 */

import {
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

/**
 Largest process id the lock file may name, the Linux `pid_max` ceiling; anything above is not a process id.
 */
export const LARGEST_PROCESS_ID = 4_194_304;

/**
 A held or refused run lock.
 */
export type RunLock = {
  /**
   Whether this process holds the lock.
   */
  readonly acquired: boolean;
  /**
   Releases the lock when this process holds it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Whether a process with this id exists.

 @param pid - process id from a lock file

 @returns true when signal 0 reaches it

 @example
 ```ts
 isAlive(process.pid,); // true
 ```
 */
export function isAlive(pid: number,): boolean {
  if ((!Number.isInteger(pid,)) || (pid <= 0)
    || (pid > LARGEST_PROCESS_ID)) {
    return false;
  }
  try {
    process.kill(
      pid,
      0,
    );
    return true;
  }
  catch (error) {
    // Signal 0 only asks; a refusal means there is no such process.
    process.stderr
      .write(`daily-update: process ${pid} is not running: ${String(error,)}\n`,);
    return false;
  }
}

/**
 Tries to create the lock file.

 @param lockPath - absolute path of the lock file

 @returns true when this process created it, false when it already exists

 @throws Error for any failure other than the file already existing
 */
async function createLockFile(lockPath: string,): Promise<boolean> {
  try {
    await writeFile(
      lockPath,
      String(process.pid,),
      { flag: 'wx', },
    );
    return true;
  }
  catch (error) {
    if ((Error.isError(error,)) && ('code' in error)
      && (error.code === 'EEXIST')) {
      return false;
    }
    throw error;
  }
}

/**
 A lock this process holds and removes when released.

 @param lockPath - absolute path of the lock file

 @returns the held lock

 @example
 ```ts
 await using lock = heldLock('/r/lock',);
 ```
 */
export function heldLock(lockPath: string,): RunLock {
  return {
    acquired: true,
    [Symbol.asyncDispose]: async function releaseLock() {
      await rm(
        lockPath,
        { force: true, },
      );
    },
  };
}

/**
 A lock another run holds; releasing it does nothing, because it is not this process's.

 @returns the refused lock

 @example
 ```ts
 await using lock = refusedLock();
 ```
 */
export function refusedLock(): RunLock {
  return {
    acquired: false,
    [Symbol.asyncDispose]: async function keepLock() {
      // The lock belongs to another run, which releases it.
    },
  };
}

/**
 Takes the run lock without waiting.

 @param lockPath - absolute path of the lock file

 @returns the lock, to be held with `await using`; `acquired` is false when a live run holds it

 @example
 ```ts
 await using lock = await takeRunLock('/r/.cache/toolchain-daily-update.lock',);
 if (!lock.acquired) { return; }
 ```
 */
export async function takeRunLock(lockPath: string,): Promise<RunLock> {
  await mkdir(
    dirname(lockPath,),
    { recursive: true, },
  );
  if (await createLockFile(lockPath,)) {
    return heldLock(lockPath,);
  }
  /**
   Process id the existing lock file names.
   */
  const holder = Number(await readFile(
    lockPath,
    'utf8',
  ),);
  if (isAlive(holder,)) {
    return refusedLock();
  }
  // The holder is gone, so its lock is stale; it is removed once and creation is tried once more.
  await rm(
    lockPath,
    { force: true, },
  );
  return (await createLockFile(lockPath,)) ? heldLock(lockPath,) : refusedLock();
}
