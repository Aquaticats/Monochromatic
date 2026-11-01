import {
  isAsciiDigit,
  isAsciiLetter,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { CodePadding, } from './consistency-markup-ranges.ts';
import { rememberedViewOf, } from './consistency-prose-text.ts';
import { commentRegionsOf, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Consistency unread units
// THE UNITS OF AN ENGLISH NO READER READS AS WORDS, by offset. The prose view (`proseViewOf`, `consistency-prose-text.ts`) blanks
// every stretch the parse does not call text: a tag, the label of a footnote reference or of a reference link, a link's
// destination with its title, the delimiters of emphasis and of code, a component's markup. Two kinds of unit stay in the prose
// as the units of a text node although no reader reads them as words: the units of an HTML comment written inside a line, and
// the units of a character reference (`&nbsp;`, `&#160;`, `&#xA0;`). This file adds those two to what the prose blanks, so the
// refusal run beside a span (`markPastRunOf`, `consistency-closing-marks.ts`) steps over everything a reader skips. White space
// is never one of them: a space shows a reader nothing whether or not a parse blanked it, and the refusal run asks for white
// space on its own.

/**
 The units of an English that are white space: a space, a tab and the line breaks.
 */
const WHITE_SPACE_UNITS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
],);

/**
 The hexadecimal digits.
 */
const HEXADECIMAL_DIGITS: ReadonlySet<string> = new Set(Array.from('0123456789abcdefABCDEF',),);

/**
 How many units a character reference may hold after its `&`, so that the search for its `;` is bounded.
 */
const REFERENCE_LONGEST = 33;

/**
 Where a character reference that starts at an `&` ends: `&name;`, `&#12;` or `&#x1F;`, by the shape of its characters and not
 by a table of names, so a name no table holds is read as a reference too.

 @param english - the English

 @param at - offset of the `&`

 @returns The offset of the `;` that ends the reference as a list of one, none when the units after the `&` are no reference

 @example
 ```ts
 const [end,] = referenceEndOf({ english: 'a&nbsp;b', at: 1, },); // 6
 ```
 */
export function referenceEndOf(
  {
    english,
    at,
  }: {
    readonly english: string;
    readonly at: number;
  },
): readonly number[] {
  /**
   The unit after the `&`, which says what kind of reference may follow.
   */
  const first = english.charAt(at + 1,);
  /**
   Whether the reference is a numeric one.
   */
  const isNumeric = first === '#';
  /**
   Whether the number is written in hexadecimal.
   */
  const isHexadecimal = isNumeric && ((english.charAt(at + 2,) === 'x') || (english.charAt(at + 2,) === 'X'));
  /**
   Where the units after the `&` start, which a `#` may open.
   */
  const afterAmpersand = at + 1;
  /**
   Where the units after the `#` start when there is one, which an `x` may open.
   */
  const afterHash = isNumeric ? (afterAmpersand + 1) : afterAmpersand;
  /**
   Where the name or the number starts.
   */
  const bodyStart = isHexadecimal ? (afterHash + 1) : afterHash;
  if ((!isNumeric) && (!isAsciiLetter({ character: first, },)))
    return [];
  for (
    let index = bodyStart;
    (index < english.length) && ((index - at) <= REFERENCE_LONGEST);
    index += 1
  ) {
    /**
     The unit under the cursor.
     */
    const unit = english.charAt(index,);
    if (unit === ';')
      return (index > bodyStart) ? [index,] : [];
    /**
     Whether the unit belongs in the body of this kind of reference.
     */
    const belongs = isHexadecimal
      ? HEXADECIMAL_DIGITS.has(unit,)
      : (isNumeric
        ? isAsciiDigit({ character: unit, },)
        : (isAsciiLetter({ character: unit, },) || isAsciiDigit({ character: unit, },)));
    if (!belongs)
      return [];
  }
  return [];
}

/**
 Whether a backslash escapes the unit at an offset: an odd number of backslashes stand directly before it, the last of them
 escaping it, while an even number are pairs that each escape the next. Only the count is asked, so the unit at the offset is
 read as an ASCII punctuation character; a backslash before any other character is no escape, which the caller knows.

 @param text - the text

 @param at - offset of the unit asked about

 @returns Whether an odd number of backslashes stand directly before the offset

 @example
 ```ts
 const escaped = isEscapedAt({ text: 'a\\&b', at: 2, },); // true
 ```
 */
export function isEscapedAt(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   Where the run of backslashes before the offset starts, mutated only inside this function.
   */
  const run = { start: at, };
  while ((run.start > 0) && (text.charAt(run.start - 1,) === '\\'))
    run.start -= 1;
  return ((at - run.start) % 2) === 1;
}

/**
 Adds the offsets of a stretch to a set, except the units that are white space.

 @param units - the set the offsets are added to

 @param english - the English

 @param from - offset where the stretch starts

 @param to - offset where the stretch ends, which is not part of it

 @example
 ```ts
 const units = new Set<number>();
 addStretch({ units, english: 'a<!-- b -->c', from: 1, to: 11, },); // units holds 1 to 4, 6, 8 to 10
 ```
 */
function addStretch(
  {
    units,
    english,
    from,
    to,
  }: {
    readonly units: Set<number>;
    readonly english: string;
    readonly from: number;
    readonly to: number;
  },
): void {
  for (let at = from; at < to; at += 1) {
    if (!WHITE_SPACE_UNITS.has(english.charAt(at,),))
      units.add(at,);
  }
}

/**
 The offsets, among those of an English, of the units of its HTML comments: the comments of the page parser's own mask
 (`commentRegionsOf`, `mask-html-comments.ts`, the one definition the parser blanks them by), white space left out. A comment
 runs from an opener to the first closer after the opener's four characters, and the scan goes on past it, so an opener
 inside a comment, or the dashes an opener shares with a closer, start none. A comment that never closes names no unit here:
 the prose view blanks the tail it swallows. One pass over the English.

 @param english - the English

 @returns The offsets of those units that are no white space

 @example
 ```ts
 const kept = commentUnitsOf({ english: 'a<!-- b -->c', },); // 1 to 4, 6, 8 to 10
 ```
 */
export function commentUnitsOf({ english, }: { readonly english: string; },): ReadonlySet<number> {
  /**
   The offsets, mutated only inside this function.
   */
  const units = new Set<number>();
  for (const region of commentRegionsOf({ text: english, },)) {
    if (!region.terminated)
      continue;
    addStretch({
      units,
      english,
      from: region.startOffset,
      to: region.endOffset,
    },);
  }
  return units;
}

/**
 The offsets, among those of an English, of the units no reader takes for words although the parse keeps them in a text node: the
 units of an HTML comment and of a character reference.

 @param english - the English

 @returns The offsets of those units that are no white space

 @example
 ```ts
 const kept = commentAndReferenceUnitsOf({ english: 'a<!-- b -->c&amp;d', },); // 1 to 4, 6, 8 to 10, and 12 to 16
 ```
 */
function commentAndReferenceUnitsOf({ english, }: { readonly english: string; },): ReadonlySet<number> {
  /**
   The offsets, mutated only inside this function.
   */
  const units = new Set<number>(commentUnitsOf({ english, },),);
  for (
    let amp = english.indexOf('&',);
    amp !== (-1);
    amp = english.indexOf(
      '&',
      amp + 1,
    )
  ) {
    /**
     Where the reference that starts here ends, none when none does.
     */
    const [end,] = referenceEndOf({
      english,
      at: amp,
    },);
    if (end !== undefined) {
      addStretch({
        units,
        english,
        from: amp,
        to: end + 1,
      },);
    }
  }
  return units;
}

/**
 How many English texts `unreadUnitsOf` remembers.
 */
const REMEMBERED_UNREAD = 4;

/**
 One English and the units of it that are not read as words.
 */
type RememberedUnread = {
  /**
   The English the units were read from.
   */
  readonly english: string;

  /**
   The definitions it was read with.
   */
  readonly definitions: string;

  /**
   The offsets of the units.
   */
  readonly units: ReadonlySet<number>;
};

/**
 The units not read as words of the most recent English texts, the newest last.
 */
const rememberedUnread: RememberedUnread[] = [];

/**
 The units of an English that a reader does not read as words: every unit the prose view blanks (a tag, the label of a footnote
 reference or of a reference link, a link's destination with its title, the delimiters of emphasis, code, a component's markup)
 and the units of an HTML comment and of a character reference, which the prose keeps as the units of a text node. White space
 is never among them. A page the grammar refuses blanks nothing, so only the comments and the references are named.

 @param english - the English of a slice

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for an English read alone: the label of a footnote
 reference or of a reference link is then plain text unless its definition stands in the same English

 @returns The offsets of those units

 @example
 ```ts
 const unread = unreadUnitsOf({ english: 'The <b>kittens</b>’ bowl.', },); // holds 4 to 6 and 14 to 17
 ```
 */
export function unreadUnitsOf(
  {
    english,
    definitions = '',
  }: {
    readonly english: string;
    readonly definitions?: string;
  },
): ReadonlySet<number> {
  /**
   The units read before for an equal English and equal definitions, none when there are none.
   */
  const known = rememberedUnread.find(function isSame(entry,): boolean {
    return (entry.english === english) && (entry.definitions === definitions);
  },);
  if (known !== undefined)
    return known.units;
  /**
   The prose of the English, with every unit the parse does not read as a word blanked.
   */
  const { prose, } = rememberedViewOf({
    english,
    definitions,
  },);
  /**
   The units the parse blanked, the comments and the references.
   */
  const units = new Set<number>(commentAndReferenceUnitsOf({ english, },),);
  for (let at = 0; at < english.length; at += 1) {
    if ((prose.charAt(at,) === ' ') && (!WHITE_SPACE_UNITS.has(english.charAt(at,),)))
      units.add(at,);
  }
  rememberedUnread.push({
    english,
    definitions,
    units,
  },);
  if (rememberedUnread.length > REMEMBERED_UNREAD)
    rememberedUnread.shift();
  return units;
}

/**
 Whether a unit is a line break, which no quotation pair and no word crosses.
 */
const LINE_BREAKS: ReadonlySet<string> = new Set([
  '\n',
  '\r',
],);

/**
 The units of an English that show a reader nothing between two pieces of text: every unit outside the stretches the parse calls
 text (the delimiters of emphasis, strike and code, a link's brackets with its destination and title, a tag, an expression, the
 markers of a list or a heading) and the units of a comment, line breaks apart. A reader skips them, so `kit*tens*`, `**kit**tens`,
 `[kit](u)tens` and `<b>kit</b>tens` each read as the one word they show. A character reference is its shown character, so it is
 not here (`unreadUnitsOf` keeps it for the refusal run, which steps over it). A unit of a hidden stretch is hidden whatever its
 width, so a reference, an escape or an astral character inside a link's destination, a tag's attribute or a comment hides every
 unit it is written with (`consistency-shown-edge.ts` reads the flag of the unit's first character). The backticks of inline code
 are hidden and the content of inline code is shown as the code word it is, without the one space CommonMark strips from each side
 of content that begins and ends with a space and is not all spaces (`consistency-markup-ranges.ts`), so a word of code beside a
 word of text reads as the one word it shows. The typography side (`typography-shown-view.ts`) keeps that padding as shown text.
 An English the grammar
 refuses blanks nothing. What separates or annotates (a line break, a block tag, a table cell, an image, a ruby annotation) is
 told apart by `unitKindsOf` (`consistency-shown-kinds.ts`), which starts from these flags.

 @param english - the English of a slice

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for an English read alone

 @param codePadding - what pads the content of inline code: a space and a line ending as CommonMark reads it, or a literal space alone
 as an artifact written before line endings padded code was read, which hides no line ending beside code; a space and a line ending
 where absent

 @returns One flag per UTF-16 unit of the English, true where the unit shows nothing

 @example
 ```ts
 const hidden = transparentUnitsOf({ english: 'kit*tens*', },); // true at 3 and 8
 ```
 */
export function transparentUnitsOf(
  {
    english,
    definitions = '',
    codePadding = 'spaces-and-line-endings',
  }: {
    readonly english: string;
    readonly definitions?: string;
    readonly codePadding?: CodePadding;
  },
): readonly boolean[] {
  /**
   What the parse made of the English.
   */
  const view = rememberedViewOf({
    english,
    definitions,
    codePadding,
  },);
  /**
   One flag per unit, false until the unit is found to show nothing.
   */
  const hidden: boolean[] = Array.from(
    { length: english.length, },
    function shown(): boolean {
      return false;
    },
  );
  if (view.refused)
    return hidden;
  for (let at = 0; at < english.length; at += 1)
    hidden[at] = !LINE_BREAKS.has(english.charAt(at,),);
  for (const {
    start,
    end,
  } of [
    ...view.stretches,
    ...view.codeContents,
  ]) {
    for (let at = start; at < end; at += 1)
      hidden[at] = false;
  }
  // A LINE ENDING THAT PADS INLINE CODE shows a reader nothing, as the padding space does: CommonMark turns each line ending of the
  // content into a space and strips one from each side, so the line ending directly outside a content stretch (a line feed, a
  // carriage return, or the two together) is hidden. What stands directly outside a content stretch is a backtick when the code is
  // not padded, so a line break between words is never reached from here. The padding of spaces alone hides none.
  for (const {
    start,
    end,
  } of ((codePadding === 'spaces-and-line-endings') ? view.codeContents : [])) {
    if ((start > 0) && LINE_BREAKS.has(english.charAt(start - 1,),)) {
      hidden[start - 1] = true;
      if ((english.charAt(start - 1,) === '\n') && (english.charAt(start - 2,) === '\r'))
        hidden[start - 2] = true;
    }
    if (LINE_BREAKS.has(english.charAt(end,),)) {
      hidden[end] = true;
      if ((english.charAt(end,) === '\r') && (english.charAt(end + 1,) === '\n'))
        hidden[end + 1] = true;
    }
  }
  // A LINE BREAK INSIDE THE RESOURCE OF A LINK OR AN IMAGE (n169) is part of the link's destination, title or reference, which the
  // grammar reads across lines, and shows a reader nothing, so it is hidden with the rest of the resource. A line break between
  // words stays shown.
  for (const {
    start,
    end,
  } of view.linkResources) {
    for (let at = start; at < end; at += 1)
      hidden[at] = true;
  }
  for (const region of commentRegionsOf({ text: english, },)) {
    if (!region.terminated)
      continue;
    for (let at = region.startOffset; at < region.endOffset; at += 1)
      hidden[at] = !LINE_BREAKS.has(english.charAt(at,),);
  }
  return hidden;
}

//endregion Consistency unread units
