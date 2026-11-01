import {
  codePointAt,
  codePointBefore,
  isAsciiDigit,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { isCasedLetter, } from '@monochromatic-dev/translation-repair-typography/ts';
import { isWordCharacter, } from './canadian-date-parts.ts';
import {
  runEnd,
  runStart,
} from './text-runs.ts';

//region Canadian spelling context
// What stands beside a listed word decides whether it is prose the pass may
// respell: a neighbour that puts it inside an identifier, a path, a label or
// a longer token stops it (ledger K10, H14), and emphasis underscores or a
// slash between words do not. `canadian-spelling-capital.ts` decides the
// same for a capitalised word's capital (ledger K11).

/**
 Neighbours that always put a word outside prose: a heading or hashtag mark,
 an assignment, an address, an escape, a footnote label's caret.
 */
const NON_PROSE_NEIGHBOURS: ReadonlySet<string> = new Set([
  '#',
  '=',
  '@',
  '\\',
  '^',
],);

/**
 How a path or address token may open.
 */
const PATH_OPENERS: readonly string[] = [
  '/',
  '~',
  './',
  '../',
];

/**
 Marks a path or address token may carry anywhere.
 */
const PATH_MARKS: readonly string[] = [
  ':',
  '=',
  '\\',
];

/**
 What stands on one side of a word.
 */
type Side = Readonly<{
  /**
   Character touching the word.
   */
  neighbour: string;

  /**
   Whether a word stands past the neighbour.
   */
  farWord: boolean;
}>;

/**
 Whether one character belongs to a whitespace-delimited token.

 @param character - one UTF-16 unit, empty past a text edge

 @returns Whether it is present and no whitespace

 @example
 ```ts
 isTokenCharacter({ character: '/', },); // true
 ```
 */
function isTokenCharacter(
  { character, }: { readonly character: string; },
): boolean {
  return (character !== '') && (character.trim() !== '');
}

/**
 Where the line holding one offset starts.

 @param text - text under scan

 @param at - offset inside the line

 @returns Offset of the line's first character

 @example
 ```ts
 lineStartOf({ text: 'a\nbc', at: 3, },); // 2
 ```
 */
export function lineStartOf(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): number {
  return text.lastIndexOf(
    '\n',
    at - 1,
  ) + 1;
}

/**
 Every word a text carries, in lower case.

 @param text - text under scan

 @returns Its words

 @example
 ```ts
 lowerCaseWordSetOf({ text: 'The id, the Ego.', },); // Set { 'the', 'id', 'ego' }
 ```
 */
export function lowerCaseWordSetOf(
  { text, }: { readonly text: string; },
): ReadonlySet<string> {
  /**
   Words found so far.
   */
  const words = new Set<string>();
  for (let at = 0; at < text.length;) {
    /**
     Where a word starting here ends, or the cursor where none starts.
     */
    const end = runEnd({
      text,
      from: at,
      keeps: isWordCharacter,
    },);
    if (end > at) {
      words.add(text.slice(
        at,
        end,
      )
        .toLowerCase(),);
    }
    at = Math.max(
      end,
      at + 1,
    );
  }
  return words;
}

/**
 Whether a token carries a dot with a letter or digit after it, as a file
 name or a domain does.

 @param token - whitespace-delimited token

 @returns Whether such a dot stands in it

 @example
 ```ts
 carriesDottedName({ token: 'cat.png', },); // true
 ```
 */
function carriesDottedName(
  { token, }: { readonly token: string; },
): boolean {
  for (let at = token.indexOf('.',); at !== (-1); at = token.indexOf(
    '.',
    at + 1,
  )) {
    /**
     Whole character after the dot.
     */
    const next = codePointAt({
      text: token,
      at: at + 1,
    },);
    if (isCasedLetter({ character: next, },) || isAsciiDigit({ character: next, },))
      return true;
  }
  return false;
}

/**
 Whether the whitespace-delimited token holding a word reads as a path or an
 address rather than words joined by a slash ("humor/behavior"): it opens
 with `/`, `~`, `./` or `../`, or carries a dot before a letter or digit, a
 colon, an equals sign or a backslash.

 @param text - text under scan

 @param start - word's first offset

 @param end - word's exclusive end

 @returns Whether the token is a path

 @example
 ```ts
 inPathToken({ text: 'see assets/color/cat.png', start: 11, end: 16, },); // true
 ```
 */
export function inPathToken(
  {
    text,
    start,
    end,
  }: {
    readonly text: string;
    readonly start: number;
    readonly end: number;
  },
): boolean {
  /**
   The token.
   */
  const token = text.slice(
    runStart({
      text,
      from: start,
      keeps: isTokenCharacter,
    },),
    runEnd({
      text,
      from: end,
      keeps: isTokenCharacter,
    },),
  );
  return PATH_OPENERS.some(function opens(opener,): boolean {
    return token.startsWith(opener,);
  },) || carriesDottedName({ token, },)
    || PATH_MARKS.some(function holds(mark,): boolean {
      return token.includes(mark,);
    },);
}

/**
 Whether the character beside a word puts it outside prose: a digit ("id3"),
 a listed neighbour, an underscore or a dot with a letter or digit on its far
 side (an identifier, a file name, a domain), or a slash inside a path.

 Before the word, the far character may be a combining mark ending the word
 there (`café_color` with a combining acute), which counts as that word;
 after it, a mark right past the underscore or dot sits on that character
 and opens no word (audit area six, ledger B18).

 @param text - text under scan

 @param start - word's first offset

 @param end - word's exclusive end

 @returns Whether the word stands in a token that is no prose

 @example
 ```ts
 besideNonProse({ text: 'my_color_var', start: 3, end: 8, },); // true
 ```
 */
export function besideNonProse(
  {
    text,
    start,
    end,
  }: {
    readonly text: string;
    readonly start: number;
    readonly end: number;
  },
): boolean {
  // THE FAR CHARACTERS ARE READ WHOLE (ledger B21). They count only where the
  // neighbour is an underscore or a dot, one unit wide, so each stands one
  // unit past the word's edge.
  /**
   Whole character past the neighbour before the word.
   */
  const farBefore = codePointBefore({
    text,
    at: start - 1,
  },);
  /**
   Whole character past the neighbour after the word.
   */
  const farAfter = codePointAt({
    text,
    at: end + 1,
  },);
  /**
   The word's two sides.
   */
  const sides: readonly Side[] = [
    {
      neighbour: text.charAt(start - 1,),
      farWord: isWordCharacter({ character: farBefore, },) || isAsciiDigit({ character: farBefore, },),
    },
    {
      neighbour: text.charAt(end,),
      farWord: isCasedLetter({ character: farAfter, },) || isAsciiDigit({ character: farAfter, },),
    },
  ];
  return sides.some(function marks(side: Side,): boolean {
    if (isAsciiDigit({ character: side.neighbour, },) || NON_PROSE_NEIGHBOURS.has(side.neighbour,))
      return true;
    if ((side.neighbour === '_') || (side.neighbour === '.'))
      return side.farWord;
    return (side.neighbour === '/') && inPathToken({
      text,
      start,
      end,
    },);
  },);
}

//endregion Canadian spelling context
