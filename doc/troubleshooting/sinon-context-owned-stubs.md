# Sinon 22.1.0: overlapping sandbox method stubs collide on a shared target

Investigation for [Monochromatic issue #481][issue],
verified on 2026-09-06.
The proposed production direction is in
[`issue-481-context-owned-stubs.md`](../planning/issue-481-context-owned-stubs.md).
The initial investigation committed documentation and the disposable experiment patch only.
Implementation is now authorized and underway.
Current acceptance state is recorded in
[`issue-481-context-owned-stubs.md`](../handover/issue-481-context-owned-stubs.md).

## Symptom

Concurrent sibling suites each call `ctx.sinon.stub(console, 'warn')`.
Sinon throws:

```text
TypeError: Attempted to wrap warn which is already wrapped
```

Giving each sibling `concurrency: 1` does not serialize the siblings against each other.
The forced-overlap control also exposed an independent failure:
a concurrent test without a stub observed the first test's fake instead of the original method.
Suppressing the double-wrap error would not fix that reader contamination.

## Root cause

Repository excerpts in this section describe the pre-implementation baseline,
not the current implementation.

### Injected context owns cleanup, not the target object

`package/test-harness/core/src/it.ts:215` creates and passes a per-test sandbox:

```ts
// package/test-harness/core/src/it.ts
await using sandbox = createSinon();
const ctx: TestContext = {
  expect: scopedExpect,
  sinon: sandbox,
};
```

`package/test-harness/core/src/sinon.ts:39` delegates to ordinary Sinon.
The file moved on 2026-10-04 to `package/test-harness/core-sandbox/src/sinon.ts`;
the snippet retains its historical location:

```ts
// package/test-harness/core/src/sinon.ts
const sandbox = createSandbox(config,) as DisposableSandbox;
```

`package/test-harness/core/src/describe.ts:184` selects an inherited concurrency value,
but `package/test-harness/core/src/describe.ts:304` allocates a limiter for this suite's children:

```ts
// package/test-harness/core/src/describe.ts
const effectiveConcurrency = concurrency ?? ctx
  .effectiveConcurrency;
```

```ts
// package/test-harness/core/src/describe.ts
const limit = pLimit(effectiveConcurrency,);
```

### Sinon replaces the real property

Installed dependency resolution selected `sinon@22.1.0`.
The matching upstream source was cloned at tag `v22.1.0`,
commit `ab289e92cdd76caf8cec2b0a8c9a391283e6c9df`.

Upstream `src/sinon/stub.js:150` delegates function replacement to `wrapMethod`:

```js
// sinon/src/sinon/stub.js
return isStubbingNonFuncProperty ? s : wrapMethod(object, property, s);
```

Upstream `src/sinon/util/core/wrap-method.js:81` rejects an existing wrapped function:

```js
// sinon/src/sinon/util/core/wrap-method.js
} else if (wrappedMethod.restore && wrappedMethod.restore.sinon) {
    error = new TypeError(
        `Attempted to wrap ${valueToString(
            property,
        )} which is already wrapped`,
    );
}
```

The same file at line 162 installs the descriptor on the supplied object:

```js
// sinon/src/sinon/util/core/wrap-method.js
Object.defineProperty(object, property, methodDesc);
```

The sandbox-specific context in upstream `src/sinon/sandbox.js:228`
concerns call-order counters:

```js
// sinon/src/sinon/sandbox.js
// Use withContext to pass sandbox context for isolated callId tracking
```

[Sinon PR #2715][call-id] introduced that counter isolation,
not execution-context-dependent target properties.
It does not remove this incident's wrapping guard.

### The harness already has an execution-context boundary

`package/test-harness/core/src/execution-node.ts:173` creates async storage:

```ts
// package/test-harness/core/src/execution-node.ts
storage: new AsyncLocalStorage<ObservedExecution | ReportingExecution>(),
```

`package/test-harness/core/src/execution-node.ts:283` runs each descriptor in that store:

```ts
// package/test-harness/core/src/execution-node.ts
return runtime.storage
  .run(
    execution,
    async function runBody(): Promise<Result> {
```

This is an integration point for production ownership,
not an already implemented mocking feature.
`package/test-harness/core/src/execution.ts:25` currently delegates directly outside detected Node:

```ts
// package/test-harness/core/src/execution.ts
if (((typeof process) === 'undefined')
  || ((typeof process.versions
    ?.node) !== 'string'))
  return options.run();
```

The [Node async-context documentation][node-context] describes propagation through promises and callbacks,
and explicitly discusses `EventEmitter` listeners running in a context different from registration.

## Verification

The experiment used repository base `089d4c953b4fec7c63bfcb8d635c2405aefdc0a0`,
Sinon 22.1.0,
and Node `v26.7.0` printed inside the final mise-driven probe.
It ran in a disposable worktree,
with read-only dependency links to installed packages.
No main-worktree implementation or logger test was edited.

The [experiment patch](sinon-context-owned-stubs.patch) preserves the helper,
harness integration,
and runnable probes.
The fixtures import the fork's real `describe` and `it` source,
not replacement test-runner functions.
This is source-level feasibility evidence,
not built-artifact acceptance evidence.

### Failure control

Before adding the adapter and execution wrapper,
the forced-overlap fixture ran through the existing package task:

```bash
# From the disposable repository root, with only the first probe fixture present.
mise run //package/test-harness/core:test:unit -- package/test-harness/core/src/issue-481-probe.unit.test.ts
```

Observed output included:

```text
Attempted to wrap warn which is already wrapped
+ 'A'
- 'original:reader'
BASELINE_CONFIRMED: collision with sequential siblings; exact descriptor restored
```

The command exited successfully because the fixture asserts that the baseline collision occurred.
That exit is not a claim that the unmodified harness isolated the tests.

### Working prototype catalog

With the adapter and execution wrapper installed in the fork:

```bash
# From the disposable repository root.
PROBE_MODE=contextual mise run //package/test-harness/core:test:unit -- \
  package/test-harness/core/src/issue-481-probe.unit.test.ts \
  package/test-harness/core/src/issue-481-lifecycle-probe.unit.test.ts
mise run //package/test-harness/core:lint:types
```

Both commands exited zero.
Consumer assertions verified:

- Different fake behavior and call histories under forced overlap.
- Original method behavior for an unstubbed concurrent reader.
- Actual `console.warn` isolation across timer awaits.
- Fake identity at property read,
  and a reference captured by one owner remaining that owner's fake when another calls it.
- Independent `stub.restore()` and `sandbox.restore()`.
- Exact final descriptor restoration.
- Repeats,
  same-owner double-stub rejection,
  `callThrough`,
  receiver preservation,
  history reset,
  restubbing,
  and repeated restoration of an older fake.
- A timed-out attempt's later method read reaching the original,
  never a subsequent test's fake.

The timeout fixture intentionally logs and asserts `Timed out after 1ms: timed-out owner`.
It is an expected failure path,
not an unresolved test failure.

### Behavioral boundary catalog

The plain `EventEmitter` probe registers under one owner and emits under another.
A method read inside that callback sees the emitting context's fake.
This passed as an explicit ownership-boundary assertion,
not as proof of registration-owner isolation.

The prototype rejects nonzero stub argument forms other than contextual two-argument method stubs,
and requires a configurable own function property.
Browser execution,
accessor stubs,
fake timers,
whole-object stubs,
spies,
and descriptor mutation interactions were not verified.

## Verified experimental workaround

The adapter asks ordinary Sinon to stub a private per-owner facade.
It installs a getter on the actual target that selects the facade using the current async context.
Readers without an active replacement see the original method.
Cleanup removes one owner and restores the original descriptor after the final owner leaves.

The test call remains `ctx.sinon.stub(target, property)`.
No test scheduling option or suite restructuring was required in the successful probes.
The prototype allocates a fresh ownership token for each test-body attempt.

Tradeoffs:

- The temporary getter changes descriptor reflection and assignment behavior.
  The probe does not claim descriptor transparency while owners are active.
- Ownership follows property-read context,
  not arbitrary causal ownership of events.
- The prototype is Node-only and covers a method subset,
  not the complete `SinonSandbox` contract.
- Retained `ctx.sinon` can still request new mutations after a test has completed.
  Production needs an irreversible completed-owner guard before this is shippable.

The existing sequential ancestor remains a fallback,
not the recommended endpoint for this method-stub incident.
It sacrifices concurrency and does not inherently stop asynchronous work after timeout.

## What does not work

- Separate ordinary sandboxes:
  the failure control proves they still share the target property.
- `concurrency: 1` on each sibling:
  the same control proves sibling suites still overlap.
- Treating the installed per-sandbox call-ID context as target isolation:
  the installed source retains direct replacement and the wrapping guard.
- Calling the experiment a drop-in implementation of Sinon:
  unsupported operations and descriptor behavior remain outside its evidence.

An initial source clone used an older tag inferred from test fixture text.
It was excluded after checking installed dependency resolution;
all upstream findings in this document use the matching 22.1.0 source.

## Upstream filing artifact

No upstream report or comment was sent.
The related [Sinon issue #2622][duplicate] requests double wrapping across sandboxes.
Its complete thread was read,
including the maintainer's objection to restoration order and shared fake state.
Our experiment implements harness-owned read-context selection instead;
it does not establish a Sinon bug.
There is no upstream bug-report artifact to add.

### Upstream filing decision

- Fault attribution:
  no.
  Sinon rejects overlapping replacements;
  this harness can provide a stronger ownership abstraction at its own boundary.
- Fix feasibility:
  the consumer-side method experiment works.
  A complete upstream design was not established.
- Supported use case:
  upstream documents sandbox grouping and cleanup,
  not independent values of one property in overlapping async executions.
- Contribution policy:
  `CONTRIBUTING.md` was read and welcomes issues with reproductions.
  A broader policy audit is unnecessary for a filing already rejected on fault attribution.
- Maintainer direction:
  issue #2622 explicitly declines the related multi-sandbox wrapping feature.
- Compatible upstream prototype:
  no upstream patch was attempted.
  This is a consumer-side experiment,
  not an upstream-ready fix.

`.out-of-scope/` filenames were checked;
none names Sinon or this bug class.
Searches for `"already wrapped" concurrent` and `"parallel" "sandbox"` returned no results.
Broader issue search for `"already wrapped"` found #2622;
broader PR search for `parallel` found #2715.
Both matching threads were read in full.

## Production boundary checks

Built-artifact boundary tests exposed an integration error in our fake-timer guard:
Sinon 22.1.0's bundled `clock.uninstall` calls `clock.setTickMode({ mode: 'manual' })`.
Closing the owner before cleanup correctly blocked new work,
but initially blocked this required shutdown call too.
The guard now allows runner restoration to stop automatic ticking,
not restart it.
A rebuilt full package test run passed after that correction.
Subsequent fixture corrections passed full package Oxlint and type checks.

The whole-object fixture initially reread the restored target instead of retaining its fake.
The corrected fixture captures the fake before completion,
then exercises its deferred `value` method.
The built test passes with completion rejection and the original target unchanged.

### Actual browser integration findings

The first real browser run could not import the neutral consumer bundle.
Firefox named `node:fs/promises`;
the neutral chunk began with static Node builtin imports.
`package/test-harness/core/src/format-error.ts` imported the module-fs-path barrel at load time.
The file moved on 2026-10-04 to `package/test-harness/core-diagnostic/src/format-error.ts`.
Its workspace lookup now dynamically imports that dependency only after Node detection.
The subsequent run passed the no-process fixtures in Chromium,
Firefox,
and WebKit.

The partial-process fixtures then reached a separate failure:
logger's `detectWebStorageRuntime` read `process.versions.node` when the injected process had no `versions`.
This is a logger runtime probe,
not a Sinon or async-context failure.
`package/logger/core/src/sink/web-storage-runtime.ts` now checks `versions` optionally.
Both the module-test browser consumer and a dedicated local-storage browser regression exercise that input.
The combined post-change run passed all nine selected cases across Chromium,
Firefox,
and WebKit in process `proc_a844`.
The subsequent direct browser error-formatting assertion passed in `proc_9107` and `proc_bced`.
After logger integration,
`proc_81bb` passed the combined process-shim browser checks again.

### Intentional Node emitter fixture

`package/test-harness/core/src/sinon-semantics.unit.test.ts` intentionally constructs Node's `EventEmitter`.
Replacing it with `EventTarget` would test a different callback API,
not the named Node boundary in the acceptance plan.

The [Oxlint rule source][event-target-rule] defines `PreferEventTarget` without configuration fields
or a `from_configuration` implementation.
Its only package exceptions are hardcoded for `@angular/core` and `eventemitter3`,
not Node's `node:events`.
There is no configurable constructor or call-site allowlist to try.
A single justified `unicorn/prefer-event-target` suppression therefore stays on this test declaration;
no package-wide lint setting is loosened.

### Intentional source and artifact interoperability

`package/test-harness/core/src/sinon-copies.unit.test.ts` imports both built entry points
and the public `/ts` entry point in one process.
The source import is not a substitute for artifact verification:
this test specifically proves shared ownership between independently loaded implementations.
Replacing the source import with another built import would remove that case.

`package/oxlint-plugin/test-import/src/require-eventual-artifact.ts` exposes only `fixturePatterns`.
`package/oxlint-plugin/test-import/src/import-classification.ts` classifies a package's own `/ts` imports
before any relative fixture-pattern handling.
No available fixture configuration can allow this bare source specifier.
The test therefore uses a single explained suppression on that import,
not a global rule change or a re-export that hides its source origin.

### Manual mock restoration generation

The production fixture in `sinon-injection.unit.test.ts` demonstrated that an already verified mock controller
could restore a newly installed contextual stub in the same still-running attempt.
The rebuilt failure was `expected 'original' to equal 'new generation'` in process `proc_2ed7`.

Sinon 22.1.0's `src/sinon/mock.js:84` restores by reading the target's current property:

```js
// Sinon 22.1.0, src/sinon/mock.js:87.
if (typeof object[proxy].restore === "function") {
    object[proxy].restore();
}
```

This lookup is not restricted to the controller's original replacement.
The adapter now retires the controller on successful restoration,
including restoration called internally by `verify`.
New `expects` calls require a fresh controller after that point.

### Call-sequence behavior authority

The built regressions in `sinon-returned.unit.test.ts` failed in `proc_46f5`:
`onCall` and its named aliases returned objects whose descriptor methods bypassed the root fake guard.
A raw Sinon 22.1.0 control confirmed descriptor changes through `.get`,
`.set`,
and `.value` on each selector result.

The source establishes the connection:

```js
// Sinon 22.1.0, src/sinon/stub.js:255.
onCall: function onCall(index) {
    if (!this.behaviors[index]) {
        this.behaviors[index] = behavior.create(this);
    }
    return this.behaviors[index];
}
```

`src/sinon/behavior.js:127` records the root:

```js
// Sinon 22.1.0, src/sinon/behavior.js:127.
behavior.stub = stub;
```

The descriptor methods then use that reference:

```js
// Sinon 22.1.0, src/sinon/default-behaviors.js:314.
value: function value(fake, newVal) {
    const rootStub = fake.stub || fake;
    Object.defineProperty(rootStub.rootObj, rootStub.propName, {
        value: newVal,
```

`sandbox-fake.ts` now guards selector results with the same owner and restoration generation as their root.
The rebuilt regression suite passed in `proc_ec54` and `proc_bced`.
Local behavior changes and saved fake calls remain usable after completion;
only descriptor-changing authority is retired.

An independent review's `withArgs` hypothesis was incorrect for this installed version.
The raw control's child `.get`,
`.set`,
and `.value` calls each threw `TypeError: Object.defineProperty called on non-object`.
`src/sinon/spy.js:45` creates an independent fake:

```js
// Sinon 22.1.0, src/sinon/spy.js:45.
const fakeInstance = this.instantiateFake();
fakeInstance.matchingArguments = args;
fakeInstance.parent = this;
```

The descriptor methods use `fake.stub || fake`,
not that parent.
No target descriptor changed in those controls.
Ordinary `withArgs` behavior remains unchanged.

### Partial installation and injection

Raw whole-object `stub` and `spy` controls installed the first method,
then threw `Attempted to wrap second which is already wrapped` on a later member.
Calling the raw sandbox's `restore()` did not restore the first method.
The registration boundary explains why:

```js
// Sinon 22.1.0, src/sinon/sandbox.js:233.
const createdStub = sinonStub.withContext.apply(sinonStub, args);
const result = commonPostInitSetup(
    arguments,
    createdStub,
```

Construction can throw before `commonPostInitSetup` receives the collection.
`sandbox-install.ts` snapshots descriptors and rolls back only introduced fakes on failure.
It does not restore unrelated successful fakes in the same sandbox.
Independent rollback steps run even when one restorer fails;
both construction and rollback errors are retained.
`sinon-install.unit.test.ts` verifies these paths.

The first rollback-failure fixture instrumented `isSinonProxy`,
but no construction failure occurred.
The deciding source is `src/sinon/util/core/wrap-method.js:81`:

```js
// Sinon 22.1.0, src/sinon/util/core/wrap-method.js:81.
} else if (wrappedMethod.restore && wrappedMethod.restore.sinon) {
```

Instrumenting `restore` reached that boundary.
The corrected test passed in `proc_e422`,
including preservation of an independent member's descriptor.

Injection has an earlier exposure boundary than return from the factory:

```js
// Sinon 22.1.0, src/sinon/sandbox.js:127.
sandbox.inject = function inject(obj) {
    obj.spy = function spy() {
        return sandbox.spy.apply(sandbox, arguments);
    };
```

An application setter receives that function immediately.
`sandbox-result.ts` therefore guards factories before assignment,
not merely after successful injection.
A fixture setter retains the spy factory before a later readonly `stub` assignment fails;
that retained factory rejects after completion.
The fixture also verifies the setter receives its original destination as `this`.

These are harness ownership guarantees,
not a claim that ordinary Sinon offers transactional construction or completed-attempt authority.
They do not change the upstream filing decision.

### Detached function spies are not whole-object mutations

The final ordinary-overload fixture in `sinon-semantics.unit.test.ts` exposed an overbroad adapter preflight.
`sinon.spy(fn)` was rejected when `fn` already had a context-owned static method,
although detached spying does not replace that method.
`proc_a541` reproduced the incorrect `ctx.sinon.spy cannot replace property "method"` diagnostic.

The distinction comes from Sinon 22.1.0's `src/sinon/spy.js:183`:

```js
// Sinon 22.1.0, src/sinon/spy.js:183.
if (!property && typeof object === "function") {
    return createSpy(object, context);
}
```

Commit `2fe8e6311` excludes that detached function form from whole-object mutation preflight.
Whole-object function stubbing still uses collision checks and collection guards.
The regression verifies invocation,
history,
and an unchanged descriptor on the independently stubbed static method.
`proc_c6c6` passed the rebuilt full module-test suite,
scoped lint,
and types after the correction.

### Build ordering and artifact control

`proc_56a3` failed before the intended setter regression ran:
the browser consumer could not resolve `../dist/final/neutral/index.mjs`.
The repository's nested task fan-out launched the neutral build twice,
allowing its output to disappear while the consumer bundled it.
Commit `b6f1a813c` changes `package/test-harness/core/mise.toml` to one declared JS dependency graph.
The client task depends on the neutral artifact.
The next build reached the intended test failure instead of failing module resolution.
Full builds and browser acceptance subsequently passed.

The production routing control used a disposable worktree at `94996e25a`:

```bash
# From the disposable repository root, using its own built Node artifact.
mise run //package/test-harness/core:build:js:node
mise run //package/test-harness/core:test:unit -- package/test-harness/core/src/sinon-context.unit.test.ts
```

`import.meta.resolve('@monochromatic-dev/test-harness-core')`,
run from the disposable package directory,
resolved to that worktree's `dist/final/node/index.mjs`.
The fixture passed in `proc_8267`.
After adding `false &&` only to the contextual dispatch condition in `sandbox-operation.ts`,
rebuilding and rerunning failed in `proc_6a49` with:

```text
Attempted to wrap method which is already wrapped
expected 'fake 0' to equal 'original:reader'
expected 'parent' to equal 'original'
```

Removing that change and rebuilding restored the passing result in `proc_fc0e`.
The control used Node 26.7.0 from the worktree's tracked tool lock.
The worktree and its dependency symlinks were removed after verification.

The actual logger breadcrumb wrapper now uses default concurrent scheduling.
Rebuilt logger unit tests and types passed in `proc_511c`;
logger Oxlint and combined real-browser checks passed in `proc_81bb`.
The console-sink tests that mutate `process.env` and `process.argv` retain their sequential setting.

[event-target-rule]: https://raw.githubusercontent.com/oxc-project/oxc/main/crates/oxc_linter/src/rules/unicorn/prefer_event_target.rs
[issue]: https://github.com/Aquaticat/Monochromatic/issues/481
[duplicate]: https://github.com/sinonjs/sinon/issues/2622
[call-id]: https://github.com/sinonjs/sinon/pull/2715
[node-context]: https://nodejs.org/api/async_context.html
