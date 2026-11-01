# DeepSeek limitations

Documenting a failed `/grilling` session on the wall-clock bound remediation
plan,
run on 2026-10-18,
so future sessions do not repeat the same failure pattern.
The session ran on the `deepseek-v4.1-flash-1017` harness model.

## Wall-clock bound remediation review and grilling (2026-10-18)

### Goal

Review
[the approved remediation plan](../planning/wall-clock-bound-test-remedies.md)
for issue [1762][i1762],
settle the plan's open questions by grilling the owner,
then execute the refined plan.
The plan covered seven real-elapsed upper bounds in the default unit suite,
a `withTimeout` cross-domain clear defect ([issue 1771][i1771]),
and documentation work.

### Files and resources involved

- `doc/planning/wall-clock-bound-test-remedies.md`
- `doc/planning/wall-clock-bound-test-remedies-review.md`
- `doc/planning/test-harness-execution-architecture.md`
- `doc/planning/wall-clock-bound-test-remedies-refinement.md`
- `doc/planning/wall-clock-bound-test-grilling-record.md`
- `doc/handover/wall-clock-bound-unit-tests.md`
- `doc/troubleshooting/cli-git-logger-sink-verify-timeout.md`
- `package/test-harness/core/`,
  `package/test-harness/core-sandbox/`,
  `package/async-time/core/`,
  `package/logger/core/`
- `mise.toml` test runner tasks
- issue [1762][i1762],
  issue [1771][i1771],
  issue [510][i510]

### What worked before the grilling

The factual review was largely correct and is still useful:

- It found the inventory error:
  seven real-elapsed upper bounds in six files,
  not seven in five.
  The missed site was
  `package/pi-plugin/bash-poke/src/job-runner.unit.test.ts:337`.
- It reran both archived probes and their no-flag variants.
  The no-flag P1 probe exited 0 with the sinon throw on stderr,
  so the recorded deterministic deadlock did not reproduce.
- It reproduced the wg-quicker timing and showed that the 250 ms bound
  does not detect representative quadratic work at 4000 prefixes.
- It identified a timing-free marker observation for the mvm site.
- It corrected the item 1 layer name:
  `withHostTimeout` is the logger wrapper,
  not the harness boundary.
- It posted the review on issue [1762][i1762]
  ([comment][c1762-review]).

The failure was not fact-finding.
It was the design conversation.

### Question round 1: per-site workarounds

After reviewing,
DeepSeek sent five questions:

1.  Add,
    keep,
    or defer the bash-poke bound.
2.  Drop the mvm bound for a marker,
    derive it,
    or use a named absolute.
3.  Retune the wg guard,
    use a two-size ratio,
    or document it.
4.  Amend the issue acceptance,
    post a superseding comment,
    or leave it.
5.  Put the logger fake-clock tests under root `concurrency: 1` or split
    the test file.

**Why it failed:**

- Every question treated the current harness architecture as fixed:
  process-global fake timers,
  per-file serialization or splitting,
  and per-site test edits as the only remedy space.
- Question 5 was the clearest failure.
  With harness ownership,
  neither option is right:
  the harness should not require process-global timer replacement at all.
  Asking the owner to choose between two workarounds for a self-imposed
  limitation is design abdication,
  not an interview.
- The questions asked the owner to make engineering choices
  (file placement,
  test shape,
  commit granularity)
  instead of presenting contracts or outcomes.
- The owner's correction:

  > Those were very very bad questions.
  > Again,
  > we control the entire test harness.

### Question round 2: harness capability menus

DeepSeek retracted the per-site framing and sent four harness questions:

1.  Interval API:
    raw start and end,
    a computed summary,
    or both.
2.  Virtual time:
    a provider seam in `module-async-time`,
    a global timer
    multiplexer,
    or an exclusivity lock for sinon fake timers.
3.  Descriptor semantics:
    memoize,
    throw on second consumption,
    or document
    the re-execution trap.
4.  Clock opt-in:
    suite-level,
    per-attempt,
    or both.

**Why it failed:**

- These were mechanism menus.
  API shape and seam placement are engineering decisions,
  not owner decisions.
- The recommendations preserved the unsafe default.
  Virtual time as an opt-in and re-execution as documented behavior keep the
  hazard for every test that does not know to opt in.
- DeepSeek ignored the incumbent ownership systems.
  `module-test-sandbox` already owns per-attempt contextual method replacement,
  and the runner already owns async-context storage.
  The first design question should have been how to extend that ownership,
  not which new seam to invent.
- DeepSeek did not question the promise-race model.
  Every option was about swapping globals,
  locking,
  or opting in.
  None asked whether a deadline should be owned outside promises.
- The owner's correction:

  > These are once again very very bad questions.

  and then:

  > Both.
  > And many more.
  > Many more:
  > For example,
  > if you believe promises inherently can't handle timeouts
  > correctly,
  > you should propose re-architecting the test runner to use
  > something else.

### Question round 3: better topics, same shape

DeepSeek sent four questions about where a timeout,
time,
observation,
and enforcement should live.
The topics were closer,
but they arrived only after the owner supplied the
promise-race critique,
and they were still option menus with hedged
recommendations.
The recommended clock design was a hidden ambient seam rather than an explicit
owner.

The owner paused the session:

> Still bad.
> I need to take a break.
> I'm not angry.
> I'm just slightly disappointed.
> Write all your findings and whatever you think in .md files.
> I'll get someone else to review it first before deciding whether to continue
> this session.

### Root cause

DeepSeek optimized inside the given architecture instead of questioning it:

- It treated the execution model,
  the runner,
  the harness,
  and the packages
  under test as fixed constraints.
- It asked the owner to choose mechanisms instead of deciding them itself.
- It wrote questions before writing a proposed design.
- It never asked the first question that mattered:
  what is the harness's execution contract,
  who owns an execution after it starts,
  and what may a test verdict depend on?
- It read code for facts but not for ownership:
  the sandbox's contextual method system and the runner's async-context
  storage were visible in the tree and were not used in the design.
- It kept proposing workarounds for a primitive that cannot express
  cancellation and cleanup.

### Correct behavior for future sessions

- Establish the contract before the remedies.
  Ask who owns execution,
  cancellation,
  time,
  cleanup,
  and observation,
  and what a verdict may depend on.
- Treat the harness and packages under test as part of the remedy.
  Read their ownership mechanisms before proposing new seams.
- When a primitive cannot express the required semantics,
  propose re-architecting the runner instead of patching around it.
- Write the design first,
  then ask only about goals,
  constraints,
  authority,
  and non-measurable preferences.
  Decide API shape,
  seam placement,
  and file organization.
- Make safe behavior the default.
  A capability that tests must opt into does not fix a class of failures.
- Enforce invariants mechanically.
  Documentation is not an acceptance test.
- Do not use a multi-round interview to derive a design that should have been
  written down first.

### Outcome

Implementation remains paused.
The session produced:

- [the plan review](../planning/wall-clock-bound-test-remedies-review.md),
  with a status note marking its superseded recommendations;
- [the execution architecture proposal](../planning/test-harness-execution-architecture.md);
- [the refined remediation plan](../planning/wall-clock-bound-test-remedies-refinement.md);
- [the grilling session record](../planning/wall-clock-bound-test-grilling-record.md).

No implementation code was changed.

### Lessons

Do not rely on DeepSeek to run a design interview on a codebase it is allowed
to change without first checking that it will:

- question the execution model instead of optimizing assertions inside it;
- distinguish owner-level decisions from engineering choices;
- inspect and extend existing ownership mechanisms;
- write the proposal before asking for design decisions;
- propose re-architecture when the primitive is wrong.

The factual layer was competent.
The architectural layer was not.

[i1762]: https://github.com/Aquaticats/Monochromatic/issues/1762
[i1771]: https://github.com/Aquaticats/Monochromatic/issues/1771
[i510]: https://github.com/Aquaticats/Monochromatic/issues/510
[c1762-review]: https://github.com/Aquaticats/Monochromatic/issues/1762#issuecomment-6255729314
