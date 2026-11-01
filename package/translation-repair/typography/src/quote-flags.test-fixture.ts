import { proseMaskReading, } from '../dist/final/node/index.mjs';

/**
 Prose flag of every straight quote mark of a text, in order of appearance.

 @param text - text whose mask is read

 @param definitions - the page's definitions, none for a slice read alone

 @returns `P` for a mark in prose, `N` for a mark that is not

 @example
 ```ts
 const flags = quoteFlags({ text: 'a "b" it\'s', },); // 'PPP'
 ```
 */
export function quoteFlags(
  {
    text,
    definitions,
  }: {
    readonly text: string;
    readonly definitions?: string;
  },
): string {
  /**
   Mask of the text.
   */
  const { mask, } = proseMaskReading({
    text,
    ...((definitions === undefined) ? {} : { definitions, }),
  },);
  return Array.from({ length: text.length, },)
    .map(function flagOf(
      _unused,
      index,
    ): string {
      /**
       Unit under the scan.
       */
      const unit = text.charAt(index,);
      if ((unit !== '\'') && (unit !== '"'))
        return '';
      return (mask[index] === true) ? 'P' : 'N';
    },)
    .join('',);
}
