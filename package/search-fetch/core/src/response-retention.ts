/**
 Retention policy for truncated full-response temp directories.

 Every truncated tool response writes its full text into a fresh
 `pi-search-fetch-response-*` directory under the scratch parent. Nothing
 removed those directories before this policy existed, so each host slowly
 accumulated them. After a successful write the pruner deletes owned response
 directories older than 7 days and anything beyond the newest 64, counting the
 directory just written and never removing it. The cap is per scratch parent,
 never machine-global, and pruning is best-effort: only the owned prefix is
 touched, concurrent writers and already-deleted entries are tolerated, and a
 deletion failure is logged without failing the search or fetch.

 @module
 */

import {
  readdir,
  rm,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

//region Constants

/**
 Prefix marking the response directories this policy owns; nothing without it
 is ever touched.
 */
export const RESPONSE_TEMP_DIR_PREFIX = 'pi-search-fetch-response-';

/**
 Age after which a response directory is deleted: 7 days in milliseconds.
 */
export const RESPONSE_MAX_AGE_MILLISECONDS: number = 604_800_000;

/**
 Number of response directories kept per scratch parent, counting the
 directory just written.
 */
export const RESPONSE_MAX_DIR_COUNT = 64;

//endregion Constants

//region Types

/**
 Owned response directory considered by one pruning pass.
 */
type ManagedResponseDir = {
  /**
   Entry name inside the scratch parent.
   */
  readonly name: string;
  /**
   Absolute directory path.
   */
  readonly dirPath: string;
  /**
   Last modification time in milliseconds.
   */
  readonly mtimeMs: number;
  /**
   Whether this is the directory just returned to the caller.
   */
  readonly keeping: boolean;
};

/**
 Input shared by the pruning pass and its per-entry helpers.
 */
type PrunePassInput = {
  /**
   Scratch parent holding the owned response directories.
   */
  readonly parentDir: string;
  /**
   Directory holding the response just returned to the caller; never deleted.
   */
  readonly keepDir: string;
  /**
   Age policy bound in milliseconds.
   */
  readonly maxAgeMs: number;
  /**
   Count policy bound.
   */
  readonly maxCount: number;
  /**
   Logger already tagged for the pruning pass.
   */
  readonly logger: Logger;
};

//endregion Types

//region Logger

/**
 Logger root for pi-search-fetch after removing the package log shim.
 */
const responseRetentionLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'response-retention',
  l: responseRetentionLogger,
},);

//endregion Logger

//region Public API

/**
 Prune owned response directories in one scratch parent under the retention policy.

 Deletes owned directories older than `maxAgeMs`, then anything beyond the
 newest `maxCount` survivors, always keeping `keepDir` and counting it toward
 the cap. Best-effort by contract: listing failures are logged and skipped,
 per-directory deletion failures are logged and left in place, and the function
 never throws, so a search or fetch can never fail because pruning failed.

 @param parentDir - scratch parent holding the owned response directories

 @param keepDir - directory holding the response just returned; never deleted

 @param maxAgeMs - age policy override in milliseconds, primarily for tests

 @param maxCount - count policy override, primarily for tests

 @example
 ```ts
 await pruneResponseTempDirs({ parentDir, keepDir: tempDir });
 ```
 */
export async function pruneResponseTempDirs(
  {
    parentDir,
    keepDir,
    maxAgeMs = RESPONSE_MAX_AGE_MILLISECONDS,
    maxCount = RESPONSE_MAX_DIR_COUNT,
  }: {
    /**
     Scratch parent holding the owned response directories.
     */
    readonly parentDir: string;
    /**
     Directory holding the response just returned to the caller; never deleted.
     */
    readonly keepDir: string;
    /**
     Age policy override in milliseconds, primarily for tests.
     */
    readonly maxAgeMs?: number;
    /**
     Count policy override, primarily for tests.
     */
    readonly maxCount?: number;
  },
): Promise<void> {
  /**
   Logger tagged for this pruning pass.
   */
  const innerL = tagged({
    tag: pruneResponseTempDirs.name,
    l,
  },);
  // Deliberate catch-and-log: pruning is best-effort retention work, and a
  // scratch parent that cannot be listed must never fail the search or fetch.
  try {
    await pruneOwnedResponseDirs({
      parentDir,
      keepDir,
      maxAgeMs,
      maxCount,
      logger: innerL,
    },);
  }
  catch (error: unknown) {
    innerL.warn(`response temp pruning skipped in ${parentDir}: ${caughtValueText(error,)}`,);
  }
}

//endregion Public API

//region Helpers

/**
 Run one full pruning pass over a scratch parent.

 @param parentDir - scratch parent holding the owned response directories

 @param keepDir - directory holding the response just returned; never deleted

 @param maxAgeMs - age policy bound in milliseconds

 @param maxCount - count policy bound

 @param logger - logger already tagged for the pruning pass

 @example
 ```ts
 await pruneOwnedResponseDirs({ parentDir, keepDir, maxAgeMs, maxCount, logger: innerL });
 ```
 */
async function pruneOwnedResponseDirs(
  {
    parentDir,
    keepDir,
    maxAgeMs,
    maxCount,
    logger,
  }: PrunePassInput,
): Promise<void> {
  /**
   Entry names currently present in the scratch parent.
   */
  const entryNames = await readdir(parentDir,);
  /**
   Owned response directories with their modification times, collected
   concurrently so a vanished entry only drops itself out of the pass.
   */
  const managed: ManagedResponseDir[] = [];
  await Promise.all(entryNames
    .filter(function isOwnedEntry(name,) {
      return name.startsWith(RESPONSE_TEMP_DIR_PREFIX,);
    },)
    .map(function collectOne(name,) {
      return collectManagedDir({
        parentDir,
        name,
        keepDir,
        managed,
        logger,
      },);
    },),);

  /**
   Cutoff instant: entries modified before it are older than the age policy.
   */
  const ageCutoffMs = Date.now() - maxAgeMs;
  /**
   Directories surviving the age policy, including the kept directory.
   */
  const survivors: ManagedResponseDir[] = [];
  /**
   Directories the age policy deletes.
   */
  const overAge: ManagedResponseDir[] = [];
  for (const entry of managed) {
    if (entry.keeping) {
      survivors.push(entry,);
      continue;
    }
    if (entry.mtimeMs < ageCutoffMs) {
      overAge.push(entry,);
      continue;
    }
    survivors.push(entry,);
  }

  /**
   Directories the count policy deletes beyond the newest `maxCount`, where the
   kept directory occupies one retained slot by sorting ahead of every timestamp.
   */
  const beyondCap = survivors
    .toSorted(function newestFirst(
      left: ManagedResponseDir,
      right: ManagedResponseDir,
    ): number {
      if (left.keeping !== right.keeping)
        return left.keeping ? -1 : 1;
      if (left.mtimeMs !== right.mtimeMs)
        return right.mtimeMs - left.mtimeMs;
      if (left.name === right.name)
        return 0;
      return left.name > right.name ? -1 : 1;
    },)
    .slice(maxCount,);

  await Promise.all([
    ...overAge,
    ...beyondCap,
  ].map(function deleteOne(entry,) {
    return deleteResponseDir({
      dirPath: entry.dirPath,
      logger,
    },);
  },),);
}

/**
 Read one candidate entry and record it when it is a managed response directory.

 Pushes into the shared collection instead of returning a possibly-absent
 value, so a concurrent writer or deleter that moved the entry simply
 contributes nothing to the pass.

 @param parentDir - scratch parent holding the entry

 @param name - candidate entry name

 @param keepDir - directory holding the response just returned

 @param managed - shared collection of managed directories

 @param logger - logger already tagged for the pruning pass

 @example
 ```ts
 await collectManagedDir({ parentDir, name, keepDir, managed, logger: innerL });
 ```
 */
async function collectManagedDir(
  {
    parentDir,
    name,
    keepDir,
    managed,
    logger,
  }: {
    readonly parentDir: string;
    readonly name: string;
    readonly keepDir: string;
    readonly managed: ManagedResponseDir[];
    readonly logger: Logger;
  },
): Promise<void> {
  /**
   Absolute path of the candidate entry.
   */
  const dirPath = join(
    parentDir,
    name,
  );
  // Deliberate catch-and-log: an entry a concurrent writer or deleter already
  // moved is expected, and it only drops itself out of this pass.
  try {
    /**
     Entry metadata deciding whether the candidate is a managed directory.
     */
    const stats = await stat(dirPath,);
    if (!stats.isDirectory())
      return;
    managed.push({
      name,
      dirPath,
      mtimeMs: stats.mtimeMs,
      keeping: dirPath === keepDir,
    },);
  }
  catch (error: unknown) {
    logger.debug(`response temp pruning skipped unreadable entry ${dirPath}: ${caughtValueText(error,)}`,);
  }
}

/**
 Delete one owned response directory, logging failures without propagating them.

 Concurrent writers and concurrent pruning passes may remove the same entry, so
 removal is forced and every failure is only logged.

 @param dirPath - owned directory path to remove

 @param logger - logger already tagged for the pruning pass

 @example
 ```ts
 await deleteResponseDir({ dirPath, logger: innerL });
 ```
 */
async function deleteResponseDir(
  {
    dirPath,
    logger,
  }: {
    readonly dirPath: string;
    /**
     Logger already tagged for the pruning pass.
     */
    readonly logger: Logger;
  },
): Promise<void> {
  // Deliberate catch-and-log: deletion is best-effort, and one undeletable
  // directory must not stop the rest of the pass or fail the caller's call.
  try {
    await rm(
      dirPath,
      {
        recursive: true,
        force: true,
      },
    );
  }
  catch (error: unknown) {
    logger.warn(`failed to delete response temp dir ${dirPath}: ${caughtValueText(error,)}`,);
  }
}

//endregion Helpers
