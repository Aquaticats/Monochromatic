import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  columnStats,
  cutoffFor,
  parseSampleRows,
  percentile,
  summarizeBuckets,
  type SampleRow,
} from './sample-stats.ts';

/**
 Builds rows at one power level with rising temperatures.

 @param watts - package watts of every row
 @param count - number of rows
 @param tctlStart - Tctl of the first row

 @returns the rows
 */
function rowsAt({ watts, count, tctlStart, }: Readonly<{ watts: number; count: number; tctlStart: number; }>,): readonly SampleRow[] {
  return Array.from({ length: count, }, function rowAt(_unused, index,): SampleRow {
    return { time: `2026-10-13T00:00:${String(index,).padStart(2, '0',)}Z`, tctl: tctlStart + index, watts, };
  },);
}

await describe({
  name: 'sample-stats',
  children: [
    describe({
      name: parseSampleRows.name,
      children: [
        it({
          name: 'skips the header, rows without Tctl or watts, and rows before the cutoff',
          fn: async function parsesRows() {
            /**
             A header and four rows: good, empty Tctl, empty watts, old.
             */
            const csv = [
              'utc_time,tctl_c,tccd1_c,tccd2_c,package_w',
              '2026-10-13T10:00:00.000Z,60.5,1,2,74.8',
              '2026-10-13T10:00:10.000Z,,1,2,74.8',
              '2026-10-13T10:00:20.000Z,60.5,1,2,',
              '2026-10-12T10:00:00.000Z,60.5,1,2,74.8',
            ].join('\n',);
            expect(parseSampleRows({ csv, cutoff: Date.parse('2026-10-13T00:00:00Z',), },),)
              .toEqual([{ time: '2026-10-13T10:00:00.000Z', tctl: 60.5, watts: 74.8, },],);
          },
        },),
      ],
    },),
    describe({
      name: percentile.name,
      children: [
        it({
          name: 'indexes into the sorted values and answers 0 for none',
          fn: async function percentiles() {
            expect(percentile({ sorted: [1, 2, 3, 4, 5,], fraction: 0.5, },),).toBe(3,);
            expect(percentile({ sorted: [1, 2, 3, 4, 5,], fraction: 0.9, },),).toBe(5,);
            expect(percentile({ sorted: [], fraction: 0.5, },),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: summarizeBuckets.name,
      children: [
        it({
          name: 'counts a sample only after twelve earlier samples in the same 20 W bucket',
          fn: async function settledOnly() {
            /**
             Fourteen rows in the 60 to 80 W bucket: rows 12 and 13 are settled.
             */
            const summary = summarizeBuckets(rowsAt({ watts: 65, count: 14, tctlStart: 70, },),);
            expect(summary,).toEqual([{ bucket: 60, count: 2, median: 83, p90: 83, max: 83, },],);
          },
        },),
        it({
          name: 'does not count samples that follow a step into another bucket',
          fn: async function stepNotSettled() {
            /**
             Twelve rows at 65 W then twelve at 125 W: the second bucket never settles within its first twelve.
             */
            const rows = [...rowsAt({ watts: 65, count: 12, tctlStart: 70, },), ...rowsAt({ watts: 125, count: 12, tctlStart: 90, },),];
            expect(summarizeBuckets(rows,),).toEqual([],);
          },
        },),
        it({
          name: 'orders buckets from lowest to highest',
          fn: async function ordersBuckets() {
            /**
             Thirteen rows at 125 W followed by thirteen at 65 W.
             */
            const rows = [...rowsAt({ watts: 125, count: 13, tctlStart: 90, },), ...rowsAt({ watts: 65, count: 13, tctlStart: 60, },),];
            expect(summarizeBuckets(rows,).map(function bucketOf(summary,): number {
              return summary.bucket;
            },),).toEqual([60, 120,],);
          },
        },),
      ],
    },),
    describe({
      name: cutoffFor.name,
      children: [
        it({
          name: 'subtracts whole days',
          fn: async function subtractsDays() {
            expect(cutoffFor({ now: 2 * 86_400_000, daysBack: 1, },),).toBe(86_400_000,);
          },
        },),
      ],
    },),
    describe({
      name: columnStats.name,
      children: [
        it({
          name: 'reports median, minimum and maximum of rows inside the time range only',
          fn: async function statsInRange() {
            /**
             Header and four rows, the last outside the range.
             */
            const csv = 'utc_time,tctl_c,package_w\n10:00,60,70\n10:10,62,72\n10:20,64,74\n11:00,99,99\n';
            /**
             Statistics of both columns plus one unknown name.
             */
            const stats = columnStats({ csv, from: '10:00', to: '10:30', names: ['tctl_c', 'package_w', 'missing',], },);
            expect(stats.rows,).toBe(3,);
            expect(stats.columns,).toEqual([
              { name: 'tctl_c', median: 62, min: 60, max: 64, },
              { name: 'package_w', median: 72, min: 70, max: 74, },
            ],);
          },
        },),
      ],
    },),
  ],
},);
