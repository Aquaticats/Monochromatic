/**
 The one owner of the whitespace-delimited token scan the plugin scanners use.

 The `@mutates` contract scanner in this package and the TSDoc block scanner in
 `@monochromatic-dev/oxlint-plugin-tsdoc` both split a block's content at its
 first token, so the scan lives here once.

 @module
 */

import { isWhitespaceChar, } from './text-character.ts';

/**
 Returns the exclusive end index of the first whitespace-delimited token of
 `text` starting at `start`.
 
 Scans forward with a bounded cursor, so the work stays linear in the scanned
 run and the stack stays flat.

 @param text - string scanned for the token

 @param start - index where the token is expected to begin

 @returns first index at or after `start` that is whitespace, or the length

 @example
 ```ts
 tokenEnd({ text: 'name - desc', start: 0 }); // 4
 ```
 */
export function tokenEnd({
  text,
  start,
}: {
  /**
   String scanned for the token.
   */
  readonly text: string;
  /**
   Index where the token is expected to begin.
   */
  readonly start: number;
},): number {
  for (let cursor = start; cursor < text
    .length; cursor += 1) {
    if (isWhitespaceChar(text.charAt(cursor,),))
      return cursor;
  }
  return text.length;
}
