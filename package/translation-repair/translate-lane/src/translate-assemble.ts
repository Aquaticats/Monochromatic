import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  adjacentRepetitionFindings,
  assertReplacementsChange,
  deriveShippedIndices,
  guardFootnoteAssembly,
  orderedChangeSets,
  repetitionFindings,
  repetitionLogLines,
  shippedSliceTexts,
  withholdLoneContainerHalves,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { buildSliceSelections, } from './slice-selection.ts';
import { alignmentRefusals, } from './translate-alignment-refusals.ts';
import type {
  TranslateDisposition,
  TranslateDocumentResult,
  TranslateSliceRecord,
  UnfilledSlice,
} from './translate-document-contract.ts';
import { translateLaneWordings, } from './translate-lane-wordings.ts';
import { wrapTranslateRecords, } from './translate-wrap.ts';
import { heardNobody, } from './translate-unheard.ts';

//region Translate assembly
// What the translate lane RETURNS, built in one place so the driver's loop and
// its report cannot drift apart in what they claim.
//
// The mirror of `repair-assemble.ts`, split out of the driver at its line cap
// on the same seam that one uses. Everything here is derived from settled
// records and the preparation: which slices moved, what the whole document
// refuses, and what a reader is told about each. Nothing here buys anything.

/**
 Whether each disposition is a guard's refusal of a replacement the judges
 chose, which `assembleTranslation` counts as refused.

 A RECORD OVER EVERY DISPOSITION, so a disposition the lane gains cannot be
 counted as taken or refused by whichever side a chain happened to end on: the
 compiler refuses the record until the new one is classified.
 */
const REFUSES_A_REPLACEMENT: Readonly<Record<TranslateDisposition, boolean>> = {
  'stage-result': false,
  'refused-alignment': true,
  'refused-quote-loss': true,
  'refused-declared-name': true,
};

/**
 Assembles settled translate records into the document and its report.

 @param prepared - preparation both the slices and the incumbents come from

 @param settled - one record per slice the lane settled, in document order

 @param unfilled - passages lane reached and could not fill

 @param carriedChunkIndices - source-only passages rendered elsewhere

 @param resumedSliceCount - slices answered from the cache, for the log line

 @param findings - run-level findings gathered before assembly, which lead the
 list ahead of every slice's own

 @param l - driver logger

 @returns Translated document with its per-slice report and status

 @example
 ```ts
 const result = assembleTranslation({ prepared, settled, unfilled, resumedSliceCount, findings, l, },);
 ```
 */
export function assembleTranslation(
  {
    prepared,
    settled: producedRecords,
    unfilled,
    carriedChunkIndices = [],
    resumedSliceCount,
    findings,
    l,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly settled: readonly TranslateSliceRecord[];
    readonly unfilled: readonly UnfilledSlice[];
    readonly carriedChunkIndices?: readonly number[];
    readonly resumedSliceCount: number;
    readonly findings: readonly string[];
    readonly l: Logger;
  },
): TranslateDocumentResult {
  /**
   Records with produced wording wrapped at its semantic boundaries.

   BEFORE ANYTHING READS THEM, because the replacements, the wordings and the
   per-slice findings all come out of this one list, and the delivery
   invariant requires the first two to agree byte for byte.
   */
  const settled = wrapTranslateRecords({
    slices: prepared.slices,
    settled: producedRecords,
    lineStructuredSlices: prepared.lineStructuredSliceIndices,
    l,
  },);

  /**
   Slices whose accepted text differs from the archive's.
   */
  const changed = settled.filter(function isChanged(record,): boolean {
    return record.changed;
  },);

  /**
   Slices where the guard refused a replacement the judges chose.
   */
  const refused = settled.filter(function wasRefused(record,): boolean {
    return REFUSES_A_REPLACEMENT[record.disposition];
  },);

  /**
   Slices no translator answered for, which stand on the incumbent and are
   deliberately absent from the cache.
   */
  const unheard = settled.filter(function answeredByNobody(record,): boolean {
    return heardNobody({ record, },);
  },);
  l.info(
    `translated ${String(settled.length,)} ${
      wordForCount({
        count: settled.length,
        one: 'slice',
        many: 'slices',
      },)
    } (${String(resumedSliceCount,)} resumed): `
    // A GUARD rather than ALIGNMENT, which is the name of one of the three
    // dispositions this count covers and sent a reader looking for an alignment
    // finding the other two never wrote. Which guard refused which slice is in
    // the findings, one per refusal, in three shapes.
      + `${String(changed.length,)} changed, ${String(refused.length,)} refused by a guard`,
  );

  /**
   What this lane wants written, checked before the guard sees it.

   A BACKSTOP rather than the defence it used to be. Every record reaching
   here has already been checked against its own text, whether it came from
   the stage or from the cache, so a contradiction at this point means a
   defect between those checks and this line rather than a bad cache file.
   */
  const replacements = changed.map(function toReplacement(record,) {
    return {
      sliceIndex: record.sliceIndex,
      replacementText: record.outputText,
    };
  },);
  assertReplacementsChange({
    slices: prepared.slices,
    replacements,
  },);
  /**
   Replacements less any container half whose partner ships nothing (class
   fifty-seven), so the guard never reads a closing tag with no opening.
   */
  const halves = withholdLoneContainerHalves({
    slices: prepared.slices,
    replacements,
  },);
  for (const finding of halves.findings)
    l.warn(finding,);

  /**
   Assembly with any replacement withdrawn that the whole document refuses.

   Runs here rather than inside a slice because everything it checks is a
   relation BETWEEN slices: a footnote's reference and definition are settled
   separately, so a candidate that drops or renumbers a marker validates
   perfectly on its own, and a set that reassembles to the archive text is a
   fact no single slice can see.
   */
  const guarded = guardFootnoteAssembly({
    targetText: prepared.targetText,
    slices: prepared.slices,
    replacements: halves.replacements,
  },);
  /**
   Every slice the assembly took back: the halves first, then the guard's.
   */
  const withdrawn = [
    ...halves.withheld,
    ...guarded.revertedChunkIndices,
  ];
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
      } ${
        wordForCount({
          count: guarded.revertedChunkIndices
            .length,
          one: 'replacement',
          many: 'replacements',
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
   Slices the returned document carries a change for, derived from the
   surviving replacements and checked against the document's own bytes.

   Derived here rather than mapped by this driver, so the text and the index
   set cannot disagree about which slices moved.
   */
  const shipped = deriveShippedIndices({
    incumbentText: prepared.targetText,
    assembledText: guarded.assembledText,
    slices: prepared.slices,
    survivingReplacements: guarded.replacements,
  },);

  /**
   What each slice contributed to the assembled document, in document order.
   */
  const shippedSlices = shippedSliceTexts({
    slices: prepared.slices,
    survivingReplacements: guarded.replacements,
  },);

  /**
   Both index sets, checked against each other and put in document order.

   The guard returns each in the order it worked, and a reader comparing two
   lanes wants document order for both.
   */
  const ordered = orderedChangeSets({
    sliceCount: prepared.slices
      .length,
    shipped,
    withdrawn,
  },);

  // WHERE A REPETITION STANDS (ledger L12): the finding counts it and names
  // no slice, so the log names the slices and the phrase.
  for (
    const line of repetitionLogLines({
      archiveText: prepared.targetText,
      shippedText: guarded.assembledText,
      shippedSlices,
    },)
  )
    l.warn(line,);

  return {
    translatedText: guarded.assembledText,
    sliceCount: prepared.slices
      .length,
    // What SHIPPED, which is not what the judges chose whenever the guard
    // withdrew one of their choices.
    changedSliceCount: guarded.replacements
      .length,
    refusedSliceCount: refused.length,
    withdrawnSliceCount: withdrawn.length,
    // The same surviving replacements `changedSliceCount` is the size of, named,
    // and checked against the withdrawn set before either is reported.
    changedSliceIndices: ordered.shipped,
    sliceSelections: buildSliceSelections({
      records: settled,
      changedSliceIndices: ordered.shipped,
    },),
    withdrawnSliceIndices: ordered.withdrawn,
    // What the guard cut, so the ledger's shipped rows carry the document's
    // text rather than the decision's where the two differ.
    trimmedReplacements: guarded.trimmed,
    // Every prepared slice paired with the archive wording it was judged
    // against. Taken from the PREPARATION rather than from the settled records,
    // which are cache values a resumed run may have written under an earlier
    // preparation of the same entry.
    sliceTexts: translateLaneWordings({
      slices: prepared.slices,
      settled,
      unfilledChunkIndices: unfilled.map(function toIndex(passage,): number {
        return passage.sliceIndex;
      },),
      belowFloorChunkIndices: unfilled
        .filter(function keptForWantOfABench(passage,): boolean {
          return passage.benchBelowFloor === true;
        },)
        .map(function toIndex(passage,): number {
          return passage.sliceIndex;
        },),
      carriedChunkIndices,
    },),
    resumedSliceCount,
    // Passages the archive has not translated and this run could not either.
    // The document carries the gap they name, so a reader counting coverage
    // has to subtract them rather than read every unshipped slice as a slice
    // the judges left alone.
    status: (unfilled.length === 0) ? 'complete' : 'unfilled',
    unfilled,
    slices: settled,
    findings: [
      ...findings,
      ...settled.flatMap(function toFindings(record,): readonly string[] {
        return record.findings;
      },),
      // Derived here rather than read out of the records, because the sentence
      // names a slice and a stored sentence would name whichever slice the
      // record was FIRST settled for. Every refusal still reaches this list;
      // only where the sentence is built moved.
      ...alignmentRefusals({ records: settled, },),
      ...halves.findings,
      ...guarded.findings,
      // BOTH LANES, not only the one whose damage was found first. Reading
      // shipped text established the repetition in `lintong`'s repair lane, and reading the
      // settled pool against each entry's true archive finds one in
      // `saurikissa`'s TRANSLATE lane too. Writing a slice from its source
      // rather than editing an incumbent does not stop a lane saying the same
      // thing twice.
      ...repetitionFindings({
        archiveText: prepared.targetText,
        shippedText: guarded.assembledText,
      },),
      // Adjacency, for the reason `assembly-adjacent-repetition.ts` gives: the
      // `repetitionFindings` check needs content words this lane's duplications may not have.
      ...adjacentRepetitionFindings({
        archiveText: prepared.targetText,
        shippedSlices,
      },),
      ...unheard.map(function toUnheardFinding(record,): string {
        return `translate-heard-no-translator chunk ${
          String(record.sliceIndex,)
        }; incumbent stands, slice not cached`;
      },),
    ],
  };
}

//endregion Translate assembly
