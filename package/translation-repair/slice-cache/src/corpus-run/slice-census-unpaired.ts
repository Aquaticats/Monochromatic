import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { EntryCensus, } from './slice-census-entry.ts';
import { sliceCensusTotal, } from './slice-census-total.ts';

//region Slice census unpaired
// HOW MUCH OF THE CORPUS REACHES NO SLICE because the aligner would not pair
// its section, on either side. Those are the calls that never happen, so the
// census names the entries that hold the most.

/**
 How many entries the unpaired-section list names, which is enough to show
 whether that text is one outlier or spread across the corpus.
 */
const UNPAIRED_ENTRIES_LISTED = 5;

/**
 Unpaired characters of both sides of one entry, which the list ranks by.

 A SUM OVER TWO SCRIPTS, BY DESIGN. A Chinese source says in one character
 what its English translation says in several, so the same passage left
 unpaired weighs more on the target side, and the sum favours entries whose
 unpaired text is English. The list ranks by how much text the aligner left
 unpaired, not by how much of the entry it carries, and every row prints each
 side's figure so a reader weighs them apart.

 @param row - measured entry, whose two figures the list ranks by together

 @returns Its unpaired source and target characters together, the key the
 list sorts on before the entry id

 @example
 ```ts
 const held = unpairedCharsOf({ row, },);
 ```
 */
function unpairedCharsOf({ row, }: { readonly row: EntryCensus; },): number {
  return row.unpairedSourceChars + row.unpairedTargetChars;
}

/**
 The census lines for the sections the aligner would not pair.

 BOTH SIDES RANK TOGETHER. Until 2026-10-06 the list ranked by unpaired source
 characters alone, so an entry whose unpaired text was all on the target side
 ranked last and fell off the list. Entries holding as much break their tie by
 id in code point order, and a list that leaves entries out says how many.

 @param rows - measured entries

 @returns The totals line, then one line for each of the entries holding the
 most unpaired characters of both sides together, at most five, then a line
 counting the entries with unpaired text the list left out, when it left any

 @example
 ```ts
 const lines = sliceCensusUnpairedLines({ rows, },);
 ```
 */
export function sliceCensusUnpairedLines(
  { rows, }: { readonly rows: readonly EntryCensus[]; },
): readonly string[] {
  /**
   Entries carrying a section the aligner would not pair, on either side.
   */
  const unpaired = rows.filter(function hasUnpaired(row,) {
    return (row.unpairedSourceSections > 0)
      || (row.unpairedTargetSections > 0);
  },);

  /**
   Entries with unpaired text the list leaves out.
   */
  const leftOut = Math.max(
    0,
    unpaired.length - UNPAIRED_ENTRIES_LISTED,
  );
  return [
    `CENSUS unpaired sections reaching no slice: source ${
      String(sliceCensusTotal({
        rows: unpaired,
        field: 'unpairedSourceSections',
      },),)
    }, target ${
      String(sliceCensusTotal({
        rows: unpaired,
        field: 'unpairedTargetSections',
      },),)
    }; entries: ${String(unpaired.length,)}; chars: source ${
      String(sliceCensusTotal({
        rows: unpaired,
        field: 'unpairedSourceChars',
      },),)
    }, target ${
      String(sliceCensusTotal({
        rows: unpaired,
        field: 'unpairedTargetChars',
      },),)
    }`,
    ...unpaired
      .toSorted(function byUnpairedChars(
        left,
        right,
      ): number {
        return (unpairedCharsOf({ row: right, },) - unpairedCharsOf({ row: left, },))
          || compareCodePoints({
            left: left.entryId,
            right: right.entryId,
          },);
      },)
      .slice(
      0,
      UNPAIRED_ENTRIES_LISTED,
      )
      .map(function unpairedLine(row,): string {
        return `CENSUS   ${row.entryId}: source sections ${
          String(row.unpairedSourceSections,)
        } (chars: ${String(row.unpairedSourceChars,)}), target sections ${
          String(row.unpairedTargetSections,)
        } (chars: ${String(row.unpairedTargetChars,)})`;
      },),
    ...((leftOut === 0)
      ? []
      : [
        `CENSUS   and ${String(leftOut,)} more ${
          wordForCount({
            count: leftOut,
            one: 'entry',
            many: 'entries',
          },)
        } with unpaired text, left out of this list`,
      ]),
  ];
}

//endregion Slice census unpaired
