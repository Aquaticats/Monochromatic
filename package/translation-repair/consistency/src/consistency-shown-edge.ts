import {
  type EdgeUnit,
  textUnitsOf,
} from './consistency-edge-units.ts';
import type { CodePadding, } from './consistency-markup-ranges.ts';
import { unitKindsOf, } from './consistency-shown-kinds.ts';
import { isSpacedScriptWordCharacter, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isDefaultIgnorable, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency shown edge
// WHETHER A LOCATED SPAN CUTS A WORD A READER SEES (n156, W2). The word test of the locator (`apartStartsOf`,
// `consistency-fold.ts`) reads the English folded and character by character, which is right for finding a stand but reads a
// reference or an escape as the characters it is written with: the `105` of `k&#105;ttens` is no letter to it, so `ttens` stands
// apart from `k` there. A quote that began inside a reference is widened to the whole reference after the stand is found
// (`consistency-whole-end.ts`), and that widened edge is the one a splice cuts at. This test therefore runs on the FINAL edges
// and reads what a reader sees: each unit (`consistency-edge-units.ts`) is its shown character, a default-ignorable character
// (a soft hyphen, a zero-width space, `renders-as-nothing.ts`) is no character at all and is stepped over, a unit the prose view
// does not read as words (the delimiters of emphasis, strike and code, a link's brackets with its destination and title, a tag, a
// comment, an expression; `transparentUnitsOf`, `consistency-unread-units.ts`) shows nothing between two pieces of one word and is
// stepped over too (`kit*tens*`, `**kit**tens`, `[kit](u)tens`; n159), a word character is a letter, a combining mark or a
// decimal digit of a script that sets its words apart by spaces (Han, Hiragana and Katakana excepted), and an ampersand or an
// underscore between two word characters joins them as a hyphen or an apostrophe does (`AT&T`, `R&D`, `snake_case`). An edge cuts a word when the
// character beside it outside the span continues a word (it is one, or it is a joiner with a word character beyond it) and the
// span's edge is a word edge: the first or last character of the words the judge quoted is a
// word character of a script that sets its words apart by spaces, or the shown characters inside the span continue a word the same
// way. A unit is hidden whatever its width
// (a reference, an escape or an astral character inside a link's destination, a tag's attribute or a comment is hidden whole), the
// content of inline code is shown as the code word it is, and markup that separates or annotates (a line break with or without a
// slash, a block tag, a table cell's pipes and padding, an image, the `<rt>` and `<rp>` of a ruby annotation) is a word boundary,
// read as a space (`unitKindsOf`, `consistency-shown-kinds.ts`; n160). A span that starts or ends at a mark is therefore no cut
// beside a word, as the locator already reads it. What the locator counts is unchanged (`standsOf`, `consistency-fold.ts`):
// this test only refuses to name a span whose edge cuts a word, so a place it refuses is one the splice leaves alone.

/**
 The shown characters that join the two word characters on either side of them into one word, as a hyphen does.
 */
const JOINERS: ReadonlySet<string> = new Set([
  '&',
  '-',
  '_',
  '\'',
  '’',
],);

/**
 How many shown characters beside an edge the test reads: the nearest and the one beyond it.
 */
const SHOWN_READ = 2;

/**
 Whether a shown character is a letter or a digit of a word.

 @param character - one shown character, empty for the end of the text

 @returns Whether it is a letter, a combining mark or a decimal digit of a script that sets its words apart by spaces (Latin,
 Cyrillic, Greek, Hebrew, Arabic, Hangul and the rest), so a word runs on through it

 @example
 ```ts
 const word = isWordCharacter({ character: 'k', },); // true
 ```
 */
function isWordCharacter({ character, }: { readonly character: string; },): boolean {
  return isSpacedScriptWordCharacter({ character, },);
}

/**
 The shown characters of the units in the order given, the default-ignorable ones stepped over, up to the number the test reads.

 @param units - the units from the edge outward, nearest first

 @returns The shown characters, nearest first, stopping at the unit that brings the count to the number the test reads

 @example
 ```ts
 const shown = nearestShown({ units: textUnitsOf({ text: '&#105;t', },), },); // ['i', 't']
 ```
 */
function nearestShown({ units, }: { readonly units: readonly EdgeUnit[]; },): readonly string[] {
  /**
   Characters met so far.
   */
  const met: string[] = [];
  for (const unit of units) {
    for (const character of Array.from(unit.shown,)) {
      if (!isDefaultIgnorable({ character, },))
        met.push(character,);
    }
    if (met.length >= SHOWN_READ)
      break;
  }
  return met;
}

/**
 Whether the shown characters beside an edge continue a word: the nearest is a word character, or it is a joiner with a word
 character beyond it.

 @param shown - the shown characters from the edge outward, nearest first

 @returns Whether a reader sees a word run on from the edge in that direction

 @example
 ```ts
 const runs = continuesWord({ shown: ['&', 'T',], },); // true
 ```
 */
function continuesWord({ shown, }: { readonly shown: readonly string[]; },): boolean {
  /**
   The shown character nearest the edge.
   */
  const [nearest, beyond,] = shown;
  if (nearest === undefined)
    return false;
  if (isWordCharacter({ character: nearest, },))
    return true;
  if (!JOINERS.has(nearest,))
    return false;
  return isWordCharacter({ character: beyond ?? '', },);
}

/**
 Whether one edge of a span cuts a word.

 @param outward - the units outside the span from the edge outward, nearest first

 @param inward - the units inside the span from the edge inward, nearest first

 @param quoted - the first or last character of the words the judge quoted, empty for empty words

 @returns Whether the outside continues a word and the span's edge is a word edge

 @example
 ```ts
 const cuts = edgeCuts({ outward: textUnitsOf({ text: 'k&#105;', },).toReversed(), inward: [], quoted: 't', },); // true
 ```
 */
function edgeCuts(
  {
    outward,
    inward,
    quoted,
  }: {
    readonly outward: readonly EdgeUnit[];
    readonly inward: readonly EdgeUnit[];
    readonly quoted: string;
  },
): boolean {
  if (!continuesWord({ shown: nearestShown({ units: outward, },), },))
    return false;
  return isWordCharacter({ character: quoted, },) || continuesWord({ shown: nearestShown({ units: inward, },), },);
}

/**
 Whether a span of an English cuts a word a reader sees at either edge, reading each reference and escape as the character it
 shows and a default-ignorable character as nothing; run on the edges the span has once a quote that began inside a unit was
 widened to the whole unit.

 @param text - the English as written

 @param start - offset where the span starts

 @param end - exclusive offset where the span ends

 @param needle - the words the judge quoted, whose first and last characters say whether an edge is a word edge

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for an English read alone

 @param codePadding - what pads the content of inline code: a space and a line ending as CommonMark reads it, or a literal space alone
 as an artifact written before line endings padded code was read; a space and a line ending where absent

 @returns Whether the start or the end of the span stands inside a shown word

 @example
 ```ts
 const cuts = cutsShownWord({ text: 'See k&#105;ttens now', start: 11, end: 16, needle: 'ttens', },); // true
 ```
 */
export function cutsShownWord(
  {
    text,
    start,
    end,
    needle,
    definitions = '',
    codePadding = 'spaces-and-line-endings',
  }: {
    readonly text: string;
    readonly start: number;
    readonly end: number;
    readonly needle: string;
    readonly definitions?: string;
    readonly codePadding?: CodePadding;
  },
): boolean {
  /**
   What each unit of the English is to a reader: hidden (the delimiters of emphasis, strike and code, a link's brackets with its
   destination and title, a tag that wraps text, a comment, an expression), a boundary (a break, a block tag, a table cell's
   pipes and padding, an image, a ruby annotation) or shown.
   */
  const kinds = unitKindsOf({
    english: text,
    definitions,
    codePadding,
  },);
  /**
   The units of the English that show a reader something, in order. A unit is hidden or not by the kind of its first character,
   whatever its width, so a reference, an escape or an astral character inside a hidden stretch is hidden whole. A boundary unit
   is read as the space it stands for.
   */
  const units = textUnitsOf({ text, },)
    .flatMap(function shownUnit(unit,): readonly EdgeUnit[] {
      /**
       What the unit's first character is to a reader.
       */
      const kind = kinds[unit.from];
      if (kind === 'hidden')
        return [];
      return (kind === 'boundary')
        ? [
          {
            from: unit.from,
            to: unit.to,
            shown: ' ',
          },
        ]
        : [unit,];
    },);
  /**
   The units that end at or before the start, nearest the start first.
   */
  const beforeStart = units.filter(function endsBefore({ to, },): boolean {
    return to <= start;
  },)
    .toReversed();
  /**
   The units from the start on, up to the end of the span.
   */
  const insideSpan = units.filter(function standsInside({
    from,
    to,
  },): boolean {
    return (from >= start) && (to <= end);
  },);
  /**
   The units that start at or after the end.
   */
  const afterEnd = units.filter(function startsAfter({ from, },): boolean {
    return from >= end;
  },);
  /**
   The characters of the quoted words, one whole character each.
   */
  const quotedCharacters = Array.from(needle,);
  if (edgeCuts({
    outward: beforeStart,
    inward: insideSpan,
    quoted: quotedCharacters.at(0,) ?? '',
  },))
    return true;
  return edgeCuts({
    outward: afterEnd,
    inward: insideSpan.toReversed(),
    quoted: quotedCharacters.at(-1,) ?? '',
  },);
}

//endregion Consistency shown edge
