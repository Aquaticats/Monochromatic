import { continuesLatinWordOrCompatibilityForm, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Consistency edge class
// THE CLASS OF THE CHARACTER AT AN EDGE OF A TEXT, as far as a single mark beside the text can tell: split from
// `consistency-loose-marks.ts`, which reads the marks inside the words, so that file stays within its length. A single mark is
// attached to a word exactly when a letter or a digit stands on both of its sides, so the class of an edge decides whether a
// replacement can change that for a mark beside the replaced text (`planConsistencyEdit`, `consistency-splice.ts`).

/**
 The kind of character at one edge of a text, as far as a single mark beside the
 text can tell: `attaches` for a Latin letter with its accent, an ASCII digit or a compatibility character of
 those (the neighbour test of the consistency pass, `continuesLatinWordOrCompatibilityForm`), which is what a
 single mark needs on both sides to be attached; `other` for every other
 character; `none` for an edge with no character, the edge of an empty text.
 */
export type EdgeClass = 'attaches' | 'other' | 'none';

/**
 The class of the character at one edge of a text.

 @param text - the words whose edge is read

 @param side - `start` for the first whole character, `end` for the last

 @returns `attaches` for a Latin letter with its accent or an ASCII digit, `none` for an empty
 text, `other` for any other character

 @example
 ```ts
 const kind = edgeClassOf({ text: 'the **cat**', side: 'end', },); // 'other'
 ```
 */
export function edgeClassOf(
  {
    text,
    side,
  }: {
    readonly text: string;
    readonly side: 'start' | 'end';
  },
): EdgeClass {
  /**
   The whole characters of the text, so a letter beyond the first plane is one character.
   */
  const points = Array.from(text,);
  /**
   The character at the edge asked about, absent for an empty text.
   */
  const [edge,] = (side === 'start') ? points : points.toReversed();
  if (edge === undefined)
    return 'none';
  return continuesLatinWordOrCompatibilityForm({ character: edge, },) ? 'attaches' : 'other';
}

/**
 Whether two texts have a character of one class at the same edge. A single mark
 directly beside a text is attached to a word exactly when a letter or a digit
 stands on both of its sides, and a replacement of the text leaves the far side
 of the mark alone, so replacing a text by another whose edge is of the same
 class cannot change whether the mark is attached, and replacing it by one whose
 edge is of another class can.

 AN EMPTY EDGE IS A CLASS OF ITS OWN. A replacement by an empty text puts the
 character that stood on the far side of the replaced text beside the mark, and
 nothing the located words record says what that character is, so an empty edge
 equals only another empty one (two empty texts are the same text).

 @param left - one text

 @param right - the other

 @param side - `start` to compare the first characters, `end` to compare the last

 @returns Whether the two edges are of one class

 @example
 ```ts
 const same = sameEdgeClass({ left: 'the cats', right: 'the **cat**', side: 'end', },); // false
 ```
 */
export function sameEdgeClass(
  {
    left,
    right,
    side,
  }: {
    readonly left: string;
    readonly right: string;
    readonly side: 'start' | 'end';
  },
): boolean {
  return edgeClassOf({
    text: left,
    side,
  },)
    === edgeClassOf({
      text: right,
      side,
    },);
}

//endregion Consistency edge class
