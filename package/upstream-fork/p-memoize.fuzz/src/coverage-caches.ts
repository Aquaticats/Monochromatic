/**
 Cache storage fixtures for the deterministic coverage driver: a recording
 cache, a cache that reports a hit while reading nothing, and a cache whose
 asynchronous `clear` rejects.
 
 @module
 */

import type {
  CacheStorage,
} from '@monochromatic-dev/upstream-fork-p-memoize/ts';

/**
 Records cache method calls in order.
 */
export const cacheLog: string[] = [];

/**
 Backing store behind the recording cache.
 */
export const cacheEntries: Map<string, number> = new Map<string, number>();

/**
 Async cache storage recording `has`, `get`, `set`, `delete`, and `clear`.
 */
export const recordingCache: CacheStorage<string, number> & {
  readonly clear: () => unknown;
} = {
  has: function has(key: string,): Promise<boolean> {
    cacheLog.push(`has ${key}`,);
    return Promise.resolve(cacheEntries.has(key,),);
  },
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors `CacheStorage.get` documented cache-miss `undefined` value
  get: function get(key: string,): Promise<number | undefined> {
    cacheLog.push(`get ${key}`,);
    return Promise.resolve(cacheEntries.get(key,),);
  },
  set: function set(
    key: string,
    value: number,
  ): Promise<void> {
    cacheLog.push(`set ${key}`,);
    cacheEntries.set(
      key,
      value,
    );
    return Promise.resolve();
  },
  delete: function remove(key: string,): boolean {
    cacheLog.push(`delete ${key}`,);
    return cacheEntries.delete(key,);
  },
  clear: function clear(): void {
    cacheLog.push('clear',);
    cacheEntries.clear();
  },
};

/**
 Cache reporting every key present while reading `undefined`, exercising
 the has-hit get-miss path.
 */
export const alwaysHitCache: CacheStorage<string, number> = {
  has: function has(_key: string,): boolean {
    return true;
  },
  get: function get(_key: string,): undefined {
    return undefined;
  },
  set: function set(
    _key: string,
    _value: number,
  ): void {
    cacheLog.push('set unexpected',);
  },
  delete: function remove(_key: string,): boolean {
    return false;
  },
};

/**
 Cache whose `clear` returns a rejecting promise, exercising the
 asynchronous-clear path of `pMemoizeClear`.
 */
export const asyncClearCache: CacheStorage<string, number> = {
  has: function has(_key: string,): boolean {
    return false;
  },
  get: function get(_key: string,): undefined {
    return undefined;
  },
  set: function set(
    _key: string,
    _value: number,
  ): void {
    cacheLog.push('set asyncClear',);
  },
  clear: async function clear(): Promise<void> {
    await Promise.resolve();
    throw new Error('asynchronous clear failure',);
  },
};
