import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { lineStartsOf, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type ContentChunk,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  REFRAIN_SLOT_MAXIMUM,
  REFRAIN_SLOT_MINIMUM,
} from './repeat-groups-bounds.ts';
import { compareFoundRepeats, } from './repeat-groups-order.ts';
import { disjointRepeatsOf, } from './repeat-groups-overlap.ts';
import {
  clausesOf,
  refrainFramesOf,
} from './repeat-groups-refrain.ts';
import {
  quotedCountsOf,
  touchesQuotation,
} from './repeat-groups-quotation.ts';
import { maximalRunsOf, } from './repeat-groups-runs.ts';
import type {
  FoundRepeat,
  LocatedSpan,
  Placement,
  RepeatGroup,
  RepeatOccurrence,
  Span,
  UnplacedOccurrence,
} from './repeat-groups-types.ts';
import {
  wordingEndAt,
  wordingStreamOf,
} from './repeat-groups-wording.ts';

//region Repeat groups entry
// The public lookup: every repeat of body wording in a page, placed in the
// slices that hold it. Detection is deterministic and judges nothing.

/**
 The slice holding a span whole, or the reason none does. Where several slices
 hold it whole, the one with the lowest index places it, so the answer never
 depends on the order the slices come in. Where none does, the span is
 reported as crossing a slice end if some slice holds its start, and as in no
 slice otherwise.

 @param slices - chunks of the page, in any order, which may overlap

 @param span - source span of the occurrence

 @returns Holding slice's index, or why no slice holds the span whole

 @example
 ```ts
 const placement = placementOf({ slices, span, },);
 ```
 */
function placementOf(
  {
    slices,
    span,
  }: {
    readonly slices: readonly ContentChunk[];
    readonly span: Span;
  },
): Placement {
  /**
   Slices holding the span's first unit.
   */
  const holdingStart = slices.filter(function holdsStart(slice,): boolean {
    return (slice.startOffset <= span.startOffset) && (span.startOffset < slice.endOffset);
  },);

  /**
   Indexes of the slices that hold the span whole.
   */
  const holdingWhole = holdingStart
    .filter(function holdsEnd(slice,): boolean {
      return span.endOffset <= slice.endOffset;
    },)
    .map(function indexOfSlice(slice,): number {
      return slice.sliceIndex;
    },);
  if (holdingWhole.length > 0) {
    return {
      kind: 'placed',
      sliceIndex: holdingWhole.reduce(function lowest(
        a,
        b,
      ): number {
        return Math.min(
          a,
          b,
        );
      },),
    };
  }
  if (holdingStart.length > 0) {
    return {
      kind: 'unplaced',
      reason: 'crosses-slice-end',
    };
  }
  return {
    kind: 'unplaced',
    reason: 'in-no-slice',
  };
}

/**
 One-based line a source offset falls on, by binary search over where the lines start.

 @param lineStarts - offset each line of the page starts at, ascending from zero

 @param offset - absolute offset in the page

 @returns Line number counted from one

 @example
 ```ts
 const line = lineOf({ lineStarts: [0, 2,], offset: 2, },);
 ```
 */
function lineOf(
  {
    lineStarts,
    offset,
  }: {
    readonly lineStarts: readonly number[];
    readonly offset: number;
  },
): number {
  /**
   Indexes the answer can still be, the lowest and the highest, narrowed until they meet.
   */
  const range = {
    low: 0,
    high: lineStarts.length - 1,
  };
  while (range.low < range.high) {
    /**
     Index halfway between, rounded up so that the range always shrinks.
     */
    const middle = Math.ceil((range.low + range.high) / 2,);
    if (nonNullishOrThrow(lineStarts[middle],) <= offset)
      range.low = middle;
    else
      range.high = middle - 1;
  }
  return range.low + 1;
}

/**
 Whether a repeat counts: no occurrence touches quoted speech anywhere, the
 quotation marks included, and its occurrences do not all sit on one line.

 @param spans - places the repeat stands

 @param lineStarts - offset each line of the page starts at

 @returns Whether the repeat is reported

 @example
 ```ts
 const reported = isReported({ spans, lineStarts, },);
 ```
 */
function isReported(
  {
    spans,
    lineStarts,
  }: {
    readonly spans: readonly Span[];
    readonly lineStarts: readonly number[];
  },
): boolean {
  if (spans.some(function isQuoted(span,): boolean {
    return span.quoted;
  },))
    return false;
  /**
   Lines the occurrences start on, each once.
   */
  const lines = new Set(spans.map(function lineOfSpan(span,): number {
    return lineOf({
      lineStarts,
      offset: span.startOffset,
    },);
  },),);
  return lines.size > 1;
}

/**
 Places one found repeat in the slices, giving its public shape. An occurrence
 no slice holds whole is reported beside the placed ones, never dropped.

 @param repeat - repeat found in the wording stream, reported

 @param slices - chunks of the page

 @param lineStarts - offset each line of the page starts at

 @returns Repeat group with each occurrence placed in its slice, or listed as unplaced

 @example
 ```ts
 const group = placedRepeat({ repeat, slices, lineStarts, },);
 ```
 */
function placedRepeat(
  {
    repeat,
    slices,
    lineStarts,
  }: {
    readonly repeat: FoundRepeat;
    readonly slices: readonly ContentChunk[];
    readonly lineStarts: readonly number[];
  },
): RepeatGroup {
  /**
   Each span with where the slices put it.
   */
  const located = repeat.spans
    .map(function withPlacement(span,): LocatedSpan {
      return {
        span,
        placement: placementOf({
          slices,
          span,
        },),
        line: lineOf({
          lineStarts,
          offset: span.startOffset,
        },),
      };
    },);

  /**
   Occurrences a slice holds whole, with the slot a refrain leaves open.
   */
  const placed = located.flatMap(function placedOnly(here,): readonly (RepeatOccurrence & {
    readonly slotStartOffset: number;
    readonly slotEndOffset: number;
  })[] {
    if (here.placement
      .kind
      !== 'placed')
      return [];
    return [{
      sliceIndex: here.placement
        .sliceIndex,
      startOffset: here.span
        .startOffset,
      endOffset: here.span
        .endOffset,
      line: here.line,
      slotStartOffset: here.span
        .slotStartOffset,
      slotEndOffset: here.span
        .slotEndOffset,
    },];
  },);

  /**
   Occurrences no slice holds whole, with why.
   */
  const unplaced = located.flatMap(function unplacedOnly(here,): readonly UnplacedOccurrence[] {
    if (here.placement
      .kind
      !== 'unplaced')
      return [];
    return [{
      startOffset: here.span
        .startOffset,
      endOffset: here.span
        .endOffset,
      line: here.line,
      reason: here.placement
        .reason,
    },];
  },);
  if (repeat.kind === 'maximal') {
    return {
      kind: 'maximal',
      length: repeat.length,
      occurrences: placed.map(function maximalOccurrence(occurrence,): RepeatOccurrence {
        return {
          sliceIndex: occurrence.sliceIndex,
          startOffset: occurrence.startOffset,
          endOffset: occurrence.endOffset,
          line: occurrence.line,
        };
      },),
      unplaced,
    };
  }
  return {
    kind: 'refrain',
    prefix: repeat.prefix,
    suffix: repeat.suffix,
    slotBounds: {
      minimum: REFRAIN_SLOT_MINIMUM,
      maximum: REFRAIN_SLOT_MAXIMUM,
    },
    occurrences: placed,
    unplaced,
  };
}

/**
 Finds every repeat of source wording in a page, body paragraphs only, and
 places each occurrence in the slice that holds it.

 Repeats are returned in page order of their first occurrence. A group is left
 out when any character of any occurrence is quoted speech or a quotation mark,
 wherever the occurrence begins, or when its occurrences all sit on one line.

 No two groups claim the same text: an occurrence of one group never overlaps
 an occurrence of another, placed or not, so a pass that splices one rendering into
 every occurrence of a group never splices two into one span. Where groups would
 overlap (periodic text, a repeat inside a longer one that fewer places share,
 a refrain frame over clauses a maximal repeat covers), the one whose
 occurrences cover more of the page stays and the others are dropped whole. A
 tie goes to the group listed first: the earlier first occurrence, then a
 maximal repeat before a refrain, then the first occurrence that ends sooner,
 then the earlier of the later occurrences.

 AN OCCURRENCE NO SLICE HOLDS WHOLE IS LISTED, NOT DROPPED AND NOT REFUSED.
 The slices a pass reads cover a page only in part: a section the alignment
 refused has none, and an original the archive's note seals reaches none. Such
 an occurrence lies in `unplaced` with its offsets and why, so a caller sees the
 group's full count and can leave those places as they stand. A group can hold
 fewer than two placed occurrences.

 @param text - full source page text, front matter included

 @param slices - chunks of that page, such as the sources of a pass's slice pairs

 @returns Repeat groups in page order

 @example
 ```ts
 const groups = repeatGroupsOf({ text, slices: chunkByHeadings({ document: parseDocument({ text, },), },), },);
 ```
 */
export function repeatGroupsOf(
  {
    text,
    slices,
  }: {
    readonly text: string;
    readonly slices: readonly ContentChunk[];
  },
): readonly RepeatGroup[] {
  /**
   Parsed page, whose body paragraphs are the only wording read.
   */
  const document = parseDocument({ text, },);

  /**
   Offset each line of the page starts at, read once for every occurrence's line.
   */
  const lineStarts = lineStartsOf({ text, },);

  /**
   Wording stream of the body paragraphs.
   */
  const wording = wordingStreamOf({
    text,
    document,
  },);

  /**
   Wording characters as one string element each, for slicing runs.
   */
  const stream = wording.chars;

  /**
   Quoted characters before each place in the stream, so each run is asked in one subtraction
   whether any character of it touches quoted speech.
   */
  const quotedCounts = quotedCountsOf({ wording, },);

  /**
   Maximal repeats, each place a run stands with its quotation flag.
   */
  const maximal: FoundRepeat[] = maximalRunsOf({ stream, },)
    .map(function maximalFound(run,): FoundRepeat {
      return {
        kind: 'maximal',
        prefix: '',
        suffix: '',
        length: run.length,
        spans: run.starts
          .map(function runSpan(start,) {
        /**
         Stream index of the run's last character.
         */
            const last = (start + run.length) - 1;
            return {
              startOffset: nonNullishOrThrow(wording.offsets[start],),
              endOffset: wordingEndAt({
                wording,
                index: last,
              },),
              quoted: touchesQuotation({
                counts: quotedCounts,
                start,
                end: start + run.length,
              },),
              width: run.length,
              slotStartOffset: nonNullishOrThrow(wording.offsets[start],),
              slotEndOffset: wordingEndAt({
                wording,
                index: last,
              },),
            };
          },),
      };
    },);

  /**
   Refrain frames, each clause with the slot it leaves open.
   */
  const refrain: FoundRepeat[] = refrainFramesOf({ clauses: clausesOf({
    wording,
    stream,
  },), },)
    .map(
    function refrainFound(frame,): FoundRepeat {
      return {
        kind: 'refrain',
        prefix: frame.prefix,
        suffix: frame.suffix,
        length: 0,
        spans: frame.members
          .map(function clauseSpan(clause,) {
          /**
           Stream index of the clause's last character.
           */
            const last = clause.end - 1;
            return {
              startOffset: nonNullishOrThrow(wording.offsets[clause.start],),
              endOffset: wordingEndAt({
                wording,
                index: last,
              },),
              quoted: clause.quoted,
              width: clause.end - clause.start,
              slotStartOffset: nonNullishOrThrow(wording.offsets[clause.start + frame.prefixLength],),
              slotEndOffset: wordingEndAt({
                wording,
                index: last - frame.suffixLength,
              },),
            };
          },),
      };
    },
    );

  /**
   Repeats that count, before any two that overlap are told apart: a repeat left out for quoted speech or
   for standing on one line must not take a place from one that stays.
   */
  const reported = [
    ...maximal,
    ...refrain,
  ].filter(function reportedOnly(repeat,): boolean {
    return isReported({
      spans: repeat.spans,
      lineStarts,
    },);
  },);
  return disjointRepeatsOf({ repeats: reported, },)
    .toSorted(function inPageOrder(
      a,
      b,
    ): number {
      return compareFoundRepeats({
        first: a,
        second: b,
      },);
    },)
    .map(function placed(repeat,): RepeatGroup {
      return placedRepeat({
        repeat,
        slices,
        lineStarts,
      },);
    },);
}

//endregion Repeat groups entry
