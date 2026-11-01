## module-quick-lru-fork

TypeScript fork of [`quick-lru`](https://github.com/sindresorhus/quick-lru),
 in-repo so its cache behavior is owned,
 tested,
 fuzzed,
 and mutation-tested here instead of trusted from a third-party package.

### Attribution

Derived from [`quick-lru`](https://github.com/sindresorhus/quick-lru) by
[Sindre Sorhus](https://sindresorhus.com),
 released upstream under the [MIT License](LICENSES/MIT.txt).
Cache semantics,
 validation rules,
 coercion quirks,
 and error message texts come from `quick-lru` 7.3.0.
Upstream's copyright and permission notice are preserved verbatim in
`LICENSES/MIT.txt`;
 this fork's own code is licensed `LGPL-3.0-or-later AND MIT`
as declared in `package.json`.

### Usage

```ts
// package/upstream-fork/quick-lru/example.ts
import { createQuickLru, } from '@monochromatic-dev/upstream-fork-quick-lru';

const lru = createQuickLru<string, string>({ maxSize: 1000, },);

lru.set({
  key: '🦄',
  value: '🌈',
},);

lru.has('🦄',); // => true
lru.get('🦄',); // => '🌈'
```

### Behavior

#### Dual-cache algorithm

The cache keeps two maps:
 a recent map for writes and reads that just hit,
 and an old map for everything else.
When the recent map fills to `maxSize`,
 it becomes the old map and a fresh recent map starts,
 so writes never pay per-item deletes.
The cache may hold between `maxSize` and `2 × maxSize` items for this
performance,
 while `size` reports at most `maxSize`.

#### Recency

`get` promotes an old-map item into the recent map;
`peek`,
 `has`,
 and `expiresIn` never change recency.
Setting a key that already sits in the recent map updates it in place,
matching upstream,
 while setting an old-map key re-inserts it as most recently used.

#### Lazy expiry

`maxAge` bounds item lifetime globally,
 and `set({ key, value, maxAge })` overrides it per item.
Expiry is applied lazily on read:
 `get`,
 `peek`,
 `has`,
 and iteration remove expired items when they touch them,
 while `expiresIn` reports remaining time without removing anything.

#### Evictions

`onEviction` fires right before an item leaves through LRU rollover
 pressure,
 TTL expiry,
 or a manual `evict` call.
It never fires for `delete` or `clear`.
When a rollover drops stale duplicate copies whose keys survive,
 those copies still notify,
 matching upstream exactly.

#### resize and evict

`resize(newSize)` updates the bound in place,
 growing keeps every item,
 shrinking evicts the oldest ones with notifications.
`evict(count)` evicts the least recently used items with notifications
 and always keeps at least one.

#### Iteration

`entriesAscending` and `entries` walk oldest first,
 `entriesDescending` walks newest first,
 and the default iterator,
 `keys`,
 and `values` walk the recent map before the old map.
Old-map keys duplicated in the recent map appear once,
 on the recent map's side.

### Deviations from upstream quick-lru

#### Factory instead of Map subclass

Upstream is a class extending `Map`.
Repository lint bans classes,
 so `createQuickLru(options)` returns a factory-built object carrying the
same members with upstream's class-prototype descriptor flags pinned.
It is not a `Map` instance;
`Map`-typed consumers need an adapter.

#### Call shape

`set` takes a destructured options object (`set({ key, value, maxAge })`)
 instead of three positional parameters,
 and `forEach({ callback, thisArgument })` takes one destructured options
object.
Repository lint bans multi-positional-parameter declarations outright.

#### Error types

Invalid configuration throws `InvalidMaxSizeError` or `InvalidMaxAgeError`
 instead of a bare `TypeError`.
Both extend `TypeError` and carry upstream's message text verbatim,
 so
migrating callers see identical diagnostics and the fuzz sidecar's
differential oracle can compare both implementations directly.

#### Absence stays undefined

`get`,
 `peek`,
 and `expiresIn` return `T | undefined` exactly as upstream declares them,
 carved out of the repository's no-nullish-union ban as genuine external
API mirror sites.
Stored `undefined` values stay indistinguishable from absence through
these members,
 as upstream.

### Layout

`quick-lru.ts` owns the cache factory and every member,
 `quick-lru-types.ts` the API surface types,
 `quick-lru-item.ts` the
stored-item model and its expiry-stamp guard,
 `quick-lru-iteration.ts` the
six dual-cache orderings,
 `quick-lru-members.ts` the descriptor-pinned
member attachment,
 `quick-lru-options.ts` configuration validation and
resolution,
 and `errors.ts` the error classes.
`test-support.ts` holds the
test-only fake clock.
Every runtime module has a stem-paired `<stem>.unit.test.ts`
 beside it,
 with `quick-lru.<feature>.unit.test.ts` sidecars for expiry,
eviction,
 and resize,
 so mutation testing selects each module's tests
automatically.

### Testing

```bash
# package/upstream-fork/quick-lru/mise.toml

# Unit tests against the built dist
mise run //package/upstream-fork/quick-lru:buildAndTest

# Property-based fuzz campaign (invariants plus an upstream differential oracle)
mise run //package/upstream-fork/quick-lru.fuzz:fuzz

# Coverage-reachability gate over this package's src
mise run //package/upstream-fork/quick-lru.fuzz:fuzz:coverage

# Container-isolated mutation testing
mise run //package/upstream-fork/quick-lru:test:mutation
```
