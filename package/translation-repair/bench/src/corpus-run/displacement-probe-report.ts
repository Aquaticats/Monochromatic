import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { EntryDisplacement, } from './displacement-probe-row.ts';
import {
  corpusTotals,
  isNotable,
} from './displacement-probe-totals.ts';

//region Displacement probe report
// The lines the displacement probe logs once every entry is read: the totals
// across the settled entries, then one block per entry that carries anything.

/**
 Lines describing one notable entry: its baseline, then each class that
 named it.

 @param row - entry to describe

 @returns Heading line, then one line per class holding positions

 @example
 ```ts
 const lines = notableLines({ row, },);
 ```
 */
function notableLines({ row, }: { readonly row: EntryDisplacement; },): readonly string[] {
  /**
   Classes in print order, each with what it names written out.
   */
  const classes: readonly {
    readonly label: string;
    readonly items: readonly (string | number)[];
  }[] = [
    {
      label: 'relocation',
      // Written as high to low.
      items: row.relocationCandidates
        .map(function toText(candidate,): string {
          return `${String(candidate.high,)}->${String(candidate.low,)}(+${
            String(candidate.surplus,)
          }/-${String(candidate.deficit,)})`;
        },),
    },
    {
      label: 'untranslated',
      items: row.untranslated,
    },
    {
      label: 'target-only',
      items: row.targetOnly,
    },
    {
      label: 'transcription suspect',
      items: row.transcriptionSuspects,
    },
    {
      label: 'other imbalance',
      items: row.otherImbalances,
    },
  ];

  /**
   Expansion this entry was read against.
   */
  const rate = row.baseline
    .toFixed(2,);
  return [
    `  ${row.entryId}: baseline ${rate} (${row.baselineFrom})`,
    ...classes
      .filter(function isNamed({ items, },): boolean {
        return items.length > 0;
      },)
      .map(function toLine({
        label,
        items,
      },): string {
        return `    ${label}: ${items.join(' ',)}`;
      },),
  ];
}

/**
 Lines the probe logs after reading every settled entry.

 @param rows - reading of every entry that carved

 @param artifactCount - settled artifacts the runs directory holds, carved or
 not

 @param defaulted - entries whose recipe had a defaulted half, each with the
 label naming the halves

 @returns Log lines in print order

 @example
 ```ts
 for (const line of reportLines({ rows, artifactCount: 1, defaulted: [], },)) log.info(line,);
 ```
 */
export function reportLines(
  {
    rows,
    artifactCount,
    defaulted,
  }: {
    readonly rows: readonly EntryDisplacement[];
    readonly artifactCount: number;
    readonly defaulted: readonly {
      readonly entryId: string;
      readonly label: string;
    }[];
  },
): readonly string[] {
  /**
   Counts across every settled entry.
   */
  const totals = corpusTotals({ rows, },);
  return [
    `settled entries carved: ${String(rows.length,)} of ${String(artifactCount,)} ${
      wordForCount({
        count: artifactCount,
        one: 'artifact',
        many: 'artifacts',
      },)
    }`,
    `  with a defaulted recipe half: ${String(defaulted.length,)}`,
    ...defaulted.map(function toLine({
      entryId,
      label,
    },): string {
      return `  ${entryId}: ${label}`;
    },),
    `slices read: ${String(totals.slices,)}`,
    `entries falling back to the corpus baseline: ${String(totals.fellBack,)}`,
    `relocation candidates: ${String(totals.relocationCandidates,)}`,
    `  of which a transcription would also explain: ${String(totals.transcriptionSuspects,)}`,
    `untranslated slices: ${String(totals.untranslated,)}`,
    `target-only slices: ${String(totals.targetOnly,)}`,
    `other imbalances: ${String(totals.otherImbalances,)}`,
    ...rows
      .filter(function carriesSomething(row,): boolean {
        return isNotable({ row, },);
      },)
      .flatMap(function toBlock(row,): readonly string[] {
        return notableLines({ row, },);
      },),
  ];
}

//endregion Displacement probe report
