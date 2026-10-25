import type { Nodes, } from 'mdast';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import { unified, } from 'unified';

import {
  firstNestingExcess,
  isStackOverflow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { withoutRawHtml, } from './without-raw-html.ts';

//region GFM block ranges
// WHERE THE STRICT GRAMMAR READS A GFM BLOCK, read off a parse with GFM on. The plain reading
// (`parse-markdown-math.ts`) has no GFM, so a caller that reads a text by it cannot see these two, and each changes
// where its nodes end. A table splits each row into cells at every pipe that is not written with a backslash, whatever
// the pipe stands in (a code span, the address of an autolink), so text read without GFM as one construct can be two
// cells. A footnote definition ends at the first line that is not indented once a block of the footnote has closed the
// paragraph the definition opened with (an unclosed fence indented under it, say), where the plain reading takes every
// line after the label for the same paragraph.
//
// TWO READINGS OF THE FOOTNOTE DEFINITIONS, and the lines of raw HTML between them. A reader of the source in GFM takes
// a tag alone on a line for an HTML block, which ends the footnote's paragraph and puts what follows in a block of its
// own. The page writes every `<` of an untranslated original as text and the strict grammar has no raw HTML, so the
// line is a line of the footnote's paragraph there and the footnote goes on. `footnoteDefinitionRangesOf` is the first
// reading, which the page is arranged to match, and `pageFootnoteDefinitionRangesOf` is the second, read with raw HTML
// turned off (`without-raw-html.ts`), which says where a code span or a brace on those lines belongs.

/**
 A stretch of a text, as offsets in UTF-16 units.
 */
export type BlockRange = {
  readonly start: number;
  readonly end: number;
};

/**
 Where a text holds blocks of one type, read with GFM on.

 A TEXT NESTED PAST THE BOUND IS READ AS HOLDING NONE, as {@link moduleLineStartsOf} reads one: the strict grammar
 refuses it before it reads a line.

 @param body - text, without front matter and a leading byte order mark

 @param type - type of the blocks to find

 @param rawHtmlIsText - whether raw HTML is read as the text the page makes of it

 @returns Start and end of each block, in text order, nested ones included

 @example
 ```ts
 const tables = rangesOfType({ body: '| a |\n| - |\n', type: 'table', rawHtmlIsText: false, },);
 ```
 */
function rangesOfType(
  {
    body,
    type,
    rawHtmlIsText,
  }: {
    readonly body: string;
    readonly type: 'footnoteDefinition' | 'table';
    readonly rawHtmlIsText: boolean;
  },
): readonly BlockRange[] {
  if (firstNestingExcess({
    body,
    grammar: 'markdown',
  },)
    .kind
    === 'beyond') {
    return [];
  }
  /**
   Blocks found, in the order the walk reaches them.
   */
  const found: BlockRange[] = [];
  /**
   Nodes still to walk, the next one on top.
   */
  const pending: Nodes[] = [];
  try {
    /**
     The parser with GFM and the site's math on, and raw HTML turned off as the page has it when the caller asks.
     */
    const parser = unified()
      .use(remarkParse,)
      .use(remarkGfm,)
      .use(remarkMath,);
    pending.push(rawHtmlIsText
      ? parser
        .use(withoutRawHtml(),)
        .parse(body,)
      : parser.parse(body,),);
  }
  catch (error) {
    // Only the engine's stack exhaustion is the refusal the strict grammar gives such a text; any other failure is an
    // unexpected state that must keep propagating.
    if (!isStackOverflow(error,))
      throw error;
    return [];
  }
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if ((node.type === type) && (node.position !== undefined)) {
      /**
       Where the block starts and ends.
       */
      const {
        start,
        end,
      } = node.position;
      found.push({
        start: start.offset ?? 0,
        end: end.offset ?? 0,
      },);
    }
    if ('children' in node) {
      /**
       The node's children, the last first so they come out in order.
       */
      const { children, } = node;
      pending.push(...children.toReversed(),);
    }
  }
  return found.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
}

/**
 Where a text holds a table, in text order.

 @param body - text, without front matter and a leading byte order mark

 @returns Start and end of each table, the end after its last row, nested tables included

 @example
 ```ts
 const tables = tableRangesOf({ body: '| a | b |\n| - | - |\n| c | d |\n', },);
 ```
 */
export function tableRangesOf({ body, }: { readonly body: string; },): readonly BlockRange[] {
  if (!body.includes('|',))
    return [];
  return rangesOfType({
    body,
    type: 'table',
    rawHtmlIsText: false,
  },);
}

/**
 Where a text holds a footnote definition, in text order.

 @param body - text, without front matter and a leading byte order mark

 @returns Start and end of each definition, the end after the last character of its last block, nested ones included

 @example
 ```ts
 const footnotes = footnoteDefinitionRangesOf({ body: '[^1]: x\n    ~~~\n<>\n', },);
 ```
 */
export function footnoteDefinitionRangesOf({ body, }: { readonly body: string; },): readonly BlockRange[] {
  if (!body.includes('[^',))
    return [];
  return rangesOfType({
    body,
    type: 'footnoteDefinition',
    rawHtmlIsText: false,
  },);
}

/**
 Where a text holds a footnote definition once the page has it, in text order: the page writes every less-than sign of an
 untranslated original as text, so a line of raw HTML is a line of the paragraph above it and the footnote goes on.

 @param body - text, without front matter and a leading byte order mark

 @returns Start and end of each definition, the end after the last character of its last block, nested ones included

 @example
 ```ts
 const footnotes = pageFootnoteDefinitionRangesOf({ body: '[^1]: x\n<q>\n', },);
 ```
 */
export function pageFootnoteDefinitionRangesOf({ body, }: { readonly body: string; },): readonly BlockRange[] {
  if (!body.includes('[^',))
    return [];
  return rangesOfType({
    body,
    type: 'footnoteDefinition',
    rawHtmlIsText: true,
  },);
}

//endregion GFM block ranges
