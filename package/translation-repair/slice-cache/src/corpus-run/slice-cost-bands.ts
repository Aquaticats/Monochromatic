import type { SliceCostRow, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Slice cost bands
// WHAT A SLICE COSTS, BY THE SIZE OF ITS ORIGINAL: the size bands the slice
// cost report groups priced slices into, and the line it prints for each.

/**
 Upper bound of the smallest band, where a slice is barely more than a heading.
 */
const TINY_SLICE_CHARS = 50;

/**
 Upper bound of the band a short paragraph falls in.
 */
const SMALL_SLICE_CHARS = 200;

/**
 Upper bound of the band an ordinary paragraph falls in.
 */
const MEDIUM_SLICE_CHARS = 500;

/**
 Upper bound of the band a long passage falls in.
 */
const LARGE_SLICE_CHARS = 1_000;

/**
 Upper bound of the last named band; anything above is open-ended.
 */
const HUGE_SLICE_CHARS = 2_000;

/**
 Buckets slices are grouped into, by size of their original.

 BOUNDARIES ARE ROUND NUMBERS CHOSEN BEFORE READING ANY DATA, so a reader can
 see they were not fitted to make a curve look like anything. They exist to
 spread the corpus's slices across several groups, nothing more.
 */
const SIZE_BUCKETS = [
  TINY_SLICE_CHARS,
  SMALL_SLICE_CHARS,
  MEDIUM_SLICE_CHARS,
  LARGE_SLICE_CHARS,
  HUGE_SLICE_CHARS,
] as const;


/**
 Milliseconds in a minute, for reporting.
 */
export const MS_PER_MINUTE = 60_000;

/**
 Column widths, so the bands line up under each other and a falling column is
 visible as a shape rather than as numbers a reader has to compare by eye.
 */
const LABEL_WIDTH = 6;

/**
 Width the slice count is padded to.
 */
export const COUNT_WIDTH = 4;

/**
 Width the per-slice minutes are padded to.
 */
export const MINUTES_WIDTH = 6;

/**
 Width the per-character milliseconds are padded to.
 */
const PER_CHAR_WIDTH = 7;

/**
 What one size bucket amounts to.

 @example
 ```ts
 const bucket: CostBucket = { upTo: 200, slices: 12, chars: 1400, ms: 90000, };
 ```
 */
export type CostBucket = {
  /**
   Largest source size in this bucket, or `Infinity` for the last.
   */
  readonly upTo: number;

  /**
   Slices that landed here.
   */
  readonly slices: number;

  /**
   Characters they carried in total.
   */
  readonly chars: number;

  /**
   Time they took in total.
   */
  readonly ms: number;
};

/**
 Groups rows by the size of the original they translated.

 ONLY `computed` ROWS COUNT. A cached slice reports the microseconds it took to
 read a file and a skipped one reports nothing worth pricing; averaging those
 in would report a pipeline far cheaper than the one that runs.

 @param rows - every parsed cost line

 @returns One bucket per size band, smallest first

 @example
 ```ts
 const buckets = bucketSliceCostsBySize({ rows, },);
 ```
 */
export function bucketSliceCostsBySize(
  { rows, }: { readonly rows: readonly SliceCostRow[]; },
): readonly CostBucket[] {
  /**
   Rows that priced real work.
   */
  const computed = rows.filter(function didWork(row,): boolean {
    return row.exit === 'computed';
  },);

  /**
   Upper bounds, with an open-ended last band.
   */
  const bounds = [
    ...SIZE_BUCKETS,
    Number.POSITIVE_INFINITY,
  ];

  return bounds.map(function summarise(
    upTo,
    at,
  ): CostBucket {
    /**
     Upper bound of the next smaller band, which is where this one starts.
     */
    const smallerBound = (at === 0) ? 0 : bounds[at - 1];
    if (smallerBound === undefined) {
      throw new Error(
        'unreachable: a band past the first has no smaller band, since the index counts up from zero over the same list',
      );
    }

    /**
     Smallest size this band accepts.
     */
    const from = smallerBound;

    /**
     Rows in this band.
     */
    const mine = computed.filter(function inBand(row,): boolean {
      return (row.sourceChars >= from) && (row.sourceChars < upTo);
    },);

    return {
      upTo,
      slices: mine.length,
      chars: mine.reduce(
        function addChars(
          sum,
          row,
        ): number {
          return sum + row.sourceChars;
        },
        0,
      ),
      ms: mine.reduce(
        function addMs(
          sum,
          row,
        ): number {
          return sum + row.elapsedMs;
        },
        0,
      ),
    };
  },);
}

/**
 Prints one bucket, with the two figures that separate the explanations.

 @param bucket - one size band

 @example
 ```ts
 printSliceCostBucket({ bucket, },);
 ```
 */
export function printSliceCostBucket({ bucket, }: { readonly bucket: CostBucket; },): void {
  if (bucket.slices === 0)
    return;

  /**
   Time a slice in this band costs.
   */
  const perSlice = bucket.ms / bucket.slices;

  /**
   Time a CHARACTER in this band costs, which is the figure that tells a
   fixed overhead from a size-driven cost.
   */
  const perChar = (bucket.chars === 0) ? 0 : (bucket.ms / bucket.chars);

  /**
   Band label, open-ended for the last one.
   */
  const label = (bucket.upTo === Number.POSITIVE_INFINITY) ? 'any' : String(bucket.upTo,);

  /**
   Minutes a slice costs here, rendered.
   */
  const minutes = (perSlice / MS_PER_MINUTE).toFixed(2,);

  /**
   Milliseconds a character costs here, rendered.
   */
  const perCharText = perChar.toFixed(1,);

  /**
   Slice count, rendered.
   */
  const count = String(bucket.slices,);

  console.log(
    `  under ${label.padStart(LABEL_WIDTH,)} chars  slices ${count.padStart(COUNT_WIDTH,)}`
      + `  min/slice ${minutes.padStart(MINUTES_WIDTH,)}`
      + `  ms/char ${perCharText.padStart(PER_CHAR_WIDTH,)}`,
  );
}

//endregion Slice cost bands
