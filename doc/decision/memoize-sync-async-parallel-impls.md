# Keep memoize sync and async as parallel implementations

## Context

`@monochromatic-dev/kv-store-memoize` ships two wrappers,
 `memoize` (`src/memoize.ts`)
and `memoizeAsync` (`src/memoize-async.ts`).
They share structure:
 destructuring the function and key derivation,
creating a default per-instance LRU store with a random `storeId`,
building the cache key,
 the get-then-compute-then-set core,
and attaching the cache-management surface to the returned callable.

An architecture review proposed extracting that shared assembly into one internal helper
parameterized over the store primitives,
 with sync and async as thin adapters.
We considered it and decided against merging;
 the two stay parallel.

The reasons a future explorer needs,
 so this is not re-suggested:

- It matches the deliberate split already in the substrate.
  `@monochromatic-dev/kv-store-core` maintains separate `Store` and `SyncStore`
  implementations rather than abstracting over both,
   and memoize sits directly on that contract.
  Mirroring the substrate's shape keeps the two layers aligned.
- Merging forces an abstraction over `Promisable` (the sync-or-async return),
  which kv-store intentionally avoids.
   The cost is a layer of conditional-async indirection
  in exchange for removing roughly thirty lines of duplication.
- The part that genuinely differs is the get-then-compute-then-set core plus the async
  in-flight deduplication (the in-flight `Map`,
   the `using`/`Disposable` cleanup,
  and the reject-clears-in-flight path).
   That is exactly the part that resists clean unification,
  so by the deletion test a shared assembler would concentrate very little.

The two variants share only the internal cache-key encoder (`src/cache-key.ts`)
and the types module (`src/types.ts`).

This decision is paired with the deepening plan at
`package/kv-store/memoize/PLAN.deepen-cache-key-seam.md`,
which keeps that shared encoder private and injective while leaving the two variants parallel.
