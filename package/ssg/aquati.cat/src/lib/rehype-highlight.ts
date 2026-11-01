/**
 Rehype plugin that pre-computes Lezer syntax highlight ranges at build time.
 
 Visits `<pre><code class="language-*">` hast nodes, parses their text content
 with the matching Lezer parser, and embeds highlight offset ranges as
 `data-hl-<group>` attributes. The client reads these attributes to create
 DOM Range objects for the CSS Custom Highlight API without shipping any
 Lezer code to the browser.
 
 Each highlight group gets its own attribute (e.g. `data-hl-keyword="0-5;15-21"`)
 with semicolon-separated `from-to` offset pairs. Groups with no tokens
 are omitted entirely.
 
 @example
 ```ts
 import rehypeHighlight from './rehype-highlight.ts';
 unified()
   .use(remarkParse)
   .use(remarkRehype)
   .use(rehypeHighlight)
   .use(rehypeStringify);
 ```
 */

import type { Parser, } from '@lezer/common';
import { parser as cssParser, } from '@lezer/css';
import { highlightTree, } from '@lezer/highlight';
import { parser as htmlParser, } from '@lezer/html';
import { parser as jsParser, } from '@lezer/javascript';
import { parser as markdownParser, } from '@lezer/markdown';
import { parser as rustParser, } from '@lezer/rust';
import { parser as xmlParser, } from '@lezer/xml';
import { parser as yamlParser, } from '@lezer/yaml';

import type {
  Element,
  ElementContent,
  Root,
} from 'hast';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  HIGHLIGHT_GROUPS,
  ssgHighlighter,
} from '../client/tags.ts';

//region Parser configuration

/**
 TypeScript parser (JavaScript parser with TS dialect).
 */
const tsParser = jsParser.configure({ dialect: 'ts', },);

/**
 JSX parser (JavaScript parser with JSX dialect).
 */
const jsxParser = jsParser.configure({ dialect: 'jsx', },);

/**
 TSX parser (JavaScript parser with TS + JSX dialects).
 */
const tsxParser = jsParser.configure({ dialect: 'ts jsx', },);

/**
 Language name to Lezer parser mapping.
 
 Covers the same language aliases as the former client-side `PARSER_LOADERS`.
 */
const PARSERS: Readonly<Record<string, Parser>> = {
  js: jsParser,
  javascript: jsParser,
  ts: tsParser,
  typescript: tsParser,
  jsx: jsxParser,
  tsx: tsxParser,
  html: htmlParser,
  css: cssParser,
  md: markdownParser,
  markdown: markdownParser,
  mdx: markdownParser,
  xml: xmlParser,
  svg: xmlParser,
  yaml: yamlParser,
  yml: yamlParser,
  json: jsParser,
  rust: rustParser,
  rs: rustParser,
};

/**
 Prefix on code element class names identifying the language.
 */
const LANGUAGE_PREFIX = 'language-';

/**
 Sentinel returned by {@link getLanguage} when a code element carries no
 `language-*` class. A unique symbol rather than the empty string, so "no
 language" is never mistaken for a (nonsensical) empty language name.
 */
const NO_LANGUAGE: unique symbol = Symbol('code block language class missing',);

//endregion Parser configuration

//region Hast utilities

/**
 Recursively extracts text content from a hast node tree.
 
 @param node - hast content node to extract text from
 
 @returns concatenated text of all descendant text nodes
 */
function extractText(node: ForeignBorrowed<ElementContent>,): string {
  if (node.type
    === 'text')
    return node.value;
  if ('children' in node)
    return node.children
      .map(extractText,)
      .join('',);
  return '';
}

/**
 Extracts the language name from a code element's class list.
 
 @param codeElement - hast `<code>` element node
 
 @returns language name, or {@link NO_LANGUAGE} when no `language-*` class is found
 */
function getLanguage(codeElement: ForeignBorrowed<Element>,): string | typeof NO_LANGUAGE {
  /**
   Destructured class-list property; rehype puts the language as a `language-*` token here.
   */
  const { className, } = codeElement.properties;
  if (!Array.isArray(className,))
    return NO_LANGUAGE;
  for (const cls of className) {
    /**
     Per-iteration string cast since className entries may be numbers in the hast spec.
     */
    const name = cls;
    if (name.startsWith(LANGUAGE_PREFIX,))
      return name.slice(LANGUAGE_PREFIX.length,);
  }
  return NO_LANGUAGE;
}

//endregion Hast utilities

//region Highlight computation

/**
 Computes highlight offset pairs for a code block with {@link ssgHighlighter}
 and sets a `data-hl-<group>` attribute on the code element for every group
 in {@link HIGHLIGHT_GROUPS} that matched.
 
 @param codeElement - hast `<code>` element to annotate
 
 @param parser - Lezer parser for the code block's language
 
 @param text - plain text content of the code block
 */
function annotateCodeBlock({
  codeElement,
  parser,
  text,
}: {
  readonly codeElement: Element;
  readonly parser: Parser;
  readonly text: string;
},): void {
  /**
   Parsed syntax tree fed to the Lezer highlighter for offset-pair extraction.
   */
  const tree = parser.parse(text,);

  /**
   Accumulated `from-to` pairs per highlight group.
   */
  const pairsByGroup = new Map<string, string[]>();

  highlightTree(
    tree,
    ssgHighlighter,
    function collectPair(
      from: number,
      to: number,
      classes: string,
    ) {
      /**
       Existing pair list for this highlight class group, or a freshly inserted empty array.
       */
      const pairs = pairsByGroup.get(classes,)
        ?? (function initGroup(): string[] {
        /**
         Newly allocated pair list inserted into the shared map.
         */
          const fresh: string[] = [];
          pairsByGroup.set(
            classes,
            fresh,
          );
          return fresh;
        })();
      pairs.push(`${from}-${to}`,);
    },
  );

  for (const group of HIGHLIGHT_GROUPS) {
    /**
     Pairs for the current highlight group; undefined when no match was found.
     */
    const pairs = pairsByGroup.get(group,);
    if ((pairs !== undefined) && (pairs.length
      > 0))
      codeElement.properties[`data-hl-${group}`] = pairs.join(';',);
  }
}

/**
 Recursively visits hast element nodes, processing `<pre><code>` blocks.
 
 @param node - hast node to visit
 */
function visitNode(node: ForeignBorrowed<Root | Element>,): void {
  for (const child of node.children) {
    if (child.type
      !== 'element')
      continue;

    if (child.tagName
      === 'pre') {
      /**
       First child node expected to be the `<code>` block; processed when present.
       */
      const [firstChild,] = child.children;
      if (
        (firstChild !== undefined)
          && (firstChild.type
            === 'element')
          && (firstChild.tagName
            === 'code')
      ) {
        /**
         Language detected from the `<code>` class list, or {@link NO_LANGUAGE} to skip.
         */
        const lang = getLanguage(firstChild,);
        if (lang !== NO_LANGUAGE) {
          /**
           Lezer parser bound to the detected language, or undefined when unsupported.
           */
          const parser = PARSERS[lang];
          if (parser !== undefined) {
            /**
             Plain text extracted from the code element prior to highlighting.
             */
            const text = firstChild.children
              .map(extractText,)
              .join('',);
            if (text.length
              > 0) {
              annotateCodeBlock({
                codeElement: firstChild,
                parser,
                text,
              },);
            }
          }
        }
      }
    }

    visitNode(child,);
  }
}

//endregion Highlight computation

/**
 Rehype plugin that pre-computes Lezer syntax highlight ranges.
 
 Walks the tree with {@link visitNode}, adding `data-hl-<group>` attributes
 to `<code>` elements inside `<pre>` blocks, encoding token offset ranges as
 semicolon-separated `from-to` pairs.
 
 @returns hast tree transformer
 
 @example
 ```ts
 import rehypeHighlight from './rehype-highlight.ts';
 unified()
   .use(remarkParse)
   .use(remarkRehype)
   .use(rehypeHighlight)
   .use(rehypeStringify);
 ```
 */
export default function rehypeHighlight(): (tree: ForeignBorrowed<Root>,) => void {
  return function transform(tree: ForeignBorrowed<Root>,): void {
    visitNode(tree,);
  };
}
