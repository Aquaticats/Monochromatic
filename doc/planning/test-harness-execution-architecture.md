# Test harness execution architecture

## Status and purpose

This is a design proposal for review,
not an implementation.
It exists because the wall-clock bound remediation plan removes symptoms
while leaving intact the execution model that produces them.
A fresh reviewer should treat it as a proposal with alternatives,
probes,
and owner decisions.

It supersedes specific recommendations in
[the remediation plan](wall-clock-bound-test-remedies.md),
[the plan review](wall-clock-bound-test-remedies-review.md),
and the [investigation handover](../handover/wall-clock-bound-unit-tests.md):
the deferred interval API,
the `concurrency: 1` fake-timer guidance,
and the per-site framing of items 2 and 3.
The review's factual findings (inventory,
measurements,
citation corrections)
stand and are reused here.

## The model behind the symptoms

The current harness represents a test execution as a promise,
a deadline as a promise race,
time as ambient globals,
dispatch as a thenable,
results as names,
and supervision as the absence of a deadline.
Every symptom in the remediation plan follows from those choices.

### A deadline is reported, not enforced

`runItAttempt` wraps the body in `withTimeout`
(`package/test-harness/core/src/it-attempt.ts:83`),
`describe` wraps the settle promise the same way
(`package/test-harness/core/src/describe.ts:316`),
and the logger wraps sink verification in `withHostTimeout`,
which delegates to `withTimeout`
(`package/logger/core/src/with-host-timeout.ts:26`).

`withTimeout` uses `Promise.race`:
it settles the race,
not the work.
When the deadline wins:

- the body keeps running and keeps its side effects;
- the attempt's sandbox closes while the body still holds stubs and logs;
- a pending write,
  verification,
  or flush continues past the verdict;
- a late rejection can become an unhandled rejection;
- a top-level await that never settles makes Node exit 13 with no `[FAIL]`
  diagnostic,
  which is the incident shape in
  `doc/troubleshooting/cli-git-logger-sink-verify-timeout.md`.

The `withTimeout` cross-domain clear defect ([issue 1771][i1771]) is one
instance of this ownership gap:
a timer scheduled in one time domain is cleared through another,
because neither the timer nor the disposer has an owner.

A synchronous hang is worse.
No promise-level mechanism can interrupt `while (true) {}`,
and the runner has no per-file deadline,
so one hung file occupies a worker
until a human intervenes.

### Time is ambient

Tests and code under test read `globalThis.setTimeout`,
`clearTimeout`,
`Date`,
and `performance`.
Determinism therefore requires replacing process globals.
Sinon's fake timers do exactly that,
which produces the sequencing rule,
the native-clear hazard,
and the descriptor deadlock trap.
The README's recipe is incomplete because the model makes completeness
impossible:
a process-global replacement cannot be per-test.

The inventory is small enough to change and large enough to matter:
17 default-run unit files call `setTimeout`,
and two default-run files install fake timers,
one for `Date` only and one for `setTimeout` and `clearTimeout`
(`package/test-harness/core/src/sinon-clock.unit.test.ts`).
A third file passes `useFakeTimers: false` while testing sandbox options.
Code under test is much larger:
231 non-test source files read ambient time.
Full dependency injection across all of it is not proportionate.

### Execution and observation are conflated

`describe` and `it` return lazy thenables.
The descriptor's `.then` both starts an execution and reports its result,
so every consumption re-runs the body
(`package/test-harness/core/src/descriptor.ts`).
The harness self-test shows the consequence:
the same failing child emits its `[FAIL]` lines twice
because two `.rejects` assertions consume one descriptor twice.
Under a fake clock the second consumption schedules timers nobody ticks,
which was the P1b probe deadlock.

Results carry `{ name }` only.
Per-attempt and suite durations are computed for log lines,
not returned,
so tests that want to prove concurrency must time the suite
from outside.
That is why the describe test asserts `elapsed < 180`.

### The runner supervises nothing

`mise.toml`'s `run_test_files` spawns `node <file>` per test file
and awaits `once(proc, 'exit')`
(`mise.toml:592` and `:593`).
There is no per-file timeout and no process-group kill.
The 32-process width is intended throughput,
but a file that hangs
occupies its worker for the rest of the run.
`mise.toml` also records that per-task timeouts are not implemented
(`mise.toml:341`).

### Verdicts are wall-clock assertions

Seven default-run assertions derive a verdict from real elapsed time.
The plan classifies them as surrogate,
discriminating,
or complexity guards,
then keeps three.
That classification is about which real-time dependencies are tolerable,
not about removing the dependency from the execution model.

## What promises can and cannot do

Promises represent eventual results.
They do not provide:

- cancellation or interruption of the underlying work;
- ownership of work after a race loses;
- scheduling control,
  virtual time,
  or timer observation;
- preemption of synchronous code;
- a distinction between observing a result and starting an execution.

A timeout built only from promises can therefore report lateness.
It cannot enforce a deadline,
cannot clean up the timed-out work,
and cannot make the timing deterministic.
Enforcement needs an owner outside the promise:
an execution scope with a clock,
a cancellation signal,
cleanup ownership,
an event journal,
and a supervisor.
`async` and `await` remain the authoring style;
the change is who owns execution.

## Requirements

- **R1 Owned execution**:
  every attempt has a scope with explicit states
  (created,
  running,
  waiting,
  cancelled,
  settled,
  disposed) and an owner.
- **R2 Deterministic time**:
  code under test can run on a virtual clock
  without replacing process globals.
- **R3 Cancellation**:
  the scope exposes an `AbortSignal`;
  a deadline aborts it and then awaits cleanup.
- **R4 Hard deadline**:
  uncooperative or synchronous work is bounded
  by the runner,
  which kills the file process group.
- **R5 Observation**:
  results carry execution events
  (task start and end,
  waits,
  virtual advances,
  cancellations,
  children,
  pending work),
  not only names.
- **R6 Explicit dispatch**:
  a descriptor executes once per dispatch;
  observing a result does not re-run it.
- **R7 Verdict purity**:
  no default-run verdict depends on real elapsed time.
  Real-time supervision is allowed;
  audited real-time observations are allowed
  by exception.
- **R8 Enforcement**:
  a repo-owned check rejects new real-time verdict bounds,
  with a small reviewable allowlist.
- **R9 Compatibility**:
  existing unit files keep working;
  new APIs are additive;
  the default clock stays real.

## Architecture

### Execution scope

One scope per attempt,
created by the harness.
A nested suite may create child scopes that inherit the clock,
cancellation,
and journal.

```ts
// package/test-harness/core/src/scope.ts (sketch)
type ScopeState = 'created' | 'running' | 'waiting' | 'cancelled' | 'settled' | 'disposed';

type ExecutionScope = {
  readonly clock: Clock;
  readonly signal: AbortSignal;
  readonly journal: ExecutionJournal;
  track(cleanup: () => Promise<void>): void;
  abort(reason: Error): void;
  dispose(): Promise<void>;
};

type ExecutionJournal = {
  readonly events: readonly ExecutionEvent[];
  record(event: ExecutionEvent): void;
};

type ExecutionEvent =
  | { readonly kind: 'task-start'; readonly name: string; readonly at: number }
  | { readonly kind: 'task-end'; readonly name: string; readonly at: number }
  | { readonly kind: 'wait-start'; readonly ms: number; readonly at: number }
  | { readonly kind: 'wait-end'; readonly at: number }
  | { readonly kind: 'advance'; readonly ms: number; readonly at: number }
  | { readonly kind: 'cancel'; readonly reason: string; readonly at: number }
  | { readonly kind: 'child'; readonly name: string; readonly state: string };
```

The scope is the only place that owns timers,
cancellation,
and cleanup.
Late work is a scope violation:
after `dispose`,
any recorded event or tracked cleanup is reported as a defect
in the verdict,
because it means the test left work running.

### Clock

```ts
// package/test-harness/core/src/clock.ts (sketch)
type Clock = {
  now(): number;
  sleep(ms: number, options?: { readonly signal?: AbortSignal }): Promise<void>;
  setTimeout(callback: () => void, ms: number): TimerHandle;
  clearTimeout(handle: TimerHandle): void;
};
```

`RealClock` wraps host timers.
`VirtualClock` owns a due-time queue and exposes `advance(ms)` and
`runUntilIdle({ maxSteps })`.
`advance` fires due timers in time order and drains microtasks between them,
so a timer scheduled by a timer callback is observed deterministically.
The clock is runtime-neutral:
browsers use `RealClock`;
hosts without timers use a no-timer mode,
which the existing `withHostTimeout` guard already anticipates.

The clock is passed explicitly into code under test.
It is not installed into `globalThis`,
and it does not replace `Date` or
`performance`.
Tests that assert time use `scope.clock.now` or the journal.

### Cancellation and deadlines

A deadline is a scope event,
not a race:

1.  the scope schedules the deadline in its clock;
2.  when it fires,
    the scope aborts `signal` with a labeled reason;
3.  cooperative work observes the signal and stops;
4.  tracked cleanup runs,
    in reverse registration order;
5.  the scope settles and disposes;
6.  work that remains is journaled and reported.

Two real-time mechanisms remain,
both at the supervision layer:

- the in-process attempt timeout,
  which uses the host clock as a safety net
  and is not a verdict input;
- the runner's per-file budget,
  which terminates the process group.

The distinction matters:
a test verdict may not depend on real elapsed time,
but the harness may still use real time to stop uncooperative work.

### Dispatch and descriptors

`makeDescriptor` keeps a factory and a memoized first execution:

- the parent scheduler invokes `RUN_WITH_CONTEXT` explicitly,
  fresh per dispatch,
  so `repeats` still re-runs;
- `.then` on a descriptor returns the memoized native promise,
  so a test that awaits it twice observes one execution;
- consuming a descriptor after a completed dispatch does not re-run,
  and a second dispatch is an explicit scheduler action.

The scheduler becomes an explicit task loop rather than an array of already
started promises.
`Promise.allSettled` may remain an implementation detail for concurrency,
but each child is a journaled task with a start and an end event.

### Observation records

`ItResult` and `DescribeResult` gain execution records:

```ts
// sketch
type ItResult = {
  readonly name: string;
  readonly execution: ExecutionRecord;
};

type ExecutionRecord = {
  readonly startedAt: number;   // scope clock
  readonly endedAt: number;     // scope clock
  readonly events: readonly ExecutionEvent[];
};
```

Helpers can be plain functions or matchers:

- `overlap(recordA, recordB)` returns whether the intervals intersect;
- `peakConcurrency(record)` returns the maximum live child count;
- `virtualAdvance(record)` returns the total advanced milliseconds;
- `pending(record)` returns tracked work still open at settle.

The data is raw and the policy stays in tests.
No helper decides what "concurrent" means.

### Runner supervisor

The file runner keeps spawning one node process per test file,
but each process gets:

- a wall-clock budget;
- a process group,
  so a kill reaches children;
- an exit classification:
  passed,
  assertion failure,
  hang,
  crash,
  or signal;
- a structured result line that names the file and the class.

A hung file becomes a bounded failure,
not a stuck run.
Worker-per-attempt is not required for this;
the process boundary is the honest hard stop for synchronous work.

### Package seam

`module-async-time` is the narrow seam for timing code under test:

```ts
// sketch
wait(ms, { clock, signal }?)
withTimeout({ promise, ms, label, clock, signal }?)
```

The default clock is the host clock,
so production behavior is unchanged.
`withTimeout` captures the schedule and clear pair from the provided clock,
which also fixes [issue 1771][i1771] without ambient globals.
The harness passes its scope clock and signal.
The logger gains an optional clock (and signal where it abandons work),
and its flush deadline uses it.

Other code that reads ambient time is out of scope for virtualization.
Tests that exercise it stay on the real clock and are covered by R7's
audited exception.

### Enforcement

A repo-owned check scans default-run `*.unit.test.ts` files for:

- `toBeLessThan` and `toBeLessThanOrEqual` applied to a clock delta;
- comparison-in-argument shapes such as `expect(delta < N,).toBe(true,)`;
- `performance.now()` or `Date.now()` feeding an upper bound.

Allowed exceptions carry an inline annotation with a reason and are listed in
an allowlist file the check reads.
The check itself is tested with allowed and rejected fixtures
(GFP and QPC:
prove it can fail before trusting it).
This makes the issue's acceptance mechanically true instead of documented.

## Migration stages

Each stage is independently reviewable and reversible.

1.  **Inventory and probes.**
    Classify the seven sites,
    the 17 timer-using unit files,
    and the two fake-timer files.
    Run the probes in `Probes required before commitment`.
2.  **Scope and journal (additive).**
    Add the scope,
    clock,
    journal,
    and result fields.
    Existing tests keep the real clock and observe no change.
    Self-tests cover scope disposal and late work.
3.  **Runner supervisor.**
    Add the per-file budget and process-group kill.
    A deliberately hanging fixture must be killed and reported,
    and the positive control must be shown to hang without the supervisor.
4.  **`module-async-time` seam and logger migration.**
    Add the optional clock and signal;
    move the logger deadline tests to the scope clock;
    delete the `shouldClearNativeTimers` workaround and the `concurrency: 1`
    requirement.
5.  **Descriptor memoization.**
    Change `.then` to memoize;
    update the self-tests that currently rely on double execution.
6.  **Site remediation.**
    Describe overlap,
    gh cancellation shape,
    mvm marker,
    bash-poke outcome,
    wg complexity decision.
7.  **Enforcement and docs.**
    Add the check and allowlist;
    rewrite the fake-timer and duration guidance around the scope clock;
    document descriptor semantics and the runner supervisor.
8.  **Verification.**
    Per-site tests,
    contention batches,
    full suite,
    and the check's positive
    control.

The old plan's items map onto stages 4 and 6;
its docs item maps onto stage 7;
its `withTimeout` item maps onto stage 4 and remains valid either way.

## Alternatives considered

### A. Keep the approved plan

Pros:
smallest change,
approved scope,
no new API.
Cons:
retains global fake timers,
the sequencing burden,
the deadlock trap,
the race-based deadline,
and three real-time verdict bounds.
It fixes the observed cases without changing the model that produced them.

### B. Global timer multiplexer

Replace `globalThis.setTimeout` once with a router that selects a clock from
the current async context,
so any code is virtualized.
Pros:
transparent to code under test;
no package API changes.
Cons:
a process-wide monkeypatch that touches browser and Node splits,
identity-sensitive tests,
libraries,
and harness internals;
`Date` and `performance` remain unvirtualized;
debugging is opaque.
Rejected as the primary route.

### C. Worker per attempt

Run each attempt in a worker thread or child process with its own realm.
Pros:
true isolation,
hard deadlines,
global replacement is safe per worker.
Cons:
heavy for 1081 files;
context,
sandbox,
and log plumbing across IPC;
browser parity is lost;
startup cost dominates microsecond tests.
Rejected for now,
retained as a possible opt-in for diagnosing sync hangs.

### D. Full dependency injection of time

Thread a clock through every production module that reads time.
Pros:
no ambient time at all.
Cons:
231 files,
public API churn across published packages,
and most of that code is not exercised by the unit harness.
Rejected as disproportionate;
the `module-async-time` seam covers the timing
code the unit suite actually exercises.

### E. Scope clock, journal, and supervisor (recommended)

Pros:
removes global timer replacement from tests,
makes time and cancellation owned,
turns timing proofs into observations,
bounds hangs at the process boundary,
and keeps production defaults unchanged.
Cons:
new harness API surface,
virtual-clock semantics to get right,
descriptor behavior change,
migration work.

### F. Generator or effect interpreter

Test bodies yield effects (`sleep`,
`awaitCancellation`,
`observe`) interpreted
by the harness.
Pros:
complete control,
no ambient anything,
deterministic by construction.
Cons:
rewrites authoring for 1081 files and 137 consumers;
not compatible with plain `async` bodies that libraries expose.
Rejected as a rewrite,
recorded as the endpoint if promises prove unworkable.

Ranking:
E > A > B > C > D > F for this repository now,
because E changes ownership with bounded API surface,
A is safe but leaves the model,
B and C trade ownership for opaque or expensive isolation,
D is disproportionate,
and F is a rewrite.

## Probes required before commitment

1.  **Scope plus logger.**
    With a virtual scope clock and no global replacement,
    the first flush is unsettled at zero,
    resolves at the virtual deadline tick,
    and the second flush advances zero.
    Pass criterion:
    same observations as the P2 sinon probe,
    without globals.
2.  **Describe overlap.**
    Two timer-held children produce journal intervals with
    `max(starts) < min(ends)`;
    sequential dispatch does not.
3.  **Supervisor.**
    A fixture that never exits is killed at the budget and reported as a hang;
    a normally failing fixture is reported as an assertion failure.
    Positive control:
    without the budget,
    the fixture hangs.
4.  **Descriptor memoization.**
    Two `await`s observe one execution;
    `repeats: 2` still runs three times through explicit dispatch.
5.  **Attempt deadline with a wedged promise.**
    The scope aborts,
    runs tracked cleanup,
    and reports pending work;
    the process still exits cleanly.
6.  **Browser and no-timer hosts.**
    The scope compiles and runs with `RealClock` in the neutral artifact;
    a host without timers keeps the existing direct-await behavior.
7.  **Seam compatibility.**
    A `wait` or `withTimeout` call without a clock behaves exactly as today.
8.  **Enforcement check.**
    Rejected fixtures for each forbidden shape;
    accepted fixtures for the allowlist.

## Risks

- **Scope creep.**
  The harness change is larger than the approved plan.
  Stage it;
  each stage has value alone.
- **Virtual-clock semantics.**
  Microtask draining,
  timers scheduled by timers,
  and real I/O mixed with virtual time are the hard parts.
  The probes must precede the migration.
- **API surface.**
  Result records and `TestContext` additions are public.
  Keep them additive;
  `module-test` is published but not cleared for release.
- **Descriptor behavior change.**
  Tests that relied on re-execution will break
  by design;
  find them with the self-tests first.
- **Browser parity.**
  The scope clock must not assume Node.
- **Enforcement false positives.**
  Start with a narrow shape set and an
  allowlist;
  expand only with fixtures.
- **Real-time supervision.**
  The attempt timeout and runner budget are
  real-time mechanisms by necessity;
  keep them out of verdict logic and label
  them as supervision in docs.

## Site remediation under this architecture

- **describe concurrency**:
  assert interval overlap or peak from the journal;
  delete the 100 ms timers and the 180 ms bound.
- **logger deadlines**:
  use the scope clock;
  assert zero virtual advance and
  the exact deadline tick;
  no sinon,
  no `concurrency: 1`,
  no `shouldClearNativeTimers`.
- **gh-process cancellation**:
  assert the `ABORT_ERR` diagnostic;
  real subprocess,
  no clock.
- **mvm lingering stream**:
  make the lingering child print a marker at hold
  end;
  assert the marker is absent from the captured stdout.
  Real subprocess,
  no clock.
- **bash-poke cancellation**:
  assert `outcome.cancelled` plus a signalled
  outcome (`exitCode` absent or nonzero);
  real subprocess,
  no clock.
- **wg-quicker complexity**:
  real CPU work cannot be virtualized.
  Options:
  (a) add a test-only operation counter to the parser and assert linear work,
  (b) keep an audited real-time bound with a positive control,
  (c) move the guard to a separate performance task.
  Probe feasibility of (a) first;
  it is the only observation of the actual
  property.

## Acceptance mapping for issue 1762

Completion means:

- no surrogate real-elapsed upper bounds in the default run;
- every surviving real-time observation is audited,
  documented,
  and has a
  positive control;
- no `*.expensive.*` quarantine;
- the inventory is complete (seven assertions in six files);
- the harness check rejects new violations,
  and its own fixtures prove it can fail;
- the runner bounds hung files;
- the docs describe the scope clock and remove the fake-timer workarounds.

## Open owner decisions

These are the decisions that are genuinely yours,
not engineering choices:

1.  Approve the architecture investment (stages 1 to 3) or prefer the smaller
    plan with per-site fixes and accepted real-time bounds.
2.  Accept additive public API on `module-test`,
    `module-async-time`,
    and `module-logger` (all published but not cleared for release).
3.  Choose the enforcement policy:
    narrow shapes with an allowlist,
    or a broader ban with per-file annotations.
4.  Choose the default clock policy after probe 1:
    suite-level virtual opt-in,
    or virtual by default with a real-clock opt-out.
5.  Choose the wg-quicker route after the operation-counter probe.
6.  Set the verification budget:
    contention batches and a full suite run,
    or per-site plus the check.

## References

[i1771]: https://github.com/Aquaticats/Monochromatic/issues/1771
