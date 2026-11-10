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

## Keys

A key is any non-empty string.
 An empty key is refused: `set`,
 `get`,
 and `delete` throw
`EmptyKeyError` before any backend is read or written.
 An empty key addresses nothing a later
call could name,
 so both variants reject it rather than deriving or storing an
unaddressable entry.

## Sync and async variants

The two variants are hand-written twins,
 not one core with two adapters.
 `create-store.ts` and `create-sync-store.ts` mirror each other step for step,
 and so do the backend query and healing pairs beside them.
 Merging a pair would need an abstraction over the sync-or-async return,
 which this package deliberately avoids;
 `doc/decision/memoize-sync-async-parallel-impls.md` records that choice.
 The parts both variants do share are explicit:
 `consensus.ts`,
 `serialize.ts`,
 and `lru-key-set.ts`.
 Because of the split,
 a behaviour one variant gains alone is a defect unless a decision says
 otherwise,
 and `store-pair-parity.unit.test.ts` holds the decisions both sides must
 make identically.

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
