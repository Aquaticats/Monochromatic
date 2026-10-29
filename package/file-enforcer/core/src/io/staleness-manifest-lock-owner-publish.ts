import {
  rename,
  rmdir,
  unlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

//region Lock owner publication paths

/**
 Metadata file made visible only after complete staging.
 */
const LOCK_OWNER_FILE_NAME = 'owner.json';

/**
 Private staging file renamed atomically after complete owner serialization.
 */
const LOCK_OWNER_PENDING_FILE_NAME = 'owner.pending.json';

/**
 Returns private staging path for lock owner metadata.
 
 @param lockPath - Lock directory path.
 
 @returns Private staging path.
 
 @example
 ```ts
 const path = pendingLockOwnerPath('/tmp/manifest.json.lock');
 ```
 */
function pendingLockOwnerPath(lockPath: string,): string {
  return join(
    lockPath,
    LOCK_OWNER_PENDING_FILE_NAME,
  );
}

/**
 Returns published path for lock owner metadata.
 
 @param lockPath - Lock directory path.
 
 @returns Published owner path.
 
 @example
 ```ts
 const path = publishedLockOwnerPath('/tmp/manifest.json.lock');
 ```
 */
function publishedLockOwnerPath(lockPath: string,): string {
  return join(
    lockPath,
    LOCK_OWNER_FILE_NAME,
  );
}

//endregion Lock owner publication paths

//region Lock owner publication lifecycle

/**
 Writes complete lock owner text to private staging path.
 
 @param lockPath - Lock directory path.
 
 @param ownerText - Complete serialized owner metadata.
 
 @example
 ```ts
 await stageLockOwnerPublication({ lockPath, ownerText });
 ```
 */
export async function stageLockOwnerPublication(
  {
    lockPath,
    ownerText,
  }: {
    readonly lockPath: string;
    readonly ownerText: string;
  },
): Promise<void> {
  await writeFile(
    pendingLockOwnerPath(lockPath,),
    ownerText,
  );
}

/**
 Atomically publishes previously staged lock owner metadata.
 
 @param lockPath - Lock directory path.
 
 @example
 ```ts
 await publishLockOwnerPublication('/tmp/manifest.json.lock');
 ```
 */
export async function publishLockOwnerPublication(lockPath: string,): Promise<void> {
  await rename(
    pendingLockOwnerPath(lockPath,),
    publishedLockOwnerPath(lockPath,),
  );
}

/**
 Removes published owner and its lock directory with one-shot operations.
 
 Avoiding recursive path removal prevents a completed release from revisiting
 and deleting a successor lock created at the same path.
 
 @param lockPath - Lock directory path owned by caller.
 
 @example
 ```ts
 await removeLockOwnerPublication('/tmp/manifest.json.lock');
 ```
 */
export async function removeLockOwnerPublication(lockPath: string,): Promise<void> {
  await unlink(publishedLockOwnerPath(lockPath,),);
  await rmdir(lockPath,);
}

//endregion Lock owner publication lifecycle
