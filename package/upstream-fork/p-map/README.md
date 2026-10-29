## module-p-map-fork

TypeScript fork of [`p-map`](https://github.com/sindresorhus/p-map),
 in-repo so its behavior is owned,
 tested,
 fuzzed,
 and mutation-tested here instead of trusted from a third-party package.

### Attribution

Derived from [`p-map`](https://github.com/sindresorhus/p-map) by
[Sindre Sorhus](https://sindresorhus.com),
 released upstream under the [MIT License](LICENSES/MIT.txt).
Iteration semantics,
 validation rules,
 and error message texts come from `p-map` 7.1.1.
Upstream's copyright and permission notice are preserved verbatim in
`LICENSES/MIT.txt`;
 this fork's own code is licensed `LGPL-3.0-or-later AND MIT`
as declared in `package.json`.

### Usage

```ts
// package/upstream-fork/p-map/example.ts
import { pMap, } from '@monochromatic-dev/upstream-fork-p-map';

const results = await pMap({
  iterable: [
    'a',
    'b',
  ],
  mapper: async function fetchUser(id: string): Promise<string> {
    return `user-${id}`;
  },
  options: { concurrency: 2, },
});
```

### Behavior

#### Scheduling

`pMap({ iterable, mapper, options })` walks a synchronous or asynchronous
input,
 awaits each iterated element,
 and runs its mapper under the
`concurrency` bound.
At most `concurrency` mapper calls run at once;
`Number.POSITIVE_INFINITY` removes the cap.
Results collect in input order regardless of completion order.

#### Results and skips

The run resolves with every mapper result in input order.
A mapper returning `pMapSkip` drops that input's slot from the collected
results.

#### Failures

With `stopOnError: true` (the default),
 the first mapper failure rejects the
run with that failure.
With `stopOnError: false`,
 every failure is collected and the
run rejects with an `AggregateError` of them once the input is exhausted.
An input that throws mid-iteration rejects the run with that throw either
way,
 because an iterable is likely to keep throwing after its first failure.
The `AggregateError` lists the failures in input order, not in the order they
happened.

#### Source protocol

Each pull takes its position in the input when the `next()` call is made, so
a source that settles its calls out of call order still gets the right
positions.
Once the source reports `done` it is never pulled again, and anything it
produces afterwards is dropped, like `for await`.
A `null` or `undefined` input fails with the same `InvalidInputError` as any
other wrong input.

#### Abort signal

Passing `options.signal` aborts the run:
 an already-aborted signal rejects
before the first pull,
 and a later abort rejects with the signal's reason and
closes the source.

#### pMapIterable

`pMapIterable({ iterable, mapper, options })` returns an async iterable that
streams mapper results in input order under `concurrency` and a
`backpressure` bound.
A mapper failure surfaces as a throw from the async iterator at its own
position: the results before it are still yielded first.
Leaving the iteration early (a `break`, or `return()` or `throw()` on the
iterator) stops further pulls and mapping at once and closes the source.
When `backpressure` is above `concurrency`, the next element is read while the
mappers run.
Skipped results count toward `backpressure` until the consumer steps over them.

### Deviations from upstream p-map

#### Call shape

One destructured object replaces upstream's three positional parameters
(`pMap(iterable, mapper, options)`),
 with `options` keeping its shape and
destructuring failure semantics.
Repository lint bans multi-positional-parameter
declarations outright.
A non-object call argument therefore fails at the call
destructuring,
 where upstream has no such argument to destructure.

#### Error types

Invalid configuration throws `InvalidInputError`, `MapperRequiredError`,
`InvalidConcurrencyError`, or `InvalidBackpressureError`
 instead of a bare
`TypeError`.
All extend `TypeError` and carry upstream's message text verbatim
 (including the value and `typeof` interpolations and the symbol-interpolation
failure),
 so migrating callers see identical diagnostics and the fuzz sidecar's
differential oracle compares both implementations directly.

#### Skip sentinel

The fork mints its own `pMapSkip` symbol.
A fork mapper and an upstream mapper
each return their own implementation's sentinel;
the two are never
interchangeable.

#### Close failures

Upstream `p-map` swallows source-close failures outright.
The fork logs them
through `@monochromatic-dev/logger-core` instead of discarding them,
 then
continues the same way.
Repository rules forbid silent catch blocks.

#### Not ported from p-map 7.1

`pMapSettled`, `pMapKeyed`, `pMapSettledKeyed`, `pMapStop`, and the
`preserveOrder` option of `pMapIterable` are not part of this fork.
No package in this repository uses them, and the differential oracle drives
only `pMap` and `pMapIterable` without them.
Upstream's link that keeps the caller in the async stack trace of a mapper
error is not ported either: a trace detail that no result or error message
depends on.

#### Exports

The entry point exports named symbols only
 (`pMap`, `pMapIterable`,
`pMapSkip`, the error classes, and the option and mapper types);
there is no
default export.

### Layout

`p-map.ts` owns the concurrent map,
 `p-map-iterable.ts` the streaming
variant,
 `map-options.ts` configuration,
 `map-iterator.ts` iterator selection
and shutdown,
 `mapper.ts` the input and mapper contracts,
 `p-map-skip.ts` the
skip sentinel,
 and `errors.ts` the error classes.
Test files sit beside each
module as `<stem>.unit.test.ts`,
 so mutation testing selects each module's
tests automatically.

### Testing

```bash
# package/upstream-fork/p-map/mise.toml

# Unit tests against the built dist
mise run //package/upstream-fork/p-map:buildAndTest

# Property-based fuzz campaign (invariants plus an upstream differential oracle)
mise run //package/upstream-fork/p-map.fuzz:fuzz

# Coverage-reachability gate over this package's src
mise run //package/upstream-fork/p-map.fuzz:fuzz:coverage

# Container-isolated mutation testing
mise run //package/upstream-fork/p-map:test:mutation
```
