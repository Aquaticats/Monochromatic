/**
 Bounded persistent effect-summary cache maintenance.
 
 @module
 */

import {
  type Dirent,
  readdirSync,
  statSync,
  unlinkSync,
} from 'node:fs';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Cache maintenance logger.
 */
const l = tagged({ tag: 'effect-summary-cache-maintenance', },);

/**
 Bytes in one kibibyte.
 */
const BYTES_PER_KIBIBYTE = 1_024;

/**
 Kibibytes in one mebibyte.
 */
const KIBIBYTES_PER_MEBIBYTE = 1_024;

/**
 Mebibytes retained per dependency root, about twice the 510 MB the translation-repair family needs.
 */
const MAX_CACHE_ROOT_MEBIBYTES = 1_024;

/**
 Days retained before age eviction.
 */
const CACHE_RETENTION_DAYS = 7;

/**
 Hours in one day.
 */
const HOURS_PER_DAY = 24;

/**
 Minutes in one hour.
 */
const MINUTES_PER_HOUR = 60;

/**
 Seconds in one minute.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Milliseconds in one second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Writes between complete root maintenance scans.
 */
const MAINTENANCE_WRITE_INTERVAL = 128;

/**
 Maximum retained content-addressed entries per root, about twice the 8300 the translation-repair family needs.
 */
const MAX_CACHE_ENTRY_COUNT = 16_384;

/**
 Maximum retained bytes per dependency root.
 */
const MAX_CACHE_ROOT_BYTES = MAX_CACHE_ROOT_MEBIBYTES
  * KIBIBYTES_PER_MEBIBYTE
  * BYTES_PER_KIBIBYTE;

/**
 Maximum entry age before removal.
 */
const MAX_CACHE_ENTRY_AGE_MS = CACHE_RETENTION_DAYS
  * HOURS_PER_DAY
  * MINUTES_PER_HOUR
  * SECONDS_PER_MINUTE
  * MILLISECONDS_PER_SECOND;

/**
 Write counts since process start by cache root.
 */
const writesByRoot = new Map<string, number>();

/**
 One immutable cache file and metadata used for eviction.
 */
type CacheFile = {
  readonly path: string;
  readonly bytes: number;
  readonly modifiedAt: number;
};

/**
 Reads immutable cache file metadata under root.
 
 @param root - Persistent cache root.
 
 @returns existing JSON cache files sorted oldest first.
 */
function cacheFiles(root: string,): readonly CacheFile[] {
  /* oxlint-disable no-restricted-syntax/no-sync -- Maintenance periodically scans local persistent cache before process exit. */
  /**
   Foreign filesystem entries returned by Node boundary.
   */
  const entries: ForeignBorrowed<Dirent[]> = readdirSync(
    root,
    {
      recursive: true,
      withFileTypes: true,
    },
  );
  /* oxlint-enable no-restricted-syntax/no-sync */
  /**
   Existing cache file metadata accumulated from filesystem entries.
   */
  const files: CacheFile[] = [];
  for (const entry of entries) {
    if ((!entry.isFile()) || (!entry.name
      .endsWith('.json',)))
      continue;
    /**
     Absolute immutable cache entry path.
     */
    const path = join(
      entry.parentPath,
      entry.name,
    );
    try {
      /* oxlint-disable no-restricted-syntax/no-sync -- Maintenance needs exact file size and age for bounded eviction. */
      /**
       Current file metadata.
       */
      const metadata = statSync(path,);
      /* oxlint-enable no-restricted-syntax/no-sync */
      files.push({
        path,
        bytes: metadata.size,
        modifiedAt: metadata.mtimeMs,
      },);
    }
    catch (error) {
      l.debug(`cache metadata skipped for ${path}: ${String(error,)}`,);
    }
  }
  return files.toSorted(function oldestFirst(
    left,
    right,
  ): number {
    return left.modifiedAt - right.modifiedAt;
  },);
}

/**
 Removes expired or overflow entries without touching current write.
 
 @param files - Oldest-first immutable cache files.
 
 @param retainedPath - Current entry retained despite limits.
 
 */
function evictCacheFiles({
  files,
  retainedPath,
}: {
  readonly files: readonly CacheFile[];
  readonly retainedPath: string;
}): void {
  /**
   Mutable aggregate tracking entries left after each successful eviction.
   */
  const retained = {
    count: files.length,
    bytes: files.reduce(
      function totalBytes(
        total,
        file,
      ): number {
        return total + file.bytes;
      },
      0,
    ),
  };
  /**
   Current wall-clock threshold for age eviction.
   */
  const expirationThreshold = Date.now() - MAX_CACHE_ENTRY_AGE_MS;
  files.forEach(function evict(file,): void {
    /**
     Whether entry age exceeds retention policy.
     */
    const expired = file.modifiedAt < expirationThreshold;
    /**
     Whether retained aggregate still exceeds count or byte policy.
     */
    const overflow = (retained.count > MAX_CACHE_ENTRY_COUNT)
      || (retained.bytes > MAX_CACHE_ROOT_BYTES);
    if (((!expired) && (!overflow)) || (file.path === retainedPath))
      return;
    try {
      // oxlint-disable-next-line no-restricted-syntax/no-sync -- Maintenance removes one immutable stale cache entry.
      unlinkSync(file.path,);
      retained.count--;
      retained.bytes -= file.bytes;
    }
    catch (error) {
      l.debug(`cache eviction skipped for ${file.path}: ${String(error,)}`,);
    }
  },);
}

/**
 Periodically bounds persistent entries under dependency root.
 
 @param root - Persistent cache root.
 
 @param retainedPath - Current atomically published entry.
 
 @example
 ```ts
 maintainPersistentEffectCache({ root, retainedPath });
 ```
 */
export function maintainPersistentEffectCache({
  root,
  retainedPath,
}: {
  readonly root: string;
  readonly retainedPath: string;
}): void {
  /**
   Number of writes observed for root in this process.
   */
  const writeCount = (writesByRoot.get(root,) ?? 0) + 1;
  writesByRoot.set(
    root,
    writeCount,
  );
  if ((writeCount !== 1) && ((writeCount % MAINTENANCE_WRITE_INTERVAL) !== 0))
    return;
  try {
    evictCacheFiles({
      files: cacheFiles(root,),
      retainedPath,
    },);
  }
  catch (error) {
    l.debug(`cache maintenance skipped for ${root}: ${String(error,)}`,);
  }
}
