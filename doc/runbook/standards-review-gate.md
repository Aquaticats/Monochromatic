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
The full review through `pi` (pinned `openai/gpt-6.1-sol`) is no longer run by anything:
the port into the resident service does not start child programs,
and the owner can ask for an embedded full-review client separately.
A landing gets its log entry,
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
  the wrapper.
  When the resident service lands a commit in a worktree that holds the tracked file
  `doc/audit/standards-review-gate-shadow.md` (the switch,
  frozen history that nothing writes any more),
  the wrapper leaves a `review` work item for that commit in `<git dir>/cli-git/work/`
  (`doc/planning/cli-git-standards-review-port.md`,
  heading `The work-item record`).
  The service claims it on its work thread after the command has returned.
  The former trigger,
  the local Git config hook `hook.standards-review-shadow` running `mise run shadow-review`,
  and that task are removed:
  cli-git runs no program that any configuration names.
- May run it:
  nobody needs to,
  and nobody can start it by hand:
  the service runs it for every landed commit in a worktree that holds the switch file.
  Its findings are required by nothing
  and checked by nothing,
  so a landing completes unchanged either way.

## Whose landings it shadows

Every agent defined above,
Claude Code first,
since its dismissals started this work.
A shadow run attaches to the candidate diff it sits beside.

The `Trigger` leaves the work item on each landing,
independent of the session that landed,
so coverage depends on nobody remembering.
An empty log means the service did not run the item
(no landings yet,
no service installed,
or a failed item in `<git dir>/cli-git/work/failed/`),
never that there was nothing to catch.
Each run appends its entry to the repository's live log,
`<common dir>/cli-git/standards-review-shadow.md`
(the common directory is `.git` of the main checkout),
an untracked file inside the repository directory,
so no checkout ever shows it as a change and no commit carries it.
There is one live log per repository,
shared by every worktree's service,
and it has no rotation:
it grows by one entry per landed commit,
and the once-per-commit check reads all of it.
Entries written before 2026-11-01 are in the tracked file;
the entry format is the same in both files,
except the receipt line the service writes last,
which the entries of that file lack.

## How to run it

Nobody runs it by hand.
The resident service runs one `review` work item per landed commit
(`package/git-policy/cli/src/native/service/service_review.rs`),
within a 20 s budget on the service's work thread:
a call deadline of 10 s for the gateway and the Git reads and the write inside the rest.
The environment of the service carries the settings:
`SHADOW_REVIEW_LOG` points the live log at a disposable fixture,
and `SHADOW_REVIEW_LLMGATEWAY_API_KEY`,
with the stop-hook key as fallback,
carries the credential.
Without a credential the entry records `Luna failed: no ... API_KEY`.

The review classifies the candidate diff and excerpt
with one choice question,
the verdict a standards review would give on superlinear cost
(`yes`,
`partial`,
or `no`),
and its answer with the probability distribution is the entry.
The steps:

1.
   Read the landed commit's names,
   subject,
   patch and the changed code files
   (two Git starts for a commit with code,
   none for a commit without).
2.
   Build the capped input and ask the gateway once.
   A failure of the call is an entry (`Luna failed: ...`),
   not a failed item.
3.
   Append the entry to the live log
   with one append-only write on an `O_APPEND` descriptor,
   once per commit,
   so entries of several worktrees' services never interleave and no lock is taken
   (`package/git-policy/cli/src/native/standards_review/standards_review_append.rs`).
   The service creates the directory and the file when missing.
   The tracked `doc/audit/standards-review-gate-shadow.md` is never written.
   The earlier spool and the repair of entries another writer overwrote are gone
   (`doc/planning/cli-git-standards-review-port.md`,
   heading `Appending to the shared log`,
   dated note).
4.
   A commit that cannot be read,
   a log that cannot be written,
   or a review that runs past its budget
   leaves its item in `<git dir>/cli-git/work/failed/` as a plain record and nothing else.

## What the shadow log holds

Per run:
date,
subject (commit or diff identifier),
input size (files and lines),
reviewer route,
wall-clock cost,
whether the review named superlinear cost,
the findings it kept,
and the receipt line,
`reviewed: <the whole commit name>`.

The receipt is the once-per-commit check's only evidence,
and it is written as the entry's last line,
line feed included.
The check compares it as one whole line,
never as a name found somewhere in the log,
so text a remote answer or a subject happens to spell cannot suppress another
commit's review,
and a receipt the system cut short before its line feed,
or an entry cut short before its receipt,
is not taken as finished:
the retry appends the whole entry and leaves the fragment where it is.
A commit whose entry was written before the receipt line existed,
and whose item is queued again,
is therefore reviewed once more.

Every append begins with a line feed of its own,
so an entry is never joined to a line an earlier write left unfinished,
and the file begins with the first entry's line feed.
An entry's own text ends with one line feed,
so two entries sit one blank line apart.

The log is read as bytes and its receipt is compared as bytes,
so a tail a cut-short write left in the middle of a character,
or of a line,
never stops a later review from being logged:

- A fragment of a character stays where it is and costs only its own line.
- A fragment of a line stays where it is,
  and the entry written after it starts on the next line.

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
