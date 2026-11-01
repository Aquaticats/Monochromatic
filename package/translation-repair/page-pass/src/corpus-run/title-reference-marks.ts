import {
  closedMarkSpans,
  wordStarts,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { straightenQuotes, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  Span,
  TitleLocation,
} from './title-reference-scope.ts';

//region Title reference marks
// A TITLE FOUND BETWEEN MARKS: title brackets or quotes, read by index scan
// on one line, a tag attribute's quoted value skipped, and several spans
// settled by the one that already carries the heading's rendering.

/**
 Newline, which no span crosses.
 */
const LINE_END = '\n';

/**
 Separator before a tag attribute's quoted value, which no title stands in.
 */
const ATTRIBUTE_EQUALS = '=';

/**
 Blanks a tag may write between an attribute's equals sign and its quoted
 value, as `<h3 align = "center">` does.
 */
const ATTRIBUTE_BLANKS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
],);

/**
 Offset of the last character before a mark that is not a blank, -1 where
 only blanks precede it.

 @param pageText - page text of the slice

 @param at - offset of the opening mark

 @returns Offset read backwards past the blanks

 @example
 ```ts
 offsetPastBlanks({ pageText: 'a = "b"', at: 4, },); // 2
 ```
 */
function offsetPastBlanks(
  {
    pageText,
    at,
  }: {
    readonly pageText: string;
    readonly at: number;
  },
): number {
  /**
   Offset read backwards past the blanks.
   */
  let before = at - 1;
  while ((before >= 0) && ATTRIBUTE_BLANKS.has(pageText.charAt(before,),))
    before -= 1;
  return before;
}

/**
 Whether an opening mark stands as a tag attribute's value, after an equals
 sign and any blanks.

 CLASS ONE HUNDRED TWENTY (XingZ6010, 2026-09-24): the first check read only
 the character before the mark, so `align = "center"` read as prose and the
 title was written into the attribute.

 @param pageText - page text of the slice

 @param at - offset of the opening mark

 @returns Whether the mark opens an attribute value

 @example
 ```ts
 opensAttributeValue({ pageText: '<h3 align = "center">', at: 11, },); // true
 ```
 */
function opensAttributeValue(
  {
    pageText,
    at,
  }: {
    readonly pageText: string;
    readonly at: number;
  },
): boolean {
  return pageText.charAt(offsetPastBlanks({
    pageText,
    at,
  },),) === ATTRIBUTE_EQUALS;
}

/**
 Every span between an opening and a closing mark on one line, a mark that
 never closed enclosing nothing (ledger B38).

 @param pageText - page text of the slice

 @param open - opening mark

 @param close - closing mark

 @returns Inner spans in order

 @example
 ```ts
 spansBetween({ pageText: '《Cat》', open: '《', close: '》', },); // [{ start: 1, end: 4, }]
 ```
 */
function spansBetween(
  {
    pageText,
    open,
    close,
  }: {
    readonly pageText: string;
    readonly open: string;
    readonly close: string;
  },
): readonly Span[] {
  return closedMarkSpans({
    text: pageText,
    open,
    close,
  },)
    .filter(function inProseOnOneLine(marks,): boolean {
      /**
       Whether the opening mark stands in prose, not after an attribute's
       equals sign.
       */
      const prose = !opensAttributeValue({
        pageText,
        at: marks.open,
      },);
      return prose && (!pageText.slice(
        marks.open + open.length,
        marks.close,
      )
        .includes(LINE_END,));
    },)
    .map(function inner(marks,): Span {
      return {
        start: marks.open + open.length,
        end: marks.close,
      };
    },);
}

/**
 Where the page renders the title inside one kind of marks: the one span
 where there is one, ambiguous where there are more.

 @param pageText - page text of the slice

 @param pairs - opening and closing marks to read

 @param kind - shape to report

 @param rendering - heading's rendering, which settles several spans where
 one already carries it

 @returns Located span, ambiguous, or none

 @example
 ```ts
 locateMarked({ pageText: '《Cat》', pairs: [['《', '》']], kind: 'bracket', rendering: 'Cat', },);
 ```
 */
export function locateMarked(
  {
    pageText,
    pairs,
    kind,
    rendering,
  }: {
    readonly pageText: string;
    readonly pairs: readonly (readonly [
      string,
      string,
    ])[];
    readonly kind: 'bracket' | 'quote';
    readonly rendering: string;
  },
): TitleLocation {
  /**
   Every span of every pair.
   */
  const spans = pairs.flatMap(function spansOf([
    open,
    close,
  ],): readonly Span[] {
    return spansBetween({
      pageText,
      open,
      close,
    },);
  },);
  /**
   First span, and the spans after it.
   */
  const [
    first,
    ...others
  ] = spans;
  if (first === undefined)
    return { kind: 'none', };
  if (others.length === 0) {
    return {
      kind,
      ...first,
    };
  }
  /**
   Rendering as the typography fold reads it.
   */
  const foldedRendering = straightenQuotes({ text: rendering, },);
  /**
   Spans that already carry the heading's rendering: opening with it as
   words (ledger B23), so "Catnip Days" does not carry "Cat", and read
   through the typography fold (ledger B24), so "The Cat's Afternoon" carries
   "The Cat’s Afternoon".
   */
  const carrying = spans.filter(function carries(span,): boolean {
    return wordStarts({
      text: straightenQuotes({ text: pageText.slice(
        span.start,
        span.end,
      ), },),
      needle: foldedRendering,
      end: 'word',
    },)
      .includes(0,);
  },);
  /**
   The span carrying the heading, and any other that carries it too.
   */
  const [
    carried,
    ...alsoCarrying
  ] = carrying;
  if ((carried === undefined) || (alsoCarrying.length > 0))
    return { kind: 'ambiguous', };
  return {
    kind,
    ...carried,
  };
}

//endregion Title reference marks
