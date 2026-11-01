# Bounded quality termination for the production pipeline

## Status

Superseded 2026-09-01 by the landed no-loop implementation recorded in
[`translation-repair-no-loop-design.md`](translation-repair-no-loop-design.md),
which removed the loops this plan proposed to bound.
Kept for the contract statement and the three-bucket terminal classification,
which the implementation carried over.

Provisional draft,
not an accepted design.
Written 2026-09-01 for the
[post-redesign direction decision](../decision/translation-repair-post-redesign-direction.md)
before the owner instructed a full reading of the failed redesign candidates.
The reading's conclusions are in
[`translation-repair-redesign-insights.md`](translation-repair-redesign-insights.md):
this plan covers the loop-bound half of the remediation,
and the owner decides whether the review verdict-form change joins it before implementation.

## Contract this implements

From the ratified corrected requirements in
[`translation-repair-pipeline-redesign.md`](translation-repair-pipeline-redesign.md):

- every invocation has finite work;
- a normal run returns one complete document,
  and quality machinery never withholds output;
- a non-producing node cannot gain authority to withhold output;
- naturalness is not a measurement;
- bounded no-output errors exist only for infrastructure and model-output failure.

The production pipeline violates these through six quality-continuation loops
whose only quality terminals are an accepted verdict,
an exact-repetition pause surfacing as `INCOMPLETE`,
or caller abort.

## The six loops

Each loop's cycle detector keys on task identity.
Whether that key accretes decides whether the loop can wedge unboundedly
or terminates into a forbidden `INCOMPLETE`.

### settleNaturalnessCorrections, `src/consolidation-naturalness-settle.ts` line 355

The digest covers `rejectedText`,
`findings`,
and `priorCorrections`.
`priorCorrections` accretes every generation,
so the detector can never fire twice on the same input and the loop is effectively unbounded.
This is the measured Carena wedge.
Exhaustion terminal:
settle with the latest reviewed candidate,
outstanding rejections demoted to findings.
`quorum-not-met` also settles with the standing candidate instead of throwing,
because a reviewer that cannot reach quorum is a non-producing node.

### buyConsolidationSlice recovery, `src/consolidate-slice-buy.ts` line 198

Key is the latest `priorFailure` only,
so exact repeats fire but varying evidence loops on.
Exhaustion terminal:
return the latest settlement with a finding.
`provider-unavailable` with no produced voice while standing may not ship stays a bounded error,
because that is producer exhaustion,
not reviewer authority.

### runTranslateStageWithRepair, `src/translate-stage-repair.ts` line 101

Key is the latest follow-up evidence only.
Exhaustion terminal:
return the latest judged outcome with a finding;
the slice settles the way a slate that heard no acceptable candidate settles today,
which downstream treats as unfilled or incumbent-kept,
never as a thrown entry.

### archive-block review, `src/archive-block-review-stage.ts` line 373

Key is block text plus latest findings.
Exhaustion terminal:
retain the block unlicensed with the findings recorded;
archive wording is the shipping default and reviewer indecision must not withhold the entry.

### pass preparation revision, `src/corpus-run/pass-prepare.ts` line 105

Key is the current archive text alone,
so a text cycle fires.
Exhaustion terminal:
proceed with the latest prepared pair,
unclaimed blocks recorded as findings.

### insertion placement, `src/corpus-run/pass-insertion-admission.ts` line 219

Keys are per-candidate latest placement tasks.
Exhaustion terminal:
unresolved passages stay unfilled and flow to the existing
`UnfilledPageError` terminal,
which remains a bounded model-output failure,
because no admissible candidate for required content was ever produced.

## Entry-error classes after the change

`src/corpus-run/entry-error-outcome.ts` currently maps ten classes to `INCOMPLETE`.
Three buckets:

- Quality-withholding,
  removed by demotion to ship-with-findings:
  `NaturalnessRepairInterruptedError` quality reasons,
  `TranslationRepairInterruptedError` reasons `final-selection-unresolved`,
  `production-cycle`,
  and `archive-block-unresolved`,
  `UnsettledFinalSelectionError`,
  and `NaturalnessCompletenessError` where it restates a review verdict.
- Deterministic integrity,
  kept:
  `ContributorCompletenessError`,
  `FrontMatterCompletenessError`,
  `DroppedDestinationError`.
  These are the contract's deterministic obligations,
  not reviewer opinion.
- Infrastructure and model-output,
  kept:
  `UnfilledPageError`,
  `TranslationRepairInterruptedError` reason `provider-unavailable`,
  `PromptPayloadStoreError`,
  `VisualEvidenceInterruptedError` where a reader failed for now.

Consequence to surface plainly:
`UnfilledPageError` remains the one no-page terminal,
so a four-entry pass can return fewer than four pages,
and the tally names why.

## Budget constants

One module exports a named budget per loop.
Every budget starts at two additional generations beyond the initial attempt,
so each invocation reviews at most three candidates.

Justification,
measured from the retained 2026-08-30 Carena run
(`~/Downloads/Carena0442-current-overlap4-run1-20260829/run.log`,
2,557,516 bytes):

- 46 absolute naturalness reviews across four in-flight slices,
  a mean of 11.5 per slice;
- 40 `unacceptable`,
  5 `acceptable`,
  1 `quorum-not-met`;
- zero of four consolidation slices completed in 8,735 seconds of consolidation;
- consolidation summed 25,899,861 ms of round time,
  86.0 percent of it post-quorum grace.

By-generation conversion was not extractable:
review log lines carry neither slice nor candidate identity,
so which generation each acceptance landed on cannot be reconstructed.
Two is therefore a design choice bounded by the observed non-convergence,
not a measured optimum,
and the constant is a named dial later measurement can move.

## Persistence at exhaustion

Following the rule that only properties of the question persist:

- budget exhaustion after heard rejections persists,
  because the roster answered and rejected;
  a warm run must resume the same settlement rather than rebuy the churn;
- `quorum-not-met` exhaustion does not persist,
  because an unheard roster is a property of the night,
  and caching it would freeze one outage into every later resume.

## Safety notes

Demoting reviewer quorum failure cannot mask a provider outage:
producing stages still throw `provider-unavailable` at the next node,
and provider preflight still gates the pass.

Rejected alternative:
requiring a rejection quorum instead of any-single-rejection decisive.
Rejected because it changes what a rejection means everywhere at once,
while capping generations is the smaller reversible change
that leaves the decisive rule intact and measurable.

Rejected alternative:
raising the whole-entry hard cap and keeping unbounded correction.
Rejected because the Carena run shows more rounds buy churn,
not convergence,
and the cap is a backstop,
not a terminal the contract permits.

## Completion set

Chosen by the owner as one small,
one medium,
one large,
plus Carena.
Picks by measured Chinese source size at corpus commit `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
92 entries carrying both `page.md` and `page.en.md`,
sizes from `git cat-file -s`:

- small:
  `keyword233`,
  738 bytes;
  the overlap smoke entry with settled baselines at overlap one and four.
- medium:
  `Toka_ls`,
  3,660 bytes,
  inside the middle third;
  three prior readings and two published pages give the richest before-and-after comparability,
  including regression detection against the page already read publishable.
- large:
  `XIEPT2`,
  20,085 bytes;
  in the owner's recorded hard list,
  alignment-hard history,
  and a 24-to-1 block-count extreme that exercises the translate and insertion machinery hardest.
- `Carena0442` from corpus pull request 386,
  head `a80634a674f94861ea3b7056fba054ca9eab1a2c`,
  still open at 2026-09-01,
  run through the surviving minimal fixture at
  `~/temp/agent/pr386-mock-home-20260829/one-among-us/data`.

`RUN_CORPUS_PIN` in `src/corpus-run/run-config.ts` hardcodes clone dir and commit,
so the Carena arm needs an environment override for both,
validated and defaulting to the pinned values,
replacing the throwaway source edits the 2026-08-29 runs used.

## Verification plan

1.  Suite,
    oxlint,
    and types green after each loop's change.
2.  GFP per new guard:
    remove the budget check,
    show the named test fail,
    restore.
3.  One fresh pass over the four-entry set on the changed build,
    overlap and grace at the measured calibration values.
4.  The actual-output reading over whatever pages publish,
    recorded as the gate for the readiness signal.
