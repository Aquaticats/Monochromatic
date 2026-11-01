# Async function `return await promise` settles the caller one microtask before `return promise`, breaking parity ports

## Symptom

An `async` function that returns a deferred promise settles its caller at a
different microtask depth depending on whether the body writes
`return await deferred.promise` or `return deferred.promise`:
 the awaited form
settles one tick early when the deferred is already settled.

Porting a third-party `async function` that returns `new Promise(executor)`
(one tick of adoption) onto a `Promise.withResolvers` deferred shows the gap
as an observable settle-timing mismatch against the upstream implementation:

```text
upstream rejection ticks: 3
fork (return await) rejection ticks: 2
```

Resolve paths can still match while reject paths diverge, because an
already-rejected deferred takes the adoption path one tick earlier when
awaited inline.

## Root cause

Step by step, for a deferred that rejects before the `async` function
returns:

1. `return deferred.promise` inside an `async` function hands the promise to
   the async return path.
   The function's own promise adopts it through the
   thenable job queue,
   exactly as `return new Promise(executor)` does,
   so the
   caller observes the rejection one microtask after the deferred rejects.
2. `return await deferred.promise` awaits the already-settled deferred
   first.
   The await reaction runs one microtask after the rejection,
   then the
   function returns the resolved value,
   so the caller observes the rejection
   one microtask earlier than the adoption path.
3. The port under review (`package/upstream-fork/p-map/src/p-map.ts`) had
   rewritten upstream `p-map`'s `return new Promise(executor)` shape into a
   `Promise.withResolvers` deferred plus `return await settlement.promise`,
   because repository lint's `require-await` rejects an `async` function with
   no `await`.
   That lint-driven rewrite changed the observable settle tick.

The fix keeps upstream's adoption shape and suppresses the lint rule at the
declaration with the reason recorded:

```ts
// package/upstream-fork/p-map/src/p-map.ts
/* oxlint-disable eslint/require-await -- `pMap` stays `async` so a destructuring failure rejects the caller's promise like upstream `p-map`'s `async` signature, and returning the deferred promise without awaiting it preserves upstream's promise-adoption tick (measured: an `await` here settles the caller one microtask early) */
export async function pMap<Element, NewElement>(
  ...
): Promise<(Exclude<NewElement, typeof pMapSkip>)[]> {
  ...
  return settlement.promise;
}
/* oxlint-enable eslint/require-await */
```

## Verification

The probe counts microtask ticks from attaching handlers until the run
promise settles,
 once for each implementation:

```js
// probe shape used against both implementations
const ticksUntilSettled = async (run) => {
  const state = { settled: false, ticks: 0 };
  run.then(() => { state.settled = true; }, () => { state.settled = true; });
  while (!state.settled && state.ticks < 20) {
    await Promise.resolve();
    state.ticks += 1;
  }
  return state.ticks;
};
```

Measured on Node 26 against upstream `p-map` 7.0.8 and the fork:

```text
upstream: rejection 3, resolution 5
fork with `return await`: rejection 2, resolution 5
fork with `return`: rejection 3, resolution 5
```

The differential fuzz campaign
(`mise run //package/upstream-fork/p-map.fuzz:fuzz`) cannot see this class of
difference: its traces are gate-controlled with macrotask separators,
 which
never resolve sub-tick differences.
 Microtask-count probes are the validating
instrument.

## Verified workarounds

- Keep the upstream return shape (`return deferred.promise`) and suppress
  `require-await` at the `async` declaration with the measured reason, as
  quoted above.
  Verified: rejection and resolution ticks both match upstream.

## What does not work

- `return await deferred.promise` written to satisfy `require-await`.
  Verified broken: settles the caller one microtask early on the reject path.
- A non-`async` function returning the deferred promise.
  Verified broken:
  skips the adoption hop entirely (rejection observed one tick after the
  deferred settles, versus upstream's three).
- Comparing settle order in gate-controlled workload traces.
  Verified
  insufficient: macrotask separators cannot resolve microtask-depth
  differences.

## Upstream filing artifact

None.
 This is ECMAScript-specified promise adoption behavior
 (`AsyncFunctionStart` resolving the returned thenable),
 not a defect in V8,
Node,
 or `p-map`,
 so the filing constraints do not apply.
 The durable artifact
is this file plus the tick probe quoted in
[Verification](#verification).
