## module-p-limit-fork

TypeScript fork of [`p-limit`](https://github.com/sindresorhus/p-limit),
 in-repo so its behavior is owned,
 tested,
 fuzzed,
 and mutation-tested here instead of trusted from a third-party package.

### Attribution

Derived from [`p-limit`](https://github.com/sindresorhus/p-limit) by
[Sindre Sorhus](https://sindresorhus.com),
 released upstream under the [MIT License](LICENSES/MIT.txt).
Scheduling semantics,
 validation rules,
 and error message texts come from `p-limit` 7.3.3.
Upstream's copyright and permission notice are preserved verbatim in
`LICENSES/MIT.txt`;
 this fork's own code is licensed `LGPL-3.0-or-later AND MIT`
as declared in `package.json`.

### Usage

```ts
// package/upstream-fork/p-limit/example.ts
import { pLimit, } from '@monochromatic-dev/upstream-fork-p-limit';

const limit = pLimit({ concurrency: 2, },);

const results = await Promise.all([
  limit({ fn: fetchUser, args: ['a'], },),
  limit({ fn: fetchUser, args: ['b'], },),
]);
```

### Behavior

#### Scheduling

`pLimit(concurrency)` and `pLimit({ concurrency, rejectOnClear })` create one
limiter.
Each scheduled call returns a promise settling with its function's result or
failure.
Calls start asynchronously,
 never inside the `limit` call that scheduled
them,
 and begin in FIFO order.
At most `concurrency` calls run at once;
`Number.POSITIVE_INFINITY` removes the cap.

#### Counters

`activeCount` counts running calls and `pendingCount` counts queued calls.
The `concurrency` member reads and writes the bound:
 raising it admits queued
calls from a microtask,
 lowering it never interrupts running calls.
Attached members are non-enumerable,
 non-writable,
 and non-configurable,
matching upstream's property descriptors.

#### clearQueue

`clearQueue()` discards queued calls only,
 leaving running calls untouched.
With `rejectOnClear`,
 each discarded call's promise rejects with
`AbortSignal.abort().reason`;
 otherwise those promises stay pending.

#### map

`map({ iterable, mapper })` processes inputs under the bound and collects
results in input order.

#### limitFunction

`limitFunction({ fn, options })` wraps one function in its own bounded call
queue,
 returning a callable that takes `{ args }` and exposes `clearQueue`.

### Deviations from upstream p-limit

#### Call shape

Calls pass an `args` tuple instead of a rest parameter
(`limit({ fn, args })`),
 `map` takes a destructured options object,
 and
`limitFunction({ fn, options })` takes one destructured options object.
Repository lint bans rest parameters and multi-positional-parameter
declarations outright.

#### Error types

Invalid configuration throws `InvalidConcurrencyError` or
`InvalidRejectOnClearError`
 instead of a bare `TypeError`.
Both extend `TypeError` and carry upstream's message text verbatim,
 so
migrating callers see identical diagnostics and the fuzz sidecar's
differential oracle can compare both implementations directly.

#### Internal queue

The queue lives in `src/task-queue.ts` instead of upstream's `yocto-queue`
dependency,
 leaving the package with zero runtime dependencies and one less
third-party trust surface on the concurrency path.

#### Settle timing

Each call's promise settles directly with its function's outcome,
 instead of
routing through upstream's promise-of-promise adoption hop.
The differential
oracle finds no observable difference in scheduling,
 counters,
 settlements,
descriptors,
 or validation.

### Layout

`p-limit.ts` owns scheduling,
 `task-queue.ts` the FIFO queue,
`limit-options.ts` configuration,
 `limit-function.ts` the wrapper factory,
and `errors.ts` the error classes.
Test files sit beside each module as `<stem>.unit.test.ts`,
 so mutation
testing selects each module's tests automatically.

### Testing

```bash
# package/upstream-fork/p-limit/mise.toml

# Unit tests against the built dist
mise run //package/upstream-fork/p-limit:buildAndTest

# Property-based fuzz campaign (invariants plus an upstream differential oracle)
mise run //package/upstream-fork/p-limit.fuzz:fuzz

# Coverage-reachability gate over this package's src
mise run //package/upstream-fork/p-limit.fuzz:fuzz:coverage

# Container-isolated mutation testing
mise run //package/upstream-fork/p-limit:test:mutation
```
