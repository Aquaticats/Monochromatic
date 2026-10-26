import remarkGfm from 'remark-gfm';
import {
  type Processor,
  unified,
} from 'unified';

//region With tables and strikethrough
// A PARSE THAT READS A TABLE AND STRIKETHROUGH AND NOTHING ELSE OF GFM. The plain reading (`parse-markdown-math.ts`) must
// give every node a position and must read `[^1]: x` as a link reference definition, which the strict grammar's footnotes
// and the autolink literals of GFM both change: a footnote definition is a footnote there, and the autolink-literal
// transform replaces a text node with nodes that carry no position. A table and strikethrough change neither, and a reader
// of the source needs them: a cell ends at its row, so a span that a row opens pairs with nothing in the row after it, and
// a line that starts a block of raw HTML ends the table; text between tildes is a node of its own that a line ending
// inside it does not cut.
//
// WHERE THE EXTENSIONS COME FROM. `remark-gfm@4.0.1` (`lib/index.js`) registers one combined extension for `micromark` and one
// list of extensions for `mdast-util-from-markdown`, the second of five: autolink literals, footnotes, strikethrough, tables
// and task list items. The packages of the single parts are not dependencies of this package, so the extensions are taken
// from a processor that uses `remark-gfm`, and what the plain reading does not have is turned off: the constructs of
// autolink literals, footnotes and task list items by name in the `micromark` extension (as `without-raw-html.ts` turns off
// raw HTML), and the autolink-literal extension of `mdast-util-from-markdown` altogether, which carries the transform that
// builds nodes without a position. The footnote and task list extensions of `mdast-util-from-markdown` stay: with their
// constructs off, no token reaches their handlers.

/**
 The constructs of the combined `micromark` extension that the plain reading does not have: the three autolink literals, the
 footnote definition and call, and the task list check.
 */
const OFF_CONSTRUCTS: readonly string[] = [
  'wwwAutolink',
  'protocolAutolink',
  'emailAutolink',
  'gfmFootnoteDefinition',
  'gfmFootnoteCall',
  'gfmPotentialFootnoteCall',
  'tasklistCheck',
];

/**
 What `remark-gfm` registers with a parser: the data of a processor that uses it, read once.
 */
const GFM_DATA = unified()
  .use(remarkGfm,)
  .freeze()
  .data();

/**
 Makes the plugin that adds GFM tables and strikethrough to the parser it is used with, and no other part of GFM.

 @returns Plugin for the parser

 @example
 ```ts
 const parser = unified().use(remarkParse,).use(withTablesAndStrikethrough(),);
 ```
 */
export function withTablesAndStrikethrough(): (this: Processor) => undefined {
  return function addExtensions(this: Processor,): undefined {
    /**
     The parser's data, in which the extensions of the `micromark` parse and of the tree it builds are listed.
     */
    const data = this.data();
    data.micromarkExtensions = [
      ...(data.micromarkExtensions ?? []),
      ...(GFM_DATA.micromarkExtensions ?? []),
      { disable: { null: [...OFF_CONSTRUCTS,], }, },
    ];
    /**
     The extensions of `remark-gfm` for the tree, the list of its parts flattened.
     */
    const gfmExtensions = (GFM_DATA.fromMarkdownExtensions ?? []).flat();
    /**
     Those that carry no transform: the autolink-literal extension carries the one that builds nodes without a position.
     */
    const kept: typeof gfmExtensions = [];
    for (const extension of gfmExtensions) {
      if (!('transforms' in extension))
        kept.push(extension,);
    }
    data.fromMarkdownExtensions = [
      ...(data.fromMarkdownExtensions ?? []),
      ...kept,
    ];
  };
}

//endregion With tables and strikethrough
