import { baseIndentAt, } from './indent.ts';
import { at, } from './range.ts';

/**
 One continuation-line break of a flattened chain.
 
 `breakOffset` is the byte offset of the leading token that renders at the
 start of that line (the `.`/`?.` for a member step, or the operator token for
 an operator step). `anchorOffset` is the byte offset of the node owning that
 token (the member step's chain head, or the operator node), so the break's
 line indents two columns past the line where the anchor renders.
 
 @internal
 */
export type ChainBreak = {
  /**
   Byte offset of the segment's leading token; start of its continuation line.
   */
  readonly breakOffset: number;
  /**
   Byte offset of the node owning the leading token; the line carrying it
   anchors this break's indent.
   */
  readonly anchorOffset: number;
};

/**
 One element of a flattened chain in source order.
 
 `isBreak: true` marks a break point (a member-name step or an operator's
 right operand) that may begin a continuation line, carrying the offsets
 {@link ChainBreak} describes. `isBreak: false` marks an attached segment (the
 head leaf, a call step, or a computed step) that always rides on the line of
 the segment before it.
 
 @internal
 */
export type ChainSegment =
  | { readonly isBreak: false; }
    | {
      readonly isBreak: true;
    /**
     Byte offset of the segment's leading token; start of its continuation line.
     */
      readonly breakOffset: number;
    /**
     Byte offset of the node owning the leading token; anchors the line's indent.
     */
      readonly anchorOffset: number;
    };

/**
 Selects the breaks that actually begin a continuation line.
 
 The layout rule reduces to a single test: a break point breaks when two or
 more segments precede it on the head line, and every break point after the
 first such one also breaks. Because segments accumulate left to right and the
 leaf occupies index `0`, "two or more segments before it" is exactly
 "segment index of two or more". Every later break point has a higher index,
 so the set of breaking points is precisely the break points at index two or
 greater; locating the first one separately is unnecessary.
 
 @param segments - flattened chain segments in source order, leaf at index 0
 
 @returns breaks in source order, empty when the chain stays on one line
 
 @example
 ```ts
 // segments for `a.b.c`: [leaf, .b(break), .c(break)]
 selectBreaks(segments); // [break for `.c`]
 ```
 
 @internal
 */
export function selectBreaks(
  segments: readonly ChainSegment[],
): readonly ChainBreak[] {
  return segments.flatMap(function pick(
    segment,
    index,
  ): ChainBreak[] {
    // The literal 2 is the magic-number exemption; it needs no name.
    return (segment.isBreak
      && (index >= 2))
      ? [{
        breakOffset: segment.breakOffset,
        anchorOffset: segment.anchorOffset,
      },]
      : [];
  },);
}

/**
 Parameters for {@link sliceIndexOf}.
 */
type SliceIndexOfParams = {
  /**
   Breaks in source order.
   */
  readonly breaks: readonly ChainBreak[];
  /**
   Byte offset whose containing slice is wanted.
   */
  readonly offset: number;
};

/**
 Finds the continuation slice containing an offset, or the head slice.
 
 Slices partition the region: the head runs to the first break, each
 continuation runs from its break to the next. A binary search over the
 ascending break offsets lands on the last break at or before the offset,
 which is exactly the slice start unless every break follows the offset and it
 sits on the head line.
 
 @returns index into `breaks`, or `-1` when the offset precedes the first break
 
 @example
 ```ts
 sliceIndexOf({ breaks, offset: 0, }); // -1
 ```
 */
function sliceIndexOf({
  breaks,
  offset,
}: SliceIndexOfParams,): number {
  let low = 0;
  let high = breaks.length - 1;
  let found = -1;
  while (low <= high) {
    /**
     Middle index probed this round; the shift halves the remaining span.
     */
    const middle = (low + high) >> 1;
    /**
     Break offset at the probed index; the search span never leaves the array.
     */
    const probe = at({
      arr: breaks,
      index: middle,
    },)
      .breakOffset;
    if (probe <= offset) {
      found = middle;
      low = middle + 1;
    }
    else {
      high = middle - 1;
    }
  }
  return found;
}

/**
 Parameters for {@link anchorIndentOf}.
 */
type AnchorIndentOfParams = {
  /**
   Full file source text.
   */
  readonly sourceText: string;
  /**
   Byte offset where the chain region begins.
   */
  readonly regionStart: number;
  /**
   Breaks in source order.
   */
  readonly breaks: readonly ChainBreak[];
  /**
   Rendered whitespace prefix of every continuation line so far, in break order.
   */
  readonly lineIndents: readonly string[];
  /**
   Whitespace prefix of the head line.
   */
  readonly headIndent: string;
  /**
   Byte offset whose line indentation is wanted.
   */
  readonly anchorOffset: number;
};

/**
 Indentation of the line where an anchor offset renders.
 
 The anchor sits on the first rendered line of its slice, whose indentation the
 caller computes, or on a verbatim interior line of a multi-line slice (a
 comment carrying the chain head), whose indentation stays the source's.
 
 @returns whitespace prefix of the anchor's rendered line
 
 @example
 ```ts
 anchorIndentOf({ sourceText, regionStart: 0, breaks, lineIndents: [], headIndent: '', anchorOffset: 0, }); // ''
 ```
 */
function anchorIndentOf({
  sourceText,
  regionStart,
  breaks,
  lineIndents,
  headIndent,
  anchorOffset,
}: AnchorIndentOfParams,): string {
  /**
   Slice index carrying the anchor; `-1` names the head line.
   */
  const sliceIndex = sliceIndexOf({
    breaks,
    offset: anchorOffset,
  },);
  /**
   First byte of the anchor's slice; the region start stands for the head slice.
   */
  const sliceStart = (sliceIndex === (-1))
    ? regionStart
    : at({
      arr: breaks,
      index: sliceIndex,
    },)
      .breakOffset;
  /**
   Whether the anchor renders on its slice's first line rather than a verbatim interior line.
   */
  const onFirstLine = !sourceText
    .slice(
    sliceStart,
    anchorOffset,
    )
    .includes('\n',);
  if (!onFirstLine) {
    return baseIndentAt({
      sourceText,
      offset: anchorOffset,
    },);
  }
  return (sliceIndex === (-1))
    ? headIndent
    : at({
      arr: lineIndents,
      index: sliceIndex,
    },);
}

/**
 Parameters for {@link renderCanonical}.
 */
export type RenderCanonicalParams = {
  /**
   Full file source text.
   */
  readonly sourceText: string;
  /**
   Byte offset where the chain region begins (the head leaf).
   */
  readonly regionStart: number;
  /**
   Byte offset where the chain region ends, past any trailing wrapper.
   */
  readonly regionEnd: number;
  /**
   Breaks in source order, as returned by {@link selectBreaks}.
   */
  readonly breaks: readonly ChainBreak[];
  /**
   Whitespace prefix of the head line; continuation lines nest two columns
   past their anchors' lines.
   */
  readonly headIndent: string;
};

/**
 Renders the chain region in canonical layout by slicing source between the
 break offsets, so comments and exact operand text inside each kept slice
 survive verbatim.
 
 The head line runs from `regionStart` to the first break offset; each
 continuation line runs from its break offset to the next break offset (or
 `regionEnd` for the last) and indents two columns past the line carrying its
 anchor, so an operand chain that broke onto an operator's line nests one
 level deeper. Trailing whitespace on every line is trimmed, so the result
 carries no trailing spaces and no whitespace-only lines, and re-rendering
 already-canonical source reproduces it byte for byte.
 
 @returns canonical multi-line rendering, or the verbatim region when there
   are no breaks
 
 @throws when a break offset has no matching end offset, which is
   unreachable because the end list is built one-to-one from the breaks
 
 @example
 ```ts
 // `a.b.c` with one break at `.c` anchored at `a`: renders `a.b\n  .c`
 renderCanonical({
   sourceText: 'a.b.c',
   regionStart: 0,
   regionEnd: 5,
   breaks: [{ breakOffset: 4, anchorOffset: 0, }],
   headIndent: '',
 });
 ```
 
 @internal
 */
export function renderCanonical({
  sourceText,
  regionStart,
  regionEnd,
  breaks,
  headIndent,
}: RenderCanonicalParams,): string {
  if (breaks.length
    === 0) {
    return sourceText.slice(
      regionStart,
      regionEnd,
    );
  }
  /**
   End offset of each continuation slice: the next break, or the region end for the last.
   */
  const ends: readonly number[] = [
    ...breaks.slice(1,)
      .map(function start(breakItem,): number {
        return breakItem.breakOffset;
      },),
    regionEnd,
  ];
  /**
   First break offset; start of the first continuation line and end of the head slice.
   */
  const firstBreak = breaks[0]
    ?.breakOffset
    ?? regionEnd;
  /**
   Head line: from region start to the first break, trailing whitespace removed.
   */
  const head = sourceText
    .slice(
    regionStart,
    firstBreak,
    )
    .trimEnd();
  /**
   Rendered whitespace prefix of every continuation line, in break order.
   
   Each break's line indents two columns past the line carrying its anchor,
   and anchors always land in the head line or an already-placed continuation
   line, so one forward pass fills the array.
   */
  const lineIndents: string[] = [];
  for (
    let index = 0;
    index < breaks.length;
    index += 1
  ) {
    /**
     Break whose line is being placed; the loop bound is the array length.
     */
    const breakItem = at({
      arr: breaks,
      index,
    },);
    /**
     Indentation of the line carrying this break's anchor.
     */
    const anchorIndent = anchorIndentOf({
      sourceText,
      regionStart,
      breaks,
      lineIndents,
      headIndent,
      anchorOffset: breakItem.anchorOffset,
    },);
    lineIndents.push(`${anchorIndent}  `,);
  }
  /**
   One `\n + indent + content` string per break offset, content sliced verbatim.
   */
  const continuations = breaks.map(function piece(
    breakItem,
    index,
  ): string {
    /**
     Slice end for this break: paired one-to-one with `breaks` by construction.
     */
    const end = at({
      arr: ends,
      index,
    },);
    return `\n${
      at({
        arr: lineIndents,
        index,
      },)
    }${
      sourceText
        .slice(
          breakItem.breakOffset,
          end,
        )
        .trimEnd()
    }`;
  },);
  return `${head}${continuations.join('',)}`;
}
