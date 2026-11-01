# Translation repair history: 2026-08-07 to 2026-08-12

Part of the [translation repair history index](translation-repair-history.md).

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Translation repair history 2026-08-07 to 2026-08-12: how to read corpus-pass progress, why runs 007 to 009 of pass 8 ended as they did, the state at the 2026-08-07 compaction and the per-entry deadline](translation-repair-history-2026-08-07-to-12-pass-8-runs-007-to-009-liveness-budget-compaction-state-and-deadline.md):
  How to tell a live corpus pass from a stalled one,
  why run 007 ended on its budget,
  the state at the 2026-08-07 compaction,
  how the per-entry deadline hit the large band,
  the evidence that a deadline costs a restart and never an entry,
  and the outside kill of run 009 that lost nothing.
  Sections:
  - [How to read corpus-pass progress without inventing a stall](translation-repair-history-2026-08-07-to-12-pass-8-runs-007-to-009-liveness-budget-compaction-state-and-deadline.md#how-to-read-corpus-pass-progress-without-inventing-a-stall)
  - [Run 007 ended on its budget, not on its target](translation-repair-history-2026-08-07-to-12-pass-8-runs-007-to-009-liveness-budget-compaction-state-and-deadline.md#run-007-ended-on-its-budget-not-on-its-target)
  - [State at the 2026-08-07 compaction](translation-repair-history-2026-08-07-to-12-pass-8-runs-007-to-009-liveness-budget-compaction-state-and-deadline.md#state-at-the-2026-08-07-compaction)
  - [The per-entry deadline is starving the large band specifically](translation-repair-history-2026-08-07-to-12-pass-8-runs-007-to-009-liveness-budget-compaction-state-and-deadline.md#the-per-entry-deadline-is-starving-the-large-band-specifically)
  - [The deadline costs wall time and a restart, never an entry (#61 answered)](translation-repair-history-2026-08-07-to-12-pass-8-runs-007-to-009-liveness-budget-compaction-state-and-deadline.md#the-deadline-costs-wall-time-and-a-restart-never-an-entry-61-answered)
  - [Run 009 was killed from outside, and nothing was lost](translation-repair-history-2026-08-07-to-12-pass-8-runs-007-to-009-liveness-budget-compaction-state-and-deadline.md#run-009-was-killed-from-outside-and-nothing-was-lost)
- [Translation repair history 2026-08-07 to 2026-08-12: the round-three gate sheet drawn and pre-graded blind](translation-repair-history-2026-08-07-to-12-round-three-gate-sheet-drawn-and-blind-pre-grades.md):
  The tension between checking that a sheet renders and keeping later pre-grades blind,
  the drawing of the round-three gate sheet at eighteen settled entries,
  and the blind pre-grades recorded for it.
  Sections:
  - [Verifying a sheet renders is in tension with pre-grading it blind](translation-repair-history-2026-08-07-to-12-round-three-gate-sheet-drawn-and-blind-pre-grades.md#verifying-a-sheet-renders-is-in-tension-with-pre-grading-it-blind)
  - [The round-three gate sheet is drawn](translation-repair-history-2026-08-07-to-12-round-three-gate-sheet-drawn-and-blind-pre-grades.md#the-round-three-gate-sheet-is-drawn)
  - [What is true right now](translation-repair-history-2026-08-07-to-12-round-three-gate-sheet-drawn-and-blind-pre-grades.md#what-is-true-right-now)
  - [Round-three blind pre-grades are recorded](translation-repair-history-2026-08-07-to-12-round-three-gate-sheet-drawn-and-blind-pre-grades.md#round-three-blind-pre-grades-are-recorded)
- [Translation repair history 2026-08-07 to 2026-08-12: the state at the 2026-08-09 compaction: the sheet awaiting its grade, what landed, a withdrawn rate and the open tasks](translation-repair-history-2026-08-07-to-12-state-at-the-2026-08-09-compaction-sheet-awaiting-grade-withdrawn-rate-and-open-tasks.md):
  The state at the 2026-08-09 compaction:
  the round-three gate sheet drawn and waiting for the user's grade,
  with the files in the order to open them,
  the pass that was running,
  what landed in that session,
  the corroboration-rate claim withdrawn as unsupported,
  and the open tasks in the order they unblock.
- [Translation repair history 2026-08-07 to 2026-08-12: what the draw estimates and how sheets are joined to verdicts](translation-repair-history-2026-08-07-to-12-sample-draw-and-scoring-join-integrity.md):
  Why round-robin across entries protects the draw,
  the probe join that attached the wrong record's verdict,
  the smaller draw-integrity fixes,
  what the precision figure estimates,
  the checks made for the first scoring run,
  the check that a re-draw cannot overwrite the draw manifest,
  and the digest that binds sheets to an exact draw.
  Sections:
  - [Entry count per band is not what protects the draw; round-robin is](translation-repair-history-2026-08-07-to-12-sample-draw-and-scoring-join-integrity.md#entry-count-per-band-is-not-what-protects-the-draw-round-robin-is)
  - [The probe join attached the wrong record's verdict, and the aggregates never showed it](translation-repair-history-2026-08-07-to-12-sample-draw-and-scoring-join-integrity.md#the-probe-join-attached-the-wrong-records-verdict-and-the-aggregates-never-showed-it)
  - [Three smaller draw-integrity fixes landed alongside](translation-repair-history-2026-08-07-to-12-sample-draw-and-scoring-join-integrity.md#three-smaller-draw-integrity-fixes-landed-alongside)
  - [What the precision figure estimates: entries, not issues](translation-repair-history-2026-08-07-to-12-sample-draw-and-scoring-join-integrity.md#what-the-precision-figure-estimates-entries-not-issues)
  - [Two things verified rather than assumed, for whoever runs #60](translation-repair-history-2026-08-07-to-12-sample-draw-and-scoring-join-integrity.md#two-things-verified-rather-than-assumed-for-whoever-runs-60)
  - [A re-draw cannot overwrite the draw manifest, and why binding sheets to a draw by digest is still worth building](translation-repair-history-2026-08-07-to-12-sample-draw-and-scoring-join-integrity.md#a-re-draw-cannot-overwrite-the-draw-manifest-and-why-binding-sheets-to-a-draw-by-digest-is-still-worth-building)
  - [Task 63 landed: sheets bind to a draw, and the reader stops faking claims](translation-repair-history-2026-08-07-to-12-sample-draw-and-scoring-join-integrity.md#task-63-landed-sheets-bind-to-a-draw-and-the-reader-stops-faking-claims)
- [Translation repair history 2026-08-07 to 2026-08-12: the runtime-neutral bundle that did not exist, and a question to a reviewer that was never answered](translation-repair-history-2026-08-07-to-12-runtime-neutral-bundle-premise-that-did-not-exist.md):
  How a lint error was held up on the belief that exporting a module would break a runtime-neutral bundle,
  the discovery that the package has no such bundle,
  and the lesson that a question sent to a reviewer and never answered leaves a premise looking examined.
- [Translation repair history 2026-08-07 to 2026-08-12: the introduced-defect probe and the naturalness audit](translation-repair-history-2026-08-07-to-12-introduced-defect-probe-and-naturalness-audit.md):
  The probe's trigger rate and where its canonical figures live,
  the probe judging wording the naturalness lane can replace,
  the audit of the naturalness lane validated live against injected damage,
  and the first live numbers from run 012.
  Sections:
  - [Probe trigger rate: the decision doc is canonical now](translation-repair-history-2026-08-07-to-12-introduced-defect-probe-and-naturalness-audit.md#probe-trigger-rate-the-decision-doc-is-canonical-now)
  - [The probe judges wording the naturalness lane can replace](translation-repair-history-2026-08-07-to-12-introduced-defect-probe-and-naturalness-audit.md#the-probe-judges-wording-the-naturalness-lane-can-replace)
  - [The naturalness lane is audited now, and the audit was checked before trusting it](translation-repair-history-2026-08-07-to-12-introduced-defect-probe-and-naturalness-audit.md#the-naturalness-lane-is-audited-now-and-the-audit-was-checked-before-trusting-it)
  - [Run 012 settled, and the naturalness audit has its first live numbers](translation-repair-history-2026-08-07-to-12-introduced-defect-probe-and-naturalness-audit.md#run-012-settled-and-the-naturalness-audit-has-its-first-live-numbers)
- [Translation repair history 2026-08-07 to 2026-08-12: the Kimi-K3 schema-mismatch degradation, the recall figure measured under it, its cause and its fix](translation-repair-history-2026-08-07-to-12-kimi-k3-schema-mismatch-degradation-recall-measured-under-it-and-its-fix.md):
  The silent failures of the naturalness lane and the editor ensemble in run 013,
  the recall re-measure
  and its correction as a degraded-ensemble figure,
  run 014,
  the two-character channel marker that caused it all,
  and the rosters widened so that no fan-out stage can meet quorum on one voice.
  Sections:
  - [Run 013, and the naturalness lane failing without saying so](translation-repair-history-2026-08-07-to-12-kimi-k3-schema-mismatch-degradation-recall-measured-under-it-and-its-fix.md#run-013-and-the-naturalness-lane-failing-without-saying-so)
  - [The silent lane was the small half: the EDITOR ensemble degraded too](translation-repair-history-2026-08-07-to-12-kimi-k3-schema-mismatch-degradation-recall-measured-under-it-and-its-fix.md#the-silent-lane-was-the-small-half-the-editor-ensemble-degraded-too)
  - [Task 51 is measured: recall on the current roster is 0.889](translation-repair-history-2026-08-07-to-12-kimi-k3-schema-mismatch-degradation-recall-measured-under-it-and-its-fix.md#task-51-is-measured-recall-on-the-current-roster-is-0889)
  - [Run 014: the degradation is now the dominant fact about the pipeline](translation-repair-history-2026-08-07-to-12-kimi-k3-schema-mismatch-degradation-recall-measured-under-it-and-its-fix.md#run-014-the-degradation-is-now-the-dominant-fact-about-the-pipeline)
  - [The model was never broken: a two-character prefix cost four runs](translation-repair-history-2026-08-07-to-12-kimi-k3-schema-mismatch-degradation-recall-measured-under-it-and-its-fix.md#the-model-was-never-broken-a-two-character-prefix-cost-four-runs)
  - [Every fan-out stage now has a quorum one voice cannot meet](translation-repair-history-2026-08-07-to-12-kimi-k3-schema-mismatch-degradation-recall-measured-under-it-and-its-fix.md#every-fan-out-stage-now-has-a-quorum-one-voice-cannot-meet)
- [Translation repair history 2026-08-07 to 2026-08-12: round three graded, the repair sheet judged ungradeable](translation-repair-history-2026-08-07-to-12-round-three-graded-and-repair-sheet-not-gradeable.md):
  The round-three grading result (the gate not met),
  the duplicate-exclusion decision and the grader calibration,
  and the reading of the repair sheet that found scope and quality failures the probe missed.
  Sections:
  - [Round three is GRADED, and the gate is not met](translation-repair-history-2026-08-07-to-12-round-three-graded-and-repair-sheet-not-gradeable.md#round-three-is-graded-and-the-gate-is-not-met)
  - [The repair sheet is not gradeable, and the reason is not the sheet](translation-repair-history-2026-08-07-to-12-round-three-graded-and-repair-sheet-not-gradeable.md#the-repair-sheet-is-not-gradeable-and-the-reason-is-not-the-sheet)
- [Translation repair history 2026-08-07 to 2026-08-12: the state at the 2026-08-12 compaction: the decision to re-plan, the two classes of false positive, the open tasks and what landed](translation-repair-history-2026-08-07-to-12-state-at-the-2026-08-12-compaction-replan-decision-false-positive-classes-and-open-tasks.md):
  The state at the 2026-08-12 compaction:
  the two user decisions taken after the repair sheet was read,
  to stop and re-plan the milestone and to let the running pass finish with its repairs discarded,
  the analysis of the eight round-three false positives as two classes,
  the open tasks in the order the plan touches them,
  and everything that landed in the session.

## Current stop condition

Historical evidence only.
Candidate M failed on 2026-09-01.
No historical next-action list authorizes a corpus run,
model call,
spent-prompt retry,
or successor implementation.
