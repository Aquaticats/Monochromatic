import { REPORTED_PERCENTILES, } from './census-spread.ts';
import type { EntryCensus, } from './slice-census-entry.ts';

//region Slice census widest
// THE TAIL OF THE SLICE SIZES, by entry. A per-call deadline meets the largest
// slice first, and a percentile hides which entry owns it.

/**
 Size at which a translate call is known to be at risk.

 The translate probe asked for a 4641-character section in one call and lost
 two voices of three: one timed out at six minutes, one returned
 schema-invalid output. That is the only measured point on this curve, so it
 is the threshold rather than a round number.
 */
const PROBE_TIMEOUT_CHARS = 4_641;

/**
 One entry with the largest single slice it produced.

 Named rather than inferred, because an inferred object literal carries
 writable properties and the comparator that sorts these then takes mutable
 parameters it never mutates.

 @example
 ```ts
 const widest: WidestSlice = { entryId: 'shihai4h', largest: 10_959, };
 ```
 */
type WidestSlice = Readonly<{
  /**
   Corpus id.
   */
  entryId: string;

  /**
   Characters in its largest slice, on either side.
   */
  largest: number;
}>;

/**
 The census lines for the largest slices: how many target sides pass the
 size a translate call is known to be at risk at, then the entries owning the
 largest slices.

 @param rows - measured entries

 @returns The count line, then one line for each of the widest entries, as
 many as there are reported percentiles

 @example
 ```ts
 const lines = sliceCensusWidestLines({ rows, },);
 ```
 */
export function sliceCensusWidestLines(
  { rows, }: { readonly rows: readonly EntryCensus[]; },
): readonly string[] {
  /**
   Every slice's target characters.
   */
  const targetChars = rows.flatMap(function toTargetChars(row,) {
    return [...row.sliceTargetChars,];
  },);

  /**
   Entries ordered by their largest slice, since the tail is what a per-call
   deadline meets first and a percentile hides which entry owns it.
   */
  const widest = rows
    .map(function toWidest(row,): WidestSlice {
      return {
        entryId: row.entryId,
        largest: Math.max(
          0,
          ...row.sliceTargetChars,
          ...row.sliceSourceChars,
        ),
      };
    },)
    .toSorted(function byLargest(
      left,
      right,
    ) {
      return right.largest - left.largest;
    },);

  /**
   Slices carrying more than the whole-section call the translate probe
   already saw time out at six minutes.
   */
  const oversized = targetChars.filter(function isLarge(chars,) {
    return chars > PROBE_TIMEOUT_CHARS;
  },);
  return [
    `CENSUS slices over ${String(PROBE_TIMEOUT_CHARS,)} target chars: ${
      String(oversized.length,)
    } of ${String(targetChars.length,)}`,
    ...widest
      .slice(
      0,
      REPORTED_PERCENTILES.length,
      )
      .map(function widestLine(row,): string {
        return `CENSUS   widest ${row.entryId}: chars in one slice: ${String(row.largest,)}`;
      },),
  ];
}

//endregion Slice census widest
