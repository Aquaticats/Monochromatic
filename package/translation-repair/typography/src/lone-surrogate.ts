import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  SURROGATE_FIRST,
  SURROGATE_LAST,
} from './semantic-wrap-binary-constants.ts';

//region Lone surrogate
// A UTF-16 string can hold half of a surrogate pair, which no UTF-8 stream can
// carry: Node writes it as U+FFFD, a different character. A text bound for a
// program's standard input is read for one before it is sent.

/**
 Finds the first UTF-16 code unit of a surrogate that has no partner.

 @param text - passage to read

 @returns Index of the first unpaired surrogate, or -1 where there is none

 @example
 ```ts
 loneSurrogateIndex({ text: 'It naps \uD83D and wakes.', },); // 8
 ```
 */
export function loneSurrogateIndex({ text, }: { readonly text: string; },): number {
  if (text.isWellFormed())
    return -1;

  // `codePointAt` reads a whole pair as one code point above the surrogate
  // range, so a code point inside the range is a surrogate with no partner.
  for (let at = 0; at < text.length;) {
    /**
     Code point starting at this index.
     */
    const codePoint = nonNullishOrThrow(text.codePointAt(at,),);
    if ((codePoint >= SURROGATE_FIRST) && (codePoint <= SURROGATE_LAST))
      return at;
    at += String.fromCodePoint(codePoint,)
      .length;
  }
  throw new Error('unreachable: a text that is not well formed holds an unpaired surrogate',);
}

//endregion Lone surrogate
