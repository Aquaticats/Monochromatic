import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Code points
// One counter, shared by everything that compares a Chinese size against an
// English one, and the two readers that take the whole code point at or before
// an offset, shared by every scan that tests a character whose class reaches
// past the first plane.
//
// EXTRACTED RATHER THAN COPIED. The refusal guard in `translate-alignment.ts`
// and the corroboration gate in `coverage-corroboration.ts` both divide one of
// these counts by another, and a copy that drifted would let the two disagree
// about the same page while both looked right. The readers lived in
// `quote-neighbours.ts` (class ninety-six) with a private copy of the forward
// one in `mdx-tag-start.ts`, until the scans of ledger B21 needed them too.

/**
 Highest code point one UTF-16 unit can carry; anything above it is a
 surrogate pair.
 */
const BMP_MAX = 0xFF_FF;

/**
 Whole code point starting at an offset, empty past either end of the text.

 @param text - text being read

 @param at - offset of the code point wanted

 @returns Code point there, as a string of one or two units

 @example
 ```ts
 const after = codePointAt({ text: 'ab', at: 1, },); // 'b'
 ```
 */
export function codePointAt({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number;
},): string {
  if (at >= text.length)
    return '';
  /**
   Code point there, read by the string's own decoding.
   */
  const point = text.codePointAt(at,);
  if (point === undefined)
    return '';
  return String.fromCodePoint(point,);
}

/**
 Whole code point ending just before an offset, empty at the text's start.

 @param text - text being read

 @param at - offset of the character whose predecessor is wanted

 @returns Code point before, as a string of one or two units

 @example
 ```ts
 const before = codePointBefore({ text: 'ab', at: 1, },); // 'a'
 ```
 */
export function codePointBefore({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number;
},): string {
  if (at <= 0)
    return '';
  /**
   Code point starting two units back, which ends just before the offset
   when it is a surrogate pair.
   */
  const paired = (at >= 2) ? text.codePointAt(at - 2,) : undefined;
  if ((paired !== undefined) && (paired > BMP_MAX))
    return text.slice(
      at - 2,
      at,
    );
  return text.charAt(at - 1,);
}

/**
 Counts every code point of a text, surrounding whitespace included: the
 characters a line calls a text's size.

 An index scan stepping one whole code point at a time ({@link codePointAt}),
 rather than spreading or `Array.from`, both of which the linter refuses over
 strings for breaking grapheme clusters. A pair counts once and a lone half
 counts once, as the string's own iteration reads them (ledger B22).

 SPLIT OUT OF {@link codePointCount} on 2026-10-06 for a page's size, where
 the whitespace at either end is part of the page: the page check called its
 UTF-16 length characters.

 @param text - page or line whose size is printed under the word characters,
 whitespace at either end included since it is part of what was written

 @returns Code points in the whole text, the unit a reader means by
 characters, which a character beyond the first plane counts once

 @example
 ```ts
 const count = codePointLength({ text: ' 其一 ', },); // 4
 ```
 */
export function codePointLength({ text, }: { readonly text: string; },): number {
  /**
   Offset reached and code points passed, mutated only inside this function.
   */
  const cursor = {
    at: 0,
    points: 0,
  };
  while (cursor.at < text.length) {
    /**
     Whole code point under the cursor, one unit or a pair.
     */
    const point = codePointAt({
      text,
      at: cursor.at,
    },);
    cursor.at += point.length;
    cursor.points += 1;
  }
  return cursor.points;
}

/**
 Counts code points rather than UTF-16 units, after trimming surrounding
 whitespace.

 `length` counts surrogate halves, so a rare CJK character measures twice on
 one side of a ratio and once on the other. Every comparison this serves runs
 between a Chinese source and an English translation, which is exactly where
 that asymmetry lands, and it lands in the unsafe direction: a doubled source
 size halves a ratio and passes a pairing a guard would otherwise refuse.

 Counted by {@link codePointLength} over the trimmed text, so a pair counts
 once and a lone half counts once (ledger B22): counting every unit that is
 not a second half, as this once did, read a lone second half as nothing.

 @param text - text to measure

 @returns Code points after trimming surrounding whitespace

 @example
 ```ts
 const count = codePointCount({ text: '其一：', },);
 ```
 */
export function codePointCount({ text, }: { readonly text: string; },): number {
  return codePointLength({ text: text.trim(), },);
}

/**
 The opening of a text, at most a number of UTF-16 units long, that never
 ends inside a character: where the limit falls between the two halves of a
 surrogate pair, the first half is left out too (ledger B21). A cut by
 `slice` alone kept half an emoji in an error's excerpt, a stream's opening
 and a refused reply's opening, and the log showed `\ud83d` or a
 replacement character. A lone first half no pair follows is a whole
 character as the string reads it, and stays (ledger B22).

 @param text - text to cut

 @param units - most UTF-16 units the opening may run to

 @returns Whole text where it fits, else its longest opening within the
 limit that ends on a whole character

 @throws {@link Error} when no code point starts at the limit's last unit of a
 text longer than the limit, which no limit above zero and text of that
 length can produce

 @example
 ```ts
 wholeOpening({ text: 'nap\u{1F431}', units: 4, },); // 'nap'
 ```
 */
export function wholeOpening({
  text,
  units,
}: {
  readonly text: string;
  readonly units: number;
},): string {
  if (text.length <= units)
    return text;
  /**
   Code point starting at the limit's last unit, read only for a limit above
   zero, since no unit precedes the limit otherwise.
   */
  const limitPoint = (units > 0) ? text.codePointAt(units - 1,) : 0;
  if (limitPoint === undefined) {
    throw new Error(
      `unreachable: no code point starts at offset ${String(units - 1,)} of a text of length ${
        String(text.length,)
      }, though the text is longer than the limit ${String(units,)} and so holds a unit there`,
    );
  }
  /**
   Whether the code point starting at the limit's last unit runs past the
   limit, which only a surrogate pair split by it does.
   */
  const cutsPair = limitPoint > BMP_MAX;
  return text.slice(
    0,
    cutsPair ? (units - 1) : units,
  );
}

/**
 Orders two texts by code point, the same on every machine: the first code
 point that differs decides, and a text that runs out first comes first.

 NOT `localeCompare`, whose order follows the runtime's locale and collation
 data, so two machines could order one list two ways and draw a different
 benchmark from one corpus (ledger B95). NOT `<` on the strings either, which
 compares UTF-16 units and so puts a character past U+FFFF, whose first unit
 is a surrogate, before one between U+E000 and U+FFFF.

 @param left - text that comes first on a negative answer

 @param right - text that comes first on a positive answer

 @returns Negative, zero or positive, as `toSorted` reads a comparator

 @example
 ```ts
 const ordered = ids.toSorted(function byId(left, right,): number {
   return compareCodePoints({ left, right, },);
 },);
 ```
 */
export function compareCodePoints({
  left,
  right,
}: {
  readonly left: string;
  readonly right: string;
},): number {
  /**
   Offset reached in both texts, which agree on every unit before it.
   */
  const cursor = { at: 0, };
  while ((cursor.at < left.length) && (cursor.at < right.length)) {
    /**
     Code point of the left text at the shared offset.
     */
    const leftPoint = nonNullishOrThrow(left.codePointAt(cursor.at,),);
    /**
     Code point of the right text at the same offset.
     */
    const rightPoint = nonNullishOrThrow(right.codePointAt(cursor.at,),);
    if (leftPoint !== rightPoint)
      return leftPoint - rightPoint;
    cursor.at += (leftPoint > BMP_MAX) ? 2 : 1;
  }
  return left.length - right.length;
}

/**
 A list of text in code-point order, as `compareCodePoints` orders two.

 FOR A LIST OF PLAIN TEXT, where an argument-less `toSorted` would order by
 UTF-16 unit and put a character past U+FFFF before one between U+E000 and
 U+FFFF (ledger B122). A list of records keys its own comparator through
 `compareCodePoints`.

 @param texts - text to order, left as it is, whose own text type the result
 keeps (a file name ending in `.json`, a provider's name)

 @returns New list holding the same text in code-point order

 @example
 ```ts
 const ordered = textsInCodePointOrder({ texts: ['mooncat', 'Tabby',], },); // ['Tabby', 'mooncat']
 ```
 */
export function textsInCodePointOrder<const TextT extends string,>(
  { texts, }: { readonly texts: readonly TextT[]; },
): TextT[] {
  return texts.toSorted(function byCodePoint(
    left,
    right,
  ): number {
    return compareCodePoints({
      left,
      right,
    },);
  },);
}

//endregion Code points
