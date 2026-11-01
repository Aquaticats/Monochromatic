//region Consistency mark neighbours
// WHAT STANDS DIRECTLY BESIDE A SINGLE MARK, and what that says about the mark. A reader of the consistency pass asks two
// questions of the character on each side of an apostrophe-like mark: is it a letter of a word (the neighbour test of the
// pass, `continuesLatinWordOrCompatibilityForm`, `latin-letters.ts`), and does it PART the mark from a word. A character
// parts a mark when it is white space, the edge of the text, or punctuation that is not a single mark (a full stop, a comma,
// a closing bracket, a double quotation mark). A letter or a digit of another script, a symbol (`™`, `+`, `²`) and a
// pictograph (`🐱`) neither continue a Latin word nor part a mark: a mark beside one is no apostrophe of a Latin word and no
// edge of a quotation either, so it reads FREE.
//
// A SINGLE MARK BESIDE A SINGLE MARK READS FREE, both of them (`''`, `’’`, `’ʼ`). The eight characters that count as a single
// mark here are the three apostrophe-like ones (the straight apostrophe, the closing curly single mark, the modifier letter
// apostrophe), the opening curly single mark, the low and the reversed single mark and both single angle quotation marks:
// each is a mark of a quotation or an apostrophe that a reader sees as a mark, so two of them side by side are a doubled
// mark and no apostrophe of a word.

/**
 The characters that count as a single mark beside a single mark: the straight apostrophe, the opening and the closing curly
 single mark, the modifier letter apostrophe, the low and the reversed single mark, and both single angle quotation marks.
 */
const NEIGHBOUR_SINGLE_MARKS: ReadonlySet<string> = new Set([
  '\'',
  '‘',
  '’',
  '\u{02BC}',
  '\u{201A}',
  '\u{201B}',
  '‹',
  '›',
],);

/* oxlint-disable no-restricted-syntax/no-regex -- the input is at most one code point, anchored at both ends with one optional class and no repetition, so the test is bounded and cannot backtrack; the Unicode general category for punctuation and the white space class have no string API */
/**
 The empty answer for an edge of the text, or a whole character that is white space or punctuation.
 */
const EDGE_WHITE_SPACE_OR_PUNCTUATION = /^[\s\p{P}]?$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Whether a whole character counts as a single mark beside a single mark.

 @param character - one whole character, empty past an edge of a text

 @returns Whether the character is one of the eight single marks

 @example
 ```ts
 const single = isNeighbouringSingleMark({ character: '’', },); // true
 ```
 */
export function isNeighbouringSingleMark({ character, }: { readonly character: string; },): boolean {
  return NEIGHBOUR_SINGLE_MARKS.has(character,);
}

/**
 Whether the character beside a mark parts the mark from a word: the edge of the text (the empty answer), white space, or
 punctuation that is not a single mark. A letter or a digit of any script, a symbol and a pictograph do not.

 @param character - one whole character, empty past an edge of a text

 @returns Whether the mark is parted from a word on that side

 @example
 ```ts
 const parted = partsMarkFromWord({ character: ',', },); // true
 const attached = partsMarkFromWord({ character: '猫', },); // false
 ```
 */
export function partsMarkFromWord({ character, }: { readonly character: string; },): boolean {
  return EDGE_WHITE_SPACE_OR_PUNCTUATION.test(character,) && (!isNeighbouringSingleMark({ character, },));
}

//endregion Consistency mark neighbours
