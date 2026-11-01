# Handover: 2026-10-20 needs-triage issue backlog triage

## Task (user instructions)

"Triage all the issues that needs-triage."

Applied to every open GitHub issue in `Aquaticats/Monochromatic` carrying the `needs-triage` label.

## Purpose and result

The `needs-triage` queue is drained. Each of the 1062 open issues that carried the label now carries
exactly one lifecycle verdict from `doc/agent/triage-labels.md`, and no open issue carries `needs-triage`.
Verdicts approved work or routed it to a human; no code was changed.

## Facts measured (re-verify before trusting)

- 1062 open issues carried `needs-triage` at 2026-10-20 15:18
  (`gh api 'search/issues?q=repo:Aquaticats/Monochromatic+is:issue+is:open+label:needs-triage'`, total_count 1062
  of 1367 open issues).
- 143 closed issues carried a stale `needs-triage`; the label was cleared from all of them
  (`doc/audit/issue-triage-2026-10-20.evidence.jsonl` does not cover these;
  the run log is `${HOME}/temp/agent/issue-triage-2026-10-20/closed-stale.applied.jsonl`).
- Final verdicts over the 1062: `ready-for-agent` 923, `ready-for-human` 99, `wontfix` 39, `needs-info` 1.
- 39 issues closed as not planned (every `wontfix`); 139 triage comments posted, each starting
  `Triage 2026-10-20: `.
- Label hygiene beyond the verdict: a second `difficulty:*` dropped on #983 and #984; a first
  `difficulty:*` added on #1768, #1770 and #1786.
- End-state check (`verify-queue.mjs`): zero open issues with `needs-triage`, zero decided issues missing
  its verdict label, one state note recorded under "Known wrinkles" (#1338).

## Decision rubric applied

Verdicts come from `doc/agent/triage-labels.md`; the procedure used for every issue was:

1. Read the body and every comment thread; never classify from the title alone.
2. A comment recording a maintainer decision ("Triage decision: ...") governs the verdict.
3. An existing lifecycle label is kept unless the read contradicts it.
4. Named location plus stated desired outcome with nothing left to choose means `ready-for-agent`.
5. A choice between competing actions, a policy or taste call, human-only access (credentials, external
   accounts, hardware, upstream contact), or open design points means `ready-for-human`.
6. A missing fact only the reporter has means `needs-info`; in-repo investigation work is `ready-for-agent`.
7. Superseded, already fixed, or retired code means `wontfix`, closed as not planned, only after probing
   the tree (grep, read, `git show`) for the claim; unverifiable claims stay `ready-for-agent` and say so.
8. The same defect as an issue that owns it means `duplicate`, closed with the owner named.
9. Torn between an approval and a reject verdict, take `ready-for-agent`: approval is reversible,
   a wrong close is not.

Hard rules: `needs-triage` is removed in every outcome; orthogonal labels (`bug`/`documentation`/
`enhancement`, `difficulty:*`, `app:*`, `deferred-new-packages`, `priority:*`, `blocked-on-*`) are kept;
exactly one `difficulty:*` per issue; `wontfix`, `duplicate`, `needs-info` and `ready-for-human` carry a
comment with the specific reason or decision question; routine `ready-for-agent` records its reason in
the ledger line only. Every reason cites something specific from the body: a path, a symbol, a symptom,
a named rule.

## Evidence

- `doc/audit/issue-triage-2026-10-20.evidence.jsonl`: one JSON object per issue, sorted by number, with
  `number`, `title`, `verdict`, `reason`, `commentPosted`, `closed`, `labelsBefore`, `labelsAfter`.
- Scratch inputs (not durable) under `${HOME}/temp/agent/issue-triage-2026-10-20`: the fetched issue and
  comment snapshots (`issues-rest.json`, `comments.json`), the ten batch files with structural priors
  (`batch-b1.json` ... `batch-b10.json`), the per-batch decision ledgers (`decisions-b*.jsonl`), and the
  scripts (`fetch-rest.mjs`, `fetch-comments.mjs`, `prepare.mjs`, `apply-decisions.mjs`,
  `reconcile-verdicts.mjs`, `verify-queue.mjs`, `build-ledger.mjs`, `cleanup-closed.mjs`).

## Execution shape

Ten disjoint batches by issue number (107 issues each except #1667-#1786 with 99), each decided by one
agent from the fetched snapshot, appending decisions to its own ledger and applying them through one
shared idempotent script. A structural pre-pass suggested a verdict per issue (decision-question phrases
push `ready-for-human`, stated criteria push `ready-for-agent`); agents were told the suggestion is a
prior and overrode it on the read, in both directions.

## Judgment calls to review

Each is recorded with its reason in the evidence ledger and is easy to flip:

- #915 and #916: property-table lookup versus case-mapping semantics; the acceptance criteria pin
  behaviour preservation, so the deterministic table path was approved. Flip both to `ready-for-human`
  if the classifier semantics should be settled first.
- #651 and #655: approved as prescribed consolidations while sibling briefs asking for "the chosen
  option is recorded here" went `ready-for-human`. The split is consistency-sensitive.
- #661: the doc write-up was approved (per `DL3`) over the body's alternative of installing mold or lld
  in dev environments and containers, which read as scope expansion.
- #666 and #667: #666 `ready-for-human` (home is the reviewer's call), #667 `ready-for-agent`
  (home `package/rust-module/error-support` is prescribed).
- #997: experimentally completed work tracked here, production adoption in #998; closed as not planned
  with `wontfix` per the rubric's superseded rule, which is semantically awkward for completed work.
  Reopen and close as completed if the maintainer prefers.
- #1022: both repo-side acceptance boxes are checked and the rest waits on an upstream `cli/cli` change;
  `ready-for-human` rather than `wontfix` only because the upstream path is still live.
- #1305: `errorOnExist: true` plus two README promises fix the intended behaviour, so the
  collision-rejection branch was approved over the overwrite-documentation branch.
- #1660 and #1662: rule-versus-practice pairs resolved on the behaviour-preserving branch the bodies
  themselves supply (amend the convention comment). The sweep-and-enforce alternative is a policy call.
- #1751: title asks to relax the four-level heading rule, body asks to promote one heading; the body
  reading was routed to `ready-for-human` rather than decided.

## Known wrinkles

- #1338 was triaged `ready-for-agent` and then completed by concurrent work on 2026-10-20
  (comment at 19:20 UTC reports the Oxlint gates green). The verdict label stays as the triage record;
  the issue is closed as completed.
- #682 was first decided `ready-for-agent` before its retirement probe ran; corrected to `wontfix` with
  the same `7ac15b19b4` legacy-source evidence as #683-#697, closed, and commented.
- Two batch ledgers needed a one-row JSON-escaping repair (#701 and #1613) after tool-call encoding broke
  the row; both repairs changed escaping only.

## Decision questions waiting on the maintainer

99 issues carry `ready-for-human`; each has a `Triage 2026-10-20:` comment restating its question.
Highest-leverage groups:

- Policy and rule-versus-practice calls: #989, #1231, #1654, #1655, #1751, #677, #679, #1650.
- External account access only the maintainer holds: #999, #1000, #1001, #1040, #1046, #675, #676.
- Contract and semantics choices with two defensible branches: #987, #1041, #1042, #1043, #1053, #1077,
  #1172, #1219, #1274, #1323, #1374, #1550, #1774, #1786.
- Ownership and home decisions: #611, #616, #666, #669, #672, #991, #995, #1619, #1627, #1643, #1667.
- Full list with reasons: `doc/audit/issue-triage-2026-10-20.evidence.jsonl`, verdict `ready-for-human`.

## What to inspect

- `doc/audit/issue-triage-2026-10-20.evidence.jsonl` for the per-issue verdict and reason.
- GitHub label state: no open issue carries `needs-triage`; every decided issue carries its verdict label.
- The judgment calls section above if any split should be re-routed.

## How to respond

- Answer the decision questions on the ready-for-human issues, or say which to batch into one
  decision doc for a single deliberation.
- Name any judgment call above that should flip; each is a one-issue label and comment change.
- The `ready-for-agent` set is approved work under `doc/handover/contributing-policy.md`; "resolve issue N"
  still means fix and commit per `doc/agent/issue-tracker.md`.

## Maintainer answers, 2026-10-20

- The 99 decision questions stay on their issues as posted; no folded decision doc.
- The nine recorded judgment calls stand as triaged; no verdict flipped at triage time.
