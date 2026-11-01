# Wall-clock bound unit tests handover (issue 1762)

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/task-util` is now `package/monorepo-task/core-exec`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/deepmerge-ts.fuzz` is now `package/upstream-fork/deepmerge-ts.fuzz`;
> `package/module/test-sandbox` is now `package/test-harness/core-sandbox`;
> `package/module/async-time` is now `package/async-time/core`;
> `@monochromatic-dev/module-async-time` is now `@monochromatic-dev/async-time-core`;
> `package/module/logger` is now `package/logger/core`;
> `package/module/test` is now `package/test-harness/core`;
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mvm` is now `package/mvm/cli`;
> `package/cli/wg-quicker` is now `package/wg/cli-quicker`.

> Moved 2026-10-31 by the search-fetch product move into `package/search-fetch`
> (`doc/planning/search-fetch-cli.md`);
> paths and names below this note are historical:
> `package/agent-harness-shared/search-fetch` is now `package/search-fetch/core`.

## Purpose and status

This handover records an investigation of
[issue 1762][i1762],
which claims four default-run unit tests bound real wall-clock elapsed time
that the parallel `test:unit` runner can stretch past.

Phase 1 (complete,
commit `2765c13607`):
verified the issue against the current tree,
measured the two speed budgets in isolation,
reproduced the contention condition through the real runner,
and changed no code,
no tests,
and no tracker state.

Phase 2 (research complete,
decisions pending,
2026-10-18):
the user corrected the remedy framing.
The test harness itself (`package/module/test`,
published as `@monochromatic-dev/module-test`)
is also ours,
so remedies are not limited to per-site test edits:
the harness can provide load-insensitive timing capabilities by construction.
The user paused all site changes
("wait before changing the sites")
and asked for harness research before any fix lands.
A second user correction shapes the findings:
where the harness README documents an omission as deliberate,
that is a prior decision to re-examine with reasons,
not a veto;
a capability gap this research judges real is itself a finding.
The research is done:
two throwaway probes verified the fake-clock remedies against the built dist,
and surfaced two harness sharp edges
(a native-timer clear deadlock and a descriptor re-execution trap),
all recorded in the phase-2 sections.

Owner decisions (2026-10-18,
after the phase-2 research):

- Fix scope:
  the test sites plus harness robustness
  (ranked remedy space items 1-7);
  the interval-exposure API change (item 8) is deferred as a design follow-up.
- Logger-site remedy:
  the fake-clock zero-advance proof (P2-verified),
  not constant scaling.
- Tracker disposition:
  the native-timer clear deadlock is filed as its own issue ([#1771][i1771]);
  the descriptor re-execution trap folds into the issue 1762 correction
  and the docs work item.
- Code changes remain paused;
  only doc and issue-tracker updates were authorized in this turn.

Tracker state:
the correction comment is posted on [issue 1762][i1762] ([comment][c1762]),
and [#1771][i1771] carries the `withTimeout` cross-domain clear deadlock.
The implementation plan for external review is
[doc/planning/wall-clock-bound-test-remedies.md](../planning/wall-clock-bound-test-remedies.md);
a reviewer colleague answers its numbered questions before implementation starts.

How to respond:
read the review verdict on the plan,
then lift the pause to start implementation;
the plan's work items elaborate the ranked remedy space
(the resolution workflow in `doc/agent/issue-tracker.md` applies).

## The framing disagreement

The issue frames the situation as correct-but-fragile tests plus an environmental stressor:
the runner spawns up to `availableParallelism()` node processes,
contention can stretch a correct run past its budget,
and the headline prescription is to quarantine the four tests as `*.expensive.*`
following the `tsc-filter` precedent.

The user's counter-framing:
the defect belongs to our own package,
module,
and test code,
meaning the test designs themselves,
and the remedy belongs inside the tests rather than in excluding them from the default run.

Assessment:
the counter-framing is correct,
with three corrections to the issue's record.

1.  The inventory is incomplete.
    The two tightest real-elapsed upper bounds in the default suite
    (`FAST_FLUSH_MS = 40` in `package/module/logger/src/create-logger.unit.test.ts`)
    are absent from the issue,
    because its line-based probe cannot see assertions whose matcher sits on the next line.
2.  The family is not homogeneous.
    Two sites replace a wall-clock surrogate with a direct observation of the property,
    two sites encode the property as a timing comparison with no timing-free alternative,
    and one is a complexity guard with roughly 176x measured slack.
    The issue's first acceptance option
    (no upper-bound wall-clock assertion in the default run)
    would ban the legitimate second kind.
3.  The flakiness is hypothetical.
    No observed failure is cited,
    and this session's repro
    (five batches at the runner's full 32-process width through the real task)
    produced zero failures.
    Phase-2 qualification:
    issue [#510][i510] records an observed failure of a sibling real-elapsed bound
    in the logger file on a two-CPU container,
    so the risk is hypothetical for the seven inventoried assertions
    but observed for the family on smaller machines.

The issue is still right about the real kernel:
the `describe` concurrency proof (79 ms margin on a 101 ms span)
and the unlisted logger fast-flush bounds (40 ms ceiling,
20 ms discrimination margin)
are genuinely load-sensitive designs,
and the issue's alternative remedy,
load-insensitive proofs,
is the correct direction for those sites.

## Verified record

### What the issue gets right

- The four cited sites exist with the cited bounds:
  `package/module/test/src/describe.unit.test.ts:58` (180 ms),
  `package/cli/wg-quicker/src/config.unit.test.ts:222` (250 ms),
  `package/cli/mvm/src/libvirt-tools.unit.test.ts:243` (30 s),
  `package/agent-harness-shared/search-fetch/src/gh-process.unit.test.ts:359` (5 s).
- The runner behavior matches:
  `mise.toml:571` bounds workers at `Math.min(availableParallelism(), files.length)`
  (the issue cites `mise.toml:564`;
  the content matches at line 571 in the current tree),
  the default glob collects 1073 `*.unit.test.ts` files,
  and `mise.toml:669` excludes `*.expensive.*` from the default run.
- Category (testing) and difficulty (easy) are plausible.

### What verification corrected

- "The rest mock the clock or bound from below" is false.
  `create-logger.unit.test.ts` measures real `performance.now()` via its `timeFlush` helper
  (line 65)
  and upper-bounds it at lines 811 and 1076 with `FAST_FLUSH_MS = 40` (line 41).
  Only three default-run files mock the clock with `useFakeTimers`:
  `package/module/logger/src/sink/file-batch.unit.test.ts`,
  `package/module/test-sandbox/src/sandbox.unit.test.ts`,
  `package/module/test/src/sinon-clock.unit.test.ts`.
  The logger assertion spans two lines
  (`expect(elapsed,)` then `.toBeLessThan(FAST_FLUSH_MS,)`),
  which is why the issue's line-based probe missed it.
- "The tightest bounds are the 250 ms parse budget and the 180 ms concurrency proof" is false.
  The tightest real-elapsed upper bound in the default suite is the logger's 40 ms.
- The `tsc-filter` precedent does not transfer.
  `package/dev-script/task-util/src/tsc-filter.expensive.unit.test.ts:52` quarantines tests
  that spawn real `tsc` (seconds-long,
  CPU-heavy)
  and records observed flakiness.
  The six sites here are millisecond-scale timer,
  parse,
  and short-lived-child tests;
  none of the seven inventoried assertions has an observed failure on record.
  The logger file does carry observed history outside the inventory:
  issue [#510][i510] records
  `sinks verify concurrently rather than one after another`
  failing `expected 108.877363 to be below 50` in a two-CPU container,
  and that test has since been rewritten in-tree as a clock-free gated-promise
  observation (`Promise.withResolvers` start flags plus a release gate).
  The repo's chosen resolution of an actually-failing load-sensitive bound
  was an observation rewrite,
  not a quarantine.

### Complete inventory: seven assertions in five files

Family A,
wall-clock surrogate bounds where a load-insensitive observation of the same property exists:

- `package/module/test/src/describe.unit.test.ts:58`:
  elapsed < `MAX_SEQUENTIAL` (180 ms) around two 100 ms timer children.
  Isolated measurement:
  101 ms every run.
  The same file already proves concurrency limits without any clock
  via active/peak counters in `limits concurrency to the given value` (lines 126-155).
- `package/agent-harness-shared/search-fetch/src/gh-process.unit.test.ts:359`:
  `elapsed < LONG_SLEEP_MILLISECONDS` (5000,
  line 44) after aborting at 60 ms (line 64).
  The bound is redundant:
  on abort the runner rethrows Node's rejection carrying `code: 'ABORT_ERR'`
  (`gh-process.ts:82` and the cancellation branch at lines 233-237),
  while an unstopped child exits cleanly at 5 s and resolves the promise,
  so the test's existing `caught instanceof Error` assertion already fails
  in the world where cancellation did not stop the child.

Family B,
discriminating bounds where the elapsed comparison is itself the property:

- `package/cli/mvm/src/libvirt-tools.unit.test.ts:243`:
  spawn must return at child exit rather than wait for a stream a lingering process holds
  for 60 s;
  the 30 s bound separates the two behaviors and has roughly 29 s of slack
  over the normal sub-second return.
- `package/module/logger/src/create-logger.unit.test.ts:811` and `:1076`:
  a second flush must not wait out the 60 ms deadline
  (`SHORT_DEADLINE_MS`,
  line 29)
  again;
  the 40 ms bound sits between immediate return and re-waited deadline.
  Discrimination margin is 20 ms and load margin is 40 ms:
  the tightest real-elapsed bound in the default suite,
  and unlisted by the issue.

Family C,
complexity guard:

- `package/cli/wg-quicker/src/config.unit.test.ts:222`:
  250 ms budget for parsing 4000 `AllowedIPs` prefixes,
  named `parses a very large AllowedIPs value without quadratic slowdown`.
  Measured in isolation:
  median 0.03 ms,
  worst 1.42 ms over 25 runs,
  so the bound carries roughly 176x slack against the worst observed run.
  A quadratic regression would take seconds and trip it reliably.

### Measurements

Isolated runs on this 32-core machine,
otherwise idle:

- wg-quicker parse,
  importing `package/cli/wg-quicker/dist/final/node/config.mjs`,
  25 iterations:
  min 0.01 ms,
  median 0.03 ms,
  p95 0.19 ms,
  max 1.42 ms against the 250 ms bound.
- describe concurrency proof,
  seven full-file runs of `node src/describe.unit.test.ts` from `package/module/test`:
  the inner `[concurrent-suite] [PASS] child-a, child-b` span measured 101 ms in every run
  against the 180 ms bound.

Contention repro through the real runner:

- Five iterations of `mise run test:unit -- <32 files>`,
  where the batch is the five files holding the seven assertions
  plus 27 stride-sampled files from the 1073-file default glob,
  reaching the runner's full 32-process width.
- All five iterations exited 0 with no failed-file list and no assertion failures.
- Zero failures in five trials is weak evidence of absence.
  A full-suite run sustains contention far longer than one 32-file batch,
  and a CI machine with fewer cores has a different contention profile.
  Issue [#510][i510] is the existence proof for the smaller-machine case:
  a sibling real-elapsed bound in `create-logger.unit.test.ts`
  failed in a two-CPU container,
  and its clock-free rewrite is the in-repo precedent for the remedy plan.

## Phase 2 correction: the harness itself is ours

The phase-1 remedy directions all live inside test files.
The user corrected that framing:
this repo also owns the test harness (`package/module/test`),
so the remedy space includes harness capabilities,
not just per-site test edits.
This also sharpens the issue's own half-heard suggestion
("inject a fake scheduler")
for the describe site:
we own the scheduler,
so a deterministic substitute is a design choice,
not an external dependency.
The inventory corrections,
the quarantine rejection,
and the measurements from phase 1 all stand unchanged.

### Verified harness facts

- Fake timers are a first-class documented capability:
  the README section `Fake timers` shows `sinon.useFakeTimers()` through the per-attempt
  context sandbox,
  with a `clock.tick()` recipe.
- The README section `Duration and hang assertions` states the harness intentionally omits
  duration matchers (`toResolveWithin`,
  `expect.poll`,
  `vi.waitFor`),
  and its only documented duration pattern is a lower bound
  (`performance.now()` plus `toBeGreaterThan`).
  No sanctioned upper-bound pattern exists anywhere in the harness docs,
  so the seven real-elapsed upper bounds are ad-hoc inventions in test code.
- Fake timers are shared host replacements,
  not per-test clocks (README `Stubs and spies` ownership notes),
  so every reader and writer touching the clock needs a common `concurrency: 1` ancestor
  (testing-practices skill sequencing rule).
- The harness timeout wraps each body in `withTimeout`
  (`src/it-attempt.ts:83`),
  which calls the global `setTimeout` at invocation time
  (`package/module/async-time/src/with-timeout.ts:56`).
  A test that installs a fake clock therefore also puts its own timeout on virtual time;
  prototypes must account for that interaction.

### Phase-2 probe results

Two throwaway probes (scratch scripts in `~/temp/agent`,
marked PROTOTYPE,
run against the built dist;
the validated patterns are captured here and in the ranked remedy space)
answered the feasibility questions empirically.

P1 (`proto-p1-fakeclock-concurrency.mjs`,
exit 0):
under `sinon.useFakeTimers`,
one `clock.tickAsync(150)` settles two concurrent 100 ms timer children of a nested describe
(18.74 ms real time);
the sequential-suite negative control is still pending at virtual 150 ms and settles at 250 ms;
the real-clock baseline costs 100.74 ms real time.
The concurrency proof is deterministic,
roughly five times faster in real time,
and the negative control shows the probe can fail.

P2 (`proto-p2-fakeclock-flush.mjs`,
exit 0):
the first logger flush is shown waiting (unsettled at virtual 0 ms)
and resolves exactly at its virtual 60 ms deadline tick;
the second flush resolves with zero virtual-time advance.
That is strictly stronger than `elapsed < FAST_FLUSH_MS (40)` and load-insensitive;
the whole probe file runs in 16 ms real time.

The probes surfaced two harness sharp edges,
both reproduced deterministically:

- Native-timer clear crash.
  Installing fake timers while the harness logger's bootstrap sink verification is in flight
  deadlocks the entire file.
  The verify deadline timer is scheduled natively;
  its disposer then calls the globally replaced `clearTimeout` with the native handle,
  and `@sinonjs/fake-timers` throws
  `FakeTimers: clearTimeout was invoked to clear a native timer instead of one created by this library.`
  (emitted by sinon `clearTimer`,
  surfaced through the harness dist path
  `withTimeout` dispose,
  `withHostTimeout`,
  `verifyAndApply`,
  logger `initialize`).
  The rejection escapes as a file-level deadlock:
  unsettled top-level await,
  node exit 13,
  no `[FAIL]` diagnostic.
  Test-side workaround:
  `shouldClearNativeTimers: true`.
  Root-fix direction:
  schedule and clear in the same captured clock domain inside `withTimeout`
  (`package/module/async-time/src/with-timeout.ts`)
  so a global replacement mid-race cannot cross domains.
  Load connection:
  the crash window stays open while sink verification (fs I/O) is in flight at attempt start
  (`verifyTimeoutMs` defaults to 5000 ms),
  so parallel load widens it.
  `doc/troubleshooting/cli-git-logger-sink-verify-timeout.md`
  corroborates the emitter (`verifyAndApply` under `withHostTimeout`)
  and documents that same verification path stalling for seconds under host load
  in plain `test:unit` runs;
  the deadlock variant is recorded there as its own incident section.
  Latent exposure in the default suite:
  the three files that install fake timers
  (`package/module/test/src/sinon-clock.unit.test.ts`,
  `package/module/logger/src/sink/file-batch.unit.test.ts`,
  `package/module/test-sandbox/src/sandbox.unit.test.ts`);
  no default-run failure observed.
  This is a latent,
  worse form of issue 1762's own theme inside the harness:
  a deadlock instead of a false red.
- Descriptor re-execution on every consumption.
  `describe` returns a lazy descriptor that re-runs the suite on every `then`/`await`
  consumption.
  The harness self-test output already shows it:
  `wraps single child failure` consumes one rejection twice through two `.rejects`
  assertions,
  so its `[failing-suite] [FAIL]` lines appear twice in a plain run of
  `describe.unit.test.ts`.
  Under a fake clock a second consumption is a deadlock trap:
  the re-run schedules virtual timers nobody ticks.
  The probes hit exactly that trap (exit 13 after otherwise-passing states)
  and were fixed by the single-consumption rule:
  consume a descriptor exactly once,
  keep the native promise returned by that single `.then`,
  and await only the native promise.

### Capability findings

Recorded per the user's second correction:
a README omission is a prior decision to re-examine with reasons,
not a veto;
a capability gap this research judges real is itself a finding.

1.  The harness should expose the child execution intervals it already measures.
    `it.ts` records per-attempt `runStart` and `durationMs` (lines 205,
    227),
    and `describe.ts` records per-child `elapsedMs` plus suite duration (lines 326,
    352,
    368),
    yet `DescribeResult` carries only `{ name }` (describe.ts:26).
    Exposing per-child start and duration (or an overlap summary)
    turns concurrency proofs into observations instead of timing races:
    load-insensitive by construction,
    no clock,
    no re-execution trap.
    This is the harness-level fix for the describe site and benefits every future
    concurrency test,
    including the harness's own self-tests.
2.  The documented fake-timer recipe is incomplete and can deadlock files.
    The README `Fake timers` section shows bare `sinon.useFakeTimers()`.
    A supported recipe needs the `shouldClearNativeTimers` caveat,
    the single-consumption rule,
    and the `concurrency: 1` sequencing rule.
    The docs should carry all three regardless of whether the root fix lands.
3.  `withTimeout` should clear in the domain that scheduled.
    The root fix for the native-timer clear crash is a robustness defect of
    `@monochromatic-dev/module-async-time`
    that affects every consumer,
    not only tests:
    any code that swaps global timer functions mid-race can turn a routine dispose
    into an unhandled throw.
4.  Upper-bound duration matcher:
    the position from this research is not to add one.
    Every upper-bound site in this repo either has a timing-free observation
    (concurrency via overlap or peak counters,
    cancellation via the `ABORT_ERR` diagnostic,
    flush speed via zero virtual advance)
    or is a discriminating bound against a real-world slower alternative that no fake
    clock can virtualize (the mvm site's real subprocess).
    For the second kind,
    a plain `expect(elapsed).toBeLessThan(ALTERNATIVE)` with a documented justification
    is the honest encoding,
    and a bespoke matcher would add API surface without adding proof power.
    What the harness docs should gain is guidance instead:
    an upper-bound duration section distinguishing surrogate bounds
    (rewrite as observations)
    from discriminating bounds
    (keep,
    derive the ceiling from the alternative's constant,
    document the slack),
    mirroring the existing lower-bound recipe.

### Ranked remedy space

Per-site remedies,
revised by the probes:

1.  `describe.unit.test.ts:58`:
    peak-counter rewrite in the test (pattern already in the same file)
    as the immediate fix,
    since it needs no clock and no harness change;
    capability finding 1 supersedes it if interval exposure lands.
    The P1 fake-clock single-tick proof remains the documented recipe demonstration.
2.  `create-logger.unit.test.ts:811` and `:1076`:
    the P2-verified fake-clock zero-advance proof,
    which needs the deadline tests grouped under a `concurrency: 1` ancestor
    (fake timers are shared host replacements).
    Fallback if the restructuring is declined:
    scale the deadline constants roughly 10x.
3.  `gh-process.unit.test.ts:359`:
    assert the `ABORT_ERR` cancellation diagnostic and drop the bound
    (unchanged from phase 1;
    real subprocess,
    no clock involved).
4.  `libvirt-tools.unit.test.ts:243`:
    keep the bound,
    derive the ceiling from the 60 s hold constant,
    document the discrimination (unchanged).
5.  `config.unit.test.ts:222`:
    keep the bound,
    document the measured slack (unchanged;
    real CPU work cannot be virtualized).

Harness and module work items,
each needing its own authorization:

6.  `withTimeout` domain-paired clear fix plus a guard test proven failing before the fix.
7.  README and testing-practices updates:
    corrected fake-timer recipe,
    descriptor re-execution semantics,
    upper-bound duration guidance section.
8.  `DescribeResult` interval exposure as a design decision
    (raw per-child intervals versus an overlap summary;
    touches the result types and verdict logging).

Sites that spawn real subprocesses (mvm,
gh-process)
stay site-level remedies regardless:
fake timers do not virtualize real process lifetimes.

## Remedy directions

Provisional phase-1 proposal,
superseded in part by the phase-2 remedy space and paused pending the research queue.
Ordered by load sensitivity:

1.  `describe.unit.test.ts`:
    replace the elapsed bound with the peak-counter observation the same file already uses
    (active/peak incremented inside each child,
    assert `peak === 2`).
    Peak 2 proves overlap directly;
    sequential dispatch cannot reach it.
    Delete the timing.
2.  `create-logger.unit.test.ts`:
    scale the deadline constants by roughly 10x
    (for example a 600 ms deadline with a 400 ms fast-flush bound and the tolerance scaled to match),
    keeping the discrimination ratio while widening the absolute margin.
    Cost:
    a fraction of a second more per deadline test.
    Alternative:
    keep 60/40 and document the slack justification.
3.  `gh-process.unit.test.ts`:
    assert the cancellation diagnostic on the caught error
    (`code === 'ABORT_ERR'`)
    and delete the elapsed bound.
    The rejection shape discriminates kill-from-cancellation from run-to-completion
    without any clock.
4.  `libvirt-tools.unit.test.ts`:
    keep the bound,
    document the discrimination relationship
    (30 s ceiling versus the 60 s lingering hold)
    in a comment,
    and derive the ceiling from the hold constant instead of a bare `30_000`.
5.  `config.unit.test.ts`:
    keep the bound and document the measured slack.
    Optional:
    convert to a ratio-scaling check;
    the precedent pattern is the `deep / shallow` ratio assertion in
    `package/module/deepmerge-ts.fuzz/src/scale-complexity.local.unit.test.ts` (S1).
    The absolute bound is already robust at 176x worst-case slack,
    so this is cosmetic.

## Rejected ideas

- Quarantining these tests as `*.expensive.*`.
  The marker means genuinely expensive tests excluded from the default run
  (`mise.toml:669`);
  these are millisecond-scale.
  Quarantining would drop the harness's only default-run concurrency proof
  and the config parser's linearity guard from the default suite,
  eroding exactly the PKG and TCV signal the issue says it protects.
- The acceptance option "no upper-bound wall-clock assertion in the default run".
  Family B bounds encode the property under test
  (returns before the 60 s hold ends;
  flushes faster than the 60 ms deadline).
  No timing-free formulation proves those,
  so the documented-slack branch of the acceptance is the only workable one for them.
- Treating the runner as the defect site.
  32 processes on 32 cores is the intended test throughput,
  not pathological oversubscription;
  the fixes belong in the tests.

## Meta finding for the review pipeline

The issue was filed by the automated repository-wide review (unit `cross-testing`)
using a line-based grep probe.
Multiline assertion chains,
where `expect(value,)` and its matcher sit on separate lines,
evade that shape.
That is how the two tightest bounds left the inventory.
A reprobing of this class needs a multiline shape such as
`rg -U 'expect\([\s\S]{0,200}?\.toBeLessThan'`.

## Open questions

- Did any of the seven inventoried assertions themselves ever fail in CI?
  Still unknown;
  the [#510][i510] history shows the family failing on two-CPU containers,
  so a CI-log check remains useful but no longer decision-relevant.
- Interval exposure API shape (deferred follow-up,
  capability finding 1):
  raw per-child start and duration on the results,
  or a computed overlap summary?
  The choice touches `DescribeResult`,
  `ItResult`,
  and the verdict logging path.

## Next actions

Code changes remain paused by the owner;
the doc and tracker updates are done
(correction comment on [issue 1762][i1762],
[#1771][i1771] filed,
this handover current).
When the pause lifts,
implement the approved scope
(ranked remedy space items 1-7):

1.  The site fixes in ranked order:
    the describe peak-counter rewrite,
    the logger fake-clock zero-advance restructuring
    (with the `shouldClearNativeTimers: true` workaround until [#1771][i1771] lands),
    the gh-process `ABORT_ERR` assertion,
    the mvm derived ceiling and documented discrimination,
    and the wg-quicker documented slack.
2.  The `withTimeout` domain-pairing fix ([#1771][i1771]),
    with a guard test proven failing before the fix.
3.  The docs work item:
    the harness README fake-timer recipe
    (`shouldClearNativeTimers` caveat,
    single-consumption rule,
    `concurrency: 1` sequencing),
    descriptor re-execution semantics,
    the upper-bound duration guidance section,
    and the matching testing-practices skill updates.
4.  Deferred beyond the approved scope:
    the `DescribeResult` interval-exposure design item.
5.  The probe scripts are throwaway
    (`~/temp/agent/proto-p1-fakeclock-concurrency.mjs`,
    `proto-p1b-negative-only.mjs`,
    `proto-p2-fakeclock-flush.mjs`);
    their validated patterns are captured here and fold into the real tests
    at implementation.

## Evidence appendix

Key commands and outputs:

- Issue text and citations:
  `gh --repo Aquaticats/Monochromatic issue view 1762 --json number,title,body,labels`.
- Multiline elapsed-bound probe over default-run tests
  (`expect(<elapsed-like>,)...toBeLessThan` within 80 chars):
  hits at `describe.unit.test.ts:58`,
  `config.unit.test.ts:222`,
  `libvirt-tools.unit.test.ts:243`,
  `create-logger.unit.test.ts:810-811` and `:1075-1076`.
- Comparison-in-argument probe (`expect(x < Y,).toBe(true,)`):
  `gh-process.unit.test.ts:359` (real elapsed)
  and `package/ssg/aquati.cat/src/component/site-footer.unit.test.ts:137`
  (computed value,
  not real elapsed;
  the file uses no clock).
- File-level intersection of real-clock users
  (`performance.now()`,
  `Date.now()`,
  `hrtime`)
  with `toBeLessThan` users:
  exactly the five inventory files.
  `scale-complexity.local.unit.test.ts` is in the intersection but its only
  `toBeLessThan` bounds a binary-searched argument capacity,
  and its timing ratio assertion is a lower bound.
- Parse timing one-liner (node,
  built dist import,
  25 iterations):
  `{"min":"0.01","median":"0.03","p95":"0.19","max":"1.42","bound":250}`.
- Describe timing:
  seven isolated runs,
  `[concurrent-suite] [PASS] child-a, child-b (101ms)` in every run.
- Contention repro:
  five `mise run test:unit -- <32-file batch>` invocations,
  every exit code 0,
  no `test files failed:` line.
- Cancellation rethrow path:
  `gh-process.ts:233-237` rethrows the raw rejection when
  `isCancellation` matches,
  and `CANCELLATION_CODE = 'ABORT_ERR'` at line 82.
- P1 probe (`node ~/temp/agent/proto-p1-fakeclock-concurrency.mjs`,
  exit 0):
  positive `{"virtualNow":150,"state":"fulfilled","realElapsedMs":18.74}`;
  negative control `{"virtualNow":150,"state":"pending"}` then
  `{"virtualNow":250,"state":"fulfilled"}`;
  baseline `realElapsedMs 100.74`.
- P2 probe (`node ~/temp/agent/proto-p2-fakeclock-flush.mjs`,
  exit 0):
  first flush `{"virtualNow":0,"firstSettled":false}` then
  `{"virtualNow":60,"firstSettled":true}`;
  second flush `{"advancedMs":0,"secondSettled":true}`.
- Native-timer clear crash repro:
  the first P1 run without `shouldClearNativeTimers` died with the sinon
  `FakeTimers: clearTimeout was invoked to clear a native timer` throw through
  `withTimeout` dispose,
  `withHostTimeout`,
  `verifyAndApply`,
  `initialize`,
  then `Warning: Detected unsettled top-level await`,
  exit 13.
- Descriptor re-execution evidence:
  duplicated `[failing-suite] [FAIL]` lines in a plain
  `node src/describe.unit.test.ts` run
  (one rejection consumed by two `.rejects` assertions);
  the P1b control (`proto-p1b-negative-only.mjs`)
  stalled at a second `await` of an already-settled descriptor,
  exit 13,
  resolved by the single-consumption pattern.
- Harness promise machinery contains no timer deferrals:
  `rg 'setTimeout|setImmediate|queueMicrotask|process.nextTick'` over
  `execution-node.ts`,
  `execution.ts`,
  `describe.ts`,
  `it.ts`,
  `verdict.ts`,
  `it-attempt.ts`,
  `sandbox-runtime.ts`
  returns no matches.
- [#510][i510] history:
  the issue records the two-CPU-container failure
  `expected 108.877363 to be below 50` for
  `expect(elapsed).toBeLessThan(SLOW_WRITE_MS * 2)`;
  the current-tree
  `sinks verify concurrently rather than one after another`
  test (create-logger.unit.test.ts:980)
  uses `Promise.withResolvers` start flags and a release gate,
  with no clock assertion.
- Filed [#1771][i1771] in phase 2
  (duplicate searches for `withTimeout clearTimeout`,
  `fake timers deadlock`,
  and `shouldClearNativeTimers` found no open matches).
- Correction comment posted on issue 1762:
  [comment permalink][c1762].

[i1762]: https://github.com/Aquaticats/Monochromatic/issues/1762
[i510]: https://github.com/Aquaticats/Monochromatic/issues/510
[i1771]: https://github.com/Aquaticats/Monochromatic/issues/1771
[c1762]: https://github.com/Aquaticats/Monochromatic/issues/1762#issuecomment-6255436442
