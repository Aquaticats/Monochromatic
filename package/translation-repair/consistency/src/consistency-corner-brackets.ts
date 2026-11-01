//region Consistency corner brackets
// THE WHITE CORNER BRACKETS THE FOLD WRITES AS THE SINGLE MARK. The fold
// (`quote-normalize.ts`) writes the curly single marks and the white corner
// brackets `『` and `』` as the straight single mark, so a reader of a folded text
// that has to tell a corner bracket from an apostrophe asks here, with the
// character the text held before the fold: neither corner bracket is ever part of
// a word (`consistency-word-joints.ts` reads a joint of a word through it).
//
// THIS FILE NO LONGER READS WHAT A SINGLE MARK IS. It once paired each single
// mark of a folded text with the one that closes it and told an apostrophe that
// opens a word from one that opens a quotation, from the shape of the words near
// it; the edge reading of a judge's quote (`consistency-edge-marks.ts`) was its
// last reader. That edge reading now takes every single mark at an edge off the
// words whether or not it pairs with anything, so no guess about what a mark is
// decides what a span covers, and the reading was removed with its tests. The
// marks of a place's English that a splice must keep in place are read as
// characters in `consistency-loose-marks.ts`.

/**
 Characters that fold to the single mark and are never part of a word: the white corner brackets.
 */
const CORNER_BRACKETS: ReadonlySet<string> = new Set([
  '『',
  '』',
],);

/**
 Whether a character that folded to the single mark is a white corner bracket,
 which is never part of a word.

 @param original - the character the text held before the fold

 @returns Whether it is a corner bracket

 @example
 ```ts
 const cornered = isCornerBracket({ original: '』', },); // true
 ```
 */
export function isCornerBracket({ original, }: { readonly original: string; },): boolean {
  return CORNER_BRACKETS.has(original,);
}

//endregion Consistency corner brackets
