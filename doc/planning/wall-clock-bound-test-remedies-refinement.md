# Wall-clock bound test remediation: refined plan

## Status

Proposal for review.
The owner paused implementation and asked for written findings before deciding
whether to continue.
Nothing here is authorized to land.
This document refines
[the approved remediation plan](wall-clock-bound-test-remedies.md)
using
[the plan review](wall-clock-bound-test-remedies-review.md)
and
[the execution architecture proposal](test-harness-execution-architecture.md).

The execution architecture is now the structural part of the remedy.
The original plan's per-site workarounds are kept only where the property
under test involves a real subprocess or real CPU work,
which no harness clock can virtualize.

## What changes from the approved plan

1.  The harness gains an execution scope:
    a clock,
    a cancellation signal,
    cleanup ownership,
    and an event journal
    (see [execution architecture](test-harness-execution-architecture.md)).
    Timing-proof tests become observations on that journal.
2.  Interval exposure moves from deferred to in scope.
    The original plan deferred the only structural fix for the describe site.
    `ItResult` and `DescribeResult` gain execution records,
    and the describe concurrency test asserts overlap or peak from them.
3.  Fake timers leave the harness-owned recipe.
    The logger deadline tests use the scope clock.
    `concurrency: 1` and `shouldClearNativeTimers` stop being landed remedies
    and become migration notes only.
4.  Descriptor semantics change:
    `.then` memoizes one execution.
    The plan documented the double-consumption trap;
    the harness now removes it.
5.  The runner gains a per-file wall-clock budget and process-group kill,
    so a hung file fails as a bounded hang instead of occupying a worker.
6.  Item 5 (mvm) drops the bound for a marker observation.
    The lingering child prints a marker at the end of its hold;
    the test asserts the marker is absent from captured stdout.
7.  Item 6 (wg-quicker) is not documentation-only.
    The measured 250 ms ceiling at 4000 prefixes is regression-blind.
    The refined plan prefers a test-only operation counter
    and falls back to an audited real-time bound with a positive control.
8.  A new site is added:
    bash-poke's cancellation bound
    (`package/pi-plugin/bash-poke/src/job-runner.unit.test.ts:337`).
    Assert the signalled outcome shape instead of elapsed time.
9.  A repo-owned check enforces the invariant mechanically.
    New default-run real-elapsed upper bounds fail the check unless the
    allowlist names them with a reason.
10. The acceptance is amended on the issue itself,
    not only restated in a comment.

## Corrected premises

From the owner's corrections in this session:

- The test harness and the packages under test are ours.
  The remedy space is not limited to test-file edits;
  harness capabilities are first-class remedies.
- Promises cannot enforce timeouts.
  A promise race reports lateness but cannot cancel,
  clean up,
  or observe work.
  The runner and execution model need an owner,
  not a larger `withTimeout`.
- Defaults must be safe.
  A capability that tests must opt into does not fix a class of failures;
  it documents it.
- The invariant must be mechanically true,
  not documented.
  The issue was found by an external probe because the repo did not own the check.

## Reviewed findings that remain valid

These from [the plan review](wall-clock-bound-test-remedies-review.md) are
unchanged by the architecture:

- The inventory has seven default-run real-elapsed upper bounds in six files,
  not five.
  The missed site is bash-poke's cancellation test.
- The `withTimeout` domain-paired clear is right,
  and the fix belongs in
  `module-async-time`,
  not the logger wrapper.
- gh-process should assert the cancellation diagnostic and drop the bound.
- The issue's written acceptance still requires a quarantine for kept bounds,
  so the record must be amended.
- The appendix's no-flag deadlock does not reproduce;
  the guard test asserts the thrown `FakeTimers` error.
- The review's item 6 measurement:
  the per-prefix split model costs
  203 ms at 4000 prefixes,
  under the 250 ms bound,
  while at 16000 it costs 3.1 s
  against a 0.116 ms linear parse.
- The review's item 5 finding:
  a marker observation exists,
  so the mvm bound can go.
- The review's citation corrections (`mise.toml:663` and `:683`,
  the default-run file count drift).

## Reviewed findings that are superseded

- Deferring interval exposure.
  It is the structural fix and moves into scope.
- Using `concurrency: 1` as the logger sequencing remedy.
  The scope clock removes global replacement,
  so the rule disappears.
- Treating the logger and describe sites as test-file edits only.
  Their landed form is an execution-scope assertion.
- Keeping the mvm and wg real-time bounds as the durable position.
  Both should become observations or audited exceptions.
- The plan's "no runner changes" position.
  It was about throughput;
  it does not cover the missing hard deadline.

## Revised work items

### 1. Execution scope and journal

Files:
`package/test-harness/core/src/scope.ts` (new),
`execution*.ts`,
`describe.ts`,
`it-attempt.ts`,
`it.ts`,
result types,
self-tests.

Acceptance:
scope lifecycle is tested;
late work after dispose is reported;
existing tests run unchanged on the real clock;
`ItResult` and `DescribeResult` expose execution records;
self-tests cover disposal,
abort,
and pending work.

Dependencies:
none.

### 2. Runner per-file budget

Files:
`mise.toml` `run_test_files`,
the root `test:unit` family and
`buildAndTest`.

Acceptance:
a hanging fixture is killed at the budget and reported as a hang;
a failing fixture is reported as an assertion failure;
the positive control hangs without the budget (GFP/QPC);
process-group kill reaches a spawned child.

Dependencies:
none;
independent of the harness library.

### 3. `module-async-time` clock seam

Files:
`package/async-time/core/src/wait.ts`,
`with-timeout.ts`,
`with-timeout.unit.test.ts`.

Acceptance:
optional clock and signal parameters;
default behavior unchanged;
`withTimeout` captures schedule and clear from one clock;
guard test installed fake timers without `shouldClearNativeTimers` and
is proven failing before the fix,
per GFP;
the reverse direction (schedule under fake,
clear after restore) is covered.

Dependencies:
item 1 for the harness clock;
item 3 can land first with its own fixture.

### 4. Logger deadline tests on the scope clock

Files:
`package/logger/core/src/create-logger.unit.test.ts`,
`package/logger/core/src/create-logger.ts` (optional clock),
possibly `with-host-timeout.ts`.

Acceptance:
the two upper-bound tests assert virtual observations
(first flush unsettled at zero,
settles at the deadline tick;
second flush advances zero);
no sinon fake timers,
no `concurrency: 1`,
no `shouldClearNativeTimers`;
the file runs concurrently;
the second test's `wait(1)` is translated and its breadcrumb and
unhandled-rejection assertions still hold;
the lower-bound deadline tests stay on real time.

Dependencies:
items 1 and 3.

### 5. Describe concurrency from the journal

Files:
`package/test-harness/core/src/describe.unit.test.ts`.

Acceptance:
the test asserts interval overlap or peak concurrency from the journal;
the 100 ms timers and the 180 ms bound are gone;
the negative control (sequential dispatch) fails to overlap;
a positive control shows the assertion can fail.

Dependencies:
item 1.

### 6. Descriptor memoization

Files:
`package/test-harness/core/src/descriptor.ts`,
`describe.ts`,
`it.ts`,
affected self-tests.

Acceptance:
two `then` consumptions observe one execution;
`repeats` still re-runs through explicit dispatch;
the duplicated `[FAIL]` lines disappear;
the P1b deadlock is impossible by construction.

Dependencies:
none;
independently testable.

### 7. gh-process cancellation

Files:
`package/search-fetch/core/src/gh-process.unit.test.ts`.

Acceptance:
assert `caught instanceof Error` and `code === 'ABORT_ERR'`;
optionally assert `name === 'AbortError'`;
do not assert `killed` or `signal`;
the elapsed measurement is deleted.

Dependencies:
none.

### 8. mvm marker observation

Files:
`package/mvm/cli/src/libvirt-tools.unit.test.ts`.

Acceptance:
the lingering child prints a marker at hold end;
the test asserts the marker is absent from `spawn`'s returned stdout;
the `30_000` bound and the 60 s interpolation constant are gone;
the kill remains as cleanup;
a positive control (a runner that reads after stream close) would capture
the marker.

Dependencies:
none.

### 9. bash-poke cancellation

Files:
`package/pi-plugin/bash-poke/src/job-runner.unit.test.ts`.

Acceptance:
assert `outcome.cancelled` and a signalled outcome
(`exitCode` absent or nonzero) instead of `Date.now()` elapsed;
the 5 s bound is gone;
a positive control (cancel not killing the child) yields `exitCode: 0`.

Dependencies:
none.

### 10. wg-quicker complexity

Files:
`package/wg/cli-quicker/src/config.unit.test.ts`,
possibly `coordinator` internals for a test-only counter.

Probe first:
can the parser expose a test-only operation count
(scan length or token count) without a public API change?
If yes,
assert linear work instead of elapsed time.
If no,
keep an audited real-time bound at 16000 prefixes with a positive
control that proves a synthetic regression trips it,
and allowlist it.

Acceptance:
either a work-count assertion,
or elapsed at 16000 prefixes with a measured
positive control and allowlist entry;
the 4000-prefix blind spot is documented if the bound stays.

Dependencies:
the probe;
item 11 enforcement for the allowlist path.

### 11. Repo enforcement check

Files:
root task (new `test:time-bounds` or a unit test in
`package/test-harness/core`),
an allowlist file,
fixtures.

Acceptance:
rejects the forbidden shapes in fixtures;
accepts allowlisted entries with reasons;
runs in the default test or CI;
the check itself is proven failing on a rejected fixture before it is trusted.

Dependencies:
items 4,
5,
7,
8,
9,
10 to reach a clean baseline.

### 12. Documentation and skill

Files:
`package/test-harness/core/README.md`,
`.agents/skills/testing-practices/SKILL.md`,
`doc/troubleshooting/cli-git-logger-sink-verify-timeout.md`.

Acceptance:
the fake-timer recipe is replaced by the scope clock guidance;
the fake-timer workarounds remain only as guidance for external code;
duration guidance distinguishes observations,
virtual advance,
and audited real-time exceptions;
descriptor semantics are documented;
the runner supervisor's budget is documented;
the skill mirrors are regenerated with `mise run sync:files`.

Dependencies:
items 1 to 11,
or co-landed with them.

### 13. Issue records

Files:
issue [1762][i1762],
issue [1771][i1771].

Acceptance:
the 1762 body's acceptance is amended to the refined completion statement;
1771 gets the no-flag reproduction qualification;
both close only after their verification.

Dependencies:
items 3 and 11.

## Ordering

```text
1 scope + journal -> 5 describe -> 11 enforcement baseline
1 scope + journal -> 4 logger
3 async-time seam -> 4 logger
2 runner budget (independent)
6 descriptor memoization (independent)
7 gh-process, 8 mvm, 9 bash-poke (independent)
10 wg probe -> 11 enforcement allowlist
12 docs after 1, 3, 4, 6, 11
13 issue records after 3, 11
```

Items 2,
6,
7,
8,
and 9 can land any time.
Items 1,
3,
4,
5 have the longest dependency chain.

## Verification

1.  Per item:
    `mise run buildAndTest -- <test file>` green,
    plus `lint:types` after TypeScript edits.
2.  Positive controls (GFP/QPC):
    each new observation is shown failing against a deliberately wrong
    implementation before it is trusted.
3.  Scope probes from the architecture document:
    logger virtual flush,
    describe overlap,
    supervisor hang,
    descriptor memoization,
    attempted deadline with wedged work,
    browser/no-timer parity,
    seam compatibility,
    enforcement fixtures.
4.  Contention:
    batches at the runner's full width including the six subject
    files and the bash-poke file.
5.  Full default suite once:
    `mise run test:unit`.
6.  Docs:
    `mise run format:monochromatic-lint -- <paths>`,
    `mise run sync:files`,
    and a rendered check of touched Markdown.
7.  Issue close only after the verification plan completes.

## Commit discipline

One commit per work item,
scoped pathspecs,
as the original plan says.
The harness stages (items 1 to 3) land before the site migrations.
Broken intermediate states name the breakage,
per GCE.

## Owner decisions requested

These are owner-level,
not mechanism choices:

1.  Approve the architecture investment or prefer the smaller plan.
2.  Accept additive API changes on the harness and the two timing packages.
3.  Choose the enforcement policy and allowlist governance.
4.  Choose the wg route after the operation-counter probe.
5.  Choose the verification budget.
6.  Confirm the issue-record edits and close conditions.

## What the next reviewer should check

- Whether the execution-scope proposal is the right level of change,
  or whether promises should be abandoned more radically (effect interpreter).
- Whether the scope clock can be introduced without changing default behavior
  for 1081 unit files.
- Whether descriptor memoization breaks any self-test or consumer.
- Whether the runner budget can be added without masking real failures.
- Whether the enforcement check's shape set is narrow enough to avoid
  false positives and broad enough to catch the seven known shapes.

## References

[i1762]: https://github.com/Aquaticats/Monochromatic/issues/1762
[i1771]: https://github.com/Aquaticats/Monochromatic/issues/1771
