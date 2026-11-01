import { edgeFreeRangeOf, } from './consistency-edge-marks.ts';
import {
  type EdgeUnit,
  textUnitsOf,
} from './consistency-edge-units.ts';
import {
  foldWithOffsets,
  writtenPointsOf,
} from './consistency-fold-offsets.ts';

//region Consistency whole end
// WHERE A LOCATED SPAN BEGINS AND ENDS ONCE A REFERENCE OR AN ESCAPE IS KEPT WHOLE. A judge copies the words of a place's English
// and stops where the words stop: before the semicolon that ends a character reference (`dogs &amp`), before the full stop a
// backslash escapes (`cats` and the backslash), or begins at the `amp;` of a reference or at the sign an escape escapes. The quote's own
// edges are cut whole (`quoteBounds`, `consistency-quote-bounds.ts`, and the fold of `consistency-edge-marks.ts`), but a quote that
// stops or starts inside a reference the parser decodes or inside a backslash escape names a span whose edge stands between the two
// halves, and a splice at that edge writes a reference or an escape in two (`The cats &amp bowl`, `\dogs’`). The locator therefore
// takes the edge of the unit it falls inside (`consistency-edge-units.ts`) and then keeps or cuts that whole unit at the edge as the
// character it shows would be: an escaped stop, a no-break space or a single mark goes, an ampersand or an escaped asterisk stays.
// The word test of the locator then runs again on these final edges and reads the shown text (`cutsShownWord`,
// `consistency-shown-edge.ts`), since an edge taken to the edge of a reference can stand against a letter the first word test
// read as the digits of a numeric reference.

/**
 The unit an offset falls strictly inside, which a span edge there would cut in two.

 @param units - the units of the text, in order, with offsets in the text

 @param offset - offset of the edge

 @returns The unit as a list of one, none when the offset falls between two units or at the text's own edge

 @example
 ```ts
 const [unit,] = unitAcross({ units: textUnitsOf({ text: 'a\\.', },), offset: 2, },); // { from: 1, to: 3, shown: '.' }
 ```
 */
function unitAcross(
  {
    units,
    offset,
  }: {
    readonly units: readonly EdgeUnit[];
    readonly offset: number;
  },
): readonly EdgeUnit[] {
  return units.filter(function holds({
    from,
    to,
  },): boolean {
    return (from < offset) && (offset < to);
  },)
    .slice(
    0,
    1,
    );
}

/**
 How long the first characters of a text are in UTF-16 units.

 @param points - the characters of a text, one whole character each

 @param index - how many characters to count

 @returns Their length in UTF-16 units

 @example
 ```ts
 const length = lengthBefore({ points: Array.from('𝓬ats',), index: 2, },); // 3
 ```
 */
function lengthBefore(
  {
    points,
    index,
  }: {
    readonly points: readonly string[];
    readonly index: number;
  },
): number {
  return points.slice(
    0,
    index,
  )
    .join('',)
    .length;
}

/**
 Where a located span ends once the reference or the escape its end falls inside is kept whole: the end of that unit, the same end
 when the end falls between two units. A unit counts when the parser decodes the reference or a backslash escapes an ASCII
 punctuation character.

 @param text - the English the span stands in

 @param end - offset where the span ends, which is after its last unit

 @returns The offset where the span ends, the same when no unit holds it inside

 @example
 ```ts
 const kept = wholeEndOf({ text: 'The dogs &amp; bowl', end: 13, },); // 14
 ```
 */
export function wholeEndOf(
  {
    text,
    end,
  }: {
    readonly text: string;
    readonly end: number;
  },
): number {
  /**
   The unit the end falls inside, none when it falls between two.
   */
  const [across,] = unitAcross({
    units: textUnitsOf({ text, },),
    offset: end,
  },);
  return across?.to ?? end;
}

/**
 Where a located span starts once the reference or the escape its start falls inside is kept whole: the start of that unit, the same
 start when the start falls between two units.

 @param text - the English the span stands in

 @param start - offset where the span starts

 @returns The offset where the span starts, the same when no unit holds it inside

 @example
 ```ts
 const kept = wholeStartOf({ text: 'See dogs&amp; cats', start: 9, },); // 8
 ```
 */
export function wholeStartOf(
  {
    text,
    start,
  }: {
    readonly text: string;
    readonly start: number;
  },
): number {
  /**
   The unit the start falls inside, none when it falls between two.
   */
  const [across,] = unitAcross({
    units: textUnitsOf({ text, },),
    offset: start,
  },);
  return across?.from ?? start;
}

/**
 A located span with each edge that fell inside a reference or an escape taken whole and then cut at the edge as the character that
 unit shows would be, by the same edge reading as the quote's own words (`edgeFreeRangeOf`): `cats` and the backslash of `cats\.` name
 `cats`, the `amp; cats` of `dogs&amp; cats` names `&amp; cats`, and the `39; cats` of `dogs&#39; cats` names `cats`, the single
 mark going like any other.

 @param text - the English the span stands in

 @param start - offset where the span starts

 @param end - offset where the span ends, which is after its last unit

 @returns The offsets of the span: the same ones when no edge falls inside a unit

 @example
 ```ts
 const span = wholeSpanOf({ text: 'See cats\\. Then', start: 4, end: 9, },); // { start: 4, end: 8 }
 ```
 */
export function wholeSpanOf(
  {
    text,
    start,
    end,
  }: {
    readonly text: string;
    readonly start: number;
    readonly end: number;
  },
): {
  readonly start: number;
  readonly end: number;
} {
  /**
   Where the span starts and ends once both its edges are whole.
   */
  const whole = {
    start: wholeStartOf({
      text,
      start,
    },),
    end: wholeEndOf({
      text,
      end,
    },),
  };
  if ((whole.start === start) && (whole.end === end)) {
    return {
      start,
      end,
    };
  }
  /**
   The span in the fold, with the characters as the English wrote them.
   */
  const folded = foldWithOffsets({
    text: text.slice(
      whole.start,
      whole.end,
    ),
  },);
  /**
   The folded characters of the span and the characters as the English wrote them.
   */
  const written = writtenPointsOf({ folded, },);
  /**
   Where the words of the whole span stand among its folded characters.
   */
  const {
    from,
    to,
  } = edgeFreeRangeOf(written,);
  if (from >= to)
    return whole;
  /**
   How many folded UTF-16 units come before the first word.
   */
  const unitsBeforeFirst = lengthBefore({
    points: written.points,
    index: from,
  },);
  /**
   How many folded UTF-16 units come up to the end of the last word.
   */
  const unitsUpToLast = lengthBefore({
    points: written.points,
    index: to,
  },);
  /**
   Where the words of the span start in the English, none when the fold has no offset for it.
   */
  const [wordsStart,] = folded
    .starts
    .slice(
    unitsBeforeFirst,
    unitsBeforeFirst + 1,
    );
  /**
   Where the words of the span end in the English, none when the fold has no offset for it.
   */
  const [wordsEnd,] = folded
    .ends
    .slice(
    unitsUpToLast - 1,
    unitsUpToLast,
    );
  return {
    start: whole.start + (wordsStart ?? 0),
    end: whole.start + (wordsEnd ?? (whole.end - whole.start)),
  };
}

//endregion Consistency whole end
