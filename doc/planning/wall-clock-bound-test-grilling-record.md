# Session record: review and grilling of the wall-clock plan

## Purpose

This record exists for the next reviewer.
It states what was established,
what was proposed,
what the owner corrected,
and why two question rounds failed to reach a usable design.
It is deliberately candid:
the same framing mistakes should not be repeated
by the next session.

It is not a decision record.
No implementation is authorized.
The current proposals are in
[the execution architecture](test-harness-execution-architecture.md)
and
[the refined remediation plan](wall-clock-bound-test-remedies-refinement.md).

## What happened

1.  I reviewed
    [the approved remediation plan](wall-clock-bound-test-remedies.md)
    against the tree,
    the issues,
    and the handover.
    I posted
    [the review](wall-clock-bound-test-remedies-review.md)
    as a comment on [issue 1762][i1762]
    ([comment][c1762-review]).
    The review found real defects:
    the inventory missed bash-poke,
    the fake-timer sequencing boundary was underspecified,
    item 5 had a timing-free observation,
    item 1's alternative was the logger wrapper rather than the harness,
    and the acceptance did not match the issue text.
2.  The owner asked to be grilled to refine the plan and then execute.
    I sent a first round of five questions about per-site remedies,
    file placement,
    acceptance edits,
    and commit granularity.
    The round was rejected.
    The correction was that we control the entire test harness,
    so the
    per-site framing was wrong.
3.  I retracted the per-site framing and sent a second round about harness
    capabilities:
    interval API shape,
    virtual-time mechanism,
    descriptor semantics,
    and clock opt-in.
    That round was rejected too.
    The correction added that promises cannot handle timeouts correctly,
    and that the response should be to re-architect the runner.
4.  I sent a third round that asked where timeouts,
    time,
    observations,
    and enforcement should live.
    It was rejected as well.
    The owner paused the session and asked for written findings before
    deciding whether to continue.
5.  This record and the two proposal documents are the result.

## The corrections, as design constraints

The owner's words carry the constraints:

- "We control the entire test harness."
  The harness,
  the runner,
  the sandbox,
  and the packages under test are ours.
  Capabilities are remedies;
  per-test workarounds are not the default answer.
- "If you believe promises inherently can't handle timeouts correctly,
  you should propose re-architecting the test runner to use something else."
  The promise-race model is not a fixed constraint.
  A design that patches around it is answering the wrong question.
- "Both.
  And many more."
  The two named mistakes (engineering choices presented as owner decisions,
  and ignoring the existing sandbox ownership seam) are examples,
  not the list.

## Why the question rounds failed

### Round 1: per-site workarounds inside a fixed model

The questions were:

- add,
  keep,
  or defer the bash-poke bound;
- drop the mvm bound for a marker,
  derive it,
  or use an absolute;
- retune the wg guard,
  use a ratio,
  or document it;
- amend the issue acceptance,
  comment,
  or leave it;
- put the logger fake-clock tests under root `concurrency: 1` or split the file.

Each question assumed the harness was fixed:

- that fake timers must remain process-global;
- that a file must serialize or split to host a clock test;
- that the site edits are independent decisions;
- that the owner should choose test file organization.

The file-placement question was the clearest failure.
With harness ownership,
the answer is neither option:
the harness should not require process-global replacement at all.
Asking the owner to choose between two workarounds for a self-imposed
limitation is not an interview,
it is a design abdication.

### Round 2: mechanism menus that preserved unsafe defaults

The questions were:

- raw intervals,
  summary only,
  or both;
- a production provider seam,
  a global timer multiplexer,
  or an exclusivity
  lock;
- memoize descriptors,
  throw on second consumption,
  or document the trap;
- suite-level clock opt-in,
  per-attempt opt-in,
  or both.

The failures:

- They asked the owner to choose between implementations.
  API shape and seam placement are engineering decisions.
- The recommendations preserved the unsafe default.
  Virtual time as an opt-in and re-execution as documented behavior keep the
  hazard for every test that does not know to opt in.
- They ignored incumbents.
  `module-test-sandbox` already owns per-attempt contextual method replacement
  and the runner already owns async-context storage.
  The first design question should have been how to extend that ownership,
  not which new seam to invent.
- They did not question the promise model.
  Every option was about swapping globals,
  locking,
  or opting in;
  none asked whether deadlines should be owned outside promises.

### Round 3: the right topics, still the wrong shape

The questions were:

- where a timeout should live;
- where time should live;
- what tests should observe;
- how the invariant should be enforced.

These were closer,
but they arrived only after the owner supplied the
promise-race critique.
They were still option menus with hedged recommendations,
and the recommended clock design was a hidden ambient seam rather than an
explicit owner.
The root question still went unasked.

### The root question that should have come first

What is the harness's execution contract?

- Who owns an execution after it starts?
- Can a deadline cancel work,
  and who runs cleanup?
- Is time a capability or an ambient global?
- What does a result contain besides a name?
- What may a verdict depend on?
- How is the contract enforced?

Once those are settled,
per-site remedies,
API shapes,
and file placement
follow.
Without them,
every question is a workaround negotiation.

## Current state of understanding

Tentative and for review,
not settled:

- An execution scope should own the clock,
  cancellation,
  cleanup,
  and journal.
- Time should be a capability passed to timing code under test,
  not a process-global replacement.
- Results should carry execution events,
  so concurrency and deadline proofs
  are observations.
- Descriptors should execute once per dispatch.
- The runner should bound hung files at the process boundary.
- Real-time supervision remains necessary for uncooperative work and real
  subprocesses;
  real-time verdicts should be audited exceptions enforced by a
  repo check.
- The seven site fixes are acceptance cases for the scope,
  not the design itself.

## What remains unresolved

The owner-level decisions are listed in
[the refined plan](wall-clock-bound-test-remedies-refinement.md#owner-decisions-requested):

1.  Approve the architecture investment or prefer the smaller plan.
2.  Accept additive public API on the harness and timing packages.
3.  Choose the enforcement policy and allowlist governance.
4.  Choose the wg-quicker route after the operation-counter probe.
5.  Choose the verification budget.
6.  Confirm the issue-record edits and close conditions.

## Process lessons for the next session

- Write the proposed design before writing questions.
  A question without a proposal is an invitation to design by proxy.
- Ask only about goals,
  constraints,
  authority,
  and non-measurable preferences.
  Decide mechanisms and API shapes in the document.
- Treat the harness and the packages under test as part of the remedy.
  Read their ownership systems before proposing a new one.
- Challenge the execution model before optimizing assertions inside it.
- Do not sequence a multi-round interview from the leaves.
  Establish the invariant first;
  the rest is derivable.
- When the owner corrects the framing,
  rewrite the proposal rather than
  rephrasing the same options.

## Artifact index

- [Approved remediation plan](wall-clock-bound-test-remedies.md)
- [Plan review](wall-clock-bound-test-remedies-review.md)
- [Execution architecture proposal](test-harness-execution-architecture.md)
- [Refined remediation plan](wall-clock-bound-test-remedies-refinement.md)
- [Investigation handover](../handover/wall-clock-bound-unit-tests.md)
- [Issue 1762][i1762],
  [issue 1771][i1771],
  [issue 510][i510]

[i1762]: https://github.com/Aquaticats/Monochromatic/issues/1762
[i1771]: https://github.com/Aquaticats/Monochromatic/issues/1771
[i510]: https://github.com/Aquaticats/Monochromatic/issues/510
[c1762-review]: https://github.com/Aquaticats/Monochromatic/issues/1762#issuecomment-6255729314
