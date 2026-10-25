import type { Root, } from 'mdast';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import { unified, } from 'unified';

import {
  firstNestingExcess,
  isStackOverflow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { MarkdownParseError, } from './parse-mdx.ts';
import { withTablesAndStrikethrough, } from './with-tables-and-strikethrough.ts';

//region Plain reading with the site's math
// THE READING A CALLER COPIES OR REWRITES A TEXT BY, apart from the strict grammar the pages are admitted under.
// CommonMark with `remark-math` and the tables and the strikethrough of GFM, and nothing else: no MDX extensions, so no text
// is refused for its syntax; no other GFM, since its autolink-literal transform replaces a text node with nodes that
// carry no position (a body ending in `www.` shows it), and a caller that edits the raw text by node positions cannot
// place those, and its footnotes turn a line like `[^1]: {` into a footnote where this reading has a link reference
// definition, which the callers rely on. A table is read because a cell ends at its row: a span that a row opens pairs
// with nothing in the row after it, and a line that starts a block of raw HTML ends the table, which a reading without
// tables gets wrong; strikethrough is read because its node holds the text between the tildes whole across a line
// (`with-tables-and-strikethrough.ts` says how the two are added). The site compiles with `remark-math` and without
// GFM (`parse-mdx.ts` says why the strict grammar here turns GFM on all the same), so a page is read by the strict
// grammar with all of GFM and the source by this reading, which differs from a reader of GFM in the footnotes, the
// autolink literals and the task lists only.

/**
 Parses body text as CommonMark with the site's math and the tables and strikethrough of GFM, the reading a caller takes
 positions from.

 NESTED PAST THE BOUND IS REFUSED BEFORE THE PARSER READS IT, as {@link parseMarkdownBody} does and for the same
 reason: the outcome of a deeply nested text must not depend on how much stack the caller has left.

 @param body - markdown source with front matter split away and no leading byte order mark; a leading mark shifts
 every position by one, which the parser drops from the input and this does not add back

 @returns mdast root whose nodes all carry positions, as offsets in UTF-16 units of the body

 @throws {@link MarkdownParseError} when the body nests past the bound or nesting exhausts the parser's stack

 @example
 ```ts
 const root = parseMarkdownMathBody({ body: '猫 $a < b$ 走\n', },);
 ```
 */
export function parseMarkdownMathBody({ body, }: { readonly body: string; },): Root {
  /**
   Where the body passes the nesting bound, when it does.
   */
  const nesting = firstNestingExcess({
    body,
    grammar: 'markdown',
  },);
  if (nesting.kind === 'beyond')
    throw new MarkdownParseError({ reading: nesting, },);
  try {
    return unified()
      .use(remarkParse,)
      .use(remarkMath,)
      .use(withTablesAndStrikethrough(),)
      .parse(body,);
  }
  catch (error) {
    // Only the engine's stack exhaustion is this reading's refusal; any other failure is an unexpected state that
    // must keep propagating.
    if (!isStackOverflow(error,))
      throw error;
    throw new MarkdownParseError({
      reading: 'stack-exhausted',
      cause: error,
    },);
  }
}

//endregion Plain reading with the site's math
