import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  caughtErrorHasCode,
  caughtErrorMessage,
  StalenessManifestPersistenceError,
} from './staleness-manifest-error.ts';
import {
  publishLockOwnerPublication,
  stageLockOwnerPublication,
} from './staleness-manifest-lock-owner-publish.ts';

//region Lock owner constants and types

/**
 Metadata file written inside each acquired lock directory.
 */
const LOCK_OWNER_FILE_NAME = 'owner.json';

/**
 Minimum valid operating-system process id.
 */
const MINIMUM_PROCESS_ID = 1;

/**
 Signal number reserved by POSIX and Node for process-existence checks.
 */
const PROCESS_EXISTS_SIGNAL = 0;

/**
 Sentinel for a missing pid record while checking stale writer liveness.
 */
const ABSENT_MANIFEST_LOCK_OWNER: unique symbol = Symbol('file-enforcer/io/staleness-manifest-lock-owner: pid record missing while checking writer liveness',);

/**
 Metadata stored in lock owner file.
 */
type ManifestLockOwner = Readonly<{
  /**
   Process id that acquired lock.
   */
  readonly pid: number;

  /**
   ISO timestamp for diagnostics.
   */
  readonly createdAt: string;
}>;

/**
 Result of reading lock owner metadata.
 */
type ManifestLockOwnerRead = ManifestLockOwner | typeof ABSENT_MANIFEST_LOCK_OWNER;

/**
 Liveness state inferred from lock owner metadata.
 */
export type ManifestLockOwnerState = 'absent' | 'dead' | 'live';

//endregion Lock owner constants and types

//region Lock owner file helpers

/**
 Returns owner metadata file path for a lock directory.
 
 @param lockPath - Lock directory path.
 
 @returns Owner metadata path.
 
 @example
 ```ts
 const path = lockOwnerPath('/tmp/manifest.json.lock');
 ```
 */
function lockOwnerPath(lockPath: string,): string {
  return join(
    lockPath,
    LOCK_OWNER_FILE_NAME,
  );
}

/**
 Returns whether parsed JSON value is lock owner metadata.
 
 @param value - Parsed JSON value.
 
 @returns Whether value is owner metadata.
 
 @example
 ```ts
 const valid = isManifestLockOwner(value);
 ```
 */
function isManifestLockOwner(value: unknown,): value is ManifestLockOwner {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  if (Array.isArray(value,))
    return false;
  /**
   Potential owner fields from parsed JSON object.
   */
  const {
    createdAt,
    pid,
  } = value as {
    readonly createdAt?: unknown;
    readonly pid?: unknown;
  };
  if ((typeof pid) !== 'number')
    return false;
  if (!Number.isInteger(pid,))
    return false;
  if (pid < MINIMUM_PROCESS_ID)
    return false;

  return (typeof createdAt) === 'string';
}

/**
 Writes owner metadata into newly acquired lock directory, at the path
 returned by {@link lockOwnerPath}.
 
 @param lockPath - Lock directory path.
 
 @example
 ```ts
 await writeLockOwner('/tmp/manifest.json.lock');
 ```
 */
export async function writeLockOwner(lockPath: string,): Promise<void> {
  /**
   Metadata describing current lock holder.
   */
  const owner: ManifestLockOwner = {
    pid: process.pid,
    createdAt: new Date().toISOString(),
  };
  await stageLockOwnerPublication({
    lockPath,
    ownerText: `${JSON.stringify(
      owner,
      null,
      2,
    )}\n`,
  },);
  await publishLockOwnerPublication(lockPath,);
}

/**
 Reads owner metadata from the {@link lockOwnerPath} file when present and
 valid per {@link isManifestLockOwner}.
 
 @param lockPath - Lock directory path.
 
 @returns Owner metadata, or the {@link ABSENT_MANIFEST_LOCK_OWNER} sentinel when absent.
 
 @throws {@link StalenessManifestPersistenceError} When owner metadata exists
 but cannot be parsed (message derived via {@link caughtErrorMessage}) or
 validated; absence is distinguished via {@link caughtErrorHasCode}.
 
 @example
 ```ts
 const owner = await readLockOwner('/tmp/manifest.json.lock');
 ```
 */
async function readLockOwner(lockPath: string,): Promise<ManifestLockOwnerRead> {
  try {
    /**
     Parsed owner metadata JSON.
     */
    const owner: unknown = JSON.parse(await readFile(
      lockOwnerPath(lockPath,),
      'utf8',
    ),);
    if (isManifestLockOwner(owner,))
      return owner;

    throw new StalenessManifestPersistenceError(
      `Invalid staleness manifest lock owner schema at ${lockOwnerPath(lockPath,)}`,
    );
  }
  catch (ownerError: unknown) {
    if (caughtErrorHasCode({
      error: ownerError,
      code: 'ENOENT',
    },))
      return ABSENT_MANIFEST_LOCK_OWNER;
    if (ownerError instanceof SyntaxError) {
      throw new StalenessManifestPersistenceError(
        `Invalid staleness manifest lock owner ${lockOwnerPath(lockPath,)}: ${caughtErrorMessage(ownerError,)}`,
        { cause: ownerError, },
      );
    }

    throw ownerError;
  }
}

//endregion Lock owner file helpers

//region Process liveness helpers

/**
 Returns whether process id appears to identify a live process, treating
 an `ESRCH` error (checked via {@link caughtErrorHasCode}) as dead.
 
 @param pid - Process id from lock owner metadata.
 
 @returns Whether process still appears alive or inaccessible.
 
 @example
 ```ts
 const alive = processAppearsAlive(process.pid);
 ```
 */
function processAppearsAlive(pid: number,): boolean {
  try {
    process.kill(
      pid,
      PROCESS_EXISTS_SIGNAL,
    );
    return true;
  }
  catch (signalError: unknown) {
    if (caughtErrorHasCode({
      error: signalError,
      code: 'ESRCH',
    },))
      return false;
    if (caughtErrorHasCode({
      error: signalError,
      code: 'EPERM',
    },))
      return true;

    return true;
  }
}

/**
 Returns liveness state for lock owner metadata read via {@link readLockOwner}
 (`'absent'` when the {@link ABSENT_MANIFEST_LOCK_OWNER} sentinel comes back),
 checking the recorded pid with {@link processAppearsAlive}.
 
 @param lockPath - Lock directory path.
 
 @returns Whether owner metadata is absent, dead, or live.
 
 @example
 ```ts
 const state = await lockOwnerState('/tmp/manifest.json.lock');
 ```
 */
export async function lockOwnerState(lockPath: string,): Promise<ManifestLockOwnerState> {
  /**
   Owner metadata read from lock directory.
   */
  const owner = await readLockOwner(lockPath,);
  if (owner === ABSENT_MANIFEST_LOCK_OWNER)
    return 'absent';
  if (processAppearsAlive(owner.pid,))
    return 'live';

  return 'dead';
}

//endregion Process liveness helpers
