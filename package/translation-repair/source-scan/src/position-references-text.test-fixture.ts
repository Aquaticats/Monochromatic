/**
 A text as the position-references scan reads it (ledger D33): its wrapped
 comment and string lines joined, so a phrase broken across two lines is one
 phrase, and its words with their offsets.

 @module
 */

import { isAsciiLetter, } from '@monochromatic-dev/translation-repair-markup';
import { CONTINUATION_MARKERS, } from './position-references-vocabulary.test-fixture.ts';

/**
 One text with its wrapped lines joined, and where each line starts in it.

 @example
 ```ts
 const joined: JoinedText = { flat: 'the case above', lineStarts: [0,], };
 ```
 */
type JoinedText = {
  /**
   Lines joined by one space, each continuation line stripped of the comment
   or string markup that opened it.
   */
  readonly flat: string;

  /**
   Offset in `flat` where each source line starts, in line order.
   */
  readonly lineStarts: readonly number[];
};

/**
 One word of a joined text.

 @example
 ```ts
 const word: Word = { text: 'above', start: 9, end: 14, };
 ```
 */
export type Word = {
  /**
   Lowercased letters.
   */
  readonly text: string;

  /**
   Offset of its first letter.
   */
  readonly start: number;

  /**
   Offset just past its last letter.
   */
  readonly end: number;
};

/**
 A continuation line without the markup that opened it: a comment's slashes
 or star, a string's joining sign and quote, a quote block's sign.

 @param line - one source line after the first

 @returns Its words, leading markup and space removed

 @example
 ```ts
 continuationOf({ line: "        + 'above the bowl'", },); // "above the bowl'"
 ```
 */
function continuationOf({ line, }: { readonly line: string; },): string {
  /**
   The line without its indentation.
   */
  const trimmed = line.trimStart();
  /**
   The markup this line opens with, longest first so a doc comment is not
   read as a plain one.
   */
  const marker = CONTINUATION_MARKERS.find(function opens(candidate,): boolean {
    return trimmed.startsWith(candidate,);
  },) ?? '';
  if (marker !== '')
    return trimmed.slice(marker.length,)
      .trimStart();
  /**
   The line without a joining sign.
   */
  const unjoined = trimmed.startsWith('+',)
    ? trimmed.slice(1,)
      .trimStart()
    : trimmed;
  return (unjoined.startsWith('\'',) || unjoined.startsWith('`',)) ? unjoined.slice(1,) : unjoined;
}

/**
 Where each piece of a joined text starts in it.

 @param pieces - pieces in order

 @returns Offset of each

 @example
 ```ts
 startsOf({ pieces: ['ab', ' c',], },); // [0, 2]
 ```
 */
function startsOf({ pieces, }: { readonly pieces: readonly string[]; },): readonly number[] {
  /**
   Offsets so far.
   */
  const starts: number[] = [];
  /**
   Offset the next piece starts at.
   */
  let offset = 0;
  for (const piece of pieces) {
    starts.push(offset,);
    offset += piece.length;
  }
  return starts;
}

/**
 Joins a text's lines, each after the first stripped of its opening markup.

 @param text - file text

 @returns The joined text and where each line starts in it

 @example
 ```ts
 joinedText({ text: '// the case\n// above', },).flat; // '// the case above'
 ```
 */
export function joinedText({ text, }: { readonly text: string; },): JoinedText {
  /**
   Each line as it joins: the first whole, the rest stripped and led by a
   space.
   */
  const pieces = text.split('\n',)
    .map(function asPiece(
      line,
      index,
    ): string {
      return (index === 0) ? line : ` ${continuationOf({ line, },)}`;
    },);
  return {
    flat: pieces.join('',),
    lineStarts: startsOf({ pieces, },),
  };
}

/**
 Every word of a joined text: maximal runs of ASCII letters, lowercased.

 @param flat - joined text

 @returns Words in order

 @example
 ```ts
 wordsOf({ flat: 'The case, above.', },).map(({ text, },) => text); // ['the', 'case', 'above']
 ```
 */
export function wordsOf({ flat, }: { readonly flat: string; },): readonly Word[] {
  /**
   Words found so far.
   */
  const words: Word[] = [];
  /**
   Offset the current word began at, -1 between words.
   */
  let start = -1;
  for (let at = 0; at <= flat.length; at += 1) {
    /**
     Whether this offset holds a letter.
     */
    const letter = (at < flat.length) && isAsciiLetter({ character: flat.charAt(at,), },);
    if (letter && (start < 0))
      start = at;
    if ((!letter) && (start >= 0)) {
      words.push({
        text: flat.slice(
          start,
          at,
        )
          .toLowerCase(),
        start,
        end: at,
      },);
      start = -1;
    }
  }
  return words;
}
