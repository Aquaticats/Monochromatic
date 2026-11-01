import { randomUUID, } from 'node:crypto';
import type { Stats, } from 'node:fs';
import {
  readFile,
  rename,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { validateNetwork, } from './asn-network.ts';

/**
 Sentinel representing cache absence or expiration.
 */
export const CACHE_ABSENT: unique symbol = Symbol('ASN cache is absent or expired',);

/**
 Type of {@link CACHE_ABSENT}.
 */
export type CacheAbsent = typeof CACHE_ABSENT;

/**
 Sentinel representing filesystem path absence.
 */
const PATH_ABSENT = Symbol('filesystem path is absent',);

/**
 Narrows unknown caught value to Node filesystem error shape.
 
 @param error - Caught value.
 
 @returns Whether value carries Node error code.
 
 @example
 ```ts
 isErrnoException({ code: 'ENOENT' }); // true
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error);
}

/**
 Stats path while representing absence explicitly.
 
 @param path - Path to inspect.
 
 @returns File metadata or absence sentinel.
 
 @throws When stat fails for reason other than absence.
 
 @example
 ```ts
 await statIfExists('/tmp/cache_AS64500.txt');
 ```
 */
async function statIfExists(path: string,): Promise<Stats | typeof PATH_ABSENT> {
  try {
    return await stat(path,);
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return PATH_ABSENT;
    throw error;
  }
}

/**
 Reads UTF-8 path while representing absence explicitly.
 
 @param path - Path to read.
 
 @returns File text or absence sentinel.
 
 @throws When read fails for reason other than absence.
 
 @example
 ```ts
 await readTextIfExists('/tmp/cache_AS64500.txt');
 ```
 */
async function readTextIfExists(path: string,): Promise<string | typeof PATH_ABSENT> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return PATH_ABSENT;
    throw error;
  }
}

/**
 Converts comma-separated cache text into validated network entries.
 
 @param text - Cache contents.
 
 @param targetAsn - ASN named when entry is invalid.
 
 @returns Validated network strings preserving database order.
 
 @example
 ```ts
 parseCachedNetworks({ text: '192.0.2.0/24', targetAsn: 'AS64500' });
 ```
 */
function parseCachedNetworks(
  {
    text,
    targetAsn,
  }: {
    readonly text: string;
    readonly targetAsn: string;
  },
): readonly string[] {
  return text
    .split(',',)
    .map(function trimNetwork(network: string,): string {
      return network.trim();
    },)
    .filter(function isPresent(network: string,): boolean {
      return network !== '';
    },)
    .map(function validateCachedNetwork(network: string,): string {
      return validateNetwork({
        network,
        targetAsn,
      },);
    },);
}

/**
 Reads and validates ASN cache when present and fresh enough.
 
 @param cachePath - Per-ASN cache path.
 
 @param targetAsn - ASN named during validation.
 
 @param earliestMtimeMs - Oldest accepted modification time, or absent to accept stale cache.
 
 @returns Validated cache entries or explicit absence.
 
 @example
 ```ts
 await readAsnCache({
   cachePath: '/tmp/cache_AS64500.txt',
   targetAsn: 'AS64500',
 });
 ```
 */
export async function readAsnCache(
  {
    cachePath,
    targetAsn,
    earliestMtimeMs,
  }: {
    readonly cachePath: string;
    readonly targetAsn: string;
    readonly earliestMtimeMs?: number;
  },
): Promise<readonly string[] | CacheAbsent> {
  /**
   Cache metadata used for presence and freshness checks.
   */
  const stats = await statIfExists(cachePath,);
  if (stats === PATH_ABSENT)
    return CACHE_ABSENT;
  if ((earliestMtimeMs !== undefined) && (stats.mtimeMs < earliestMtimeMs))
    return CACHE_ABSENT;
  /**
   Cache text read after metadata check.
   */
  const text = await readTextIfExists(cachePath,);
  if (text === PATH_ABSENT)
    return CACHE_ABSENT;
  return parseCachedNetworks({
    text,
    targetAsn,
  },);
}

/**
 Replaces ASN cache through same-directory temporary path.
 
 @param cacheDirectory - Directory shared by temporary and final paths.
 
 @param cachePath - Final cache path.
 
 @param targetAsn - ASN used in temporary filename.
 
 @param text - Complete comma-separated database result.
 
 @example
 ```ts
 await writeAsnCache({
   cacheDirectory: '/tmp',
   cachePath: '/tmp/cache_AS64500.txt',
   targetAsn: 'AS64500',
   text: '192.0.2.0/24',
 });
 ```
 */
export async function writeAsnCache(
  {
    cacheDirectory,
    cachePath,
    targetAsn,
    text,
  }: {
    readonly cacheDirectory: string;
    readonly cachePath: string;
    readonly targetAsn: string;
    readonly text: string;
  },
): Promise<void> {
  /**
   Unique same-directory path preserving atomic rename semantics.
   */
  const temporaryPath = join(
    cacheDirectory,
    `cache_${targetAsn}.${randomUUID()}.txt`,
  );
  /**
   Cleanup guard removing temporary path after rename or failed write.
   */
  await using temporaryFile = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        temporaryPath,
        { force: true, },
      );
    },
  };
  await writeFile(
    temporaryPath,
    text,
    { flag: 'wx', },
  );
  await rename(
    temporaryPath,
    cachePath,
  );
}
