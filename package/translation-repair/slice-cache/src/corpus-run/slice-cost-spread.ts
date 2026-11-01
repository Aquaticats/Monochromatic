import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { SliceCostRow, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  COUNT_WIDTH,
  MINUTES_WIDTH,
  MS_PER_MINUTE,
} from './slice-cost-bands.ts';

//region Slice cost spread
// WHAT THE BANDS AVERAGE AWAY: the cheapest and the dearest priced slice, side
// by side.

/**
 Reports the widest gap between two slices, and what sizes they were.

 WHY THE BANDS ALONE CAN MISLEAD, and the reason this exists. The bands answer
 "does cost scale with size" by averaging within a size range, which is only
 meaningful when slices of one size cost roughly alike. On the first entry
 priced under the two-lane pipeline they did not: a 208-character slice cost
 14 seconds and a 141-character slice cost 11.3 minutes, a factor of 48 in the
 WRONG DIRECTION for size. The cheap one raised no critic claims, so it never
 reached the editor, the panel, the judge or the checker.

 So the cost driver is HOW MUCH A SLICE TURNS OUT TO NEED, which no size band
 can show, and a run with few slices can put one such outlier in a band and
 produce a shape that looks like an answer. This line makes that visible in
 the same breath as the bands rather than in a document nobody reads next to
 them.

 @param rows - every parsed cost line

 @example
 ```ts
 printSliceCostSpread({ rows, },);
 ```
 */
export function printSliceCostSpread({ rows, }: { readonly rows: readonly SliceCostRow[]; },): void {
  /**
   Rows that priced real work.
   */
  const computed = rows.filter(function didWork(row,): boolean {
    return row.exit === 'computed';
  },);
  if (computed.length < 2) {
    console.log('\nSPREAD, WHICH THE BANDS AVERAGE AWAY',);
    console.log(
      `  NO SPREAD TO SHOW: it takes two priced slices to compare, and this log holds ${String(computed.length,)} ${
        wordForCount({
          count: computed.length,
          one: 'priced slice',
          many: 'priced slices',
        },)
      }.`,
    );
    return;
  }

  /**
   Slice that cost least.
   */
  const cheapest = computed.reduce(function cheaper(
    best,
    row,
  ): SliceCostRow {
    return (row.elapsedMs < best.elapsedMs) ? row : best;
  },);

  /**
   Slice that cost most.
   */
  const dearest = computed.reduce(function dearer(
    worst,
    row,
  ): SliceCostRow {
    return (row.elapsedMs > worst.elapsedMs) ? row : worst;
  },);

  /**
   Minutes the cheapest slice took.
   */
  const cheapMinutes = cheapest.elapsedMs / MS_PER_MINUTE;

  /**
   Minutes the dearest slice took.
   */
  const dearMinutes = dearest.elapsedMs / MS_PER_MINUTE;

  /**
   Cheapest slice's minutes, rendered.
   */
  const cheapText = cheapMinutes.toFixed(2,);

  /**
   Dearest slice's minutes, rendered.
   */
  const dearText = dearMinutes.toFixed(2,);

  /**
   Cheapest slice's size, rendered.
   */
  const cheapChars = String(cheapest.sourceChars,);

  /**
   Dearest slice's size, rendered.
   */
  const dearChars = String(dearest.sourceChars,);

  console.log('\nSPREAD, WHICH THE BANDS AVERAGE AWAY',);
  console.log(
    `  cheapest  ${cheapChars.padStart(COUNT_WIDTH,)} ${
      wordForCount({
        count: cheapest.sourceChars,
        one: 'char',
        many: 'chars',
      },)
    }`
      + `  ${cheapText.padStart(MINUTES_WIDTH,)} min  ${cheapest.lane}`,
  );
  console.log(
    `  dearest   ${dearChars.padStart(COUNT_WIDTH,)} ${
      wordForCount({
        count: dearest.sourceChars,
        one: 'char',
        many: 'chars',
      },)
    }`
      + `  ${dearText.padStart(MINUTES_WIDTH,)} min  ${dearest.lane}`,
  );
  // A CHEAPEST SLICE THAT TOOK NO TIME HAS NO RATIO TO SHOW, and the report
  // says so rather than leaving a reader to wonder whether the spread was left
  // out.
  if (cheapest.elapsedMs === 0) {
    console.log('  ratio     not defined: the cheapest slice took no measurable time',);
    return;
  }

  /**
   How many times more the dearest slice cost, rendered.
   */
  const ratio = (dearest.elapsedMs / cheapest.elapsedMs).toFixed(1,);
  console.log(`  ratio     ${ratio}x`,);
  console.log(
    '  A large ratio with the DEARER slice no larger than the cheaper one refutes'
      + ' size as the driver outright, whatever this report\'s bands show, and points at'
      + ' how much each slice turned out to need instead.',
  );
}

//endregion Slice cost spread
