//region Coverage control decoy
// The cut that must NOT change the answer.
//
// Deleting the spans a roster anchored on and watching it vote absence proves
// the vote is reachable. It does not yet prove the vote tracks THAT PASSAGE: a
// wire that answered `absent` after any deletion at all would produce the same
// flip, and would be worthless for the same reason a wire that never votes
// absence is.
//
// So the same case is also asked with an EQUALLY LARGE cut taken from
// somewhere the roster did not point at. A sound wire keeps saying `carried`,
// because the passage's rendering is still there. Both cuts are made against
// the same document, for the same passage, in the same run, so nothing but the
// position of the cut differs between them.

/**
 Where an unrelated cut of a given size can be taken from.
 */
export type DecoyCut = {
  /**
   Exact document text to delete, blank when the document has no room for a
   cut of this size clear of the anchored spans.
   */
  readonly span: string;

  /**
   Offset it starts at, or `-1` when there is none.

   REPORTED rather than kept private, because a decoy that does change the
   answer needs diagnosing: a cut landing on a title or a frontmatter block is
   structural damage of a different kind, and its offset is what says so.
   */
  readonly at: number;
};

/**
 Nothing to cut, named so the two absent cases read alike.
 */
const NO_CUT: DecoyCut = {
  span: '',
  at: -1,
};

/**
 A stretch of a document one anchored span occupies.
 */
type CoveredRegion = {
  /**
   Offset it starts at.
   */
  readonly from: number;

  /**
   Offset just past its end.
   */
  readonly to: number;
};

/**
 Every region of a document covered by one of the named spans.

 @param text - document searched

 @param spans - spans to locate, every occurrence of each

 @returns End offset of each occurrence, keyed by where it starts

 @example
 ```ts
 const covered = coveredRegions({ text, spans, },);
 ```
 */
function coveredRegions(
  {
    text,
    spans,
  }: {
    readonly text: string;
    readonly spans: readonly string[];
  },
): readonly CoveredRegion[] {
  return spans
    .filter(function isLocatable(span,): boolean {
      return span !== '';
    },)
    .flatMap(function occurrencesOf(span,) {
      /**
       Every place this span appears.
       */
      const found: CoveredRegion[] = [];

      /**
       Cursor walking the document, as a plain index rather than a recursion
       over the remaining text.
       */
      let at = text.indexOf(span,);

      while (at !== (-1)) {
        found.push({
          from: at,
          to: at + span.length,
        },);
        at = text.indexOf(
          span,
          at + 1,
        );
      }

      return found;
    },);
}

/**
 Finds a cut of the requested size that misses every anchored span.

 TAKEN AS LATE IN THE DOCUMENT AS IT FITS, because the front of a page carries
 its frontmatter and title, and deleting those is structural damage rather
 than the ordinary body deletion this control is trying to make.

 @param text - document to cut from

 @param avoid - spans the roster anchored on, which the cut must not touch

 @param chars - size of the cut, matched to the cut it is being compared with

 @returns Cut to make, or a blank one when the document has no room for it

 @example
 ```ts
 const decoy = decoyCut({ text, avoid, chars, },);
 ```
 */
export function decoyCut(
  {
    text,
    avoid,
    chars,
  }: {
    readonly text: string;
    readonly avoid: readonly string[];
    readonly chars: number;
  },
): DecoyCut {
  if (chars <= 0)
    return NO_CUT;

  if (text.length < chars)
    return NO_CUT;

  /**
   Regions the cut may not overlap.
   */
  const covered = coveredRegions({
    text,
    spans: avoid,
  },);

  /**
   Latest offset a cut of this size can start at.
   */
  const last = text.length - chars;

  // Walks backwards from the last position that fits. The cursor lives in the
  // loop head rather than at the function root, so nothing after the scan can
  // see where it stopped.
  for (let cursor = last; cursor >= 0;) {
    /**
     Start of the window under test, bound once per turn so the `covered.find`
     search cannot see the cursor move under it.
     */
    const from = cursor;

    /**
     Anchored region this window runs into, if any.
     */
    const hit = covered.find(function overlaps(region,): boolean {
      return (region.from < (from + chars)) && (region.to > from);
    },);

    if (hit === undefined)
      return {
        span: text.slice(
          from,
          from + chars,
        ),
        at: from,
      };

    // Jump to the last window that ends before this region starts, rather than
    // stepping back one character at a time: every position in between overlaps
    // the same region and would be rejected for the same reason.
    cursor = hit.from - chars;
  }

  return NO_CUT;
}

//endregion Coverage control decoy
