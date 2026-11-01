# Status: the terminal quality-refusal audit of 2026-08-29 (superseded)

The audit as recorded on 2026-08-29 of the terminal and bypass findings and how each was integrated,
kept as history after its continuous-correction design was superseded on 2026-09-01.

Part of [Status](status.md),
which lists every file of this record.

## Terminal quality-refusal audit, 2026-08-29 (superseded)

This is the audit as recorded on 2026-08-29 and is not standing guidance.
Its continuous-correction design was superseded:
`1ba8f713a` (2026-09-01) made consolidation a single attempt
and turned the final-selection gate into the reporter `finalSelectionFindings`,
and the finite producer constraints in `Standing redesign constraints` in the package README
replace its unbounded correction.
Present tense in this section describes the pipeline of that day.

Naturalness rejection no longer ends with `do not publish`:
consolidation cache generation 14 continuously corrects,
checkpoints payloads,
and pauses only on operational interruption or exact deterministic task cycle.
Publication gates remain strict.

Audited terminal and bypass findings are integrated as follows:

- Archive-only block repair landed in `ccaad1f53`,
  with contributor-floor test in `78ab244a2`.
  Preparation now scopes each unclaimed target block to expected aligned source section,
  requires exact anchored source support or deterministically shaped editorial apparatus,
  and lets any revise voice block retention.
  Retained wording receives defect-discovery naturalness review plus distinct acceptance challenge.
  Corrections preserve target-authoritative contributors,
  undergo independent whole-roster selection,
  splice in reverse offset order,
  and re-enter preparation before lanes.
  Exact correction and archive-state cycles pause as `INCOMPLETE`.
  A sibling revision intentionally causes remaining unclaimed blocks to be reviewed again;
  this costs calls but avoids carrying a license across changed parser locations and source context.
  The old `assertArchiveReviewed` terminal and `UnreviewedArchiveError` are removed.
- Unfilled-passage continuation landed in `ed756993b`,
  with cycle guard test in `1649c480d`.
  Translate version 11 produces from latest exact rejected slate and findings without finite correction count.
  Insertion placement rechecks latest semantic,
  destination,
  and shortfall evidence until admitted,
  proven carried elsewhere,
  operationally interrupted,
  or exact task cycle repeats.
  Carried passages map to `not-applicable` at insertion anchor;
  final would-ship page must retain every exact anchored region that proved full coverage.
  This exact-region rule can conservatively pause semantically equivalent rewrites as `INCOMPLETE`.
  `assertPublishableTranslation` is now defensive invariant and cannot trigger fresh whole-entry retry.
- Source-destination recovery landed in `0dc6e510c`.
  Translate version 12 excludes archive fallback that misses source atoms and continues as absent-mode translation.
  Ordinary repair-lane or archive standing is validated before consolidation eligibility,
  so a contest-endorsed linkless candidate enters version-16 recovery instead of becoming final.
  Final destination comparison runs before any page write and pauses as names-only `INCOMPLETE` invariant.
- Continuous final-selection recovery landed in `a84bb3a7a`,
  with role-alias guard in `f858ab538`.
  Consolidation version 16 treats every unendorsed or publication-ineligible standing baseline as unfinished.
  It threads prior selection slate,
  anonymized producer and judge roles,
  selection ballots,
  gate ballots,
  terminal,
  and findings into substantively distinct producer recovery.
  Recovery continues until a fresh consolidation wins selection and fidelity gate,
  provider operation stops,
  caller aborts,
  or exact evidence cycle repeats.
  Identical unsafe twins share one recovery chain;
  only final safe settlement persists.
  `assertFinalSelectionSettled` was then a defensive invariant,
  classified as `INCOMPLETE` rather than whole-entry quality retry;
  `1ba8f713a` (2026-09-01) removed it,
  and `finalSelectionFindings` reports the same condition beside the tally
  (`src/corpus-run/final-selection-completeness.ts` (moved on 2026-10-18 to
  `package/translation-repair/page-check/src/corpus-run/final-selection-completeness.ts`)).
- One-sided front-matter support landed in `56a47cb81` and creates insertion slice for source-only metadata,
  deterministically admits it,
  and validates candidate against source YAML shape when archive has none.
  Target-only metadata remains exact archive content outside localized slices.
  Missing/misplaced slice and contradictory assembled-page structure pause as `INCOMPLETE` invariants;
  incumbent fallback and candidate YAML/schema invalidity remain handled before publication.
- `assertContributorNamesComplete` throws when final attribution drops target-authoritative public form.
  Consolidation cache generation 15 landed in `5211c54dd` and floors translation candidates,
  excludes unrepairable violator voices without fabricated authorship,
  rejects unsafe ordinary lane winners,
  and floors consolidation and polish candidates.
  Repair/refine text that differs reaches lane winner floor;
  uncontested identical text matches translate lane already floored.
  Invalid slate deterministically retains target-authoritative incumbent.
  Body-slice floor does not replace front-matter comment authority,
  which remains in metadata validator.
  Final completeness guard remains defensive.
  Cache generations move to translate 12,
  refine 4,
  lane contest 5,
  and consolidation 16.

- Visual-evidence guard landed in `d14e641e6` and gives each image retry substantively distinct responsibility:
  complete transcription,
  small text,
  layout,
  then identifier verification.
  Corroborated and reviewed no-text outcomes proceed.
  Missing or unavailable image-dependent evidence pauses as `INCOMPLETE` before lanes,
  so visual content cannot bypass publication review.

On a fresh corpus path under consolidation cache generation 14,
`assertFinalNaturalnessComplete` is runtime persistence invariant after continuous naturalness,
not ordinary quality outcome.
Existing artifact readers do not invoke it.
Parser,
ledger,
assembly,
filesystem,
provider,
and cancellation failures are operational errors rather than quality verdicts.
The code-level terminal-quality register is closed.
Continuous-repair verification is recorded in `doc/audit/translation-repair-continuous-repair-invariant.md`.
Fresh validation remains gated by Carena completion-path analysis.
Exact deterministic cycles deliberately pause as `INCOMPLETE` until strategy or evidence changes;
they never authorize fallback publication.

Corpus-readiness work is defect-driven:
once output proves reproducible systemic blocker,
stop unrelated corpus arms,
fix and verify blocker,
then rerun affected entry before resuming measurement.
More samples do not compensate for known mechanism.

Remaining corpus arms,
hard-case output reading,
live calibration checks,
site-grammar gaps,
package reading CLI,
Hyper catalog drift check,
and declined-archive seam remain open.
Read milestone figures as history:
they were measured under earlier pipeline shapes and none is readiness claim.
The evidence and traces of that period are in `doc/audit/translation-repair-output-reading-20260826.md`
and `doc/planning/translation-repair-corpus-overlap-measurement.md`.
