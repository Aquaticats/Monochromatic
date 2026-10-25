import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships readable
// A TEXT AS THE PARSE READS IT, which the encoder and the closing of a flow share: the comments that close blanked,
// so what stands inside them is not parsed, and a byte order mark that opens the text an ordinary character, since
// the page carries the text after a note and a blank line and the mark then stands in the middle of the page.

/**
 The code point of the byte order mark.
 */
const BYTE_ORDER_MARK_CODE_POINT = 0xFE_FF;

/**
 The character a text may start with that the parser drops from its input.
 */
const BYTE_ORDER_MARK = String.fromCodePoint(BYTE_ORDER_MARK_CODE_POINT,);

/**
 What the parse is given in place of a byte order mark that opens a text. The page carries the text after a note and a
 blank line, so the mark stands in the middle of the page: an ordinary character, after which the first line of the text
 starts no block, where the parser drops a mark that opens its input and reads the line as the start of the document.
 */
const LEADING_MARK_STAND_IN = 'x';

/**
 Replaces every character of a text but its line endings with a space.

 @param text - text to blank

 @returns Text of the same length

 @example
 ```ts
 const spaces = spacesLike({ text: '<!-- a\n-->', },);
 ```
 */
function spacesLike({ text, }: { readonly text: string; },): string {
  return text
    .split('\n',)
    .map(function spaces(line,): string {
      return ' '.repeat(line.length,);
    },)
    .join('\n',);
}

/**
 Replaces the characters of each stretch with spaces and keeps the line endings.

 @param text - text to blank

 @param ranges - stretches to blank, in text order

 @returns Text of the same length

 @example
 ```ts
 const blanked = blanking({ text: 'a<!-- b -->c', ranges: [{ start: 1, end: 11, },], },);
 ```
 */
export function blanking(
  {
    text,
    ranges,
  }: {
    readonly text: string;
    readonly ranges: readonly TextRange[];
  },
): string {
  /**
   Pieces of the result, joined at the end.
   */
  const pieces: string[] = [];
  /**
   Where the text not yet copied starts.
   */
  const copied = { upTo: 0, };
  for (const range of ranges) {
    pieces.push(
      text.slice(
        copied.upTo,
        range.start,
      ),
      spacesLike({
        text: text.slice(
          range.start,
          range.end,
        ),
      },),
    );
    copied.upTo = range.end;
  }
  pieces.push(text.slice(copied.upTo,),);
  return pieces.join('',);
}

/**
 Gives a text to the parse as the page holds it: a byte order mark that opens it is an ordinary character there.

 @param text - text

 @returns Text of the same length, with a letter in place of a mark that opens it

 @example
 ```ts
 const given = asTheParseReadsIt({ text: '\uFEFF- a', },);
 ```
 */
export function asTheParseReadsIt({ text, }: { readonly text: string; },): string {
  return text.startsWith(BYTE_ORDER_MARK,) ? `${LEADING_MARK_STAND_IN}${text.slice(1,)}` : text;
}

//endregion Source ships readable
