# Deepen the memoize cache-key seam

Plan for turning the shallow,
 leaky cache-key handling into a deep,
 private seam,
correcting the stale `undefined` documentation,
 and simplifying the one real consumer.
Produced by an architecture grilling session;
 every decision below is locked.

The companion decision `keep memoize sync and async as parallel implementations`
lives at `doc/decision/memoize-sync-async-parallel-impls.md`.

## Why

The cache key is built as `` `${argKey}:${salt}` `` in `src/cache-key.ts`.
That builder is a shallow pass-through (one template literal in its own module,
 with no test),
and the format it produces leaks out of the module two ways:
`.delete(fullKey)` forces callers to reconstruct it,
 and `.store` exposes the raw keyed store.
The bare colon join is not injective:
 `("user:1", "v2")` and `("user", "1:v2")` both yield `"user:1:v2"`,
so distinct argument-key and salt pairs can collide.
This is latent today only because the single consumer keys on `''`.
Separately,
 the package-level docs claim a stored `undefined` recomputes on every call,
while the code,
 the function-level docs,
 and the passing tests all cache it.

The only consumer (`package/rss/app/src/index.ts`)
never touches `.delete`,
 `.clear`,
 `.store`,
 or `.size`,
so the entire cache-management surface has a test-only blast radius.

## Locked decisions

- Make the composite cache-key string fully private to the module;
   it must never appear in the interface.
- Encode it injectively in `(argKey, salt)`,
   with adversarial delimiter tests (the repo SYB and STB rules).
- Keep sync and async as parallel implementations;
   do not merge them (see the decision doc).
- Adopt an argless default:
   `keyFn` and per-call `args` become optional only for argless functions.
- Lean management surface:
   keep `clear()` and (sync) `size`;
   drop the `.store` output and the per-entry `delete`.
- Preserve the `ABSENT` miss semantics,
   so a stored `undefined` stays cached.

## Phase 1: injective encoder, kept internal

- [ ] Rewrite `buildCacheKey` to a length-prefixed form:
       `` `${String(argKey.length)}:${argKey}${salt}` ``.
- [ ] Confirm injectivity:
       the prefix is decimal digits then a colon,
       so the first colon always marks the
      `argKey` boundary and `argKey` is recovered by its declared length,
       leaving the remainder as `salt`.
      The split is unambiguous even when `argKey` or `salt` contains colons,
       newlines,
       or leading digits.
- [ ] Keep it regex-free and a single linear construction;
       never export it.
- [ ] Note:
       the encoded key is non-empty even when `argKey` is `''` (`"0:" + salt`),
      so the kv-store empty-key content-hash path never fires and salt invalidation stays exact.

## Phase 2: argless default in the types

- [ ] Gate `keyFn` (in the factory options) and `args` (in `MemoizedCallOptions`) on the empty tuple.
- [ ] Use `TArgs extends readonly []` for the optional branch,
       not `[] extends TArgs`.
      The precise form makes `keyFn`/`args` optional only for an exactly-empty tuple;
      a function with arguments,
       including a variadic `(...xs: T[])` whose `TArgs` is `T[]`,
      still requires `keyFn`,
       so the all-collide footgun of a defaulted `() => ''` stays a compile error.
- [ ] Default `keyFn` to `() => ''` and `args` to `[]` internally when the empty-tuple branch is taken.

```ts
type KeyFnField<TArgs extends readonly unknown[]> = TArgs extends readonly []
  ? { readonly keyFn?: (this: void, ...args: TArgs) => string }
  : { readonly keyFn: (this: void, ...args: TArgs) => string };

type MemoizedCallOptions<TArgs extends readonly unknown[]> =
  & { readonly salt: string }
  & (TArgs extends readonly [] ? { readonly args?: TArgs } : { readonly args: TArgs });
```

## Phase 3: lean management surface

- [ ] Sync `MemoizedFunction`:
       the callable plus `clear(): void` and `size: number`.
- [ ] Async `MemoizedAsyncFunction`:
       the callable plus `clear(): Promise<void>` that clears the store
      and the in-flight map;
       no `size` (the async store is asynchronous).
- [ ] Remove the `.store` output property and the key-string `.delete` from both variants.
- [ ] Keep `store?` as a construction input (the local-substitutable seam);
       introduce no port.
- [ ] Internalize `DEFAULT_MAX_CACHE_SIZE` (no external importer;
       confirm with a fresh search before removing the export).
- [ ] Preserve the async in-flight dedup,
       its `using`/`Disposable` cleanup,
       and the reject-clears-in-flight path.

## Phase 4: correct the undefined docs (candidate 2)

- [ ] In `src/index.ts` `@packageDocumentation`,
       state that a stored `undefined` is cached and
      `ABSENT` is the only miss signal.
- [ ] Make the same correction in `README.md`;
       drop the source-escape-hatch and management-surface
      sections that no longer match the interface.

## Phase 5: simplify the consumer (rss)

- [ ] In `package/rss/app/src/index.ts`,
       drop `keyFn: () => ''` from both
      `memoizeAsync` calls and `args: []` from the call sites,
       relying on the argless default.
- [ ] Verify rss still builds and serves (see verification below).

## Tests

The interface is the test surface;
 with the key format private,
 injectivity is observed through behavior,
 not by reading keys.

- [ ] Replace assertions that read the key format (`store.get('1:v1')`,
       `delete('1:v1')`,
       `.store`)
      with behavioral assertions through the surviving surface.
- [ ] Add an STB adversarial collision suite:
       argument keys and salts containing colons,
       leading digits,
      newlines,
       empty strings,
       and source-escaped variants must produce distinct cached results
      (the pairs that collide under the old colon join must not collide now).
- [ ] Add a fast-check property:
       distinct `(argKey, salt)` pairs produce distinct keys
      (the repo already uses fast-check;
       see `doc/decision/fast-check.md`).
- [ ] Keep the behavioral cases:
       cache hit,
       salt-change recompute,
       miss versus stored `undefined`,
      async dedup via `Promise.all`,
       async reject clears in-flight then recomputes,
      LRU eviction observed as a recompute,
       and `clear()`/`size`.

## Verification at the boundary

- [ ] `mise run //package/kv-store/memoize:lint:types`
- [ ] `mise run //package/kv-store/memoize:lint`
- [ ] `mise run //package/kv-store/memoize:test:unit`
- [ ] `mise run //package/rss/app:lint:types`
- [ ] Run the rss server and confirm it serves the index page without errors,
      exercising the memoized pipeline through a real request rather than only compiling it.

## Deferred, not in scope

Recorded as leads,
 not built now:

- An `onEvent` observability hook (`hit`/`miss`/`join`):
   add it when a second consumer actually wants metrics.
- Predicate or salt-prefix invalidation,
   richer introspection,
   and an `on-evict` hook:
  all blocked until the kv-store `Store`/`SyncStore` contracts gain enumeration and an eviction callback.
- A caller-supplied low-level key encoder:
   it would re-leak the private format;
   `keyFn` plus `salt`
  is already the key-customization seam at the right altitude.
- A dedicated `MemoizeKeyFnError`:
   the type system already enforces a string return from `keyFn`.
