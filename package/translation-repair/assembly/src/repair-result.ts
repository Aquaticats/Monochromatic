import type { SliceCriticRecord, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { LaneSliceText, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { ChunkRepairOutcome, } from './repair-contract.ts';
import type { RepairIssueRecord, } from './repair-record.ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Repair result
// What the repair driver RETURNS, kept beside neither the driver nor the
// assembler so both can name it without importing each other.

/**
 Completion status of one repair run;
 never an unqualified "corrected translation".

 @example
 ```ts
 const status: RepairStatus = 'repaired';
 ```
 */
export type RepairStatus =
  | 'repaired'
    | 'unchanged'
    | 'blocked-non-translation';

/**
 Output contract of the batch driver.

 @example
 ```ts
 const { repairedText, status, issues, } = await repairTranslation({ ... },);
 ```
 */
export type RepairTranslationResult = {
  /**
   Best translation the run can justify;
   equals the input when nothing demonstrably beat it.
   */
  readonly repairedText: string;

  /**
   How the run ended.
   */
  readonly status: RepairStatus;

  /**
   Every adjudicated issue with its chunk and resolution fate.
   */
  readonly issues: readonly RepairIssueRecord[];

  /**
   Alignment and stage findings in scorecard-stable wording.
   */
  readonly findings: readonly string[];

  /**
   Per-chunk critic calibration: who answered, and who raised each claim.

   Separate from {@link RepairTranslationResult.issues} because a chunk whose
   critics raised nothing produces no issue record, and that chunk is exactly
   the one a rate needs: it is the difference between a critic that was asked
   and stayed quiet and a critic that was never asked.
   */
  readonly sliceCritics: readonly SliceCriticRecord[];

  /**
   Slices the preparation produced, which every index set of this result is out of.

   Reported because a consumer holding only this result could not otherwise
   range-check the index sets, nor tell a document with one changed slice out
   of two from one changed out of two hundred. The translate lane has always
   reported it; this side did not.
   */
  readonly sliceCount: number;

  /**
   Slices the returned document CARRIES a repair for, in document order.

   Named rather than counted, because the question this lane is measured
   against is per slice: which slices did this lane change, and did the other
   lane change the same ones. A count answers neither. Empty on a blocked run,
   which returns its input whatever each slice decided.
   */
  readonly changedSliceIndices: readonly number[];

  /**
   Slices whose repair the assembly guard took back, in document order.

   Ordered by `orderedChangeSets` rather than left in the order the guard
   worked, so a reader joining two lanes slice by slice reads both sets by one
   rule. Disjoint from {@link RepairTranslationResult.changedSliceIndices} by
   construction. Kept apart from the issue records because a withdrawal is a
   fact about the DOCUMENT, and a slice can be withdrawn while carrying no
   adjudicated issue of its own.
   */
  readonly withdrawnSliceIndices: readonly number[];

  /**
   Shipped slices whose text the assembly guard trimmed, with the text the
   document carries: the decision with an orphan definition block cut.

   The delivery ledger reads a shipped row's text here before it reads the
   decision, so the rows say what the document carries. Empty when every
   shipped slice carries its decision whole.
   */
  readonly trimmedReplacements: readonly SliceReplacement[];

  /**
   What this lane DECIDED for every prepared slice, beside the archive's own
   wording, in document order.

   One entry per slice whether or not anything changed, because a rate needs
   its denominator and "this lane looked and left it alone" is a decision.
   Carries no shipped flag: {@link RepairTranslationResult.changedSliceIndices}
   is that fact, and repeating it per slice would let the two disagree.
   */
  readonly sliceTexts: readonly LaneSliceText[];

  /**
   Every slice this run settled, with the judged rounds that decided each.

   THE REPAIR LANE'S COUNTERPART TO `TranslateDocumentResult.slices`, added
   because it had none: the outcome went into the slice cache and no further,
   so a settled artifact carried this lane's repaired text with no record of
   which panel chose it or why. That is the whole reason the declared-name
   defect had to be found with a live probe.

   COVERAGE. One entry per slice the run reached, in slice order. An ordinary
   run reaches every prepared slice, so the count equals `sliceCount`. A run
   blocked for non-translation stops at the crossing and carries only the
   slices decided before it, which is why the two numbers are reported
   separately rather than one being derived from the other.

   Withdrawn slices stay here. What assembly took back is `withdrawnSliceIndices`;
   this side says what the lane decided, and a withdrawal is only readable
   against the decision it withdrew.
   */
  readonly chunks: readonly ChunkRepairOutcome[];
};

//endregion Repair result
