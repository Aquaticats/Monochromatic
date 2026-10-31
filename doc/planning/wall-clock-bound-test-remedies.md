# Wall-clock bound test remediation plan (issue 1762)

## Purpose and review request

This plan covers the owner-approved remediation of the real-elapsed-time upper bounds
in the default unit-test suite,
plus the harness robustness fix discovered while investigating them.
It exists because the owner disagreed with the framing of
[issue 1762][i1762]
(filed by the automated repository review),
an investigation confirmed the disagreement,
and the owner approved a fix scope while pausing implementation pending review.

A new colleague is asked to review this plan before any code changes.
Implementation is paused;
nothing here is authorized to land until the owner lifts the pause after reading the review.
The section `Questions for the reviewer` lists the specific decisions the review should settle,
and `How to respond` says where to put the verdict.

Provenance,
in reading order for background:

- [issue 1762][i1762]
  and its [correction comment][c1762]:
  the original claim and the corrected record.
- [doc/handover/wall-clock-bound-unit-tests.md](../handover/wall-clock-bound-unit-tests.md):
  the full investigation record (measurements,
  probes,
  capability findings,
  ranked remedy space,
  owner decisions).
- [issue 1771][i1771]:
  the `withTimeout` cross-domain clear deadlock filed from the research.
- [doc/troubleshooting/cli-git-logger-sink-verify-timeout.md](../troubleshooting/cli-git-logger-sink-verify-timeout.md),
  section `Fake-timer cross-domain clear deadlock on 2026-10-18`:
  the incident record.
- [issue 510][i510]:
  the observed-failure precedent (a logger elapsed bound failing in a two-CPU container,
  later rewritten clock-free in-tree).

## Situation summary

Issue 1762 claims four default-run unit tests bound real wall-clock elapsed time
that the parallel `test:unit` runner
(up to `availableParallelism()` node processes,
`mise.toml:571`)
can stretch past,
and proposes quarantining them as `*.expensive.*` or replacing the bounds.

The investigation corrected the record:

- The inventory is seven real-elapsed upper-bound assertions in five files.
  The issue's line-based probe missed the two tightest:
  `FAST_FLUSH_MS = 40` at
  `package/logger/core/src/create-logger.unit.test.ts:811` and `:1076`,
  where the assertion spans two lines.
- The sites are three families,
  not one defect:
  - **Surrogate bounds** prove a property that has a load-insensitive observation
    (concurrency,
    cancellation).
    The clock adds flake risk and no proof power.
  - **Discriminating bounds** encode the property itself as
    "settles before the known slower alternative would"
    (return at process exit versus a 60 s stream hold;
    flush immediately versus re-waiting a 60 ms deadline).
    No timing-free formulation proves those.
  - **A complexity guard** (250 ms ceiling over a 0.03 ms median parse)
    detects quadratic regressions,
    not slow machines.
- Quarantine is rejected:
  `*.expensive.*` means genuinely expensive tests excluded from default runs
  (`mise.toml:669`);
  these are millisecond-scale,
  and excluding them would drop the harness's only default-run concurrency proof
  and the parser's linearity guard.
- Measured margins on a 32-core host:
  parse median 0.03 ms and worst 1.42 ms over 25 runs against the 250 ms bound;
  the describe concurrency span 101 ms against its 180 ms bound in seven isolated runs.
  Five contention batches at the runner's full 32-process width
  (five subject files plus 27 sampled of 1073)
  through `mise run test:unit` were green.
- Observed history exists for the family:
  [issue 510][i510] records
  `expected 108.877363 to be below 50`
  in a two-CPU container for a sibling logger bound,
  resolved by rewriting that test as a clock-free gated-promise observation
  (`Promise.withResolvers` start flags plus a release gate,
  visible at `create-logger.unit.test.ts:980` in the current tree).
  The repo's own precedent is observation rewrites,
  not quarantine.

Two harness facts constrain the design:

- The `withTimeout` deadlock ([#1771][i1771]):
  a test that installs sinon fake timers while the harness logger's bootstrap sink
  verification is in flight deadlocks the whole file
  (native timer handle cleared through the replaced global `clearTimeout`;
  sinon throws;
  the rejection escapes the init path;
  node exits 13 with no `[FAIL]` diagnostic).
  Verified workaround:
  `shouldClearNativeTimers: true`.
  The window is up to `verifyTimeoutMs` (5000 ms default)
  and host load widens it.
- Descriptor re-execution:
  `describe` returns a lazy descriptor that re-runs the suite on every `then`/`await`
  consumption.
  Under a fake clock,
  a second consumption schedules virtual timers nobody ticks (deadlock).
  Rule:
  consume a descriptor exactly once,
  keep the native promise from that single `.then`,
  await only the native promise.

Both constraints were validated by throwaway probes against the built dist;
condensed runnable forms are in the `Reviewer verification appendix`.

## Scope

In scope (owner-approved,
2026-10-18):

1.  The `withTimeout` domain-pairing fix ([#1771][i1771]).
2.  Five test-site remedies (work items 2 to 6 in this plan).
3.  Documentation updates:
    harness README fake-timer recipe and a new duration-upper-bound guidance section,
    descriptor re-execution semantics,
    matching testing-practices skill updates.

Deferred (explicitly out of the approved scope):

- Exposing per-child execution intervals on `DescribeResult`/`ItResult`
  (the harness already measures them at `it.ts:205/227` and `describe.ts:326-368`
  but returns only `{ name }`).
  Recorded as a design follow-up in the handover;
  a reviewer may recommend pulling it in,
  which needs a fresh owner decision.

Not proposed:

- No `*.expensive.*` quarantine of any site.
- No runner changes (32 processes on 32 cores is intended throughput).
- No upper-bound duration matcher.
  Position from the research:
  every site either has a timing-free observation or is a discriminating bound
  where plain `expect(elapsed).toBeLessThan(ALTERNATIVE)` with documented slack
  is the honest encoding;
  guidance covers the rest.
  The README's deliberate omission stands.

## Work items

### Item 1: `withTimeout` domain-paired clear (issue 1771)

File:
`package/async-time/core/src/with-timeout.ts`
(global `setTimeout` at line 56;
disposer clears through the global `clearTimeout`).

Change:
capture `setTimeout` and `clearTimeout` as a pair from one read of the global scope
at schedule time,
schedule through the captured `setTimeout`,
and clear through the captured `clearTimeout`,
so a global replacement mid-race cannot cross clock domains.
The reviewer question below covers the alternative of fixing at the harness
`withHostTimeout` boundary instead.

Guard test (in `package/async-time/core`):
install sinon fake timers **without** `shouldClearNativeTimers`,
start and complete a `withTimeout` race whose timer is still pending at completion,
assert clean disposal with no throw.
Per the GFP rule,
the guard test lands first and is proven failing before the fix
(expected symptom:
the sinon native-clear throw),
then the fix lands and the test passes.

Blast radius:
every `withTimeout` consumer,
including the harness attempt timeout (`it-attempt.ts:83`)
and the logger flush deadline.
Run the `module-async-time` suite,
the harness suite,
and the logger suite after the change.

### Item 2: describe concurrency proof without a clock

File:
`package/test-harness/core/src/describe.unit.test.ts`,
test `runs children concurrently` (bound at line 58).

Change:
replace the elapsed bound with the active/peak observation the same file already uses in
`limits concurrency to the given value` (lines 126-155):
each child increments a counter on entry and decrements before exit;
the test asserts the observed peak equals two.
The children keep their timer waits
(the delay holds both bodies in flight so overlap is observable;
an instant child would peak at one even under concurrent dispatch).
The delay may shrink from 100 ms since no bound depends on it anymore;
the review questions cover the value.

Sketch of the replacement body:

```ts
// package/test-harness/core/src/describe.unit.test.ts (sketch)
it({
  name: 'runs children concurrently',
  fn: async () => {
    const DELAY = 20;
    let active = 0;
    let peak = 0;
    const observe = async () => {
      active += 1;
      if (active > peak)
        peak = active;
      await new Promise(resolve => {
        setTimeout(resolve, DELAY,);
      },);
      active -= 1;
    };
    await describe({
      name: 'concurrent-suite',
      children: [
        it({ name: 'child-a', fn: observe, },),
        it({ name: 'child-b', fn: observe, },),
      ],
    },);
    expect(peak,).toBe(2,);
  },
},);
```

Why this is load-insensitive:
sequential dispatch cannot reach a peak of two
(child-a would decrement before child-b increments),
and contention cannot lower the peak of timer-held overlapping bodies.

### Item 3: logger fast-flush proofs on a virtual clock

File:
`package/logger/core/src/create-logger.unit.test.ts`,
tests at lines 811 and 1076 (`FAST_FLUSH_MS = 40` against a 60 ms deadline).

Change:
rewrite the two upper-bound tests to the probe-verified virtual-clock pattern:

- Group the clock-using tests under a `concurrency: 1` ancestor
  (fake timers are shared host replacements;
  the skill's sequencing rule).
- Install `sinon.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'], ... })`.
  Include `shouldClearNativeTimers: true` only while [#1771][i1771] is unlanded;
  remove it once item 1 lands (the review questions cover whether to keep it
  defensively).
- First flush:
  show it waiting at virtual time zero (unsettled after a microtask-only flush),
  then resolve it by ticking exactly the deadline.
  This replaces the existing lower-bound wait with an exact virtual event.
- Second flush:
  resolve it with zero virtual advance
  (flush,
  microtask-only settle,
  assert `clock.now` unchanged).
  Zero advance is strictly stronger than `elapsed < 40`:
  it proves the deadline was not re-armed and waited,
  not merely that the wait was short.

Probe-verified behavior (P2,
against the built dist,
exit 0):
first flush `{virtualNow: 0, firstSettled: false}` then `{virtualNow: 60, firstSettled: true}`;
second flush `{advancedMs: 0, secondSettled: true}`;
whole probe file 16 ms real time versus the current tests' real 60 ms deadline waits.

Open design point for the review:
whether the rest of the file's deadline family
(the `toBeGreaterThanOrEqual(SHORT_DEADLINE_MS - DEADLINE_TOLERANCE_MS)` tests)
migrates to the same virtual-clock ancestor for uniformity,
or stays on real timers.
The approved scope covers only the two upper-bound tests.

### Item 4: gh-process cancellation diagnostic instead of a bound

File:
`package/search-fetch/core/src/gh-process.unit.test.ts:359`,
test `throws and stops the child when cancellation arrives mid-run`.

Change:
delete the `expect(elapsed < LONG_SLEEP_MILLISECONDS,).toBe(true,)` bound and the
surrounding `performance.now()` measurement;
assert the cancellation diagnostic on the caught error instead.
The runner rethrows the raw cancellation rejection
(`gh-process.ts:233-237`,
`CANCELLATION_CODE = 'ABORT_ERR'` at `:82`),
while an unstopped child exits cleanly at the sleep end and resolves the promise,
which already fails the existing `expect(caught,).toBeInstanceOf(Error,)`.
So the shape assertion discriminates without any clock:

```ts
// package/search-fetch/core/src/gh-process.unit.test.ts (sketch)
expect(caught,).toBeInstanceOf(Error,);
expect((caught as NodeJS.ErrnoException).code,).toBe('ABORT_ERR',);
```

Implementation note:
per the EXT rule,
fetch the current Node `child_process` documentation at implementation time to confirm
which fields (`code`,
`killed`,
`signal`)
Node guarantees on an aborted `execFile` rejection,
and assert only the guaranteed ones.

### Item 5: mvm derived ceiling with documented discrimination

File:
`package/mvm/cli/src/libvirt-tools.unit.test.ts:243`,
test `returns when the command exits although a process it left behind keeps the output open`.

Change:
keep the bound (it encodes the property:
return at child exit,
not at stream close),
but derive it from the scenario instead of a bare `30_000`:

```ts
// package/mvm/cli/src/libvirt-tools.unit.test.ts (sketch)
/** The lingering process holds the output stream open for this long. */
const LINGERING_HOLD_MS = 60_000;
/** Midpoint ceiling: far above any real exit latency, far below the hold. */
const RETURN_CEILING_MS = LINGERING_HOLD_MS / 2;
```

and interpolate `LINGERING_HOLD_MS` into the spawned child's
`setTimeout(() => {}, ...)` string so the two cannot drift.
Add a comment stating the discrimination relationship
(a runner that waits for stream close cannot finish under the hold;
one that returns at exit finishes in milliseconds).
The review questions cover the ceiling choice.

### Item 6: wg-quicker documented complexity-guard slack

File:
`package/wg/cli-quicker/src/config.unit.test.ts:222`.

Change:
keep the 250 ms bound;
document it as a quadratic-regression detector with the measured slack
(median 0.03 ms,
worst 1.42 ms over 25 isolated runs,
roughly 176x worst-case margin;
a quadratic regression on 4000 prefixes takes seconds and trips the bound reliably).
No behavior change.
The optional ratio-scaling rewrite
(the `deep / shallow` pattern in
`package/upstream-fork/deepmerge-ts.fuzz/src/scale-complexity.local.unit.test.ts`)
is explicitly not planned:
the measured slack already makes the absolute bound robust.

### Item 7: documentation updates

- `package/test-harness/core/README.md`,
  `Fake timers` section:
  add the single-consumption rule for descriptors,
  the `concurrency: 1` sequencing rule for clock readers and writers,
  and the `shouldClearNativeTimers` caveat
  (kept as general guidance even after [#1771][i1771] lands,
  because tests' own native timers can still cross domains;
  reviewer question below).
- `package/test-harness/core/README.md`:
  new section on duration upper bounds,
  distinguishing surrogate bounds
  (rewrite as observations:
  peak counters,
  gated promises,
  cancellation diagnostics,
  virtual-clock zero advance)
  from discriminating bounds
  (keep;
  derive the ceiling from the slower alternative's constant;
  document the slack),
  mirroring the existing lower-bound recipe in `Duration and hang assertions`.
- `package/test-harness/core/README.md`:
  document descriptor re-execution semantics
  (every consumption re-runs;
  existing self-test output shows duplicated `[FAIL]` lines from double consumption).
- `.agents/skills/testing-practices/SKILL.md`:
  mirror the fake-timer constraints and the upper-bound guidance in the skill's
  async/sinon sections.
  Check `file-enforcer.config.ts` for management status before editing.
- `doc/troubleshooting/cli-git-logger-sink-verify-timeout.md`:
  after [#1771][i1771] lands,
  update the deadlock section with the fix reference
  (keeping the original text per the WR6 convention).

## Ordering and dependencies

```text
item 1 (withTimeout fix, guard test first per GFP)
  └─> item 3 (logger fake-clock rewrite; needs item 1 or the shouldClearNativeTimers workaround)
items 2, 4, 5, 6: independent of everything, any order
item 7 (docs): lands last, reflecting landed behavior;
  the README caveat may co-land with item 3 if item 1 slips
```

Commit discipline:
one commit per work item with scoped pathspecs;
`mise run //package/<path>:lint:types` after each TypeScript edit;
`mise run buildAndTest -- <file>` for test verification
(unit tests import built dist;
`mise run test` alone can exercise stale artifacts).

## Verification plan

1.  Per site:
    `mise run buildAndTest -- <test file>` green.
2.  Item 1 guard test:
    proven failing before the fix (GFP sequence),
    passing after;
    then the P1 probe pattern reruns green **without** `shouldClearNativeTimers`.
3.  Item 3:
    the rewritten logger tests pass;
    the virtual-advance assertions hold;
    the file's real runtime drops (probe reference:
    16 ms for the pattern).
4.  Contention rerun:
    five batches at the runner's full width through `mise run test:unit`
    with the five subject files included,
    all green.
5.  Full default suite once:
    `mise run test:unit`.
6.  Docs:
    `mise run format:monochromatic-lint -- <paths>` and a rendered check of every
    touched markdown file.

## Questions for the reviewer

1.  Item 1 layer:
    fix domain pairing inside `withTimeout`
    (all consumers benefit)
    or at the harness `withHostTimeout` boundary
    (smaller blast radius)?
    The plan proposes `withTimeout`.
2.  Item 2:
    is the peak-counter proof with a shortened delay (20 ms sketch value)
    the right landed form,
    or should the fake-clock single-tick proof
    (appendix pattern,
    deterministic and five times faster in real time)
    land instead as the recipe demonstration?
3.  Item 3 scope:
    migrate only the two upper-bound tests,
    or the file's whole deadline family to the virtual clock?
4.  Item 3 caveat:
    keep `shouldClearNativeTimers: true` defensively after [#1771][i1771] lands,
    or remove it to keep tests honest about the fixed behavior?
5.  Item 4:
    assert only `code === 'ABORT_ERR'`,
    or also Node's `killed`/`signal` fields if the current docs guarantee them?
6.  Item 5:
    is `HOLD / 2` the right ceiling,
    or a named absolute (for example 45 s) with a wider margin?
7.  Item 6:
    confirm documentation-only is acceptable for the complexity guard
    (the issue's acceptance allows it via documented slack justification).
8.  Acceptance restatement for issue 1762
    (in the correction comment):
    drop every surrogate bound,
    keep discriminating and complexity bounds with derived constants and documented slack,
    no quarantine.
    Is this a sound resolution of the issue's acceptance clause?
9.  Deferred interval exposure:
    leave deferred,
    or recommend the owner pull it into scope?

## How to respond

Write the review as a comment on [issue 1762][i1762]
(or per-item comments on [issue 1771][i1771] for item 1),
answering the numbered questions and flagging anything the plan gets wrong.
The owner reads the review,
then lifts or extends the implementation pause.
Until then this plan and the handover are the record;
no code changes.

## Reviewer verification appendix

Condensed runnable forms of the two probes that validated items 2 and 3.
Save as `.mjs` files anywhere and run with `node <file>` from the repository root's
machine (they import the built dist by absolute path;
adjust the two import paths for a different checkout).
Both patterns assume a green `mise run build` first.

Concurrency proof under a fake clock (validates the item 2 alternative in question 2):

```js
// probe: fake-clock concurrency proof (single descriptor consumption!)
import { describe, expect, it } from '<repo>/package/test-harness/core/dist/final/node/index.mjs';

const timerChild = name =>
  it({ name, fn: async () => new Promise(r => { setTimeout(r, 100); }) });

const observeOnce = descriptor => {
  let state = 'pending';
  const native = descriptor.then(() => { state = 'fulfilled'; }, () => { state = 'rejected'; });
  return [native, () => state];
};

await describe({
  name: 'probe',
  concurrency: 1,
  children: [
    it({
      name: 'concurrent suite settles in one 150ms virtual tick; sequential does not',
      fn: async ({ sinon }) => {
        const clock = sinon.useFakeTimers({
          toFake: ['setTimeout', 'clearTimeout'],
          shouldClearNativeTimers: true, // workaround for issue 1771 until it lands
        });
        const [concurrent, concurrentState] = observeOnce(describe({
          name: 'concurrent-suite',
          children: [timerChild('a'), timerChild('b')],
        }));
        await clock.tickAsync(150);
        expect(concurrentState()).toBe('fulfilled');
        await concurrent;

        const [sequential, sequentialState] = observeOnce(describe({
          name: 'sequential-suite',
          concurrency: 1,
          children: [timerChild('a'), timerChild('b')],
        }));
        await clock.tickAsync(150);
        expect(sequentialState()).toBe('pending'); // negative control
        await clock.tickAsync(100);
        expect(sequentialState()).toBe('fulfilled');
        await sequential;
      },
    }),
  ],
});
```

Logger flush proof on a virtual clock (validates the item 3 pattern):

```js
// probe: zero-virtual-advance second flush
import { describe, expect, it } from '<repo>/package/test-harness/core/dist/final/node/index.mjs';
import { createLogger } from '<repo>/package/logger/core/dist/final/node/index.mjs';

await describe({
  name: 'probe',
  concurrency: 1,
  children: [
    it({
      name: 'second flush resolves with zero virtual advance',
      fn: async ({ sinon }) => {
        const clock = sinon.useFakeTimers({
          toFake: ['setTimeout', 'clearTimeout'],
          shouldClearNativeTimers: true, // workaround for issue 1771 until it lands
        });
        sinon.stub(console, 'warn');
        const wedged = { verify: () => Promise.resolve(true), write: () => new Promise(() => {}) };
        const { logger, initPromise } = createLogger({ flushDeadlineMs: 60, sinks: [wedged] });
        await initPromise;
        logger.info('stuck');

        let firstSettled = false;
        const first = logger.flush().then(() => { firstSettled = true; });
        await clock.tickAsync(0);
        expect(firstSettled).toBe(false); // positive control: the wait is observable
        await clock.tickAsync(60);
        await first;

        const before = clock.now;
        let secondSettled = false;
        const second = logger.flush().then(() => { secondSettled = true; });
        await clock.tickAsync(0); // microtasks only, no virtual advance
        expect(secondSettled).toBe(true);
        expect(clock.now - before).toBe(0);
        await second;
      },
    }),
  ],
});
```

Expected probe behavior:
both files exit 0.
Without `shouldClearNativeTimers` and before item 1 lands,
the first file instead deadlocks with the sinon
`FakeTimers: clearTimeout was invoked to clear a native timer` throw and node exit 13,
which is itself the [#1771][i1771] repro.

[i1762]: https://github.com/Aquaticats/Monochromatic/issues/1762
[i1771]: https://github.com/Aquaticats/Monochromatic/issues/1771
[i510]: https://github.com/Aquaticats/Monochromatic/issues/510
[c1762]: https://github.com/Aquaticats/Monochromatic/issues/1762#issuecomment-6255436442
