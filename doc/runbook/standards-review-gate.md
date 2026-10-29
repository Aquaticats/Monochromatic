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
`SHADOW_REVIEW_SHA` replays any commit instead of `HEAD`):

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

The runner judges nothing,
so it writes `Named superlinear cost` as `unjudged`
and a scorer reads the findings to settle `yes` or `no`.
The reviewer route is pinned to `openai/gpt-6.1-sol` through `pi`
because `pi`'s default provider answered a `402` `billing_error` on 2026-10-29
(`doc/troubleshooting/pi-advisor-provider-billing.md`);
`--no-tools` keeps the reviewer from running `git`,
which is the recursion guard.

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
