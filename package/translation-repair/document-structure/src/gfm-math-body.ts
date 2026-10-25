import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import { unified, } from 'unified';

import type { parseMarkdownMathBody, } from './parse-markdown-math.ts';

//region GFM and math reading
// WHAT A READER OF PLAIN MARKDOWN SEES IN THE FLAVOUR THE PACKAGE PARSES PAGES IN
// (CommonMark with GFM and the site's math, no MDX): the footnotes, tables and autolink literals of GFM and the math of
// `remark-math`, and none of the MDX extensions, so no text is refused for its syntax. A suite that puts a source text
// beside the page carrying it reads the source through this, because the page is read with footnotes and math on, and a
// source read without them (`parseMarkdownMathBody` has no GFM, so a footnote definition made of one address is a link
// reference definition there) would differ from the page where the two are the same text. The page check reads the
// source through this to say whether the page shows what the source shows (`source-reading-difference.ts` in
// `page-check`).

/**
 The tree the package's plain reading returns, which this reading returns too.
 */
type PlainRoot = ReturnType<typeof parseMarkdownMathBody>;

/**
 Whether a parse returned a root node, the one thing the parser's own types promise
 and the built types of the package spell differently.

 @param value - what the parser returned

 @returns Whether it is a root node with children

 @example
 ```ts
 const isTree = isRoot(parsed,);
 ```
 */
function isRoot(value: unknown,): value is PlainRoot {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  if (!('type' in value))
    return false;
  if (value.type !== 'root')
    return false;
  return ('children' in value) && Array.isArray(value.children,);
}

/**
 Parses body text as CommonMark with GFM and the site's math.

 @param body - markdown source with front matter split away

 @returns mdast root; the GFM autolink-literal transform builds nodes without a position, so a caller that edits the
 text by node positions reads through `parseMarkdownMathBody` instead

 @throws Error when the parser returns something that is not a root node, which its own types rule out

 @example
 ```ts
 const root = parseGfmMathBody({ body: '猫[^1] $a < b$\n\n[^1]: 注\n', },);
 ```
 */
export function parseGfmMathBody(
  { body, }: { readonly body: string; },
): PlainRoot {
  /**
   What the parser returned, before it is known to be a root.
   */
  const parsed: unknown = unified()
    .use(remarkParse,)
    .use(remarkGfm,)
    .use(remarkMath,)
    .parse(body,);
  if (!isRoot(parsed,))
    throw new Error('the parser returned something that is not a root node',);
  return parsed;
}

//endregion GFM and math reading
