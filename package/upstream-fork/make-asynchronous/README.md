## module-make-asynchronous-fork

TypeScript fork of [`make-asynchronous`](https://github.com/sindresorhus/make-asynchronous),
 in-repo so its behavior is owned,
 tested,
 fuzzed,
 and mutation-tested here instead of trusted from a third-party package.

### Attribution

Derived from [`make-asynchronous`](https://github.com/sindresorhus/make-asynchronous)
 by
[Sindre Sorhus](https://sindresorhus.com),
 released upstream under the [MIT License](LICENSES/MIT.txt).
Worker semantics,
 reply protocol,
 and error restoration rules come from `make-asynchronous` 2.1.0.
Upstream's copyright and permission notice are preserved verbatim in
`LICENSES/MIT.txt`;
 this fork's own code is licensed `LGPL-3.0-or-later AND MIT`
as declared in `package.json`.

### Usage

```ts
// package/upstream-fork/make-asynchronous/example.ts
import { makeAsynchronous, } from '@monochromatic-dev/upstream-fork-make-asynchronous';

const fn = makeAsynchronous({
  fn: function double(value: number,): number {
    return performExpensiveOperation(value,);
  },
});

console.log(await fn({ args: [2], }));
```

```ts
// package/upstream-fork/make-asynchronous/example-iterable.ts
import { makeAsynchronousIterable, } from '@monochromatic-dev/upstream-fork-make-asynchronous';

const fn = makeAsynchronousIterable({
  fn: function * (count: number,): Generator<number> {
    yield * performExpensiveOperation(count,);
  },
});

for await (const value of fn({ args: [2], })) {
  console.log(value,);
}
```

### Behavior

#### makeAsynchronous

`makeAsynchronous({ fn, options })` returns an async function running `fn`
in a worker thread.
Each call spawns its own worker,
 posts its `args` tuple,
awaits the single reply,
 restores the result,
 and terminates the worker.
The main thread never blocks,
 even while the worker spins.

#### makeAsynchronousIterable

`makeAsynchronousIterable({ fn, options })` returns a function producing an
async iterable per call.
Each iteration spawns its own worker:
 the first
message runs `fn` and normalizes its return into an iterator,
 later
messages pull the next item with no arguments attached.
Iterator state never
leaks between iterations.

#### Serialization

The wrapped function is serialized with `Function.prototype.toString`,
 so it
cannot close over outer variables or imports.
Pass everything through
arguments instead,
 exactly like upstream.

#### Options

Both factories accept `options.baseUrl`,
 resolving bare dynamic imports
inside Node.js workers.
Pass `import.meta.url` when the wrapped function
dynamically imports dependencies from the module that creates the wrapper.

#### withSignal

Both wrapped forms carry `withSignal(signal)`,
 binding one `AbortSignal`
and returning the same call shape.
Abortion fails the worker with
`signal.reason`,
 which the call or iteration rejects with verbatim,
 including
falsy reasons.
Each call detaches its abort listener when it settles.

#### Results and errors

Results and failures travel back keyed by request id,
 so a wrapped
function posting its own messages cannot take over the channel.
Failure
detection uses the `error` key rather than the value,
 so falsy thrown values
reject and falsy returned values resolve.
Error name and extra own
properties (including `AggregateError.errors`) travel beside the cloned
error and are defined back onto it,
 with getter-only names handled by
definition rather than assignment.

### Deviations from upstream make-asynchronous

#### Call shape

Factories take one destructured object (`makeAsynchronous({ fn, options })`)
and calls pass an `args` tuple (`fn({ args })`),
 while the iterable factory
takes `{ fn, options }` with `{ args }` per iteration.
Repository lint bans
rest parameters and multi-positional-parameter declarations outright.
The
worker still receives the tuple spread as real arguments,
 so serialized
functions keep their upstream shape.

#### Type shape

Async forms derive the awaited return inline (`AsyncForm`,
`AsyncIterableForm`)
instead of upstream's `type-fest` `Asyncify` and `SetReturnType` imports,
leaving the package with zero runtime and type dependencies.

#### Cleanup structure

Upstream's `try`/`finally` worker cleanup becomes a `using` disposer
(repository lint bans `try`/`finally`),
 terminating the worker on every exit
from the call or iteration,
 including early `break` and `return`.
The
iterable pull loop uses an explicit `for (;;)` scan with the exit on top
rather than upstream's `while (true)`,
 matching repository loop style.

#### Internal modules

Worker-source templates live in `src/worker-source.ts`,
 the worker
lifecycle in `src/worker-lifecycle.ts`,
 reply settlement in `src/result.ts`,
options in `src/options.ts`,
 and one factory per module
(`src/make-asynchronous.ts`,
 `src/make-asynchronous-iterable.ts`),
 instead of
upstream's single `index.js`.
The templates stay byte-identical to
upstream's so the fuzz sidecar's differential oracle can compare built
worker sources verbatim.

### Layout

`make-asynchronous.ts` owns single-call wrapping,
`make-asynchronous-iterable.ts` owns iteration,
 `worker-lifecycle.ts` the
worker creation and request channel,
 `worker-source.ts` the worker module
templates,
 `result.ts` reply settlement,
 and `options.ts` the shared
options.
Test files sit beside each module as `<stem>.unit.test.ts`,
 so
mutation testing selects each module's tests automatically.

### Testing

```bash
# package/upstream-fork/make-asynchronous/mise.toml

# Unit tests against the built dist
mise run //package/upstream-fork/make-asynchronous:buildAndTest

# Property-based fuzz campaign (invariants plus an upstream differential oracle)
mise run //package/upstream-fork/make-asynchronous.fuzz:fuzz

# Coverage-reachability gate over this package's src
mise run //package/upstream-fork/make-asynchronous.fuzz:fuzz:coverage

# Container-isolated mutation testing
mise run //package/upstream-fork/make-asynchronous:test:mutation
```
