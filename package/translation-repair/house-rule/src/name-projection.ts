import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  codePointAt,
  codePointBefore,
  continuesLatinWord,
  isAsciiDigit,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  isCapitalLetter,
  isSmallLetter,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Name projection
// A NAME AS THE LETTERS AND DIGITS IT IS WRITTEN WITH, and where each sits in
// the text, for the declared-name survival guard (`declared-name-survival.ts`)
// and the link-name floor (`translate-declared-link-name.ts`).
//
// WHY A PROJECTION. A handle is written several ways across one archive: with
// `\_` where the declaration writes `_`, with a space where the other side
// writes an underscore or nothing, split over a wrapped line behind a `> `
// prefix. Reduced to its letters and digits, lowercased, each of those is the
// same key, which a raw substring comparison cannot see through (the guard's
// header has the measurement).
//
// WHY THE OFFSETS (ledger B23). The projection drops the spaces and marks
// between words along with those inside a handle, so containment alone finds
// a key across a word edge: `ann` inside `cannot`. A found key counts only
// where each of its Latin edges meets a word edge in the text, read in the
// text itself, which is why every projected unit keeps where its character
// sits.

/**
 Whether one character belongs to a name rather than to the punctuation,
 spacing or markup written around it.

 @param character - one whole character

 @returns Whether it is a letter or a digit in any script

 @example
 ```ts
 const kept = isNameCharacter({ character: '猫', },);
 ```
 */
function isNameCharacter({ character, }: { readonly character: string; },): boolean {
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- Unicode letter and number classes have no string-API equivalent and the corpus writes Han, Latin and digits inside one handle; input is ONE character and the pattern carries no quantifier or alternation, so it cannot backtrack.
  return /[\p{L}\p{N}]/u.test(character,);
}

/**
 A text reduced to the characters a name is made of, with where each came
 from.

 THE OFFSETS INDEX THE TEXT AS COMPOSED (NFC), case kept, not the text as
 given: composing can shorten it, and a lowered character can be longer than
 the one it lowers, so neither the given text nor the lowered one lines up
 with the key. The composed text keeps case, which the edges read.

 @example
 ```ts
 const projection: NameProjection = projectName({ text: 'Mittens naps.', },);
 ```
 */
export type NameProjection = {
  /**
   Letters and digits only, lowercased, in order.
   */
  readonly key: string;

  /**
   Text composed, case kept, which the offsets index.
   */
  readonly composed: string;

  /**
   Per UTF-16 unit of the key, where its character starts in the composed
   text.
   */
  readonly starts: readonly number[];

  /**
   Per UTF-16 unit of the key, where its character ends in the composed text.
   */
  readonly ends: readonly number[];
};

/**
 A text projected onto the characters a name is made of.

 COMPOSED BEFORE ANYTHING ELSE, because a combining mark is neither a letter
 nor a digit and would be dropped where a precomposed one is kept: `Mikä`
 written the two ways would otherwise yield two keys, and the guard would
 report a name lost that is sitting right there.

 SCANNED BY CODE POINT, not by grapheme and not by UTF-16 unit. A letter
 beyond the first plane (a Han ideograph from Extension B, a letter in
 mathematical script) is two units, and neither half is a letter, so a scan
 by unit dropped it from the key (ledger B22). An emoji is neither a letter
 nor a digit, so it drops out of the key, from both sides alike.

 LOWERED ONE CHARACTER AT A TIME, so each kept character knows the character
 it came from; a key and a text are both projected here, so both lower alike.

 @param text - any text

 @returns Key with the offsets of each unit's character

 @example
 ```ts
 const projection = projectName({ text: 'Mittens\_the\_Cat', },); // key 'mittensthecat'
 ```
 */
export function projectName({ text, }: { readonly text: string; },): NameProjection {
  /**
   Text composed, case kept.
   */
  const composed = text.normalize('NFC',);

  /**
   Characters kept, in order.
   */
  const kept: string[] = [];

  /**
   Start offset per kept unit.
   */
  const starts: number[] = [];

  /**
   End offset per kept unit.
   */
  const ends: number[] = [];

  for (let offset = 0; offset < composed.length; offset += codePointAt({
    text: composed,
    at: offset,
  },)
    .length) {
    /**
     Character read here, whole.
     */
    const character = codePointAt({
      text: composed,
      at: offset,
    },);

    /**
     Where it ends.
     */
    const end = offset + character.length;
    for (const lowered of character.toLowerCase()) {
      if (!isNameCharacter({ character: lowered, },))
        continue;
      kept.push(lowered,);
      // One entry per UTF-16 unit of the kept character, and a code point is
      // one unit or two.
      starts.push(offset,);
      ends.push(end,);
      if (lowered.length > 1) {
        starts.push(offset,);
        ends.push(end,);
      }
    }
  }
  return {
    key: kept.join('',),
    composed,
    starts,
    ends,
  };
}

/**
 Text reduced to the characters a name is made of, lowercased.

 @param text - any text

 @returns Letters and digits only, in order

 @example
 ```ts
 const key = nameProjection({ text: 'Mittens\_the\_Cat', },);
 ```
 */
export function nameProjection({ text, }: { readonly text: string; },): string {
  /**
   Projection whose key this is.
   */
  const projection = projectName({ text, },);
  return projection.key;
}

/**
 Whether two adjacent characters belong to one word, so a name can neither
 begin nor end between them.

 A LATIN WORD RUNS ON through letters, digits and combining marks, EXCEPT
 where a letter meets a digit or a small letter meets a capital: that is how
 a handle joins a name to what follows it (`Mittens2024`, `MittensBlossom`),
 measured as the glue on every handle the stored artifacts carry a declared
 name inside (ledger B23). A small letter running on into another small
 letter is one word, which is how `ann` sits inside `cannot`. A Han
 character, and anything that is no Latin word character, joins nothing.

 @param left - character before the edge, empty at a text's start

 @param right - character after it, empty at a text's end

 @returns Whether the edge falls inside one word

 @example
 ```ts
 joinsWord({ left: 'n', right: 'n', },); // true
 joinsWord({ left: 's', right: '2', },); // false
 ```
 */
function joinsWord(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): boolean {
  if ((!continuesLatinWord({ character: left, },)) || (!continuesLatinWord({ character: right, },)))
    return false;
  if (isAsciiDigit({ character: left, },) !== isAsciiDigit({ character: right, },))
    return false;
  return !(isSmallLetter({ character: left, },) && isCapitalLetter({ character: right, },));
}

/**
 Where one occurrence of a name sits in the composed text.

 @example
 ```ts
 const place: NameOccurrence = { start: 4, end: 11, };
 ```
 */
export type NameOccurrence = {
  /**
   Offset of the occurrence's first character in the composed text.
   */
  readonly start: number;

  /**
   Offset just past the occurrence's last character in the composed text.
   */
  readonly end: number;
};

/**
 Every occurrence of a key in a projected text that falls between word edges.

 @param projection - text projected by {@link projectName}

 @param key - name projected the same way

 @returns Occurrences in text order, each opening and closing at a word edge

 @example
 ```ts
 const places = nameOccurrences({ projection: projectName({ text: 'Ann naps. Ann wakes.', },), key: 'ann', },);
 ```
 */
export function nameOccurrences(
  {
    projection,
    key,
  }: {
    readonly projection: NameProjection;
    readonly key: string;
  },
): readonly NameOccurrence[] {
  if (key === '')
    return [];

  /**
   Text the offsets index.
   */
  const text = projection.composed;

  /**
   Projected text the key is looked for in.
   */
  const projected = projection.key;

  /**
   Units the key spans, so a match's last unit is its start plus this less
   one.
   */
  const keyUnits = key.length;

  /**
   Occurrences found, in text order.
   */
  const found: NameOccurrence[] = [];
  for (let at = projected.indexOf(key,); at !== (-1); at = projected.indexOf(
    key,
    at + 1,
  )) {
    /**
     Where the match's first character starts in the composed text; every
     unit of the key has an entry, so a missing one is a broken projection.
     */
    const start = nonNullishOrThrow(projection.starts[at],);

    /**
     Where its last character ends there.
     */
    const end = nonNullishOrThrow(projection.ends[(at + keyUnits) - 1],);

    /**
     Whether the match opens at a word edge.
     */
    const opensApart = !joinsWord({
      left: codePointBefore({
        text,
        at: start,
      },),
      right: codePointAt({
        text,
        at: start,
      },),
    },);

    /**
     Whether it closes at one.
     */
    const closesApart = !joinsWord({
      left: codePointBefore({
        text,
        at: end,
      },),
      right: codePointAt({
        text,
        at: end,
      },),
    },);
    if (opensApart && closesApart) {
      found.push({
        start,
        end,
      },);
    }
  }
  return found;
}

/**
 Whether a projected text carries a key as a name: found in the key, with
 each end of the match at a word edge in the composed text.

 @param projection - text projected by {@link projectName}

 @param key - name projected the same way

 @returns Whether some occurrence stands apart

 @example
 ```ts
 carriesName({ projection: projectName({ text: 'The cat cannot nap.', },), key: 'ann', },); // false
 ```
 */
export function carriesName(
  {
    projection,
    key,
  }: {
    readonly projection: NameProjection;
    readonly key: string;
  },
): boolean {
  return nameOccurrences({
    projection,
    key,
  },)
    .length
    > 0;
}

//endregion Name projection
