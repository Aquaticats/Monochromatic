import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  FENCE_MARKS,
  rangeOf,
  startsAny,
  type TextEdit,
} from './source-ships-ranges.ts';

//region Source ships list marker
// A LINE THAT LOOKS LIKE A LIST MARKER RIGHT AFTER INDENTED CODE, which the parse reads as text. The list construct
// treats the indented code before it as it treats a paragraph: an item that cannot interrupt a paragraph, an empty one or
// an ordered one that does not start at one, does not start a list there, with a blank line between the two or none.
// The encoder writes a blank line after indented code, the page has no indented code, and on the page the same line
// opens a list: the account of the page lacks the marker the source shows. The marker is written with a backslash, so the
// page shows it as text, as the source does. An item that can interrupt a paragraph is a list in the parse and is left.

/**
 Characters that open an item of a bullet list.
 */
const BULLETS = '*+-';

/**
 Characters that end the number of an item of an ordered list.
 */
const ORDERED_ENDINGS = '.)';

/**
 Characters of a number.
 */
const DIGITS = '0123456789';

/**
 Characters after a marker that make it a marker: a space, a tab, or the end of the line.
 */
const AFTER_MARKER = ' \t\r\n';

/**
 The most digits a list marker may hold.
 */
const MAX_DIGITS = 9;

/**
 What the index of a marker is when a line starts with none.
 */
const NO_MARKER = -1;

/**
 Whether a character is one of a set, an empty answer of `charAt` at the end of the text being none.

 @param set - characters

 @param character - one character, or none

 @returns Whether the character is there

 @example
 ```ts
 const found = isOneOf({ set: '*+-', character: '*', },);
 ```
 */
function isOneOf(
  {
    set,
    character,
  }: {
    readonly set: string;
    readonly character: string;
  },
): boolean {
  return (character !== '') && set.includes(character,);
}

/**
 Counts the digits a number starts with, up to one more than a marker may hold.

 @param source - text

 @param start - index of the first character of the line's content

 @returns How many digits stand from the start

 @example
 ```ts
 const digits = digitsLengthOf({ source: '2. x', start: 0, },);
 ```
 */
function digitsLengthOf(
  {
    source,
    start,
  }: {
    readonly source: string;
    readonly start: number;
  },
): number {
  for (let length = 0; length <= MAX_DIGITS; length += 1) {
    if (!isOneOf({
      set: DIGITS,
      character: source.charAt(start + length,),
    },))
      return length;
  }
  return MAX_DIGITS + 1;
}

/**
 Finds the character that makes a list marker at the start of a line: the bullet itself, or the full stop or bracket that
 ends the number.

 @param source - text

 @param start - index of the first character of the line's content

 @returns Index of the character, none when the line starts with no marker

 @example
 ```ts
 const at = markerIndexOf({ source: '2. x', start: 0, },);
 ```
 */
function markerIndexOf(
  {
    source,
    start,
  }: {
    readonly source: string;
    readonly start: number;
  },
): number {
  if (isOneOf({
    set: BULLETS,
    character: source.charAt(start,),
  },))
    return start;
  /**
   How many digits the line starts with.
   */
  const digits = digitsLengthOf({
    source,
    start,
  },);
  if ((digits === 0) || (digits > MAX_DIGITS))
    return NO_MARKER;
  /**
   Index of the character after the number.
   */
  const ending = start + digits;
  if (!isOneOf({
    set: ORDERED_ENDINGS,
    character: source.charAt(ending,),
  },))
    return NO_MARKER;
  return ending;
}

/**
 Whether a block is indented code: a code node that no fence opens.

 @param source - text

 @param node - block

 @returns Whether the block is indented code

 @example
 ```ts
 const indented = isIndentedCodeBlock({ source: '    c\n', node, },);
 ```
 */
export function isIndentedCodeBlock(
  {
    source,
    node,
  }: {
    readonly source: string;
    readonly node: TreeNode;
  },
): boolean {
  if (node.type !== 'code')
    return false;
  /**
   Where the block starts.
   */
  const { start, } = rangeOf({ node, },);
  return !startsAny({
    text: source,
    at: start,
    candidates: FENCE_MARKS,
  },);
}

/**
 Writes the marker of a line that looks like a list item with a backslash.

 @param source - text

 @param start - index of the first character of the line's content

 @returns Edit that escapes the character that makes the marker, none when the line starts with no marker

 @example
 ```ts
 const edits = markerEscapeEdits({ source: '2.\n', start: 0, },);
 ```
 */
export function markerEscapeEdits(
  {
    source,
    start,
  }: {
    readonly source: string;
    readonly start: number;
  },
): readonly TextEdit[] {
  /**
   Index of the character that makes the marker.
   */
  const at = markerIndexOf({
    source,
    start,
  },);
  if (at === NO_MARKER)
    return [];
  /**
   The character after the marker, none at the end of the text.
   */
  const next = source.charAt(at + 1,);
  if ((next !== '') && (!AFTER_MARKER.includes(next,)))
    return [];
  return [
    {
      start: at,
      end: at + 1,
      replacement: `\\${source.charAt(at,)}`,
    },
  ];
}

//endregion Source ships list marker
