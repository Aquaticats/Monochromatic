import { slatePickBallotsOf, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import type { TranslateStageResult, } from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Translate lane winner ballots
// WHAT THE CONSOLIDATION MUST KNOW ABOUT THE TRANSLATE LANE'S TEXT and the artifact rows do not carry: how many judges
// chose it in the lane's own round. The owner's answer of 2026-10-29 ("Ship the two-ballot text (Recommended)") ships
// that text where the final round cannot seat two judges only if two judges chose it, so the count is read off the
// lane's settled records, where every ballot is kept (`TranslateSliceRecord.stageResult`), rather than inferred from a
// text that differs from the archive.

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
  readonly stageResult: Pick<TranslateStageResult, 'origin' | 'perCandidate' | 'shippedIndex' | 'text'>;
};

/**
 Counts, per slice, the ballots that named the text the translate lane would ship.

 A SLICE IS COUNTED ONLY WHERE THE LANE'S ROUND CHOSE FRESH WORDING AND THAT WORDING IS WHAT THE DELIVERY LEDGER
 CARRIES. A round that kept the archive chose nothing, and an assembly guard that trimmed the shipped text left a
 wording the judges never saw, so neither earns the two-ballot credit.

 @param records - the lane's settled records, one per slice it asked

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
    if (stageResult.text !== translateTexts.get(record.sliceIndex,))
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
