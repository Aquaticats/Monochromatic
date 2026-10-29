## module-p-memoize-fork

TypeScript fork of [`p-memoize`](https://github.com/sindresorhus/p-memoize),
 in-repo so its behavior is owned,
 tested,
 fuzzed,
 and mutation-tested here instead of trusted from a third-party package.

### Attribution

Derived from [`p-memoize`](https://github.com/sindresorhus/p-memoize) by
[Sindre Sorhus](https://sindresorhus.com),
 released upstream under the [MIT License](LICENSES/MIT.txt).
 Memoization semantics,
 validation rules,
 and error message texts come from `p-memoize` 8.0.0.
 The function-identity copying in `src/mimic-function.ts` is inlined from
 [`mimic-function`](https://github.com/sindresorhus/mimic-function) 5.0.1,
 same author and license.
 Both upstreams' copyright and permission notice are preserved verbatim in
`LICENSES/MIT.txt`;
 this fork's own code is licensed `LGPL-3.0-or-later AND MIT`
as declared in `package.json`.

### Usage

```ts
// package/upstream-fork/p-memoize/example.ts
import { pMemoize, } from '@monochromatic-dev/upstream-fork-p-memoize';

const memoized = pMemoize({
  fn: async function fetchUser(id: string): Promise<string> {
    return `user-${id}`;
  },
},);

const results = await Promise.all([
  memoized({ args: ['a'], },),
  memoized({ args: ['a'], },),
]);
```

### Behavior

#### Memoization

`pMemoize({ fn, options })` wraps one promise-returning function.
 Each call derives a cache key from its `args` tuple,
 answers hits from the
cache,
 and runs the wrapped function on misses.
 Only fulfilled values are cached;
 rejections are never written and the next call
recomputes.

#### Cache keys

`options.cacheKey(args)` derives the key and defaults to the first argument,
so primitives and identity-comparable references work out of the box.
 A
serializer like `JSON.stringify` keys by all arguments instead.

#### Cache storage

`options.cache` defaults to a fresh `Map` per memoized function and accepts
any storage implementing `has`,
 `get`,
 `set`,
 and `delete`,
 with optional
`clear`.
 All of `has`,
 `get`,
 and `set` may return promises, so asynchronous
storages work.
 `false` disables caching entirely: only concurrent calls with
one key share a result.

#### In-flight promises

Concurrent calls with one key share one promise object,
 so the wrapped
function runs once per key until that promise settles.
 The in-flight entry
is dropped the moment the call settles,
 whether it fulfilled or rejected.

#### shouldCache

`options.shouldCache(value, { key, argumentsList })` runs after the wrapped
function fulfills and before `cache.set`.
 Return `false` to skip the write
while still clearing in-flight de-duplication,
 or throw to propagate the
failure and skip caching.
 Reads are unaffected.

#### pMemoizeClear

`pMemoizeClear(memoized)` drops every cached value of one memoized function.
It throws `NotMemoizedError` for functions never memoized,
 `CacheDisabledError`
for functions created with `cache: false`,
 and `UnclearableCacheError` for
storages without `clear`.

#### pMemoizeDecorator

`pMemoizeDecorator(options)` decorates class methods under the new
ECMAScript decorators.
 Each instance gets its own memoized method as an own,
non-enumerable,
 writable,
 configurable property,
 so `pMemoizeClear(instance.method)`
clears one instance's cache.
 Private methods are not supported.

### Deviations from upstream p-memoize

#### Call shape

A memoized call passes an `args` tuple instead of a rest parameter
(`memoized({ args })`).
 Repository lint bans rest parameters and
multi-positional-parameter declarations outright.
 The wrapped function still
receives the tuple spread as real arguments,
 and `this` forwards through
unchanged,
 so prototype-method memoization behaves like upstream's.

#### Type shape

Options and results are typed on the argument tuple and result type
(`MemoizeOptions<TArgs, TResult, CacheKeyType>`,
 `MemoizedFunction<TArgs, TResult>`)
instead of upstream's function-type parameterization,
 matching this repo's
`p-limit-fork` idiom.
 Runtime behavior is unaffected.

#### Error types

Failures throw `NotMemoizedError`,
 `CacheDisabledError`,
 `UnclearableCacheError`,
`NonMethodDecorationError`,
 or `PrivateMethodDecorationError` instead of bare
`TypeError` values.
 All extend `TypeError` and carry upstream's message text
verbatim,
 so migrating callers see identical diagnostics and the fuzz
sidecar's differential oracle can compare both implementations directly.

#### Internal dependencies

Upstream `p-memoize` depends on `mimic-function` for function-identity
copying and `type-fest` for types.
 Both are gone: `mimic-function` is inlined
in `src/mimic-function.ts` (same MIT license,
 attribution above) and the
awaited return type is derived inline,
 leaving the package with zero runtime
dependencies and one less third-party trust surface on the memoization path.
 The wrapper's `name` descriptor is derived from the wrapper itself instead
of read from `Function.prototype.toString.name`;
 observably identical on
any conforming runtime.

#### Cleanup structure

Upstream's `try`/`finally` cleanup becomes `try`/`catch` with the in-flight
cleanup duplicated before each `return` (repository lint bans `try`/`finally`).
 The duplicate placements preserve upstream's exact ordering: cleanup runs
before each settlement and inside the same synchronous stretch when the
wrapped function throws synchronously.

### Retained upstream behavior

Upstream `p-memoize` registers a call's in-flight promise after the call
body starts.
 When the wrapped function throws synchronously with caching
disabled (`cache: false`),
 the cleanup runs before the registration lands, so
the rejected promise stays in the in-flight map and later calls with the
same key replay that rejection instead of recomputing.
 The fork reproduces
this ordering exactly;
 the differential oracle pins it.

### Layout

`p-memoize.ts` owns memoization,
 `memoized-cache-store.ts` the memoized
function registry,
 `cache-storage.ts` the cache contracts,
 `mimic-function.ts`
the inlined identity copy,
 `p-memoize-decorator.ts` the class-method
decorator,
 `p-memoize-clear.ts` cache clearing,
 and `errors.ts` the error
classes.
 Test files sit beside each module as `<stem>.unit.test.ts`,
 so
mutation testing selects each module's tests automatically.

### Testing

```bash
# package/upstream-fork/p-memoize/mise.toml

# Unit tests against the built dist
mise run //package/upstream-fork/p-memoize:buildAndTest

# Property-based fuzz campaign (invariants plus an upstream differential oracle)
mise run //package/upstream-fork/p-memoize.fuzz:fuzz

# Coverage-reachability gate over this package's src
mise run //package/upstream-fork/p-memoize.fuzz:fuzz:coverage

# Container-isolated mutation testing
mise run //package/upstream-fork/p-memoize:test:mutation
```
