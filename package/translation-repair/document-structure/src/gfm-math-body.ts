import type { Root, } from 'mdast';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import { unified, } from 'unified';


//region GFM and math reading
// WHAT A READER OF PLAIN MARKDOWN SEES IN THE FLAVOUR THE PACKAGE PARSES PAGES IN
// (CommonMark with GFM and the site's math, no MDX): the footnotes, tables and autolink literals of GFM and the math of
// `remark-math`, and none of the MDX extensions, so no text is refused for its syntax. A suite that puts a source text
// beside the page carrying it reads the source through this, because the page is read with footnotes and math on, and a
// source read without them (`parseMarkdownMathBody` has no GFM footnotes, so a footnote definition made of one address is a link
// reference definition there) would differ from the page where the two are the same text. The page check reads the
// source through this to say whether the page shows what the source shows (`source-reading-difference.ts` in
// `page-check`).

/**
 Parses body text as CommonMark with GFM and the site's math.

 @param body - markdown source with front matter split away

 @returns mdast root; the GFM autolink-literal transform builds nodes without a position, so a caller that edits the
 text by node positions reads through `parseMarkdownMathBody` instead

 @example
 ```ts
 const root = parseGfmMathBody({ body: '猫[^1] $a < b$\n\n[^1]: 注\n', },);
 ```
 */
export function parseGfmMathBody(
  { body, }: { readonly body: string; },
): Root {
  return unified()
    .use(remarkParse,)
    .use(remarkGfm,)
    .use(remarkMath,)
    .parse(body,);
}

//endregion GFM and math reading
