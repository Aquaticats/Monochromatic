# @monochromatic-dev/kv-store-core

Multi-backend key-value store with majority consensus,
 self-healing,
 LRU eviction,
 and
SuperJSON serialization.
 Synchronous and asynchronous variants.
 Runtime-neutral:
 works in
browsers,
 Node,
 Bun,
 Deno,
 and Workers.

## Concepts

A store wraps an ordered,
 non-empty list of backends.
 Each backend implements `get`,
 `set`,
and `delete` over string keys and string values,
 optionally exposing a numeric `priority`
tier and a `clear` method.
 `Map<string, string>` and the Web Storage API both satisfy the
shape.
 When no backends are supplied the store falls back to a single in-memory `Map`,
 unless
a platform default-backends builder has been registered.

Values are serialized with SuperJSON by default,
 so structured data (dates,
 maps,
 sets,
nested objects) round-trips through string backends.
 Circular graphs are decycled and stored
lossily when `lossyForCircular` is true (the default),
 or rejected with a `TypeError` when it
is false.

## Reads: consensus and healing

`get` queries every backend,
 then resolves a canonical value by majority vote:

- A majority across all tiers wins.
- On a cross-tier tie,
   the majority within the highest-priority tier wins.
- A tie within the highest tier throws.

After resolving,
 the store heals divergent backends:
 mismatched entries are rewritten to the
canonical value,
 and entries with no canonical value are deleted.

## Async store

```ts
import { createStore } from '@monochromatic-dev/kv-store-core';

const store = await createStore({
  storeId: 'cache',
  eviction: [{ policy: 'lru', maxSize: 256 }],
});

await store.set('user-1', { name: 'Ada' });
const user = await store.get<{ name: string }>('user-1');
await store.delete('user-1');
await store.clear();
```

Passing an empty key to the async `set` derives a stable key from the SHA-256 hash of the
serialized value,
 so content-addressed writes still address something deterministic.

## Sync store

```ts
import { createSyncStore } from '@monochromatic-dev/kv-store-core';

const store = createSyncStore({ storeId: 'sync-cache' });
store.set('answer', 42);
const answer = store.get<number>('answer');
const count = store.size; // entries in the primary backend, or 0
```

## Eviction

Pass one or more eviction policies to bound capacity.
 The LRU policy tracks access order and
drops the least recently used key once `maxSize` is exceeded.
 Reads and writes both refresh a
key's position.
 Without a policy the store grows unbounded.

```ts
const store = createSyncStore({
  storeId: 'bounded',
  eviction: [{ policy: 'lru', maxSize: 1024 }],
});
```

## Platform default backends

Register a builder once at module load to supply platform-specific backends (for example a
file or OPFS backend layered behind an in-memory `Map`).
 Stores created without an explicit
`backends` list then use it.

```ts
import { configureDefaultBackendsBuilder } from '@monochromatic-dev/kv-store-core';

configureDefaultBackendsBuilder(async function buildDefaults({ storeId }) {
  return [new Map<string, string>(), await createFileBackend(storeId)];
});
```

## Source escape hatch

The package ships built output for its `.` entry.
 Source modules are reachable through the
`./ts` and `./ts/*` exports for exceptional source-level imports.
