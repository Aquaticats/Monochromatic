//region Closing punctuation
// CLASS ONE HUNDRED EIGHTY-ONE (TianqiChen66613, 2026-09-27): the page is
// written in Canadian English, and Canadian Press style sets a period or a
// comma inside the closing quotation mark. The archive follows it, 224 marks
// inside to 32 outside across the pin, but the bench wrote
// `the “high-performance robot”.` on a page that also wrote
// `“Old Man Chen,”`, so one page carried both conventions. Moved in code,
// like the quote shapes beside it, because a model asked to follow the style
// mostly will.

/**
 Right double quotation mark.
 */
const CLOSE_DOUBLE = '\u{201D}';

/**
 Marks Canadian Press style sets inside the closing quotation mark.
 */
const INSIDE_MARKS: ReadonlySet<string> = new Set([
  '.',
  ',',
],);

/**
 Marks that already end the quoted words, after which a second mark inside
 would read as doubled punctuation.
 */
const ENDING_MARKS: ReadonlySet<string> = new Set([
  '.',
  ',',
  '?',
  '!',
  '\u{2026}',
],);

/**
 Whether the closing quote at an offset takes the mark after it inside.

 @param text - replacement being read

 @param mask - which units are prose

 @param at - offset of the closing quote

 @returns Whether the quote and the mark after it swap

 @example
 ```ts
 const swaps = takesMarkInside({ text: 'the “nap”.', mask, at: 8, },);
 ```
 */
function takesMarkInside(
  {
    text,
    mask,
    at,
  }: {
    readonly text: string;
    readonly mask: readonly boolean[];
    readonly at: number;
  },
): boolean {
  /**
   Whether the quote and the unit after it are both prose.
   */
  const inProse = (mask[at] === true) && (mask[at + 1] === true);
  if (!inProse)
    return false;
  if (text.charAt(at,) !== CLOSE_DOUBLE)
    return false;
  /**
   Mark right after the quote.
   */
  const mark = text.charAt(at + 1,);
  if (!INSIDE_MARKS.has(mark,))
    return false;
  // A run of dots is an ellipsis, which stays where the writer put it.
  if ((mark === '.') && (text.charAt(at + 2,) === '.'))
    return false;
  return !ENDING_MARKS.has(text.charAt(at - 1,),);
}

/**
 Moves a period or comma that follows a closing double quotation mark inside
 it, in prose only.

 Leaves an ellipsis after the quote, a quotation that already ends in its
 own mark, a question or exclamation mark, and anything inside code or markup
 alone. Swaps two units for two, so offsets into the text still hold.

 @param text - replacement with its quote shapes restored

 @param mask - which units of the text are prose

 @returns Text with each such mark inside its quotation

 @example
 ```ts
 placeClosingPunctuation({ text: 'the “nap champion”.', mask, },);
 ```
 */
export function placeClosingPunctuation(
  {
    text,
    mask,
  }: {
    readonly text: string;
    readonly mask: readonly boolean[];
  },
): string {
  return (function scan(): string {
    /**
     Characters emitted so far.
     */
    const rebuilt: string[] = [];
    for (let index = 0; index < text.length; index += 1) {
      if (takesMarkInside({
        text,
        mask,
        at: index,
      },)) {
        rebuilt.push(
          text.charAt(index + 1,),
          CLOSE_DOUBLE,
        );
        index += 1;
        continue;
      }
      rebuilt.push(text.charAt(index,),);
    }
    return rebuilt.join('',);
  })();
}

//endregion Closing punctuation
