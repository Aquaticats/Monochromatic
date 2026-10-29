import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { compareFoundRepeats, } from './repeat-groups-order.ts';
import type {
  FoundRepeat,
  FoundSpan,
} from './repeat-groups-types.ts';

//region Repeat groups overlap
// Keeps the groups apart. The finders read each repeat alone, so periodic text
// (a unit stated several times in a row) yields several repeats that share a
// start with different lengths, a repeat can sit inside a longer one that fewer
// places share, and a refrain frame can cover clauses a maximal repeat covers. The pass
// that reads the groups splices one rendering into every occurrence of a
// group, and two groups claiming one stretch of text would have it splice two.
// So the groups returned never overlap: where two would, the one covering more
// text stays and the other is dropped whole. This is a pass over the found
// repeats and not a reordering inside the run finder, because a refrain frame
// is no run, and because the repeats that overlap without periodic text (a
// sub-run that fewer places share) come from different sets of places, which no
// order of extension and overlap removal inside one set can join.

/**
 Wording characters a repeat's occurrences cover, one per code point, counting
 every occurrence whether or not a slice holds it. A character beyond the basic
 plane counts once, and a comment inside an occurrence adds nothing, so the group
 kept does not change with the encoding of the text. Counting every occurrence
 keeps the groups from depending on the slices a caller reads the page through.

 @param repeat - found repeat

 @returns Sum of its occurrences' widths

 @example
 ```ts
 const covered = coverageOf({ repeat, },);
 ```
 */
function coverageOf({ repeat, }: { readonly repeat: FoundRepeat; },): number {
  return repeat.spans
    .reduce(
    function widthSum(
      total,
      span,
    ): number {
      return total + span.width;
    },
    0,
    );
}

/**
 Index of the first kept occurrence that ends after an offset, by binary
 search. The kept occurrences are sorted and disjoint, so their ends ascend too.

 @param kept - occurrences kept so far, ascending and disjoint

 @param offset - absolute offset to look past

 @returns Index, or the count of kept occurrences when none ends after the offset

 @example
 ```ts
 const at = firstEndingAfter({ kept, offset: 12, },);
 ```
 */
function firstEndingAfter(
  {
    kept,
    offset,
  }: {
    readonly kept: readonly FoundSpan[];
    readonly offset: number;
  },
): number {
  /**
   Indexes the answer can still be, the lowest and the highest, narrowed until they meet.
   */
  const range = {
    low: 0,
    high: kept.length,
  };
  while (range.low < range.high) {
    /**
     Index halfway between, rounded down so that the range always shrinks.
     */
    const middle = Math.floor((range.low + range.high) / 2,);

    /**
     Kept occurrence at that index.
     */
    const probe = nonNullishOrThrow(kept[middle],);
    if (probe.endOffset > offset)
      range.high = middle;
    else
      range.low = middle + 1;
  }
  return range.low;
}

/**
 Whether an occurrence shares a character of the page with any kept one. Two
 occurrences that touch, one ending where the other begins, share none.

 @param kept - occurrences kept so far, ascending and disjoint

 @param span - occurrence to test

 @returns Whether some kept occurrence overlaps it

 @example
 ```ts
 const clash = overlapsKept({ kept, span, },);
 ```
 */
function overlapsKept(
  {
    kept,
    span,
  }: {
    readonly kept: readonly FoundSpan[];
    readonly span: FoundSpan;
  },
): boolean {
  /**
   First kept occurrence that ends after this one starts, the only one that can overlap it from the left or inside.
   */
  const next = kept[firstEndingAfter({
    kept,
    offset: span.startOffset,
  },)];
  return (next !== undefined) && (next.startOffset < span.endOffset);
}

/**
 Order the groups are tried in: more covered text first, then the order the
 groups are listed in, so a tie is decided by the page alone.

 @param first - one found repeat

 @param second - other found repeat

 @returns Negative when the first is tried before the second

 @example
 ```ts
 const order = compareByCoverage({ first, second, },);
 ```
 */
function compareByCoverage(
  {
    first,
    second,
  }: {
    readonly first: FoundRepeat;
    readonly second: FoundRepeat;
  },
): number {
  /**
   Text the second covers less the text the first covers, positive when the second covers more.
   */
  const byCoverage = coverageOf({ repeat: second, },) - coverageOf({ repeat: first, },);
  if (byCoverage !== 0)
    return byCoverage;
  return compareFoundRepeats({
    first,
    second,
  },);
}

/**
 Keeps from found repeats a set no two of which overlap: tries them from the
 one covering the most text, ties by the order the groups are listed in (first
 occurrence earliest, a maximal repeat before a refrain, and so on), and keeps
 each that shares no character of the page with one kept already. A dropped
 repeat is dropped whole, with its other occurrences, so every repeat that
 stays lists all the places its wording stands. A repeat whose occurrences
 are not all clear of the kept ones is dropped even where some are.

 @param repeats - found repeats, each with its occurrences apart from one another

 @returns Repeats kept, most text first; sort them for the page

 @throws {@link Error} when a repeat's own occurrences overlap, which both finders keep apart

 @example
 ```ts
 const kept = disjointRepeatsOf({ repeats, },);
 ```
 */
export function disjointRepeatsOf({ repeats, }: { readonly repeats: readonly FoundRepeat[]; },): readonly FoundRepeat[] {
  /**
   Occurrences of the repeats kept so far, ascending by start and disjoint.
   */
  const keptSpans: FoundSpan[] = [];

  /**
   Repeats kept, in the order they were tried.
   */
  const keptRepeats: FoundRepeat[] = [];
  for (
    const repeat of repeats.toSorted(function byCoverage(
      first,
      second,
    ): number {
      return compareByCoverage({
        first,
        second,
      },);
    },)
  ) {
    /**
     Places this repeat stands.
     */
    const { spans, } = repeat;
    if (spans.some(function clashes(span,): boolean {
      return overlapsKept({
        kept: keptSpans,
        span,
      },);
    },))
      continue;
    for (const span of spans) {
      if (overlapsKept({
        kept: keptSpans,
        span,
      },)) {
        throw new Error(
          'unreachable: two occurrences of one repeat overlap, though both finders keep the occurrences of a repeat apart',
        );
      }
      keptSpans.splice(
        firstEndingAfter({
          kept: keptSpans,
          offset: span.startOffset,
        },),
        0,
        span,
      );
    }
    keptRepeats.push(repeat,);
  }
  return keptRepeats;
}

//endregion Repeat groups overlap
