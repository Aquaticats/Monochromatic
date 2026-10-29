import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { guardFootnoteAssembly, } from './assembly-integrity.ts';
import { adjacentRepetitionFindings, } from './assembly-adjacent-repetition.ts';
import { contentSurvivalFindings, } from './assembly-content-survival.ts';
import {
  contentLossLogLines,
  repetitionLogLines,
} from './assembly-damage-log.ts';
import { repetitionFindings, } from './assembly-repetition.ts';
import {
  assertReplacementsChange,
  deriveShippedIndices,
  orderedChangeSets,
  shippedSliceTexts,
} from './assembly-invariant.ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { buildSliceCriticRecords, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { repairLaneWordings, } from './repair-lane-wordings.ts';
import type { ChunkRepairOutcome, } from './repair-contract.ts';
import { buildIssueRecords, } from './repair-record.ts';
import { repairReplacements, } from './repair-replacements.ts';
import { wrapRepairOutcomes, } from './repair-wrap.ts';
import type { RepairTranslationResult, } from './repair-result.ts';

//region Repair assembly
// What a settled run RETURNS, built once so the driver's exits cannot drift
// apart in what they report.
//
// This is the layer that decides what reached the reader. A slice's own
// selection cannot answer that: the assembly guard can take a repair back to
// keep a footnote relation whole, and a repair the document does not carry is
// not a repair a measurement may count.

/**
 Assembles settled outcomes into the document and its report.

 @param targetText - translation as it stands, which is the fallback

 @param slices - prepared slice pairs in document order

 @param outcomes - settled per-slice outcomes, refinement included

 @param lineStructuredSlices - global indices the line-structure rule governs,
 whose lines the wrap leaves alone

 @param findings - alignment and phase findings to carry through

 @param l - driver logger

 @returns Repaired document with its issue report and status

 @example
 ```ts
 const result = assembleRepair({ targetText, slices, outcomes, lineStructuredSlices, findings, l, },);
 ```

 @internal
 */
export function assembleRepair(
  {
    targetText,
    slices,
    outcomes: producedOutcomes,
    lineStructuredSlices,
    findings,
    l,
  }: {
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly outcomes: readonly ChunkRepairOutcome[];
    readonly lineStructuredSlices: ReadonlySet<number>;
    readonly findings: readonly string[];
    readonly l: Logger;
  },
): RepairTranslationResult {
  /**
   Outcomes with produced wording wrapped at its semantic boundaries.

   BEFORE ANYTHING READS THEM, because the replacements, the wordings and the
   issue records all come out of this one list, and the delivery invariant
   requires the first two to agree byte for byte.
   */
  const outcomes = wrapRepairOutcomes({
    slices,
    outcomes: producedOutcomes,
    lineStructuredSlices,
    l,
  },);

  /**
   What this lane wants written, checked before the guard sees it.

   A replacement identical to its incumbent survives the footnote guard and
   lands in the shipped set beside a document nobody changed, so it is refused
   here rather than counted there.
   */
  const replacements = repairReplacements({ outcomes, },);
  assertReplacementsChange({
    slices,
    replacements,
  },);

  /**
   Document rebuilt slice by slice, with any replacement withdrawn that would
   leave the footnote graph worse than the archive's.

   A footnote is a relation BETWEEN slices, and every stage works inside one,
   so this is the only layer that can see it. The per-envelope footnote gate
   bounds what one edit does; it cannot see a definition in another slice.
   */
  const guarded = guardFootnoteAssembly({
    targetText,
    slices,
    replacements,
  },);
  if (guarded.revertedChunkIndices
    .length
    > 0) {
    // Deliberately does not name a cause. The guard withdraws for footnote
    // damage, for structural regressions, and for a set that reassembles to the
    // archive text; only its findings say which, and a warning that guessed
    // would send a reader looking for a footnote that is not there.
    l.warn(
      `withdrew ${
        String(guarded.revertedChunkIndices
          .length,)
      } slice ${
        wordForCount({
          count: guarded.revertedChunkIndices
            .length,
          one: 'repair',
          many: 'repairs',
        },)
      } at assembly; the findings say why`,
    );
  }
  // THE FINDINGS REACH THE LOG. An entry the pass stops before its artifact is
  // written leaves the log as the only trace, and the twenty-first hakureico
  // pass of 2026-09-09 left "withdrew 2 replacements" with nothing saying why.
  for (const finding of guarded.findings)
    l.warn(finding,);

  /**
   What each slice contributed to the assembled document, in document order.
   */
  const shippedSlices = shippedSliceTexts({
    slices,
    survivingReplacements: guarded.replacements,
  },);

  /**
   How many slices the document CARRIES a repair for.

   Read off the guard's surviving replacements rather than recomputed from the
   outcomes, because the guard is what decides this. Reconstructing it from
   `changed` and the reverted list would agree today and would go on agreeing
   silently for exactly as long as those two stay in step.
   */
  const shippedSliceCount = guarded.replacements
    .length;

  /**
   Both index sets, checked against each other and put in document order.

   The guard returns each in the order it worked, and a reader comparing two
   lanes wants document order for both. Checking them here is also the only
   place that can: it is the one point holding the prepared slice count and
   both sets at once.
   */
  const ordered = orderedChangeSets({
    sliceCount: slices.length,
    // Derived from the surviving replacements and checked against the
    // document's own bytes, so the text and the index set cannot disagree
    // about which slices moved.
    shipped: deriveShippedIndices({
      incumbentText: targetText,
      assembledText: guarded.assembledText,
      slices,
      survivingReplacements: guarded.replacements,
    },),
    withdrawn: guarded.revertedChunkIndices,
  },);

  /**
   Whole-document issue report.
   */
  const issues = buildIssueRecords({
    outcomes,
    blocked: false,
    // The NORMALIZED set rather than the guard's raw one, which is the same
    // membership either way, since this is a lookup and order cannot change what
    // it finds. Reading from one set rather than from two spellings of it is the
    // point: the raw set is the normalizer's input and nothing else should take
    // it, or a later change to what normalizing means would reach one reader and
    // not the other.
    withdrawnSliceIndices: ordered.withdrawn,
  },);

  /**
   Whether any slice shipped a repair.
   */
  const anyChanged = shippedSliceCount > 0;
  // SLICES rather than chunks: both arrays hold slice outcomes, and a section
  // subdivides into several, so reporting them as chunks understates the
  // denominator against every other count in the artifact.
  l.info(
    `repair ${anyChanged ? 'shipped' : 'kept input'}: ${
      String(shippedSliceCount,)
    }/${String(outcomes.length,)} ${
      wordForCount({
        count: outcomes.length,
        one: 'slice',
        many: 'slices',
      },)
    } changed, ${String(issues.length,)} ${
      wordForCount({
        count: issues.length,
        one: 'issue',
        many: 'issues',
      },)
    }`,
  );

  // WHERE THE DOCUMENT-SCALE DAMAGE IS (ledger L12): `repetitionFindings`
  // counts it and names no slice, so the log names the slices, the phrase and the words.
  for (
    const line of repetitionLogLines({
      archiveText: targetText,
      shippedText: guarded.assembledText,
      shippedSlices,
    },)
  )
    l.warn(line,);
  for (
    const line of contentLossLogLines({
      archiveText: targetText,
      shippedText: guarded.assembledText,
      archiveSlices: slices.map(function toArchive(slice,) {
        return {
          sliceIndex: slice.target
            .sliceIndex,
          text: slice.target
            .text,
        };
      },),
    },)
  )
    l.info(line,);

  return {
    sliceCritics: buildSliceCriticRecords({ outcomes, },),
    // The WRAPPED outcomes, so the rounds sit beside the wording that shipped
    // rather than beside a pre-wrap rendering of it.
    chunks: outcomes,
    repairedText: guarded.assembledText,
    sliceCount: slices.length,
    status: anyChanged ? 'repaired' : 'unchanged',
    // Read off the guard's surviving replacements, which is the only place that
    // knows what the document carries, and checked against the withdrawn set
    // before either is reported.
    changedSliceIndices: ordered.shipped,
    withdrawnSliceIndices: ordered.withdrawn,
    // What the guard cut, so the ledger's shipped rows carry the document's
    // text rather than the decision's where the two differ.
    trimmedReplacements: guarded.trimmed,
    // Every prepared slice, decided or left alone, paired with the archive's
    // own wording. Built from the outcomes rather than from the surviving
    // replacements, because this side of the record is what the lane CHOSE and
    // `changedSliceIndices` and `withdrawnSliceIndices` are what the document carries.
    sliceTexts: repairLaneWordings({
      slices,
      outcomes,
      // Every slice was visited to reach assembly at all, so a gap here is a
      // defect rather than an early stop.
      undecided: 'refuse',
    },),
    issues,
    findings: [
      ...findings,
      ...guarded.findings,
      // REPETITION IS A RELATION BETWEEN SLICES, exactly like the footnote graph
      // the page guard (`guarded`) exists for, so this is the only layer that can
      // see it.
      // The introduced-defect probe was measured against `lintong`'s
      // duplicated farewell and it could not have caught it at any setting: the
      // probe compares one edited region against itself, and no single region
      // contains the duplication.
      ...repetitionFindings({
        archiveText: targetText,
        shippedText: guarded.assembledText,
      },),
      // ADJACENCY IS A SEPARATE CHECK because `repetitionFindings`, the
      // document-scale one, cannot see the `lintong` example: that duplication carries no word of
      // five letters, so the content gate drops it. Two neighbouring slices
      // shipping the same wording is specific enough to need no such gate.
      ...adjacentRepetitionFindings({
        archiveText: targetText,
        shippedSlices,
      },),
      // THE DAMAGE CLASS NOTHING ELSE HERE SEES. A slice rewritten into generic
      // prose keeps the document's length and structure and loses what it was
      // about. Measured on `saurikissa`, where one baseline slice went from 136
      // words to 21 and the whole document shrank by 70 characters of 4479.
      ...contentSurvivalFindings({
        archiveText: targetText,
        shippedText: guarded.assembledText,
      },),
    ],
  };
}

//endregion Repair assembly
