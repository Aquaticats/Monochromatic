import type { Root, } from 'mdast';

import { referenceEndOf, } from './consistency-unread-units.ts';
import { parseMdxBody, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Consistency edge units
// THE UNITS A SPAN EDGE IS CUT BY. A quote's words, and the span they name in an English, begin and end at the edge of a
// unit, never inside one. A unit is one character, a BACKSLASH ESCAPE (a backslash and the one ASCII punctuation character it
// escapes, `\.`, `\)`, `\*`, `\\`) or a CHARACTER REFERENCE the Markdown parser decodes (`&amp;`, `&#39;`, `&nbsp;`), and a
// unit shows a reader one character: the escaped character, the decoded one. At an edge a unit is kept or cut exactly as the
// character it shows would be: `\.` goes like `.`, `&nbsp;` like a space, `&#39;` like `'`, and `\*` and `&amp;` stay like `*` and
// `&`. This file is the one place the units are read, so the three readers of an edge agree: the bounds of a quote
// (`consistency-quote-bounds.ts`), the fold of the edge marks (`consistency-edge-marks.ts`) and the span located in an English
// (`consistency-whole-end.ts`).
//
// WHAT IS A REFERENCE HERE IS WHAT THE PARSER DECODES, nothing wider. `Q&A;`, `R&D;`, `a&b;`, `a&zzzz;` and a numeric
// reference past the parser's digits (eleven digits) stay plain text in the parse, so their semicolon is a plain
// semicolon at an edge. The shape reader of the refusal run (`referenceEndOf`, `consistency-unread-units.ts`) reads a name no
// table holds as a reference too, which is the safe direction for stepping over a unit and the unsafe one for cutting; it only
// proposes the candidate here, and the parser itself says whether it decodes and to what. The package holds no table of the
// parser's names, and the parser's own is not reachable from here, so the answer is read from the parse of the reference alone.
//
// AN ESCAPE IS READ FROM THE CHARACTERS THE JUDGE WROTE. The fold turns a curly mark straight, so `\’` reaches the fold as `\'`,
// which would look like an escape; the parser does not escape a curly mark, so the caller hands over the written characters
// beside the folded ones, and a backslash escapes only what is ASCII punctuation as written.

/**
 The ASCII punctuation characters, which are the only characters a backslash escapes.
 */
export const ASCII_PUNCTUATION: ReadonlySet<string> = new Set(Array.from('!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~',),);

/**
 One unit of a text: where it stands among the characters and the character a reader sees for it.

 @example
 ```ts
 const unit: EdgeUnit = { from: 4, to: 6, shown: '.', };
 ```
 */
export type EdgeUnit = {
  /**
   Index of the first character of the unit.
   */
  readonly from: number;

  /**
   Exclusive index of the end of the unit.
   */
  readonly to: number;

  /**
   What a reader sees for the unit: the escaped character, the character a reference decodes to, the character itself.
   */
  readonly shown: string;
};

/**
 How many characters a reference may span from its ampersand to its semicolon, so that the search for its end is bounded.
 */
const REFERENCE_SPAN = 35;

/**
 How many answers the memory of decoded references holds before it starts again, so a text of many different references cannot
 grow it without bound.
 */
const REFERENCE_MEMORY = 4_096;

/**
 What the parser decoded each reference to, by the reference as written: a list of the one text shown, none when the parser
 leaves the text as it is.
 */
const decodedReferences = new Map<string, readonly string[]>();

/**
 What a reader sees for the text of one reference, by what the Markdown parser makes of it alone.

 @param tree - the parse of the reference

 @returns The text of the one paragraph with the one text node the parse holds, none when the parse holds anything else
 */
function loneTextOf({ tree, }: { readonly tree: ForeignBorrowed<Root>; },): readonly string[] {
  /**
   The one paragraph the parse holds.
   */
  const [paragraph, ...others] = tree.children;
  if ((paragraph?.type !== 'paragraph') || (others.length > 0))
    return [];
  /**
   The one text node the paragraph holds.
   */
  const [text, ...rest] = paragraph.children;
  if ((text?.type !== 'text') || (rest.length > 0))
    return [];
  return [text.value,];
}

/**
 The text a character reference shows, as the Markdown parser decodes it.

 @param reference - the reference as written, from its ampersand to its semicolon

 @returns The shown text as a list of one, none when the parser leaves the text as it is, so it is no reference

 @example
 ```ts
 const [shown,] = referenceShownAs({ reference: '&amp;', },); // '&'
 ```
 */
function referenceShownAs({ reference, }: { readonly reference: string; },): readonly string[] {
  /**
   The answer given for this reference before.
   */
  const remembered = decodedReferences.get(reference,);
  if (remembered !== undefined)
    return remembered;
  /**
   What the parse of the reference alone shows.
   */
  const [text,] = loneTextOf({ tree: parseMdxBody({ body: reference, },), });
  /**
   The answer: the decoded text when it differs from what was written.
   */
  const answer = ((text === undefined) || (text === reference)) ? [] : [text,];
  if (decodedReferences.size >= REFERENCE_MEMORY)
    decodedReferences.clear();
  decodedReferences.set(
    reference,
    answer,
  );
  return answer;
}

/**
 The unit that starts at an ampersand, when it starts a reference the parser decodes.

 @param points - the characters of a text, one whole character each

 @param at - index of the ampersand

 @returns The unit as a list of one, none when the ampersand starts no such reference
 */
function referenceUnitAt(
  {
    points,
    at,
  }: {
    readonly points: readonly string[];
    readonly at: number;
  },
): readonly EdgeUnit[] {
  /**
   The characters a reference could span from the ampersand.
   */
  const english = points.slice(
    at,
    at + REFERENCE_SPAN,
  )
    .join('',);
  /**
   Where the reference ends by its shape, none when the characters after the ampersand have the shape of none.
   */
  const [end,] = referenceEndOf({
    english,
    at: 0,
  },);
  if (end === undefined)
    return [];
  /**
   The text the parser decodes the reference to.
   */
  const [shown,] = referenceShownAs({
    reference: english.slice(
      0,
      end + 1,
    ),
  },);
  if (shown === undefined)
    return [];
  return [
    {
      from: at,
      to: at + (end + 1),
      shown,
    },
  ];
}

/**
 The unit that starts at a character.

 @param points - the characters of a text, one whole character each, possibly folded

 @param originals - the characters as they were written, one for each of the points

 @param at - index of the character the unit starts at

 @returns The unit: a backslash with the ASCII punctuation it escapes, a reference the parser decodes, or the one character

 @example
 ```ts
 const unit = edgeUnitAt({ points: Array.from('a\\.',), originals: Array.from('a\\.',), at: 1, },); // { from: 1, to: 3, shown: '.' }
 ```
 */
function edgeUnitAt(
  {
    points,
    originals,
    at,
  }: {
    readonly points: readonly string[];
    readonly originals: readonly string[];
    readonly at: number;
  },
): EdgeUnit {
  /**
   The character at the index.
   */
  const point = points[at] ?? '';
  if (point === '\\') {
    /**
     The character the backslash escapes as it was written, none when it escapes nothing.
     */
    const escaped = originals[at + 1] ?? '';
    if (ASCII_PUNCTUATION.has(escaped,)) {
      return {
        from: at,
        to: at + 2,
        shown: points[at + 1] ?? escaped,
      };
    }
  }
  /**
   The reference that starts at an ampersand, none when none does.
   */
  const [reference,] = (point === '&')
    ? referenceUnitAt({
      points,
      at,
    },)
    : [];
  return reference ?? {
    from: at,
    to: at + 1,
    shown: point,
  };
}

/**
 The units of a text, read from its first character on, so an escaped backslash leaves the character after it alone and an
 escaped ampersand starts no reference.

 @param points - the characters of a text, one whole character each, possibly folded

 @param originals - the characters as they were written, one for each of the points, the points themselves when not folded

 @returns The units in order, each with the characters it covers and the character a reader sees for it

 @example
 ```ts
 const units = edgeUnitsOf({ points: Array.from('cats\\.',), },); // c, a, t, s, then { from: 4, to: 6, shown: '.' }
 ```
 */
export function edgeUnitsOf(
  {
    points,
    originals = points,
  }: {
    readonly points: readonly string[];
    readonly originals?: readonly string[];
  },
): readonly EdgeUnit[] {
  /**
   The units found so far, mutated only inside this function.
   */
  const units: EdgeUnit[] = [];
  /**
   Index of the next character to read, mutated only inside this function.
   */
  const cursor = { at: 0, };
  while (cursor.at < points.length) {
    /**
     The unit that starts at the cursor.
     */
    const unit = edgeUnitAt({
      points,
      originals,
      at: cursor.at,
    },);
    units.push(unit,);
    cursor.at = unit.to;
  }
  return units;
}

/**
 The units of a text with their offsets in the text, in UTF-16 units.

 @param text - the text as written

 @returns The units in order, with offsets in the text instead of indexes among its characters

 @example
 ```ts
 const units = textUnitsOf({ text: '𝓬ats\\.', },); // the last has from 5, to 7, shown '.'
 ```
 */
export function textUnitsOf({ text, }: { readonly text: string; },): readonly EdgeUnit[] {
  /**
   The characters of the text, one whole character each.
   */
  const points = Array.from(text,);
  /**
   The offset in the text of each character, and the offset after the last.
   */
  const offsets: number[] = [0,];
  for (const point of points)
    offsets.push((offsets.at(-1,) ?? 0) + point.length,);
  return edgeUnitsOf({ points, },)
    .map(function inOffsets({
      from,
      to,
      shown,
    },): EdgeUnit {
      return {
        from: offsets[from] ?? text.length,
        to: offsets[to] ?? text.length,
        shown,
      };
    },);
}

//endregion Consistency edge units
