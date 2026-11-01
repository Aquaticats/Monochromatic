import { slatePickBallotsOf, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import type { TranslateSliceRecord, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import type { TranslateStageResult, } from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Translate lane winner ballots
// WHAT THE CONSOLIDATION MUST KNOW ABOUT THE TRANSLATE LANE'S TEXT and the artifact rows do not carry: how many judges
// chose it in the lane's own round. The owner's answer of 2026-10-29 ("Ship the two-ballot text (Recommended)") ships
// that text where the final round cannot seat two judges only if two judges chose it, so the count is read off the
// lane's settled records, where every ballot is kept (`TranslateSliceRecord.stageResult`), rather than inferred from a
// text that differs from the archive.
//
// THE LANE'S OWN DELIVERED TEXT IS WHAT THE RECORD IS COMPARED WITH (B1180). The record is the lane's settled record
// after `wrapTranslateRecords`, so its `outputText` is the wording the delivery ledger carries, byte for byte, and the
// round's `stageResult.text` is the wording the judges saw, before the semantic wrap added its line breaks. The count
// used to compare `stageResult.text` with the ledger row, so it missed every wording the wrap changed: 57 of 116
// slices of XingZ6022's artifact, and four of the five plain-prose slices XingZ6023 settled with two or three ballots.
// No fold of the wrap is needed, and none is written: the wrap runs once, inside the lane, and this reads its output.

/**
 The part of a translate lane record this reads.
 */
export type TranslateWinnerRecord = {
  /**
   Slice the lane settled.
   */
  readonly sliceIndex: number;

  /**
   What the lane's round decided there.
   */
  readonly stageResult: Pick<TranslateStageResult, 'origin' | 'perCandidate' | 'shippedIndex'>;

  /**
   Text the lane delivers for the slice, after the semantic wrap: the wording the delivery ledger carries unless an
   assembly guard cut it.
   */
  readonly outputText: string;

  /**
   Whether that text differs from the archive's, re-derived by the wrap.
   */
  readonly changed: boolean;

  /**
   What the driver did with the round's result; only `stage-result` delivers the wording the round chose.
   */
  readonly disposition: TranslateSliceRecord['disposition'];
};

/**
 Counts, per slice, the ballots that named the text the translate lane would ship.

 A SLICE IS COUNTED ONLY WHERE THE LANE'S ROUND CHOSE FRESH WORDING AND THAT WORDING IS WHAT THE DELIVERY LEDGER
 CARRIES. A round that kept the archive chose nothing; a guard that refused the choice (`refused-quote-loss`,
 `refused-declared-name`, `refused-alignment`) delivers the archive; a wording the wrap made the archive's own bytes
 chose nothing the page shows; and an assembly guard that trimmed the shipped text left a wording the judges never saw.
 None of those earns the two-ballot credit.

 @param records - lane's settled records, one per slice it asked

 @param comparison - one row per slice, whose translate text is what the translate document carries there

 @returns Ballots by slice index, for the slices where the round chose the text the ledger carries and at least one
 ballot named it

 @example
 ```ts
 const ballots = translateWinnerBallotsOf({ records: lanes.translate.slices, comparison: projected.comparison, },);
 ```
 */
export function translateWinnerBallotsOf(
  {
    records,
    comparison,
  }: {
    readonly records: readonly TranslateWinnerRecord[];
    readonly comparison: readonly {
      readonly sliceIndex: number;
      readonly translateText: string;
    }[];
  },
): ReadonlyMap<number, number> {
  /**
   What the translate document carries at each slice.
   */
  const translateTexts = new Map(comparison.map(function textOf(row,): readonly [
    number,
    string,
  ] {
    return [
      row.sliceIndex,
      row.translateText,
    ];
  },),);
  return new Map(records.flatMap(function counted(record,): readonly (readonly [
    number,
    number,
  ])[] {
    /**
     What the round decided.
     */
    const { stageResult, } = record;
    if (stageResult.origin !== 'fresh')
      return [];
    if (record.disposition !== 'stage-result')
      return [];
    if (!record.changed)
      return [];
    if (record.outputText !== translateTexts.get(record.sliceIndex,))
      return [];
    /**
     Ballots that named the shipped position.
     */
    const ballots = slatePickBallotsOf({ decided: stageResult, },);
    return (ballots === 0)
      ? []
      : [[
        record.sliceIndex,
        ballots,
      ] as const,];
  },),);
}

//endregion Translate lane winner ballots
