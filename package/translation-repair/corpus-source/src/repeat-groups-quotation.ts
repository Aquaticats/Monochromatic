import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  QUOTE_CLOSERS,
  QUOTE_OPENERS,
  STRAIGHT_QUOTE,
} from './repeat-groups-bounds.ts';
import type { WordingStream, } from './repeat-groups-types.ts';

//region Repeat groups quotation
// Which stretches of the wording stream touch quoted speech. A repeat that
// touches it anywhere stays out of the groups, because the pass that reads the
// groups rewrites the occurrences' spans and must never rewrite part of a
// quotation. Every character is judged on its own, so the answer for a stretch
// is the same wherever the stretch starts and ends.

/**
 Whether one character of the stream is part of quoted speech: it stands at an
 open quotation depth, or it is itself a quotation mark. The marks count
 whatever the depth the reading gave them, since the reading leaves a closing
 mark outside its quotation, and a closing mark that no opening mark matches
 (the end of a quotation that began in another paragraph) leaves the depth at
 zero.

 @param wording - wording stream of the page

 @param index - stream index of the character

 @returns Whether the character is quoted speech or a quotation mark

 @example
 ```ts
 const touches = isQuotationCharacter({ wording, index: 3, },);
 ```
 */
function isQuotationCharacter(
  {
    wording,
    index,
  }: {
    readonly wording: WordingStream;
    readonly index: number;
  },
): boolean {
  if (nonNullishOrThrow(wording.depths[index],) > 0)
    return true;

  /**
   Character at this index, or a break.
   */
  const char = nonNullishOrThrow(wording.chars[index],);
  return QUOTE_OPENERS.includes(char,)
    || QUOTE_CLOSERS.includes(char,)
    || (char === STRAIGHT_QUOTE);
}

/**
 For each place in the wording stream, how many characters before it are
 quoted speech or quotation marks, so that the question for any stretch
 is two reads and a subtraction however long the stretch is.

 @param wording - wording stream of the page

 @returns One count per stream index and one more for the end, ascending from zero

 @example
 ```ts
 const counts = quotedCountsOf({ wording, },);
 ```
 */
export function quotedCountsOf({ wording, }: { readonly wording: WordingStream; },): readonly number[] {
  /**
   Counts so far, the first being the count before any character.
   */
  const counts: number[] = [0,];

  /**
   Quoted characters seen so far.
   */
  let quoted = 0;

  /**
   Wording characters of the page.
   */
  const { chars, } = wording;
  for (let index = 0; index < chars.length; index++) {
    if (isQuotationCharacter({
      wording,
      index,
    },))
      quoted++;
    counts.push(quoted,);
  }
  return counts;
}

/**
 Whether any character of a stretch of the wording stream is quoted speech or
 a quotation mark, the opening and the closing mark included.

 @param counts - quoted characters before each stream index, from {@link quotedCountsOf}

 @param start - stream index of the stretch's first character

 @param end - stream index one past the stretch's last character

 @returns Whether the stretch touches quoted speech anywhere

 @throws {@link Error} when the stretch is empty, starts before the stream or ends past it, which no finder builds

 @example
 ```ts
 const touches = touchesQuotation({ counts, start: 0, end: 12, },);
 ```
 */
export function touchesQuotation(
  {
    counts,
    start,
    end,
  }: {
    readonly counts: readonly number[];
    readonly start: number;
    readonly end: number;
  },
): boolean {
  if ((end <= start) || (start < 0))
    throw new Error('unreachable: a repeat or clause stretch is empty or starts before the wording stream, though both finders read inside it',);
  if (end >= counts.length)
    throw new Error('unreachable: a repeat or clause stretch ends past the wording stream, though both finders read inside it',);
  return nonNullishOrThrow(counts[end],)
    > nonNullishOrThrow(counts[start],);
}

//endregion Repeat groups quotation
