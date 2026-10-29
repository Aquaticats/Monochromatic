import {
  rm,
  stat,
} from 'node:fs/promises';

import { caughtErrorHasCode, } from './staleness-manifest-error.ts';
import { lockOwnerState, } from './staleness-manifest-lock-owner.ts';

//region Stale lock recovery constants

/**
 Age after which directory lock is considered abandoned.
 */
const LOCK_STALE_AFTER_MS = 60_000;

//endregion Stale lock recovery constants

/**
 Sentinel for a coordination path missing before its mtime can be read.
 */
const ABSENT_LOCK_DIRECTORY_AGE: unique symbol = Symbol('file-enforcer/io/staleness-manifest-lock-recovery: coordination path missing before mtime read',);

/**
 Result of reading lock directory age.
 */
type LockDirectoryAge = number | typeof ABSENT_LOCK_DIRECTORY_AGE;

/**
 Action selected from one observed lock directory generation.
 */
type LockDirectoryRecoveryState = 'absent' | 'held' | 'recoverable';

//region Stale lock recovery helpers

/**
 Returns age of existing lock directory in milliseconds.
 
 @param lockPath - Lock directory path.
 
 @returns Lock directory age, or the {@link ABSENT_LOCK_DIRECTORY_AGE} sentinel
 when directory disappeared.
 
 @throws When lock metadata cannot be read for a reason other than absence.
 
 @example
 ```ts
 const age = await lockDirectoryAgeMs('/tmp/manifest.json.lock');
 ```
 */
async function lockDirectoryAgeMs(lockPath: string,): Promise<LockDirectoryAge> {
  try {
    /**
     Filesystem metadata for existing lock directory.
     */
    const lockStat = await stat(lockPath,);
    return Date.now() - lockStat.mtimeMs;
  }
  catch (statError: unknown) {
    if (caughtErrorHasCode({
      error: statError,
      code: 'ENOENT',
    },))
      return ABSENT_LOCK_DIRECTORY_AGE;

    throw statError;
  }
}

/**
 Returns whether existing lock directory is safe to reclaim: trusts
 {@link lockOwnerState} when it reports a definite verdict, otherwise
 falls back to {@link lockDirectoryAgeMs}.
 
 @param lockPath - Lock directory path.
 
 @returns Whether lock is absent, held, or recoverable.
 
 @example
 ```ts
 const stale = await lockDirectoryIsRecoverable('/tmp/manifest.json.lock');
 ```
 */
async function lockDirectoryRecoveryState(lockPath: string,): Promise<LockDirectoryRecoveryState> {
  /**
   Liveness state reported by lock owner metadata.
   */
  const ownerState = await lockOwnerState(lockPath,);
  if (ownerState === 'dead')
    return 'recoverable';
  if (ownerState === 'live')
    return 'held';

  /**
   Lock age from directory metadata.
   */
  const ageMs = await lockDirectoryAgeMs(lockPath,);
  if (ageMs === ABSENT_LOCK_DIRECTORY_AGE)
    return 'absent';
  if (ageMs >= LOCK_STALE_AFTER_MS)
    return 'recoverable';

  return 'held';
}

/**
 Removes stale lock directory when {@link lockDirectoryRecoveryState} says
 it is safe to reclaim.
 A lock that disappeared during observation retries without path removal,
 so a successor created at the same path cannot be deleted.
 
 @param lockPath - Lock directory path.
 
 @returns Whether acquisition should retry immediately.
 
 @example
 ```ts
 const retry = await recoverStaleManifestLock('/tmp/manifest.json.lock');
 ```
 */
export async function recoverStaleManifestLock(lockPath: string,): Promise<boolean> {
  /**
   Recovery action for observed lock directory generation.
   */
  const recoveryState = await lockDirectoryRecoveryState(lockPath,);
  if (recoveryState === 'held')
    return false;
  if (recoveryState === 'absent')
    return true;

  await rm(
    lockPath,
    {
      recursive: true,
      force: true,
    },
  );
  return true;
}

//endregion Stale lock recovery helpers
