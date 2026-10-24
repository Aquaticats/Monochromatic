import type { Nodes, } from 'mdast';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import { unified, } from 'unified';

import {
  firstNestingExcess,
  isStackOverflow,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Table ranges
// WHERE THE STRICT GRAMMAR READS A TABLE, read off a parse with GFM on. A table splits each row into cells at every
// pipe that is not written with a backslash, whatever the pipe stands in (a code span, the address of an autolink), so
// text a caller reads without GFM as one construct can be two cells to the grammar the pages are admitted under. The
// plain reading (`parse-markdown-math.ts`) has no GFM, so this is how a caller learns which of its nodes a table cuts.

/**
 A stretch of a text, as offsets in UTF-16 units.
 */
export type TableRange = {
  readonly start: number;
  readonly end: number;
};

/**
 Where a text holds a table, in text order.

 A TEXT NESTED PAST THE BOUND IS READ AS HOLDING NONE, as {@link moduleLineStartsOf} reads one: the strict grammar
 refuses it before it reads a line.

 @param body - text, without front matter and a leading byte order mark

 @returns Start and end of each table, the end after its last row, nested tables included

 @example
 ```ts
 const tables = tableRangesOf({ body: '| a | b |\n| - | - |\n| c | d |\n', },);
 ```
 */
export function tableRangesOf({ body, }: { readonly body: string; },): readonly TableRange[] {
  if (!body.includes('|',))
    return [];
  if (firstNestingExcess({
    body,
    grammar: 'markdown',
  },)
    .kind
    === 'beyond') {
    return [];
  }
  /**
   Tables found, in the order the walk reaches them.
   */
  const tables: TableRange[] = [];
  /**
   Nodes still to walk, the next one on top.
   */
  const pending: Nodes[] = [];
  try {
    pending.push(unified()
      .use(remarkParse,)
      .use(remarkGfm,)
      .use(remarkMath,)
      .parse(body,),);
  }
  catch (error) {
    // Only the engine's stack exhaustion is the refusal the strict grammar gives such a text; any other failure is an
    // unexpected state that must keep propagating.
    if (!isStackOverflow(error,))
      throw error;
    return [];
  }
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if ((node.type === 'table') && (node.position !== undefined)) {
      /**
       Where the table starts and ends.
       */
      const {
        start,
        end,
      } = node.position;
      tables.push({
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
  return tables.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
}

//endregion Table ranges
