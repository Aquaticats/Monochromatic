# Review of the wall-clock bound test remediation plan

## Status note

This review's factual findings stand:
the inventory correction,
the measurements,
the no-flag probe result,
and the citation corrections.

Its harness-level recommendations are superseded by
[the execution architecture proposal](test-harness-execution-architecture.md)
and
[the refined remediation plan](wall-clock-bound-test-remedies-refinement.md).
Specifically:

- interval exposure is in scope,
  not deferred;
- `concurrency: 1` is not the logger remedy;
  the harness should own virtual
  time instead of replacing process globals;
- items 2 and 3 are acceptance cases for scope-owned execution,
  not test-file
  workarounds;
- the plan's "no runner changes" position is superseded by a runner-level
  hard deadline for hung files.

One measurement annotation:
the character-scan quadratic model in item 6 is
JIT-dependent and not reproducible across runs.
Use the per-prefix split model (203 ms at 4000 prefixes,
3.1 s at 16000)
and the linear parse figures only.

## Verdict

The reframing in the plan is correct and better evidenced than the original issue:
the four visible sites are not one defect,
the quarantine is the wrong remedy,
and the logger and harness findings are real.
The remedy direction is sound and the owner-approved scope is the right shape.

Implementation should stay paused until the seven blocking corrections land.
None of them overturns the remedy space;
two change an item's landed design (items 3 and 5),
one adds a missed default-run surrogate site,
one corrects a layer label (item 1),
and the rest make the acceptance and the appendix match what the repository can prove.

Blocking corrections:

1.  The inventory is seven assertions in six files,
    not seven in five.
2.  The fake-timer sequencing boundary must be the test file's root suite.
3.  Item 3's probe covers only one of the two tests it rewrites.
4.  Item 5 has a timing-free proof,
    so the plan's "no timing-free formulation" premise fails.
5.  Item 1's alternative is the logger wrapper,
    not a harness boundary.
6.  The issue's written acceptance is not satisfied by the restatement as written.
7.  The appendix's expected no-flag behavior does not reproduce.

## Review boundary

Read in full or at the cited lines:
the [plan](wall-clock-bound-test-remedies.md),
the [handover](../handover/wall-clock-bound-unit-tests.md),
[issue 1762][i1762] and its correction comment,
[issue 1771][i1771],
[issue 510][i510],
the [incident record](../troubleshooting/cli-git-logger-sink-verify-timeout.md),
every cited test and source site at its cited lines,
the harness README and testing-practices skill,
`mise.toml`,
and `file-enforcer.config.ts`.

Ran directly:
both archived probes and their no-flag variants,
the harness self-test for the duplicated failure lines,
the wg-quicker parse timing and synthetic quadratic models,
and repository-wide probes for real-elapsed upper bounds.

Not verified:
the recorded five-batch contention rerun (not repeated,
see the evidence section),
any two-CPU container behavior,
and CI history for the inventoried assertions.

Built artifacts used by the probes:
`package/logger/core/dist` from 2026-10-18 12:00,
`package/test-harness/core/dist` from 2026-10-04.
The probes therefore tested the tree as of those builds,
not a fresh one.

## Blocking findings

### The inventory is seven assertions in six files

The plan and the handover say "seven real-elapsed upper-bound assertions in five files"
and then list six assertions:
`describe.unit.test.ts:58`,
`gh-process.unit.test.ts:359`,
`libvirt-tools.unit.test.ts:243`,
`create-logger.unit.test.ts:811` and `:1076`,
and `config.unit.test.ts:222`.
The handover itself says "The six sites here" at one point,
so the count is internally inconsistent.

A seventh site exists and was never inventoried:
`package/pi-plugin/bash-poke/src/job-runner.unit.test.ts:337`,
test `ends a long job early when cancelled and marks it cancelled` at line 321.
It measures `Date.now() - startedAt` against `CANCEL_DEADLINE_MS = 5_000` (line 31)
around a `sleep 30` child with a 200 ms kill grace.
It is a default-run unit test,
it predates the investigation
(file committed 2026-09-17 as `3962d99199`),
and it is a surrogate bound:
an unstopped `sleep 30` exits 0,
while the cancelled job's `close` payload reports `code: null`,
so `JobOutcome.exitCode` is absent
(`package/pi-plugin/bash-poke/src/job-runner.ts:200-217` and `:53-67`).
Asserting `outcome.cancelled` plus `outcome.exitCode` not equal to 0
discriminates without a clock;
`JobOutcome` does not currently expose the closing signal.

Why it was missed:
the handover's "file-level intersection of real-clock users with `toBeLessThan` users"
only considers files that already use `toBeLessThan`,
and this site uses a parenthesised comparison in the matcher argument,
the exact shape the comparison probe was meant to catch.
The intersection heuristic excludes any site with a different assertion syntax,
so it cannot establish completeness.
A completeness probe needs every `toBeLessThan`,
`toBeLessThanOrEqual`,
comparison-in-argument,
and any clock delta compared from above,
followed by manual review of every `performance.now()` and `Date.now()` user.

Consequences:
the contention batch used "the five files holding the seven assertions",
so the missed file was not part of the load evidence;
the approved scope covers five test-site remedies,
so it covers six sites;
and no restatement of the acceptance can be complete while this site is absent.
The plan needs a new work item (or an explicit decision to keep that bound with reasons),
and the handover,
correction comment,
and scope count need the correction.

### The fake-timer sequencing ancestor must be the file root

Item 3 says "Group the clock-using tests under a `concurrency: 1` ancestor".
Implemented as a new inner group,
that is not enough.
Fake timers are shared host replacements,
and the testing-practices skill says the sequencing ancestor must cover
every affected reader and writer,
while "separate sequential sibling suites can still overlap".

The logger test file has no `concurrency` setting at any level,
and its root `describe` at `create-logger.unit.test.ts:354` dispatches children
with the harness default of 16 (`package/test-harness/core/src/descriptor.ts:65`).
Its own helper comment at line 83 says sibling tests run concurrently.
So the two rewritten tests would install process-global fake timers while up to
15 sibling suites keep running with real timers.
That is exactly the class of failure the plan is trying to remove:
a sibling's `wait`,
flush deadline,
or attempt timeout would become a virtual timer
nobody ticks,
and the file would hang.

Set `concurrency: 1` on the root `describe` at line 354
(or restructure so one sequential ancestor contains every sibling),
and state the runtime cost in the plan.
The same rule applies to item 1's guard test:
`package/async-time/core/src/with-timeout.unit.test.ts` has a sibling test
that schedules real timers (its timeout case waits 300 ms),
and its root at line 13 also needs `concurrency: 1`.
The plan mentions the sequencing rule for item 3 but not for item 1.

### Item 3's probe does not cover the second rewritten test

The appendix probe validates the shape of the test at line 811:
a wedged sink whose write never settles,
first flush waiting at virtual 0,
tick 60,
second flush with zero advance.
The test at line 1076 is different:
a parked first write that is later rejected,
`await wait(1)` before the next log,
`logger.info('after')`,
`captureUnhandledRejections()`,
and breadcrumb and message assertions that the probe does not exercise.

Under a fake clock,
`wait(1)` becomes a virtual timer,
so the rewrite must tick it explicitly;
skipping that step hangs the file.
The probe also does not show that the breadcrumb count and the unhandled-rejection
capture survive the clock substitution.
Item 3 needs a second probe,
or a per-test translation note that names
every real-timer dependency (`wait(1)`,
`timeFlush`,
`performance.now`)
and what replaces it.

Related detail:
the rewritten tests can no longer use `timeFlush`
(`create-logger.unit.test.ts:65-72`) because it measures real time.
The plan should say the two tests drop that helper and assert virtual state,
while the untouched lower-bound siblings keep it.

### Item 5 already has a timing-free proof

The plan classifies `libvirt-tools.unit.test.ts:243` as a discriminating bound
and states that no timing-free formulation proves those.
That is not true for this site.
The lingering child holds the output stream open for 60 s
and prints nothing after its pid.
If it printed a marker at the end of its hold,
a runner that waits for stream close would necessarily capture that marker
before the stream closes,
while a runner that returns at child exit would not.
Asserting the marker is absent is deterministic and needs no clock.
The current `process.kill(pid, 'SIGKILL')` already implies liveness,
but it is not asserted as the property and is weaker than the marker,
because a zombie can accept a signal.

So item 5 can drop the real-elapsed bound entirely
and satisfy the first branch of the issue's acceptance for this site.
If the owner prefers to keep a bound anyway,
the derived `HOLD / 2` is the right form,
not a bare absolute.

### Item 1's alternative is the logger wrapper, not the harness

Question 1 calls the smaller-blast-radius alternative
"the harness `withHostTimeout` boundary".
`withHostTimeout` lives in `package/logger/core/src/with-host-timeout.ts:26`
and is called from `create-logger.ts:318` and `:515`.
The harness calls `withTimeout` directly at
`package/test-harness/core/src/it-attempt.ts:83` and `describe.ts:316`,
for the attempt and suite timeouts.
A fix in the logger wrapper would leave those two paths crossing domains,
and the attempt timeout is one of the paths a fake-timer test exercises.

So the plan's proposal is right,
for a stronger reason than stated:
the invariant belongs to `withTimeout`,
and it is the only layer that covers
the logger bootstrap path and the harness attempt and suite paths with one change.
The plan text should stop calling the logger wrapper a harness boundary.

The guard test also needs two branches,
not one:
schedule under the native clock and clear after a fake clock installs (the bug),
and schedule under a fake clock and clear after it restores.
The second direction is currently benign because Node's `clearTimeout`
ignores a fake handle,
but the fix claims symmetry and the test should prove it.

### The acceptance restatement does not satisfy the issue's written acceptance

The issue body's acceptance is:
"The default test:unit run contains no upper-bound wall-clock assertion on real elapsed time,
or every such bound carries documented slack justification and a `*.expensive.*` quarantine
consistent with the tsc-filter precedent."

The plan's restatement keeps discriminating and complexity bounds
without quarantine.
That is the correct engineering choice,
and the quarantine branch is wrong for millisecond-scale tests
for the reasons the plan gives.
But the issue record does not say the acceptance was replaced:
the correction comment restates it,
while the issue body still carries the original clause.
As written,
the plan cannot close the issue by its own text.

The owner-approved reframing is enough authority to change the acceptance,
but the change has to be recorded where the issue's acceptance lives:
edit the issue body,
or post a comment that explicitly supersedes that clause
and says why the quarantine branch is rejected.
The completion statement should also cover the missed bash-poke site,
otherwise the first branch fails on a site the plan never saw.

### The appendix's no-flag expectation does not reproduce

The appendix says:
"Without `shouldClearNativeTimers` and before item 1 lands,
the first file instead deadlocks with the sinon ... throw and node exit 13".

I ran the archived probe with the flag removed,
three times,
against the current dist.
Every run exited 0.
The sinon error did surface on stderr,
with the path `withTimeout` dispose,
`withHostTimeout`,
`verifyAndApply`,
logger `initialize`,
exactly as recorded,
but the file did not deadlock and did not exit 13.
The logger probe without the flag also exited 0,
with no throw at all.

The recorded exit 13 may have come from an earlier probe revision,
or from the descriptor double-consumption trap (the handover's P1b),
not from the native-clear throw.
Whatever the history,
a reviewer following the appendix will not see the deadlock,
so the appendix is currently wrong as an instruction.
The deterministic symptom to assert is the thrown
`FakeTimers: clearTimeout was invoked to clear a native timer`
error,
which the guard test in item 1 can surface directly.
The handover and the incident record should qualify "deterministically deadlocked"
as a race whose consequence depends on how the escaping rejection is handled.
This does not weaken the fix;
it changes the expected-failure description.

## Non-blocking findings and plan edits

### Item 6 is load-robust but regression-blind at 4000 prefixes

I reproduced the parse measurement:
25 runs at 4000 prefixes gave min 0.009 ms,
median 0.033 ms,
p95 0.164 ms,
max 1.60 ms against the 250 ms bound.
That is roughly 156x worst-case measured,
consistent with the plan's 176x
from a 1.42 ms worst run;
the difference is ordinary run-to-run spread.
Parse cost scales linearly:
median 0.116 ms at 16000 prefixes and 0.187 ms at 32000.

The problem is the other side of that slack.
The plan says a quadratic regression "takes seconds and trips the bound reliably",
but that was not measured.
I modeled the class the guard targets,
one pass over the stored value per prefix,
at n = 4000:

- per-prefix full-value character scan:
  39.9 ms median,
  under the bound;
- per-prefix split of the value:
  198.6 ms median,
  also under the bound.

So a representative quadratic regression does not trip the guard on this host.
The bound catches order-of-magnitude regressions only,
and the comment planned in item 6 would overstate what it detects.

Options,
with measured support:

- Raise the input size.
  At 16000 prefixes the split model scales to roughly 3.2 s,
  the character-scan model to roughly 640 ms,
  while the linear parse stays near 0.12 ms.
  The two distributions separate cleanly.
- Compare two sizes (the `deep / shallow` pattern the plan rejects)
  so load cancels and the ratio exposes a superlinear slope.
- Keep the absolute bound but document it honestly as a coarse ceiling,
  and add a positive control that applies a synthetic regression and shows
  the bound tripping.

Documentation-only is acceptable only in the third,
weakened sense.
Before the comment lands,
run the positive control (GFP and QPC both apply),
because the current claim is unmeasured.

### The deferred interval premise is inaccurate

Question 9 and the deferred scope say the harness "already measures"
per-child execution intervals at `it.ts:205/227` and `describe.ts:326-368`.
It does not.
`it.ts` measures one attempt's start and duration for verdict logging,
and `describe.ts` measures suite-level start and duration for the summary line.
`DescribeResult` and `ItResult` both return only `{ name }`
(`describe.ts:26-31`,
`it.ts:77-83`).
Per-child start and end intervals do not exist,
so exposing them needs new instrumentation,
result-type changes,
and verdict-path work,
not only a return-type change.

Keep the item deferred.
The corrected premise matters because intervals would be a genuinely
load-insensitive concurrency proof (an overlap assertion
`max(starts) < min(ends)` is order-based,
not a duration bound),
so this is the durable fix worth scheduling later,
not a cosmetic API tweak.

### Verification must rebuild the dist bundles

Item 1 changes `module-async-time` source,
which is bundled into `module-logger/dist` and `module-test/dist`.
The logger tests import `module-logger` from dist but `module-test` from `/ts`;
the harness self-test imports `module-test` from dist;
and the P1 probe imports `module-test/dist`.
`mise run buildAndTest -- <file>` builds only the containing package,
which does not rebuild `module-test/dist`.
Add explicit rebuild steps to the verification plan:

- after item 1,
  rebuild `//package/test-harness/core` before rerunning the no-flag probe;
- after item 1,
  rebuild `//package/logger/core` before the logger tests
  (the normal `buildAndTest` path does this);
- verify the probes report the fresh build,
  not the 2026-10-04 harness artifact.

Also add to verification:
run the rewritten logger file without `shouldClearNativeTimers` after item 1
(this is the gate for question 4),
run the wg positive control,
run the mvm marker observation on both the healthy and the broken path,
and include the bash-poke file in the contention batch and the full run.

### Citation and record drift

- `mise.toml:669` is cited for the `*.expensive.*` meaning.
  The description is at line 663 and the glob filter at line 683;
  line 669 is inside the `test:unit` run body.
- The default-run unit glob measured 1081 files today
  (1086 active files minus 5 `.expensive.` files),
  not the handover's 1073.
  The drift is expected from intervening commits,
  but the plan should say whether the count is historical or re-measured.
- The "three default-run files install fake timers" record overstates the timer
  exposure:
  `file-batch.unit.test.ts` fakes only `Date`,
  `sandbox.unit.test.ts` passes `useFakeTimers: false`,
  and only `sinon-clock.unit.test.ts` fakes `setTimeout` and `clearTimeout`
  today.
- `CANCELLATION_CODE` is a module-local constant at `gh-process.ts:82`.
  The test will duplicate the literal unless the plan exports it or cites it.
- Skills under `.agents/skills` are canonical and mirrored to
  `.claude/skills` and `.factory/skills` by file-enforcer.
  Item 7 should add `mise run sync:files` after the skill edit and commit the
  mirrored outputs;
  the harness README is not managed and is edited directly.

## Answers to the nine questions

### 1. Item 1 layer

Recommendation:
fix inside `withTimeout`.

Fix at the primitive.
The timer is created there,
so the disposer is the only code that can know
which clock domain owns the handle.
It also covers the harness attempt timeout and suite timeout,
which the logger wrapper cannot reach.

- Option A,
  `withTimeout` pairing.
  Pros:
  one invariant,
  every consumer fixed,
  guard test at the source,
  no signature or API change.
  Cons:
  every async-time consumer is rebuilt;
  the async-time test file needs
  root sequencing for the guard test.
- Option B,
  logger `withHostTimeout` pairing.
  Pros:
  smaller diff for the observed logger path.
  Cons:
  leaves `it-attempt.ts:83` and `describe.ts:316` unfixed;
  duplicates the same pairing logic if the harness is fixed later;
  the plan's label for it as a harness boundary is wrong.
- Option C,
  import `setTimeout` and `clearTimeout` from `node:timers`.
  Pros:
  immune to global replacement with no pairing.
  Cons:
  breaks legitimate virtual clocks,
  which item 3 depends on;
  withTimeout would always use real timers under a fake clock.

Ranking:
A > B > C,
because A fixes the invariant once at the owner,
B fixes only one of three known paths and mislabels the layer,
and C makes the virtual-clock remedy impossible.

### 2. Item 2 proof

Recommendation:
land the peak-counter proof,
keep the existing 100 ms delay,
and keep the fake-clock single-tick pattern as the README recipe demonstration.

The peak counter is deterministic for the property that matters:
concurrent dispatch invokes both bodies in the same dispatch turn,
so both increment before any timer callback can run,
and sequential dispatch cannot reach a peak of two.
The timer is only there to keep a sequential child from completing before the
next child starts,
so its length does not affect the peak result.
The current 100 ms is already exercised by the plan's contention batches;
shrinking it to 20 ms saves 80 ms per run and changes the measured configuration
without a measurement,
so keep 100 ms until a dispatch-gap measurement justifies
a smaller value.

The fake-clock pattern is deterministic and faster,
but it adds global timer replacement and root sequencing to the harness
self-test file,
plus the `shouldClearNativeTimers` dependency,
which is a worse trade for the harness's own suite.
Keep it documented in the README as the recipe it already is.

Option C worth recording before implementation:
a gated-promise start/release proof,
the #510 in-tree precedent,
is fully clock-free and proves the same overlap.
It needs a `timeout` so a sequential regression fails as a timeout rather than
hanging,
and it changes the failure mode from an assertion to a timeout.
Implement it only if the owner wants zero real timers in this file.

Ranking:
A (peak counter,
100 ms) > C (gated promise) > B (fake clock),
because A is the smallest change that keeps the existing measured timing,
C is conceptually cleanest but changes the failure mode,
and B imports the timer-replacement hazard into the file being protected.

### 3. Item 3 scope

Recommendation:
migrate only the two upper-bound tests,
as the approved scope says.

Lower-bound tests cannot produce the false red the issue is about:
contention makes them slower,
not faster.
Migrating them would enlarge the fake-timer window and the serialized portion
of the file for no flake reduction.
It would also force the untouched tests to drop `timeFlush`,
which measures real time,
so the change is not the mechanical substitution
the plan implies.

Uniformity is a later cleanup,
in a change that also documents which clock each
test uses.
The two rewritten tests should say in their names or comments that
they read virtual time,
so the file is not misleading.

Ranking:
two tests only > whole deadline family,
for this plan.
The whole-family option becomes reasonable only after the root sequencing lands
and a separate review covers the lower-bound tests.

### 4. Item 3 caveat

Recommendation:
remove `shouldClearNativeTimers` once item 1 lands,
and gate the removal on running the rewritten tests without it.

The flag forwards native clears to the real clock,
which is exactly the cross-domain shortcut the fix removes.
Leaving it in keeps the tests honest-looking while hiding the behavior the
guard test exists to protect.
If removal exposes a non-`withTimeout` native timer created before the fake
install,
fix or name that source;
that is a finding,
not a reason to restore the flag.

Keep the README caveat as general guidance with the narrow legitimate use:
timers created before installation by code you do not own.
The plan's item 7 wording already allows this.

Ranking:
remove after item 1 > keep with a written reason > keep silently.

### 5. Item 4 fields

Recommendation:
assert only the diagnostic code.

Node's documentation says aborting through the signal option makes the callback
receive "an `AbortError`";
it does not document `killed` or `signal` on that rejection.
Those properties describe the `ChildProcess`,
not the error.
`code === 'ABORT_ERR'` is the repository's own classification contract
(`gh-process.ts:82`,
used by `isCancellation` at `:233-237`),
so the test can rely on it without adding new coupling.
Asserting `name === 'AbortError'` is a reasonable optional addition;
`killed` and `signal` are not.

Because `CANCELLATION_CODE` is not exported,
either export it and assert against it,
or cite `gh-process.ts:82` in a comment
so the literal has a stated owner.
Keep `expect(caught).toBeInstanceOf(Error)`.

Ranking:
code only > code plus `name` > code plus `killed` or `signal`.
The last option would fail on Node versions that satisfy the documented
contract without the undocumented fields.

### 6. Item 5 ceiling

Recommendation:
neither ceiling;
drop the bound and prove the property
observationally.

Make the lingering child print a marker at the end of its hold,
then assert the spawn output does not contain the marker.
A runner that returns at child exit cannot see a 60 s marker;
a runner that waits for stream close cannot avoid seeing it.
That is deterministic and load-insensitive,
and it removes a real-elapsed
upper bound from the default run.
Keep the kill as cleanup.

If the owner wants a numeric ceiling anyway:
derive it from `LINGERING_HOLD_MS` as the plan sketches,
and comment the
discrimination.
The derived form beats a named absolute because the two constants cannot drift.

Ranking:
marker observation > `HOLD / 2` > named absolute,
because the observation proves the property directly,
the derived constant keeps the two behaviors separated with no drift,
and an absolute has no relationship to the hold.

### 7. Item 6 documentation-only

Recommendation:
not documentation-only as currently framed.
Retune the guard,
or document a weakened claim with a positive control.

The measured evidence supports the plan's load margin and contradicts its
regression claim.
At 4000 prefixes,
representative quadratic string work costs 40 ms to 200 ms,
under the 250 ms bound.
The guard therefore does not detect generic quadratic slowdown;
it detects regressions that are roughly an order of magnitude worse.

Preferred:
raise the input to 16000 prefixes (linear cost about 0.12 ms)
or add a two-size ratio check,
and add the positive control that proves a
synthetic regression trips the guard.
Fallback:
keep 4000 and the absolute bound,
but the comment must say it is a
coarse ceiling and name the class it does not catch.
The issue's "documented slack justification" is met in the fallback only in
that weakened sense,
and the issue record should say so.

Ranking:
`retune plus positive control > honest documentation of a coarse ceiling > unchanged plan text`.
The first restores the guard's stated purpose,
the second is truthful about its limits,
and the third asserts a measurement that does not hold.

### 8. Acceptance restatement

Recommendation:
sound in substance,
not compliant as written;
amend the issue record.

Dropping surrogate bounds and keeping discriminating and complexity bounds
with derived constants and documented slack is the right resolution.
But the issue's written acceptance's second branch requires both documented
slack and a `*.expensive.*` quarantine,
and the plan correctly rejects quarantine.
Quarantine means genuinely expensive tests excluded from the default run,
not millisecond-scale proofs (`mise.toml:663`,
`:683`).
The owner's reframing is authority to replace the clause,
but the issue body has to carry the replacement,
or the correction comment has
to say explicitly that it supersedes the body's acceptance.

The restated completion should read:
no surrogate real-elapsed upper bounds in the default run;
every surviving bound is discriminating or a complexity guard,
with a constant derived from the scenario,
documented measured slack,
and a positive control;
no `*.expensive.*` quarantine;
the inventory covers every default-run site,
including the bash-poke site.
Also derive the acceptance from a complete probe,
not the file intersection
that missed it.

Ranking:
`amend the issue body plus complete inventory > superseding comment only > leave the text as is`.
The last option leaves a closed issue whose acceptance the code does not meet.

### 9. Deferred interval exposure

Recommendation:
keep it deferred,
and correct the premise.

The harness does not currently measure per-child intervals,
so this is new instrumentation plus result-type and verdict-path work,
not exposing existing values.
It is a good follow-up because interval overlap is load-insensitive and would
make future concurrency proofs observational by construction.
It is not needed for item 2 once the peak-counter or gated-promise proof lands,
and pulling it into a paused,
already-broad scope needs a fresh owner decision.

Ranking:
defer > pull in,
for this plan;
revisit as the durable concurrency-proof
design after the current items land.

## Required edits before the pause lifts

1.  Add the bash-poke site to the inventory,
    the scope,
    and the contention batch;
    correct "seven in five files" in the plan,
    the handover,
    and the correction
    comment;
    state a remedy (outcome-shape assertion) or a decision to keep it.
2.  Move the item 3 sequencing requirement to the root suite at
    `create-logger.unit.test.ts:354`,
    add the same requirement to item 1's guard
    test file,
    and state the runtime cost.
3.  Validate test 1076's rewrite with a second probe or a translation note that
    covers `wait(1)`,
    `timeFlush`,
    breadcrumbs,
    and unhandled rejections.
4.  Change item 5 to the marker observation,
    or justify keeping a derived bound
    after acknowledging that a timing-free proof exists.
5.  Rename item 1's alternative to the logger `withHostTimeout` wrapper;
    keep `withTimeout` as the chosen layer and add the reverse-direction guard.
6.  Amend issue 1762's acceptance on the issue itself,
    with the completion
    statement from question 8.
7.  Correct the appendix's expected behavior and qualify the deadlock wording in
    the handover and incident record.
8.  Retune or honestly narrow item 6,
    and add its positive control.
9.  Correct the deferred interval premise in the plan and handover.
10. Add the rebuild,
    sequencing,
    marker-control,
    positive-control,
    and skill-mirror
    steps to the verification plan;
    fix the `mise.toml:669` citation and the
    default-run count.

## Evidence from this review

Probes:
`node ~/temp/agent/proto-p1-fakeclock-concurrency.mjs` exited 0 with
`{"virtualNow":150,"state":"fulfilled","realElapsedMs":17.33}`,
the negative control pending at virtual 150 and fulfilled at 250,
and the real-clock baseline at 101.05 ms.
`node ~/temp/agent/proto-p2-fakeclock-flush.mjs` exited 0 with
first flush `{"virtualNow":0,"firstSettled":false}` then
`{"virtualNow":60,"firstSettled":true}`,
and second flush `{"advancedMs":0,"secondSettled":true}`.
Both probes ran green,
matching the handover.

No-flag variants,
created by removing `shouldClearNativeTimers` from copies:
P1 exited 0 in three runs,
with the sinon `clearTimeout` throw on stderr
through `withTimeout`,
`withHostTimeout`,
`verifyAndApply`,
`initialize`;
P2 exited 0 with no throw.
No deadlock and no exit 13 were observed.

Descriptor re-execution:
`node src/describe.unit.test.ts` from `package/test-harness/core` printed the
`failing-suite` failure lines twice,
four lines in total,
confirming the double-consumption claim.

wg-quicker timing:
`parseConfigText` over 4000 prefixes gave min 0.009 ms,
median 0.033 ms,
p95 0.164 ms,
max 1.60 ms (25 runs);
16000 prefixes gave median 0.116 ms,
32000 gave 0.187 ms.
Synthetic quadratic models at 4000 prefixes:
full-value character scan 39.9 ms median,
per-prefix split 198.6 ms median,
both under the 250 ms bound.

Inventory:
repository-wide probes over `*.unit.test.ts` for `toBeLessThan`,
`toBeLessThanOrEqual`,
and comparison-in-argument shapes found
the six listed assertions plus the bash-poke assertion,
so seven assertions in six files.

Default-run file count:
1086 active `*.unit.test.ts` files,
1081 after excluding `.expensive.`.

## References

[i1762]: https://github.com/Aquaticats/Monochromatic/issues/1762
[i1771]: https://github.com/Aquaticats/Monochromatic/issues/1771
[i510]: https://github.com/Aquaticats/Monochromatic/issues/510
