import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  endLineOf,
  footnoteBodyStart,
} from './source-ships-footnote.ts';
import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships footnote html
// A FOOTNOTE DEFINITION THAT OPENS WITH A BLOCK OF RAW HTML. A tag alone on the definition's line, or the start of an
// element such as a `div` or a `pre`, starts a block of raw HTML to a reader of the source, and that block holds the lines
// that follow it up to the next blank line. A fence mark or the mark of a display math on one of them is text there, not
// the start of a block of the footnote, so the lines are not the footnote's own blocks as the lines after any other
// definition are, and the encoder writes their marks as text.

/**
 A line the text after the label is followed by, to tell whether the block that starts there goes on over a line.
 */
const NEXT_LINE = 'x';

/**
 Whether a footnote definition opens with a block of raw HTML that goes on over the next line.

 @param source - text

 @param range - where the definition stands, from its label

 @returns Whether the text after the label starts a block of raw HTML that the line after it belongs to

 @example
 ```ts
 const html = footnoteOpensRawHtml({ source: '[^1]: <b>\n', range: { start: 0, end: 10, }, },); // true
 ```
 */
export function footnoteOpensRawHtml(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
  },
): boolean {
  /**
   Where the text after the label starts.
   */
  const from = footnoteBodyStart({
    source,
    range,
  },);
  /**
   Where its line ends.
   */
  const feed = source.indexOf(
    '\n',
    from,
  );
  /**
   The text after the label on the definition's line.
   */
  const line = source.slice(
    from,
    (feed === (-1)) ? source.length : feed,
  );
  /**
   The text after the label less the white space before it.
   */
  const text = line.trimStart();
  if (!text.startsWith('<',))
    return false;
  /**
   The block it starts, read with a line after it.
   */
  const { children, } = parseMarkdownMathBody({ body: `${line}\n${NEXT_LINE}`, },);
  /**
   The first block of the parse.
   */
  const block = children.at(0,);
  if (block?.type !== 'html')
    return false;
  /**
   The line the block ends on, the second when the block goes on over the line after the first.
   */
  const endLine = endLineOf({
    node: block,
    fallback: 0,
  },);
  return endLine > 1;
}

//endregion Source ships footnote html
