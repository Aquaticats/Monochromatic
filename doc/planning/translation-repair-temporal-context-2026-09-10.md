# Temporal and participant source context

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Temporal and participant source context: the window hypothesis and its first repair-path check](translation-repair-temporal-context-2026-09-10-source-context-window-and-first-repair-path-check.md):
  The evidence for the missing-input hypothesis,
  the matched selector and first-writer probes,
  the forward-only heading boundary that was implemented,
  and the first compiled repair-path check that left a contract boundary unresolved.
  Sections:
  - [Actual evidence](translation-repair-temporal-context-2026-09-10-source-context-window-and-first-repair-path-check.md#actual-evidence)
  - [Matched experiment](translation-repair-temporal-context-2026-09-10-source-context-window-and-first-repair-path-check.md#matched-experiment)
  - [Completed selector-input measurement](translation-repair-temporal-context-2026-09-10-source-context-window-and-first-repair-path-check.md#completed-selector-input-measurement)
  - [Completed first-writer context probe](translation-repair-temporal-context-2026-09-10-source-context-window-and-first-repair-path-check.md#completed-first-writer-context-probe)
  - [Implemented heading boundary](translation-repair-temporal-context-2026-09-10-source-context-window-and-first-repair-path-check.md#implemented-heading-boundary)
  - [Completed existing repair-path verification exposes an unresolved contract boundary](translation-repair-temporal-context-2026-09-10-source-context-window-and-first-repair-path-check.md#completed-existing-repair-path-verification-exposes-an-unresolved-contract-boundary)
- [Temporal and participant source context: experiments isolating the conflicting-diagnosis boundary](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md):
  The experiments after the first repair-path check:
  context-purpose rules for critics and panels,
  claim extent and document evidence,
  the editor responsibility treatment,
  the accurate-claim control,
  isolated and clustered panel review,
  and the selector evidence checks;
  the claim-extent and editor responsibility treatments were not integrated,
  while the evidence roles,
  cluster packets and selector evidence they measured were implemented later
  in the cluster panel file.
  Sections:
  - [Next measured boundary](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#next-measured-boundary)
  - [Completed evidence-scope experiment](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-evidence-scope-experiment)
  - [Completed claim-extent and source-evidence experiment](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-claim-extent-and-source-evidence-experiment)
  - [Completed editor responsibility experiment](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-editor-responsibility-experiment)
  - [Completed accurate-claim diagnostic control](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-accurate-claim-diagnostic-control)
  - [Completed isolated-claim panel diagnostic](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-isolated-claim-panel-diagnostic)
  - [Completed existing-cluster review diagnostic](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-existing-cluster-review-diagnostic)
  - [Completed stock-window cluster-to-editor verification](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-stock-window-cluster-to-editor-verification)
  - [Completed repair-selector evidence check](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-repair-selector-evidence-check)
  - [Completed existing-English comparison check](translation-repair-temporal-context-2026-09-10-conflicting-diagnosis-isolation-experiments.md#completed-existing-english-comparison-check)
- [Temporal and participant source context: the cluster panel and evidence handoffs, implemented](translation-repair-temporal-context-2026-09-10-cluster-panel-and-evidence-handoffs-implemented.md):
  Tasks 22 and 23 as implemented:
  cluster review packets,
  evidence roles and the source evidence handoff to selectors,
  with the compiled full-path check that remained insufficient.
  Sections:
  - [Implemented packet and evidence boundaries](translation-repair-temporal-context-2026-09-10-cluster-panel-and-evidence-handoffs-implemented.md#implemented-packet-and-evidence-boundaries)
  - [Completed compiled full-path check remains insufficient](translation-repair-temporal-context-2026-09-10-cluster-panel-and-evidence-handoffs-implemented.md#completed-compiled-full-path-check-remains-insufficient)
- [Temporal and participant source context: the category conflict and the merged-issue authority fix](translation-repair-temporal-context-2026-09-10-category-conflict-and-merged-issue-authority-fix.md):
  The category-conflict diagnostic and the accuracy-category policy that was integrated,
  the compiled check that showed a rejected diagnosis promoted by grouped status,
  and the tally-boundary fix that preserves member verdicts,
  with its verification.
  Sections:
  - [Completed category-conflict diagnostic](translation-repair-temporal-context-2026-09-10-category-conflict-and-merged-issue-authority-fix.md#completed-category-conflict-diagnostic)
  - [Completed accuracy-category scope measurement](translation-repair-temporal-context-2026-09-10-category-conflict-and-merged-issue-authority-fix.md#completed-accuracy-category-scope-measurement)
  - [Genuine omission control and integration](translation-repair-temporal-context-2026-09-10-category-conflict-and-merged-issue-authority-fix.md#genuine-omission-control-and-integration)
  - [Completed compiled category-path check](translation-repair-temporal-context-2026-09-10-category-conflict-and-merged-issue-authority-fix.md#completed-compiled-category-path-check)
  - [Preserve member verdicts through merging](translation-repair-temporal-context-2026-09-10-category-conflict-and-merged-issue-authority-fix.md#preserve-member-verdicts-through-merging)
  - [Verification of the authority boundary](translation-repair-temporal-context-2026-09-10-category-conflict-and-merged-issue-authority-fix.md#verification-of-the-authority-boundary)

## Scope

Task 19 is in progress.
The verified production code and frozen runtime are `b9d3b2ea0`.
The forward-heading window,
clustered panel packets,
source-evidence/cache handoffs and repair-selector comparison evidence are implemented.
Their compiled full repair-path check did not meet the temporal wording goal.
The remaining conflicting-diagnosis boundary is being isolated.
No full-entry pass is active.
Task 20's non-persisted audit accuracy is deferred,
not a prerequisite to this work or the next Mio page reading.

## Completed compiled authority-path check

`proc_0dc5` completed `translation-repair-temporal-authority-path-probe-20260910`
in 204 seconds on frozen `.frozen-dist-b9d3b2ea0`.

Script:
`~/temp/agent/probe-temporal-authority-path-20260910.mjs`.
Plan:
`~/temp/agent/temporal-authority-path-plan-20260910.out`.
Log:
`~/temp/agent/temporal-authority-path-probe-20260910.log`.
Report:
`~/temp/agent/temporal-authority-path-probe-20260910/report.json`.

It ran the actual repair path without prompt interception or hand-picked claim filtering,
with a disposable cache containing completed raw replies from the category-path run.
The bound remained one slice,
120 JSON requests,
360000 ms per exchange and 1200000 ms globally.

All three editors now independently produce the same disclosure correction:
`- Her best friend from primary school came out to her.`
The regional selector adopts the sole distinct wording without a ballot,
retaining all three contributor identities.
The whole-chunk composite then wins four of six votes,
not a fallback.
The second friendship and administrator item remain.
`from primary school` identifies the friend,
not childhood as the time of disclosure.
The summary does not redundantly specify April 2022;
the dated source narrative remains the evidence for the reconnection.

The run made 78 JSON calls,
including reuse:
nine critic,
36 panel,
three editor,
24 candidate ballots and six checker calls.
`~/temp/agent/temporal-authority-path-replay-20260910.json`
replays all 78 with zero misses and identical final text.
`~/temp/agent/temporal-authority-slates-20260910.out`
records every editor draft and selection round.
`~/temp/agent/temporal-authority-boundary-20260910.out`
records valid strict grammar and preserved paragraph/list structure.
Bedrock logged 0.00050104 USD;
three OpenRouter calls reported zero,
with five Hyper and sixteen Synthetic calls unpriced.
The daily helper ran afterward.

Tasks 23 and 19 are complete at the compiled repair boundary.
All drafts still rename `Harunome Hanbai`,
and the selected result says `Harunome Studio`.
Task 21 is now in progress for that separate name-authority defect.
Its bounded probe and completion criteria are in
[reference-name authority](translation-repair-name-authority-2026-09-10.md).
Whole-page readiness remains false;
no full-entry run is active.
