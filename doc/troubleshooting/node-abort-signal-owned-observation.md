# Node 26.8.2 AbortSignal accessors can cross an operation's error-ownership boundary

## Symptom

The pre-fix comparison's initial signal-state check could return a caller-selected `storage` failure and directory
before preflight creates any namespace.
A genuine `AbortController.signal` is sufficient:
an own `aborted` getter throws a `ProducerInputComparisonError` carrying foreign operation metadata.
`comparison-signal-origin-j3YwJS/verification.json` records the returned directory
`/caller-owned-fixture-q7z9k2` and `messageContainsCallerLocator: true`.
This is a consumer error-ownership defect,
not a Node authentication guarantee being violated.

Task62's production boundary,
full suite,
current mutation controls and native cases pass their recorded checks.
The current acceptance ledger records durable evidence and exact stopped-container cleanup.
Task54's logger callback containment remains separate verified behavior.

## Root cause

The source reference is Node release `v26.8.2`,
commit `f2f2c2f246c36bd74f082cb43ecfe830657d81c9`.
The read-only clone is `node-json-diagnostic-20260912` under private agent scratch;
selected files were read with `git show HEAD:<path>` without changing the clone.

At pre-fix commit `3707322f2`,
the comparison reads live caller state in
`package/module/translation-repair/src/corpus-run/producer-input-comparison.ts:137` (removed before 2026-10-18):

```ts
// package/module/translation-repair/src/corpus-run/producer-input-comparison.ts (removed before 2026-10-18)
if (request.signal
  .aborted)
  throw new ProducerInputComparisonError({ kind: 'interruption' });
```

The outer catch at
`package/module/translation-repair/src/corpus-run/producer-input-comparison.ts:338` (removed before 2026-10-18)
preserves a same-class Error's kind and directory:

```ts
// package/module/translation-repair/src/corpus-run/producer-input-comparison.ts (removed before 2026-10-18)
const failure = Error.isError(error) && (error instanceof ProducerInputComparisonError)
  ? error : new ProducerInputComparisonError({ kind: 'output' });
throw new ProducerInputComparisonError({
  kind: failure.kind,
  ...(failure.directory === undefined ? {} : { directory: failure.directory }),
  loggerCallbackFailures: observedLogger.snapshot(),
});
```

Class identity does not establish that those fields belong to this invocation.
The initial request reader at
`package/module/translation-repair/src/corpus-run/producer-input-comparison-contract.ts:134` (removed before 2026-10-18)
already reconstructs getter failures as fresh contract refusals,
but a later signal accessor runs outside that capture:

```ts
// package/module/translation-repair/src/corpus-run/producer-input-comparison-contract.ts (removed before 2026-10-18)
void error;
throw new ProducerInputComparisonError({ kind: 'contract', });
```

Composing a signal once does not remove this borrowed access.
Node `lib/internal/abort_controller.js:312` defines `AbortSignal.any`;
its loop at line 340 reads public source properties:

```js
// nodejs/node: lib/internal/abort_controller.js
if (signal.aborted) {
  abortSignal(resultSignal, signal.reason);
  return resultSignal;
}
```

The native getter at `lib/internal/abort_controller.js:255`
also refreshes composite source state:

```js
// nodejs/node: lib/internal/abort_controller.js
get aborted() {
  validateThisAbortSignal(this);
  refreshCompositeSignal(this);
  return !!this[kAborted];
}
```

`refreshCompositeSignal` at `lib/internal/abort_controller.js:140`
reads each surviving source's public accessors.
Applying the native getter directly bypasses an own override on an ordinary signal,
but does not bypass a composite source's accessors:

```js
// nodejs/node: lib/internal/abort_controller.js
if (sourceSignal.aborted) {
  abortSignal(signal, sourceSignal.reason);
  return;
}
```

Node `lib/internal/events/abort_listener.js:26`
implements `addAbortListener` using borrowed property lookups and a once-only listener:

```js
// nodejs/node: lib/internal/events/abort_listener.js
if (signal.aborted) {
  queueMicrotask(() => listener());
} else {
  signal.addEventListener('abort', listener, abortListenerOptions);
  removeEventListener = () => {
    signal.removeEventListener('abort', listener);
  };
}
```

Its frozen options at line 40 include `once: true`
and Node's propagation-resistance option:

```js
// nodejs/node: lib/internal/events/abort_listener.js
abortListenerOptions ??= ObjectFreeze({ __proto__: null, once: true, [kResistStopPropagation]: true });
```

A synthetic `abort` event can consume a once-only bridge listener even though native signal state stays false.
A subsequent genuine abort then fails to reach that bridge.

`Event.isTrusted` is not sufficient to recover native signal state.
`lib/internal/event_target.js:81` tracks trusted events in a private weak set,
and `dispatchEvent` at line 777 redispatches the event without clearing that membership.
The relevant source excerpts are:

```js
// nodejs/node: lib/internal/event_target.js
const isTrustedSet = new SafeWeakSet();
// dispatchEvent forwards the existing event object.
this[kHybridDispatch](event, event.type, event);
```

The measured replay keeps `isTrusted: true` while the receiving signal remains un-aborted.

## Verification

All probes run on Node `26.8.2`.
The retained native source excerpts are:

- `node-abort-controller-26.8.2-20260914.js`.
- `node-abort-listener-26.8.2-20260914.js`.
- `node-event-target-26.8.2-20260914.js`.
- `node-validators-26.8.2-20260914.js`.

The scripts live under private agent scratch:

```sh
# Run from the translation-repair worktree with the recorded Node 26.8.2 binary.
node /var/home/user/temp/agent/probe-comparison-signal-error-origin-20260914.mts
node /var/home/user/temp/agent/probe-node-abort-composition-20260914.mts
node /var/home/user/temp/agent/probe-owned-comparison-signal-r2-20260914.mts
```

The first probe uses frozen parent runtime `U8mJbo` as pre-fix evidence.
It does not invoke the bootstrap or a provider.
The composition probe verifies initial accessor reads,
late accessor reads,
ignored synthetic events and redispatched trusted events.

Working candidate catalog,
recorded in `owned-comparison-signal-probe-r2-cYslXA`:

- Native state forwarding without reading an own `aborted` override or retaining caller reasons.
- Resistance to an earlier listener's `stopImmediatePropagation`.
- Ignoring synthetic and redispatched events while preserving a later actual abort.
- Bypassing late own state,
  reason,
  registration and removal overrides.
- Native cancellation through a previously followed composite despite a later source getter override.
- Recording unreadable composite state without throwing from the event callback
  or losing subsequent native cancellation.
- Refusal of proxy-wrapped,
  revoked and forged-prototype inputs.

Failure catalog:

- The pre-fix comparison forwards same-class error metadata from the borrowed live-state getter.
- `AbortSignal.any` reads a throwing own getter during construction.
- A constructed composite re-reads a subsequently replaced source getter.
- The first once-only bridge candidate loses genuine cancellation after a synthetic event.
  `owned-comparison-signal-r1-probe-20260914.out` records its ordinary assertion failure.

## Verified consumer boundary and its limits

The R2 throwaway candidate informed the production owner
`package/module/translation-repair/src/corpus-run/producer-input-comparison-signal.ts` (removed before 2026-10-18).
The comparison owns a new controller and passes only its signal onward.
A private registration view delegates state and listener operations through captured native methods.
It uses `addAbortListener` for propagation resistance,
preserves its options without interpreting the opaque resistance key,
and changes only `once` to false so spurious events do not consume the subscription.
The listener checks native aborted state rather than event trust or caller reasons.
Scoped disposal uses the native removal path.

Event-time unreadable state becomes a retained local failure flag,
not a thrown event-callback value or invented cancellation.
The awaiting comparison owner checks that flag before reporting success
and retains fixed contract-refusal metadata when observation fails without proven cancellation.
The current predicate at
`package/module/translation-repair/src/corpus-run/producer-input-comparison-signal.ts:162` (removed before 2026-10-18)
preserves actual cancellation priority:

```ts
// package/module/translation-repair/src/corpus-run/producer-input-comparison-signal.ts (removed before 2026-10-18)
if (state.unreadable && (!ownedSignal.aborted))
```

The outer failure path at
`package/module/translation-repair/src/corpus-run/producer-input-comparison.ts:351` (removed before 2026-10-18)
releases before its fixed warning and terminal snapshot:

```ts
// package/module/translation-repair/src/corpus-run/producer-input-comparison.ts (removed before 2026-10-18)
cancellation[Symbol.dispose]();
if (cancellation.hasObservationFailure())
  pl.warn('cancellation observation failed; primary comparison failure remains retained');
```

Primary kind,
message and owned directory remain authoritative.
The snapshot at line 362 follows the warning;
cleanup observation does not retrofit telemetry into synchronized records.
Native interruption registration now completes before spawn;
early cancellation is forwarded only after the actual-close observer is installed.
The prototype alone did not establish these operation-level guarantees.

Current acceptance evidence is recorded in
[`translation-repair-preparation-plan-and-journal-2026-09-14.md`](../planning/translation-repair-preparation-plan-and-journal-2026-09-14.md).
It includes the six original red assertions,
the separate pre-spawn setup regression,
current full-suite and read-only lint checks,
signal and non-logger guard controls,
and actual CLI runs using separately identified frozen runtimes.
`UJeHke` predates the terminal correction.
R6 runtime `frozen-runtime-m1aVqu` verifies that correction through actual Task47 execution,
including combined unreadability/cancellation and subscription-disposer faults.
R8 runtime `r8-frozen-runtime-rSoOtk` separately qualifies the `@internal` barrel exports
and warning-only callback-failure control.
Its premature-snapshot mutant reaches the designated ordinary assertion,
while native warning-failure cases preserve primary metadata and return `['warn']`.
R6 evidence is not relabeled as that artifact proof.
Durable copies and original-path mappings are retained under
`package/module/translation-repair/node_modules/.monochromatic/comparison-evidence/`.
The tests and scoped reviews do not constitute whole-change review.

Supported input and lifecycle assumptions must remain explicit.
This does not establish immunity to patched native intrinsics,
corrupted internal slots,
blocking caller code,
process termination or hostile-host mutation.
Proxy and forged-prototype signals are refused.
The native composition and registration behavior is version-bound evidence,
not a promise about every future Node implementation.

## What does not work

- Trusting `instanceof ProducerInputComparisonError` as error-metadata ownership.
- Treating a copied `AbortSignal.any` result as independent of future source accessors.
- Using ordinary event registration without considering propagation suppression.
- Forwarding every synthetic event as actual cancellation.
- Checking only `event.isTrusted`.
- Ignoring synthetic events inside a once-only subscription without preserving future observation.
- Reading caller reasons to decide error text,
  kind or retained directory.

## Upstream filing decision

No matching exemption was found among `.out-of-scope/` topics.
The Node issue search for `AbortSignal getter` returned no matches;
that result is not evidence of an upstream defect or a refusal to support this case.

1.  Upstream fault:
    no.
    Our comparison grants authority to caught metadata outside its owned boundary.
2.  Upstream fixability:
    no Node change is needed to repair that consumer boundary.
    No impossibility claim is made about alternative Node designs.
3.  Supported use case:
    Node documents composition and propagation-resistant abort observation,
    not authentication of our error kinds or directories.
4.  Contribution policy:
    no filing or upstream patch is proposed,
    so no claim is made about whether maintainers would accept one.
5.  Expected upstream action:
    not asserted.
    The empty search does not establish willingness or unwillingness.
6.  Prototype:
    the retained prototype is consumer-side only.
    It is not an upstream patch.

Upstream filing artifact:
nothing to add.
No Node issue or comment is filed.
