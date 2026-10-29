/**
 Memoization for sync and async functions, backed by `@monochromatic-dev/kv-store-core`.
 
 Each memoized function derives a cache key from a required `keyFn` plus a per-call salt,
 persists results in a SyncStore (sync) or Store (async) with LRU eviction, and exposes
 `.store`, `.clear()`, and `.delete()` for cache management. The async variant additionally
 deduplicates concurrent in-flight calls onto a single computation. A stored `undefined` is
 treated as a miss, so `undefined`-returning functions recompute on every call.
 
 The cache-key builder is an internal helper, not a public export.
 
 @example
 Sync:
 ```ts
 import { memoize } from '\@monochromatic-dev/kv-store-memoize';
 
 const memoizedAdd = memoize({ fn: (a: number, b: number) => a + b, keyFn: (a, b) => `${String(a)}:${String(b)}` });
 memoizedAdd({ args: [1, 2], salt: 'v1' });
 ```
 
 @example
 Async:
 ```ts
 import { memoizeAsync } from '\@monochromatic-dev/kv-store-memoize';
 
 const memoized = await memoizeAsync({ fn: fetchUser, keyFn: (id) => id });
 await memoized({ args: ['user-1'], salt: 'v1' });
 ```
 
 @packageDocumentation
 */

export { DEFAULT_MAX_CACHE_SIZE, } from './types.ts';

export type {
  MemoizeAsyncNamedOptions,
  MemoizeAsyncOptions,
  MemoizedAsyncFunction,
  MemoizedCallOptions,
  MemoizedFunction,
  MemoizeNamedOptions,
  MemoizeOptions,
} from './types.ts';

export { memoize, } from './memoize.ts';

export { memoizeAsync, } from './memoize-async.ts';
