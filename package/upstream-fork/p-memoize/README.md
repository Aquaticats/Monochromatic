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
 and error message texts follow `p-memoize` 9.0.0
(see "Tracking upstream 9.0.0"),
 which the fork was first derived from at 8.0.0.
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
 and `set`,
 with optional
`delete` and `clear`.
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

`pMemoizeClear(memoized)` drops every cached value of one memoized function,
 and the in-flight calls of that function (see "Tracking upstream 9.0.0").
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
 Private methods are not supported,
 and a subclass override is left alone.

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
 The duplicate placements preserve upstream's ordering: cleanup runs
before each settlement.

### Tracking upstream 9.0.0

The fork tracks `p-memoize` 9.0.0.
 The behavior below changed between 8.0.0 and 9.0.0 and is followed here;
 the differential oracle in the fuzz sidecar compares the fork with whatever
upstream release the catalog entry installs.

#### Synchronous throw with caching disabled

Upstream 8.0.0 registered a call's in-flight promise after the call body
started.
 When the wrapped function threw synchronously with caching disabled
(`cache: false`),
 the cleanup ran before the registration landed,
 so the rejected promise stayed in the in-flight map and later calls with the
same key replayed that rejection.
 Upstream 9.0.0 (commit `a8cec6126`, issues 55 and 62) yields one microtask
before running the wrapped function when caching is disabled,
 so registration comes first and a later call recomputes.

#### Clearing and calls in flight

`pMemoizeClear` bumps a generation counter shared by every memoized function
over one cache,
 so a call that was running at clear time does not write its result back and
a `has` hit followed by a clear before `get` counts as a miss.
 The clear also drops the cleared function's own in-flight promises,
 so a later call starts fresh;
 other functions over the same cache keep their in-flight calls.
 A `cache.set` that already started is not undone.
 A `clear()` that returns a promise finishes after `pMemoizeClear` returns,
 and a rejection from it is dropped.
 A `clear()` that throws synchronously still propagates,
 after the bookkeeping ran.

#### Messages and options

`UnclearableCacheError` carries upstream 9.0.0's text
`The cache can't be cleared!` (8.0.0 said `The cache Map can't be cleared!`).
 `shouldCache` is read once when the function is memoized,
 like `cache` and `cacheKey`.
 Only `cache: false` disables caching.
 `CacheStorage.delete` is optional because the memoizer never calls it.

#### Keys

`cacheKey` receives a copy of the argument tuple,
 so a key built by sorting it in place cannot reorder what the wrapped
function receives.
 A `cacheKey` that throws makes the call reject instead of throwing
synchronously,
 and a `cacheKey` typed to return a promise is rejected by the type.

#### Re-entrant calls and failing storages

A call re-entered for the same key from `cache.has`,
 `cache.get`,
 or the wrapped function joins the running call instead of running again.
 A `cache.has` that throws synchronously rejects that call without leaving a
stuck in-flight entry,
 so the next call retries.

#### Function-identity copy

Copying the wrapped function's name,
 properties,
 and source into the memoized function is cosmetic,
 and a function that makes it hard
(a non-configurable own `toString`,
 a `toString` that throws,
 a proxy whose `ownKeys` throws)
is still memoized with whatever the copy reached.

#### Decorator and subclasses

`pMemoizeDecorator` asks where the decorated method is defined
(the prototype for an instance method,
 the class for a static one),
 by property descriptor so no subclass getter runs,
 and stands down when that is not the decorated function
(a subclass override,
 a method deleted from the chain,
 an accessor that replaced it).
 Defining the memoized copy on the instance would otherwise shadow the
override.
 A `super.method()` call from an override reaches the original,
 unmemoized method.

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
