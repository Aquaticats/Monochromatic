import {
  mkdir,
  rm,
} from 'node:fs/promises';
import { dirname, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';

import { recoverStaleManifestLock, } from './staleness-manifest-lock-recovery.ts';
import { removeLockOwnerPublication, } from './staleness-manifest-lock-owner-publish.ts';
import { writeLockOwner, } from './staleness-manifest-lock-owner.ts';
import {
  caughtErrorHasCode,
  StalenessManifestPersistenceError,
} from './staleness-manifest-error.ts';

//region Locking constants

/**
 Suffix appended to manifest path for directory locks.
 */
const LOCK_DIRECTORY_SUFFIX = '.lock';

/**
 Poll delay while waiting for another process to release manifest lock.
 */
const LOCK_RETRY_MS = 10;

/**
 Maximum time to wait for another process to release manifest lock.
 */
const LOCK_TIMEOUT_MS = 5_000;

//endregion Locking constants

//region Lock acquisition helpers

/**
 Removes lock directory if owner metadata cannot be written.
 
 @param lockPath - Lock directory path.
 
 @param ownerError - Error thrown while writing owner metadata.
 
 @throws Always rethrows `ownerError` after cleanup.
 
 @example
 ```ts
 await cleanupFailedOwnerWrite({ lockPath, ownerError });
 ```
 */
async function cleanupFailedOwnerWrite(
  {
    lockPath,
    ownerError,
  }: {
    readonly lockPath: string;
    readonly ownerError: unknown;
  },
): Promise<never> {
  await rm(
    lockPath,
    {
      recursive: true,
      force: true,
    },
  );
  throw ownerError;
}

/**
 Records lock owner metadata after directory acquisition.
 
 @param lockPath - Lock directory path.
 
 @example
 ```ts
 await recordLockOwner('/tmp/manifest.json.lock');
 ```
 */
async function recordLockOwner(lockPath: string,): Promise<void> {
  try {
    await writeLockOwner(lockPath,);
  }
  catch (ownerError: unknown) {
    await cleanupFailedOwnerWrite({
      lockPath,
      ownerError,
    },);
  }
}

/**
 Returns async disposable handle that releases lock directory.
 
 @param lockPath - Lock directory path.
 
 @returns Async disposable release handle.
 
 @example
 ```ts
 await using lock = lockReleaseHandle('/tmp/manifest.json.lock');
 ```
 */
function lockReleaseHandle(lockPath: string,): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await removeLockOwnerPublication(lockPath,);
    },
  };
}

//endregion Lock acquisition helpers

//region Lock acquisition

/**
 Acquires manifest directory lock, retrying contended locks via
 {@link recoverStaleManifestLock}, and returns a {@link lockReleaseHandle}
 async disposable release handle.
 
 @param manifestPath - Absolute manifest path whose lock should be held.
 
 @returns Async disposable lock release handle.
 
 @throws {@link StalenessManifestPersistenceError} When lock cannot be acquired before timeout.
 
 @example
 ```ts
 await using lock = await acquireManifestLock('/tmp/manifest.json');
 ```
 */
export async function acquireManifestLock(manifestPath: string,): Promise<AsyncDisposable> {
  await mkdir(
    dirname(manifestPath,),
    { recursive: true, },
  );
  /**
   Directory path used as inter-process lock.
   */
  const lockPath = `${manifestPath}${LOCK_DIRECTORY_SUFFIX}`;
  /**
   Last timestamp at which lock acquisition may keep retrying.
   */
  const deadline = Date.now() + LOCK_TIMEOUT_MS;

  /**
   Acquired release handle; absence explicitly keeps lock acquisition polling.
   */
  const acquisition: { release?: AsyncDisposable; } = {};
  while (acquisition.release === undefined) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- lock acquisition must observe each create attempt before deciding whether to retry.
      await mkdir(lockPath,);
      // oxlint-disable-next-line eslint/no-await-in-loop -- owner metadata belongs to the directory acquired by the immediately preceding mkdir.
      await recordLockOwner(lockPath,);
      acquisition.release = lockReleaseHandle(lockPath,);
    }
    catch (lockError: unknown) {
      if (!caughtErrorHasCode({
        error: lockError,
        code: 'EEXIST',
      },))
        throw lockError;
      // oxlint-disable-next-line eslint/no-await-in-loop -- stale recovery must run against the currently observed contended lock.
      if (await recoverStaleManifestLock(lockPath,))
        continue;
      if (Date.now() > deadline) {
        throw new StalenessManifestPersistenceError(
          `Timed out waiting for staleness manifest lock ${lockPath}`,
          { cause: lockError, },
        );
      }
      // oxlint-disable-next-line eslint/no-await-in-loop -- retries intentionally poll one lock state snapshot per delay.
      await wait(LOCK_RETRY_MS,);
    }
  }
  return acquisition.release;
}

//endregion Lock acquisition
