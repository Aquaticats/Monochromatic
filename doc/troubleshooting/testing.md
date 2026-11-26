# `@monochromatic-dev/test-harness-core` test harness: confusing error chains with duplicate describes, skipped tests after first failure, shared-state flakes, assertion-count + sinon-restore pitfalls

This file groups five independent quirks of the in-tree test harness
(`@monochromatic-dev/test-harness-core`).
 Each gets its own canonical
section.

---

## Bug 1: Duplicate `describe` names at the same scope produce ambiguous error chains

### Symptom

Test output reports a failing test under the wrong suite name,
 or
two suites print the same name,
 making it hard to locate the
actual failure:

```text
FAIL  suite-name > test-a   ← which "suite-name"?
FAIL  suite-name > test-b   ← (there are two)
```

### Root cause

`describe` propagates child failures as
`new Error(name, { cause: childError })`.
 When two sibling
describes share the same `name`,
 both errors wrap their causes with
the same message string.
 The runner's output cannot distinguish
them;
 the cause chain is technically correct (the `cause` field
preserves the inner error),
 but the human-readable summary loses
the disambiguation.

The harness does not merge or deduplicate suites with the same name
(unlike bun:
test's reporter),
 so the output prints both with their
shared name verbatim.

### Verification

Version under test:
 `@monochromatic-dev/test-harness-core` at workspace
HEAD.

Reproduce:

```ts
await describe({
  name: 'root',
  children: [
    describe({ name: 'dup', children: [it({ name: 'a', fn: () => {
      throw new Error('boom',);
    }, },),], },),
    describe({ name: 'dup', children: [it({ name: 'b', fn: () => {}, },),], },),
  ],
},);
// Output: both "dup" lines visible; failure attribution requires
// reading the cause chain manually.
```

### Verified workaround

Keep `describe` names unique within a scope:

- Prefer `functionName.name` for imported functions;
   that is
  inherently unique per import.
- Use region markers (`//region X` / `//endregion`) to visually
  separate groups and catch accidental duplication during review.

Tradeoff:
 enforced by convention,
 not by the runner.
 A future
linter rule could detect duplicates but is not in place.

### What does not work

- Renaming nested suites only:
   top-level duplicates still produce
  the ambiguous chain.
- Relying on the cause chain in output:
   present but not surfaced
  by the default reporter.

### Why we do not file this upstream

The runner is in-tree.
 Walking the 5 constraints as if `module-test`
were the upstream:

1.  **Is it really upstream's fault?
    ** Borderline.
     Deduplication
    matching bun:
    test's reporter would help but adds complexity.
2.  **Can upstream fix it?
    ** Yes;
     augment the reporter to mark
    duplicates with a suffix or path.
3.  **Are they supporting this use case?
    ** The runner accepts any
    name;
     duplicates are not rejected.
4.  **Will they likely fix it?
    ** Not at this priority.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 keep the convention rule.

---

## Bug 2: Multiple top-level `await describe(...)` calls; first failure skips the rest

### Symptom

A test file has multiple top-level `await describe({ ... })` calls.
The first one fails;
 the second never runs:

```text
FAIL  suite A > test-1
(no further output for suite B; process exits)
```

### Root cause

`describe` throws on child failure.
 Multiple `await describe(...)`
at the top level means a throw in the first kills the surrounding
module evaluation,
 so subsequent describes never reach the
scheduling step.

### Verification

```ts
await describe({ name: 'A', children: [it({ name: 'fails', fn: () => {
  throw new Error('boom',);
}, },),], },);
await describe({ name: 'B',
  children: [it({ name: 'never runs', fn: () => {}, },),], },);
// Suite B is silently skipped.
```

### Verified workaround

Wrap all suites in a single empty-named parent describe.
 The empty
parent runs children through `Promise.allSettled`,
 so every suite
executes regardless of earlier failures:

```ts
await describe({
  name: '',
  children: [
    describe({ name: 'A', children: [...] }),
    describe({ name: 'B', children: [...] }),
  ],
});
```

Tradeoff:
 every test file has a wrapper at the top.
 Trivial
boilerplate but easy to forget;
 reviewing for two top-level
`await describe(...)` calls is the simplest catch.

### What does not work

- Wrapping each describe in its own try/catch:
   catches the error,
  but the runner does not see the original failure for reporting.
- Using `Promise.all` over `Promise.allSettled`:
   same fail-fast
  behaviour as sequential awaits.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** Borderline.
     The throw-on-fail
    contract is intentional;
     the wrapper pattern is the documented
    shape.
2.  **Can upstream fix it?
    ** Could change top-level behaviour to use
    `allSettled` implicitly,
     but that hides bugs where the file
    forgets to declare a parent.
3.  **Are they supporting this use case?
    ** Yes;
     the wrapper pattern
    is documented.
4.  **Will they likely fix it?
    ** No.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 keep the wrapper convention.

---

## Bug 3: Concurrent tests sharing mutable state interfere

### Symptom

Tests pass individually but fail when run together.
 Assertions see
values written by another test.
 Failures depend on execution order
and are flaky on reruns.

### Root cause

Children of a describe run concurrently by default (via
`Promise.allSettled`).
 Tests that mutate module-level state
(counters,
 caches,
 singletons) race with their siblings.

### Verification

```ts
// module under test:
let counter = 0;
export function increment(): number {
  return ++counter;
}

// tests:
describe({
  name: 'counter',
  children: [
    it({ name: 'first call returns 1',
      fn: ({ expect, },) => expect(increment(),).toBe(1,), },),
    it({ name: 'second call returns 2',
      fn: ({ expect, },) => expect(increment(),).toBe(2,), },),
  ],
},);
// Both run concurrently; one of them returns 2 first, the other returns 1,
// the assertion ordering breaks.
```

### Verified workaround

Isolate state per test (preferred):
 create fresh state inside each
`fn`,
 so siblings cannot interfere.

When shared state is unavoidable,
 set `concurrency: 1` on the
describe:

```ts
describe({
  name: 'shared resource',
  concurrency: 1,
  children: [
    it({ name: 'first', fn: async () => {/* ... */}, },),
    it({ name: 'second', fn: async () => {/* ... */}, },),
  ],
},);
```

Children are lazy descriptors and dispatched by the parent,
 so no
thunk wrapping is needed;
 the parent's effective concurrency is
inherited by nested describes.

Tradeoff:
 `concurrency: 1` serialises the whole describe,
 which
slows the suite.
 Use it only when isolation is genuinely
impractical.

### What does not work

- Restructuring tests to "assume execution order":
   the runner does
  not promise order;
   even with `concurrency: 1`,
   relying on order
  outside the explicit sequencing is fragile.
- Wrapping shared state in a `Map` keyed by test name:
   works but
  spreads state-tracking concerns into every test;
   the runner-level
  `concurrency: 1` is cleaner.

### Why we do not file this upstream

The default-concurrent behaviour is intentional.
 Walking the
constraints concludes:
 no upstream report;
 document the convention.

---

## Bug 4: `expect.assertions(n)` only works when destructured from the test context

### Symptom

`expect.assertions(n)` is not available on the imported `expect`:

```ts
import { expect, } from '@monochromatic-dev/test-harness-core';
// `expect.assertions` is undefined.
```

Worse,
 async tests where one branch never reaches its assertion
silently pass,
 because no count is being enforced.

### Root cause

The global `expect` (imported directly from the package) does not
support assertion counting.
 Only the scoped `expect` from the test
context (passed via `fn`'s parameter) tracks counts;
 that is the
instance that has the `assertions` method.

### Verification

```ts
import {
  describe,
  expect as globalExpect,
  it,
} from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'count check',
  children: [
    it({
      name: 'scoped',
      fn: async ({ expect, },) => {
        expect.assertions(2,); // works
        expect(await fetchA(),).toBe('a',);
        expect(await fetchB(),).toBe('b',);
      },
    },),
    it({
      name: 'global',
      fn: async () => {
        globalExpect.assertions(2,); // TypeError or undefined
      },
    },),
  ],
},);
```

### Verified workaround

Always destructure `expect` from the `fn` parameter:

```ts
it({
  name: 'all branches assert',
  fn: async ({ expect, },) => {
    expect.assertions(2,);
    expect(await fetchA(),).toBe('a',);
    expect(await fetchB(),).toBe('b',);
  },
},);
```

Tradeoff:
 every test that needs assertion counting must use the
scoped form.
 The global `expect` is still useful for ad-hoc
assertions,
 but should not be used inside `fn`.

### What does not work

- Patching `expect.assertions` onto the global instance:
   the
  counter state lives in the per-test context,
   so a shimmed global
  cannot find the right counter.
- Relying on the test runner to detect unreached assertions
  heuristically:
   not implemented;
   assertion counting is the only
  guarantee.

### Why we do not file this upstream

In-tree harness.
 The scoped-only design is intentional (state lives
in the context).
 Walking the constraints concludes no upstream
report.

---

## Bug 5: `sinon` imported directly leaks stubs across tests

### Symptom

A stub installed in test A is still installed when test B runs.
Test B fails with "TypeError:
 Attempted to wrap …" because the
already-wrapped method cannot be wrapped a second time.

### Root cause

The directly-imported `sinon` is the singleton;
 stubs installed on
it persist until explicitly restored.
 The per-test `sinon` exposed
via context is wrapped in a sandbox that auto-restores at the end
of the test (via `await using`).

### Verification

```ts
import sinon from 'sinon';

it({
  name: 'stubs leak',
  fn: async () => {
    sinon.stub(obj, 'method',).returns('mocked',);
    // No restore; the stub persists into the next test.
  },
},);
```

### Verified workaround

Use `ctx.sinon` from the test context:

```ts
it({
  name: 'stubs cleanly',
  fn: async ({ sinon, },) => {
    sinon.stub(obj, 'method',).returns('mocked',);
    // Sandbox auto-restores via `await using`.
  },
},);
```

Tradeoff:
 every test that needs stubs must destructure `sinon` from
the context.
 Refactoring legacy tests that import `sinon` directly
is mechanical but tedious.

### What does not work

- Calling `sinon.restore()` manually in a `finally` block:
   works
  but reintroduces the boilerplate the sandbox replaced;
   missing a
  single `finally` re-creates the leak.
- Wrapping every test in `beforeEach`/`afterEach`:
   this runner does
  not have lifecycle hooks;
   the sandbox model is the substitute.

### Why we do not file this upstream

In-tree convention;
 sinon itself is upstream-correct.
 No
upstream report.

---

## Bug 6: A control that passes alone fails in the parallel suite

### Symptom

`mise run //package/pi-shared/subagent-driver:test:unit` fails on a control that starts a real child,
or on one that measures the clock,
while the same file alone passes:

```text
[workOutlivingTheTurnIn] [names a real child a shell started ...] [FAIL] (609ms)
  Error: ENOENT: no such file or directory, open '/proc/3894226/comm'
[drive-pi-subagent] [ends a run at its wall clock while the build it left still runs ...] [FAIL] (18.8s)
  AssertionError: expected +0 to equal 1
```

Two causes,
measured on 2026-11-26 while resolving issue 6504,
and a third,
measured the same day while resolving issue 6508.

### Cause 1: a `/proc` read of a process that ended between the listing and the read

A shell started as `sh -lc 'sleep 60; :'` runs this machine's profile scripts first,
and each of those scripts forks a child that ends before the work starts.
A control that takes the first number out of `/proc/<pid>/task/<pid>/children` can take one of those,
and its `comm` is gone by the time the control reads it:

```text
$ bash -lc 'sleep 6; :' &   # then sample the children file every 5 ms
      1 3915398 comm=bash
      1 3915408 comm=bash
      3 3915418 comm=bash
      1 3915439 comm=GONE
    230 3915442 comm=sleep
```

Linux answers `ENOENT` for a record already removed,
and `ESRCH` for one whose task it released;
the driver counts both as a process that is gone
(`package/pi-shared/subagent-driver/src/process-ended-codes.ts`).
The read's own mechanism is in
`package/pi-shared/subagent-driver/doc/troubleshooting/a-process-that-ends-while-its-record-is-read.md`.
The control does the same:
it waits for the child that runs the program it started,
treats a child whose program cannot be read as one of the profile scripts,
and owns that child before it reads any record of it,
because that child outlives the test.

### Cause 2: a wall clock sized below the start-up the suite makes

`--wall-seconds` bounds Pi's start-up as well as the watch,
as `package/pi-shared/subagent-driver/README.md` says under that option.
So a run whose subject is its own clock has to reach that state before the clock ends.
How long the start-up takes is what the other test files leave the machine:
the driver took its first prompt at 473 ms on an idle host,
and the moment each control measures took about 25 s inside the full suite,
so a fixed 8 s ended a run before its prompt was ever sent.
A control sizes its clock from a run it makes first,
of the same driver and stand-in,
which reaches the same feed line and ends of its own accord,
and leaves that figure room to take half again.
The work such a run leaves behind lives longer than the clock,
so the run still ends on its clock rather than after its work.

### Cause 3: a control that spends its own bound in real time

A control that arms a bound and then sleeps almost all of it has given the host the decision:
`package/pi-shared/subagent-driver/src/wait-for-an-answer.unit.test.ts`
armed a 100 ms silence bound,
slept 90 ms of it,
and only then paused the silence;
it failed once in a full parallel run on 2026-11-26:

```text
[startWaitingForAnAnswer] [the silence starts afresh when a compaction ends, so a wait that was nearly out gets the whole bound again] [FAIL] (178ms)
  AssertionError: expected true to equal false  @wait-for-an-answer.unit.test.ts:260
```

Issue 6508 read that as a resume more than 10 ms late:
the pause reaches `pauseSilence` after the bound settled,
and `pauseSilence` returns on `state.over` without arming anything.
That reading is not what decides the outcome.
A probe of the control's own sequence,
run against the package's built code,
stalled its own thread at each candidate place with `Atomics.wait`,
a block rather than a spin:

```text
gap between arming the bound and registering the sleep: p99 0.011 ms, max 0.110 ms over 500 passes
control as written, no stall:                                            0 of 100 failed
40 ms stall inside the sleep's continuation, after the sleep came back: 0 of 100 failed
100 ms block across both due times, sleep and bound due together:       0 of 100 failed
 5 ms stall before the sleep is registered:                             0 of 100 failed
11 ms stall before the sleep is registered:                           100 of 100 failed
```

A late continuation cannot lose:
when the sleep and the bound are both due,
Node runs the sleep's continuation,
whose `clearTimeout` retires the bound,
before the bound's own callback.
What loses is a stall greater than the 10 ms the control leaves itself
**between arming the bound and registering the sleep**,
which is one synchronous stretch of a few statements:
a preemption there,
a stop-the-world pause or
a page-fault storm,
any of which delays the sleep's due time past the bound's.
Every failure the probe made that way ended `silence`,
which is the settle the issue quotes.
So the window is 10 ms wide,
and ordinary runs cross it with 0.011 ms to spare,
which is why it failed once in a full suite and never alone.

### What a control does instead: own the clock it reads

The same control now installs a clock of its own
through the harness's context-owned `Date.now` stub,
which `package/test-harness/core/README.md` describes under "Stubs and spies".
A bound is then due at a moment the control decides:

```ts
const clock: { now: number } = { now: Date.now(), };
sinon.stub(Date, 'now',)
  .callsFake(() => clock.now,);
```

That stub follows the reading async context,
so the other controls in the same file keep the real clock
while running concurrently in the same process.
A sibling read the real clock while the stubbing child held the fake,
which is what makes the stub usable in a file whose controls run together.
The control then advances its own clock over the 90 ms of served silence
and over a 300 ms compaction that outlives the bound,
and no timer can be due between arming the bound and pausing the silence,
because that stretch yields nothing.
It asserts what the bound owes:
not settled after `resumeSilence`,
still not settled after 90 ms of the bound is served again,
and `silence` with `no answer in 100 ms` once the whole bound is served.
No shipped code changed;
the behaviour held is the README's own words,
"`compaction_start` stops the count and `compaction_end` starts it again,
which means a compaction Pi is running never spends the silence a prompt is allowed".

A control proves nothing until it is shown to fail:
removing `state.silentSince = Date.now()` from `resumeSilence`
(`package/pi-shared/subagent-driver/src/wait-for-an-answer.ts`)
makes this control fail deterministically in 113 ms,
at the assertion after `resumeSilence`:
`AssertionError: expected true to equal false`.

### Verification

Each control fails in a full run of the package's unit suite when its cause is present,
and passes in it after the change.

```text
$ mise run //package/pi-shared/subagent-driver:test:unit
$ mise run //package/pi-shared/subagent-driver:test:unit -- \
    src/drive-pi-subagent.unit.test.ts src/work-the-run-left-running.unit.test.ts
```

The second run needs both files named:
the harness runs a whole file and has no filter that would run one test alone.

The clock control needs only its own file:
`mise run //package/pi-shared/subagent-driver:test:unit -- src/wait-for-an-answer.unit.test.ts`,
which passes all six controls in it.
