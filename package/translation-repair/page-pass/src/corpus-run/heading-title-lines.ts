import {
  isHeadingLine,
  splitHeading,
} from './assembly-page-text.ts';

//region Heading title lines
// A HEADING'S TITLE READ FROM EITHER SHAPE THE CORPUS WRITES: an ATX line
// (`### 猫`) or an HTML heading element on one line
// (`<h3 align = "center">猫</h3>`), the shape the XingZ60 memorial uses.

/**
 Opening of an HTML heading element.
 */
const HTML_HEADING_OPEN = '<h';

/**
 Closing of an HTML heading element's opening tag.
 */
const TAG_CLOSE = '>';

/**
 Opening of an HTML heading element's closing tag.
 */
const HTML_HEADING_CLOSE = '</h';

/**
 Lowest heading level.
 */
const LEVEL_FIRST = '1';

/**
 Highest heading level.
 */
const LEVEL_LAST = '6';

/**
 Title of an HTML heading element standing alone on a line, empty where
 the line is no such element.

 @param line - one line

 @returns Inner text, trimmed

 @example
 ```ts
 htmlHeadingTitle({ line: '<h3 align = "center">猫</h3>', },); // '猫'
 ```
 */
function htmlHeadingTitle({ line, }: { readonly line: string; },): string {
  /**
   Line without its outer whitespace.
   */
  const trimmed = line.trim();
  if (!trimmed.startsWith(HTML_HEADING_OPEN,))
    return '';
  /**
   Level digit after `<h`.
   */
  const level = trimmed.charAt(HTML_HEADING_OPEN.length,);
  if ((level < LEVEL_FIRST) || (level > LEVEL_LAST))
    return '';
  /**
   Offset of the opening tag's close, -1 for none.
   */
  const open = trimmed.indexOf(TAG_CLOSE,);
  if (open === (-1))
    return '';
  /**
   Offset of the closing tag, -1 for none.
   */
  const close = trimmed.indexOf(
    `${HTML_HEADING_CLOSE}${level}${TAG_CLOSE}`,
    open,
  );
  if (close === (-1))
    return '';
  return trimmed.slice(
    open + 1,
    close,
  )
    .trim();
}

/**
 Titles of every heading in a text, ATX or HTML, in line order.

 @param text - text that may carry headings

 @returns One title per heading line

 @example
 ```ts
 headingTitles({ text: '### 猫\n\n<h3>猫猫</h3>', },); // ['猫', '猫猫']
 ```
 */
export function headingTitles({ text, }: { readonly text: string; },): readonly string[] {
  return text.split('\n',)
    .flatMap(function titleOf(line,): readonly string[] {
      if (isHeadingLine({ line, },)) {
        /**
         Title of the ATX heading.
         */
        const { title, } = splitHeading({ line, },);
        return [title,];
      }
      /**
       Title where the line is an HTML heading.
       */
      const html = htmlHeadingTitle({ line, },);
      return (html === '') ? [] : [html,];
    },);
}

//endregion Heading title lines
