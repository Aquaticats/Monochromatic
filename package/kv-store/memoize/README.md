# @monochromatic-dev/kv-store-memoize

Memoization for synchronous and asynchronous functions,
 backed by
`@monochromatic-dev/kv-store-core`.
 Each memoized function derives a cache key from a
required `keyFn` plus a per-call salt,
 persists results in a store with LRU eviction,
 and
exposes cache-management methods.
 Runtime-neutral.

## Concepts

A memoized function is created from a `fn` to wrap and a `keyFn` that derives a deterministic
string key from the arguments.
 `keyFn` is required because hashing variadic arguments
automatically is error-prone;
 the caller owns key derivation.
 Each call also supplies a
`salt`,
 appended to the key as `${argKey}:${salt}`,
 so the cache can be invalidated by
changing the salt without recreating the function (for example `String(time % 3600000)` for
hourly expiry).

By default a fresh in-memory store is created with LRU eviction at `DEFAULT_MAX_CACHE_SIZE`
(1024 entries).
 Pass a `store` to share a cache,
 change the eviction bound,
 or use a
non-default backend.

Only pure functions should be memoized.
 A stored value of `undefined` is indistinguishable
from a cache miss,
 so functions returning (or resolving to) `undefined` recompute on every
call.

## Sync

```ts
import { memoize } from '@monochromatic-dev/kv-store-memoize';

const memoizedAdd = memoize({
  fn: (a: number, b: number) => a + b,
  keyFn: (a, b) => `${String(a)}:${String(b)}`,
});

memoizedAdd({ args: [1, 2], salt: 'v1' }); // computed: 3
memoizedAdd({ args: [1, 2], salt: 'v1' }); // cached: 3

memoizedAdd.size;            // number of cached entries
memoizedAdd.store;           // underlying SyncStore
memoizedAdd.delete('1:v1');  // drop one entry by full cache key
memoizedAdd.clear();         // drop all entries
```

If `fn` throws,
 the throw propagates and nothing is cached for that key.

## Async

```ts
import { memoizeAsync } from '@monochromatic-dev/kv-store-memoize';

const memoized = await memoizeAsync({ fn: fetchUser, keyFn: (id) => id });

await memoized({ args: ['user-1'], salt: 'v1' }); // fetched
await memoized({ args: ['user-1'], salt: 'v1' }); // cached

await memoized.delete('user-1:v1');
await memoized.clear();
```

Concurrent calls with the same key share a single in-flight promise,
 so `fn` runs once even
under a burst of simultaneous calls.
 If `fn` rejects,
 the rejection propagates,
 the in-flight
entry is cleared,
 and nothing is cached,
 so the next call recomputes.
 The async variant has no
`.size` because its backing store is asynchronous.

## Custom store

```ts
import { memoize } from '@monochromatic-dev/kv-store-memoize';
import { createSyncStore } from '@monochromatic-dev/kv-store-core';

const store = createSyncStore({
  storeId: 'my-memo',
  eviction: [{ policy: 'lru', maxSize: 256 }],
});

const memoized = memoize({ fn: compute, keyFn: String, store });
```

## Source escape hatch

The package ships built output for its `.` entry.
 Source modules are reachable through the
`./ts` and `./ts/*` exports for exceptional source-level imports.
