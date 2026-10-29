import { MS_PER_DAY, } from './constants.ts';

/**
 Watts per bucket when judging temperature against power.
 */
export const BUCKET_WATTS = 20;

/**
 Previous samples that must share a sample's power bucket for it to count as settled (12 samples, 2 minutes).
 */
export const SETTLED_SAMPLES = 12;

/**
 Index of the `package_w` column in the sample log.
 */
const PACKAGE_WATTS_COLUMN = 4;

/**
 Fraction of the sorted temperatures at the median.
 */
const MEDIAN_FRACTION = 0.5;

/**
 Fraction of the sorted temperatures at the 90th percentile.
 */
const P90_FRACTION = 0.9;

/**
 The columns of a sample row the summary needs.
 */
export type SampleRow = Readonly<{
  /**
   ISO UTC time of the row.
   */
  time: string;
  /**
   Tctl in degrees Celsius.
   */
  tctl: number;
  /**
   Package watts.
   */
  watts: number;
}>;

/**
 Temperature statistics of the settled samples in one power bucket.
 */
export type BucketSummary = Readonly<{
  /**
   Lower edge of the bucket in watts.
   */
  bucket: number;
  /**
   Number of settled samples.
   */
  count: number;
  /**
   Median Tctl.
   */
  median: number;
  /**
   90th percentile Tctl.
   */
  p90: number;
  /**
   Highest Tctl.
   */
  max: number;
}>;

/**
 Parses the sample CSV lines the summary reads, skipping the header and rows with an empty Tctl or watts field.
 
 @param csv - contents of one daily sample file
 
 @param cutoff - epoch milliseconds before which rows are dropped

 @returns the parsed rows in file order

 @example
 ```ts
 parseSampleRows({ csv, cutoff: 0, },).length;
 ```
 */
export function parseSampleRows({
  csv,
  cutoff,
}: Readonly<{
  csv: string;
  cutoff: number
}>,): readonly SampleRow[] {
  return csv.split('\n',)
    .slice(1,)
    .flatMap(function parseLine(line,): readonly SampleRow[] {
    /**
     Fields of one CSV line.
     */
      const fields = line.split(',',);
    /**
     UTC time of the row.
     */
      const time = fields[0] ?? '';
    /**
     Tctl field of the row.
     */
      const tctl = fields[1] ?? '';
    /**
     Package watts field of the row.
     */
      const watts = fields[PACKAGE_WATTS_COLUMN] ?? '';
      if ((tctl === '') || (watts === '')
        || (Date.parse(time,) < cutoff)) return [];
      return [{
        time,
        tctl: Number(tctl,),
        watts: Number(watts,),
      },];
    },);
}

/**
 Returns the value at a fraction of an already sorted array.

 @param sorted - ascending values
 
 @param fraction - position from 0 to 1

 @returns the value at that position, 0 for an empty array

 @example
 ```ts
 percentile({ sorted: [1, 2, 3, 4,], fraction: 0.5, },); // 3
 ```
 */
export function percentile({
  sorted,
  fraction,
}: Readonly<{
  sorted: readonly number[];
  fraction: number;
}>,): number {
  return sorted[Math.min(
    sorted.length - 1,
    Math.floor(sorted.length * fraction,),
  )] ?? 0;
}

/**
 Summarizes Tctl per package-power bucket, counting only settled samples:
 a sample counts when the previous {@link SETTLED_SAMPLES} samples were all in its bucket,
 so alert-time spikes do not hide the steady state.

 @param rows - sample rows in time order

 @returns one summary per bucket, lowest bucket first

 @example
 ```ts
 summarizeBuckets(parseSampleRows({ csv, cutoff: 0, },),);
 ```
 */
export function summarizeBuckets(rows: readonly SampleRow[],): readonly BucketSummary[] {
  /**
   Power bucket of each row.
   */
  const buckets = rows.map(function bucketOf(row,): number {
    return Math.floor(row.watts / BUCKET_WATTS,) * BUCKET_WATTS;
  },);
  /**
   Tctl of each settled row with its bucket.
   */
  const settled = rows.flatMap(function settledRow(
    row,
    index,
  ): readonly Readonly<{
    bucket: number;
    tctl: number
  }>[] {
    /**
     Power bucket of this row.
     */
    const bucket = buckets[index];
    /**
     Whether the previous rows all share this row's bucket.
     */
    const isSettled = (index >= SETTLED_SAMPLES)
      && buckets.slice(
        index - SETTLED_SAMPLES,
        index,
      )
        .every(function sameBucket(other,) {
          return other === bucket;
        },);
    return isSettled ? [{
      bucket: bucket ?? 0,
      tctl: row.tctl,
    },] : [];
  },);
  /**
   Settled rows grouped by bucket.
   */
  const grouped = Map.groupBy(
    settled,
    function byBucket(row,): number {
      return row.bucket;
    },
  );
  return grouped.entries()
    .toArray()
    .toSorted(function byBucketAscending(
      [a,],
      [b,],
    ): number {
      return a - b;
    },)
    .map(function summarize([bucket, bucketRows,]): BucketSummary {
    /**
     Ascending Tctl values of the bucket.
     */
      const sorted = bucketRows.map(function tctlOf(row,): number {
        return row.tctl;
      },)
        .toSorted(function ascending(
          a,
          b,
        ): number {
          return a - b;
        },);
      return {
        bucket,
        count: sorted.length,
        median: percentile({
          sorted,
          fraction: MEDIAN_FRACTION,
        },),
        p90: percentile({
          sorted,
          fraction: P90_FRACTION,
        },),
        max: sorted.at(-1,) ?? 0,
      };
    },);
}

/**
 Epoch-millisecond cutoff for a look-back window.
 
 @param now - current epoch milliseconds
 
 @param daysBack - days to look back

 @returns the cutoff

 @example
 ```ts
 cutoffFor({ now: Date.now(), daysBack: 1, },);
 ```
 */
export function cutoffFor({
  now,
  daysBack,
}: Readonly<{
  now: number;
  daysBack: number
}>,): number {
  return now - (daysBack
    * MS_PER_DAY);
}

/**
 Median, minimum and maximum of one column.
 */
export type ColumnStats = Readonly<{
  /**
   Column name from the CSV header.
   */
  name: string;
  /**
   Median value.
   */
  median: number;
  /**
   Smallest value.
   */
  min: number;
  /**
   Largest value.
   */
  max: number;
}>;

/**
 Computes statistics of named columns over the rows between two UTC times.
 
 @param csv - contents of one daily sample file, header first
 
 @param from - earliest UTC time (ISO string, compared as text)
 
 @param to - latest UTC time (ISO string, compared as text)
 
 @param names - columns to summarize

 @returns the number of rows in range and one entry per column that exists in the header

 @example
 ```ts
 columnStats({ csv, from: '2026-10-13T20:00', to: '2026-10-13T21:00', names: ['tctl_c',], },);
 ```
 */
export function columnStats({
  csv,
  from,
  to,
  names,
}: Readonly<{
  csv: string;
  from: string;
  to: string;
  names: readonly string[];
}>,): Readonly<{
  rows: number;
  columns: readonly ColumnStats[]
}> {
  /**
   Header line and data lines of the CSV.
   */
  const [headerLine = '', ...lines] = csv.trim()
    .split('\n',);
  /**
   Column names.
   */
  const header = headerLine.split(',',);
  /**
   Data rows within the time range.
   */
  const rows = lines.map(function splitLine(line,): readonly string[] {
    return line.split(',',);
  },)
    .filter(function inRange(row,): boolean {
    /**
     UTC time of the row.
     */
      const time = row[0] ?? '';
      return (time >= from) && (time <= to);
    },);
  /**
   Statistics per requested column.
   */
  const columns = names.flatMap(function statsOf(name,): readonly ColumnStats[] {
    /**
     Position of the column in the header.
     */
    const index = header.indexOf(name,);
    if (index === (-1)) return [];
    /**
     Ascending values of the column inside the range.
     */
    const values = rows.map(function valueOf(row,): number {
      return Number(row[index],);
    },)
      .toSorted(function ascending(
        a,
        b,
      ): number {
        return a - b;
      },);
    return [{
      name,
      median: values[Math.floor(values.length / 2,)] ?? 0,
      min: values[0] ?? 0,
      max: values.at(-1,) ?? 0,
    },];
  },);
  return {
    rows: rows.length,
    columns,
  };
}
