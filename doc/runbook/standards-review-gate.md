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

When a diff about to land touches code that consumes unbounded input
(text,
rows,
file trees,
anything whose length the code itself does not bound).
In shadow phase any agent may run it on such a diff;
nothing is required,
and nothing is blocked either way.

## Whose landings it shadows

Agent sessions landing diffs in this repo:
Claude Code first,
since its dismissals started this work
and `CLAUDE.md` is generated for it,
and every other harness that lands here,
this one included.
The subject is the diff,
not the harness:
a shadow run attaches to the landing it sits beside.

In shadow phase the landing session triggers the run
on its own diffs
and appends to the log.
Nothing forces it,
so an empty log means no run happened
and never means there was nothing to catch.
A non-blocking landing hook that runs the review
and only logs is the unvolunteered alternative,
kept for the graduation discussion.

## How to run it

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

## Graduation

Blocking use needs three things measured on real diffs:
per-diff cost,
determinism across repeats of the same diff,
and catch rate on real landings.
Then decide with the owner between an `AGENTS.md` rule
and a git-policy landing check,
and only then unshadow.
Until a decision lands,
the gate records and never gates.
