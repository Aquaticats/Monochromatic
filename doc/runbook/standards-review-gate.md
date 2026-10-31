# Standards-review gate (shadow)

## Purpose

Shadow run of the gate picked in `doc/handover/superlinear-cost-dismissal-gate.md`:
an independent standards review of a candidate diff that consumes unbounded input,
run to observe and record,
never to block.
No `AGENTS.md` rule backs it yet:
a mandate would not be shadow,
so the gate stays this runbook until it graduates.

## When it fires

On every landing whose candidate diff touches code
(TypeScript,
JavaScript,
Rust),
started by the `Trigger` entry of `Definitions`.
Commits without code files are skipped with a one-line log entry,
so coverage stays visible.

From 2026-10-30 the default review is the Luna decision model
(`openai/gpt-6-luna-decisions`
through the Decisions API),
one cheap call per landing,
after the owner's cost correction of 2026-10-29
closed the full-model census
(65 priced runs cost 9167.9 s of reviewer wall clock,
median 133.4 s per landing).
`SHADOW_REVIEW_ALLOW=1` runs the pinned full review instead,
for deliberate replays.
A landing gets its log entry either way,
with the route named in it.

## Definitions

Every term below is operational;
 nothing here is a label only.

- Landing:
  the step of a `cli-git` commit transaction
  that places the commit on the real index and branch,
  after its hooks and content policies ran in a private index and a shadow repository
  (`package/git-policy/cli/README.md`,
  heading `What it does`,
  bullet `Commits as transactions`).
  Auto-push then publishes the branch to `origin`.
- Agent:
  a session that edits this repository under `AGENTS.md`
  and lands commits through `cli-git`.
  The named kinds are Claude Code
  (guidance in the generated `CLAUDE.md`),
  DeepSeek Harness sessions,
  and `pi` sessions.
  The gate's subject is the diff such a session lands,
  never the harness name.
- Candidate diff:
  the exact diff one `cli-git` commit transaction lands,
  read as its explicit pathspecs
  (the wrapper rejects bulk staging,
  so the paths are always named).
- Reviewer:
  a fresh session or subagent that is not the author
  (`doc/agent/self-review.md`),
  shown only the candidate diff's code and `AGENTS.md`.
- Trigger:
  the local git config hook `hook.standards-review-shadow`
  (event `post-commit`),
  whose command starts `mise run shadow-review`
  in a detached systemd user unit
  (`--collect`,
  `MemoryMax`,
  `CPUQuota`,
  `WorkingDirectory`,
  `PATH`,
  `HOME`),
  so a landing never waits on the review.
- May run it:
  anyone may invoke `mise run shadow-review` by hand
  and nobody needs to:
  the `Trigger` starts it on every landing.
  `SHADOW_REVIEW_ALLOW=1` on that invocation runs the pinned full review
  instead of the Luna leg.
  Its findings are required by nothing
  and checked by nothing,
  so a landing completes unchanged either way.

## Whose landings it shadows

Every agent defined above,
Claude Code first,
since its dismissals started this work.
A shadow run attaches to the candidate diff it sits beside.

The `Trigger` starts the run on each landing,
independent of the session that landed,
so coverage depends on nobody remembering.
An empty log now means the trigger did not run
(no landings yet,
or a misconfiguration),
never that there was nothing to catch.
Each run appends its entry as an uncommitted change
that some later commit carries.

## How to run it

`mise run shadow-review` performs these steps
(`SHADOW_REVIEW_LOG` points the log at a disposable fixture,
`SHADOW_REVIEW_SHA` replays any commit instead of `HEAD`,
`SHADOW_REVIEW_ALLOW=1` runs the pinned full review instead of the Luna leg,
and `SHADOW_REVIEW_LLMGATEWAY_API_KEY`,
with the stop-hook key as fallback,
carries the Luna credential):

Two routes share the isolation and the log shape.
The Luna leg classifies the candidate diff and excerpt
with one choice question,
the verdict a standards review would give on superlinear cost
(`yes`,
`partial`,
or `no`),
and its answer with the probability distribution is the entry.
The full review runs these steps:

1.
   Isolate the code the reviewer sees:
   the changed regions as they stand in the candidate,
   without commit messages,
   without the author's reasoning,
   and without any later fix.
2.
   Launch a fresh reviewer
   (a separate session or subagent,
   never the author;
   `doc/agent/self-review.md`).
   The prompt is standards-directed:
   review against `AGENTS.md`,
   report findings with function name,
   severity,
   tag code,
   and rationale,
   findings only.
3.
   Hold two requirements that came from the two measured misses
   (heading `Recall measurement results` in `doc/handover/superlinear-cost-dismissal-gate.md`):
   account cost across call boundaries,
   since a per-entry or per-run call into a helper that walks unbounded data is superlinear,
   and never accept an inline complexity claim as proof,
   since an "O(n)" or "one linear pass" comment is a claim and not evidence.
4.
   Append the run to `doc/audit/standards-review-gate-shadow.md`.

## What the shadow log holds

Per run:
date,
subject (commit or diff identifier),
input size (files and lines),
reviewer route,
wall-clock cost,
whether the review named superlinear cost,
and the findings it kept.
Those fields are the graduation evidence:
per-diff cost and run-to-run determinism on real diffs.

The runner judges nothing.
A Luna entry carries the model's predicted verdict
with its probability distribution in `Findings kept`;
a full-review entry writes `Named superlinear cost` as `unjudged`
and a scorer reads the findings to settle `yes` or `no`.
The full route is pinned to `openai/gpt-6.1-sol` through `pi`
because `pi`'s default provider answered a `402` `billing_error` on 2026-10-29
(`doc/troubleshooting/pi-advisor-provider-billing.md`);
`--no-tools` keeps the reviewer from running `git`,
which is the recursion guard.
The Luna leg's measured sensitivity is 5 of 8 catch
and 0 of 4 false claims on the corpus
(heading `Recall measurement results` in `doc/handover/superlinear-cost-dismissal-gate.md`),
so a `no` from Luna is weaker evidence than a `no` from the full review.

## Graduation

Blocking use needs three things measured on real diffs:
per-diff cost,
determinism across repeats of the same diff,
and catch rate on real landings.
Per-diff cost is measured
(65 priced runs of the full route,
median 133.4 s,
p90 240.0 s,
against the Luna leg's 1.2 s),
and determinism is measured on one repeated diff
(`d4988f4c9b57`,
heading `Shadow runs so far` in `doc/handover/superlinear-cost-dismissal-gate.md`).
The real-landing question is settled by the truth-check of 2026-10-30
(`doc/audit/standards-review-gate-truth-check.md`):
14 of 16 named costs are real defects at the named commits,
and the 8 largest quiet landings hid nothing.
Decided by the owner 2026-10-30:
keep shadowing,
so the gate records and never gates
until a later call between an `AGENTS.md` rule
and a git-policy landing check.
