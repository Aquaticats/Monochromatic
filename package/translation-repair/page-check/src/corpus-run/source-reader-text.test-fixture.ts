import type {
  Nodes,
  Root,
} from 'mdast';
import {
  commentRegionsOf,
  maskHtmlComments,
  parseMarkdownBody,
  parseMdxBody,
} from '@monochromatic-dev/translation-repair-document-structure';

//region Reader text of a source run
// WHAT A READER OF A PAGE SEES OF A TEXT, put side by side for the text as the artifact records it and the text as the
// page carries it. A page is read the way the package admits it (HTML comments masked, then the strict MDX grammar),
// the source the way any Markdown reader would read it (the plain grammar), and both are reduced to the same account:
// the text of every leaf, with a link's target marked where the link stands. The account of a page that holds a
// construct the strict grammar reads as syntax (an expression, a tag) is refused rather than guessed at, since such a
// page does not show the source text.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 The note a run of untranslated slices carries, which the shipped page puts before the original.
 */
const NOTE = '> This passage has not been translated yet. The original text follows.';

/**
 Nodes whose children are blocks, which a reader sees on lines of their own whether the block is a paragraph or raw HTML.
 */
const BLOCK_PARENTS: ReadonlySet<string> = new Set(['root', 'blockquote', 'list', 'listItem', 'footnoteDefinition',],);

/**
 Runs of whitespace, which two readings may break differently without a reader seeing the difference.
 */
const WHITE_SPACE = /\s+/gv;

/**
 The text a reader sees at one node and in its descendants.

 @param node - node of a tree read by either grammar

 @returns Text of the leaves, a link's target in marks, a footnote's label in brackets

 @throws {@link Error} for a node the strict grammar reads as syntax

 @example
 ```ts
 const seen = leavesOf({ node: root, },);
 ```
 */
function leavesOf({ node, }: { readonly node: Nodes; },): string {
  switch (node.type) {
    case 'text':
    case 'inlineCode':
    case 'code':
    case 'math':
    case 'inlineMath':
    case 'html':
      return node.value;
    case 'image':
      return node.alt ?? '';
    case 'break':
      return '\n';
    case 'footnoteReference':
      return `[^${node.identifier}]`;
    case 'link':
      return `⟦${decodeURI(node.url,)}⟧${childrenLeaves({ node, },)}`;
    case 'footnoteDefinition':
      return `[^${node.identifier}]:${childrenLeaves({ node, },)}\n`;
    case 'paragraph':
    case 'heading':
    case 'listItem':
    case 'tableRow':
      return `${childrenLeaves({ node, },)}\n`;
    case 'mdxFlowExpression':
    case 'mdxTextExpression':
    case 'mdxJsxFlowElement':
    case 'mdxJsxTextElement':
    case 'mdxjsEsm':
      throw new Error(`the page holds a ${node.type} node, which the strict grammar reads as syntax`,);
    default:
      return ('children' in node) ? childrenLeaves({ node, },) : '';
  }
}

/**
 The text a reader sees in the children of a node.

 @param node - node with children

 @returns The leaves of the children, in order

 @example
 ```ts
 const seen = childrenLeaves({ node: root, },);
 ```
 */
function childrenLeaves({ node, }: { readonly node: Nodes & { readonly children: readonly Nodes[]; }; },): string {
  return node.children
    .map(function leaves(child,): string {
      return leavesOf({ node: child, },);
    },)
    .join(BLOCK_PARENTS.has(node.type,) ? '\n' : '',);
}

/**
 Reduces a tree to the account the two readings are compared by.

 @param root - tree to reduce

 @returns Leaves with each run of white space as one space and none at the ends

 @example
 ```ts
 const account = accountOf({ root, },);
 ```
 */
function accountOf({ root, }: { readonly root: Root; },): string {
  return leavesOf({ node: root, },)
    .replaceAll(
      WHITE_SPACE,
      ' ',
    )
    .trim();
}

/**
 What a reader sees of the page that carries the text after the note.

 @param carried - text as the page carries it, encoded or as written

 @returns Account of everything after the note

 @throws {@link Error} when the page holds syntax the reader would not see as text, and whatever the strict grammar
 raises when the page does not parse

 @example
 ```ts
 const seen = readerOfPage({ carried: String.raw`她说 \{ 然后走了。`, },);
 ```
 */
export function readerOfPage({ carried, }: { readonly carried: string; },): string {
  /**
   The page as the package reads it, comments masked.
   */
  const { masked, } = maskHtmlComments({ text: `${NOTE}\n\n${carried}\n`, },);
  /**
   Tree of the strict grammar.
   */
  const root = parseMdxBody({ body: masked, },);
  return accountOf({
    root: {
      ...root,
      children: root.children.slice(1,),
    },
  },);
}

/**
 What a reader of plain Markdown sees of the text as the artifact records it.

 @param source - text as the artifact records it

 @returns Account of the text, its closed HTML comments blanked the way the page's reading blanks them

 @example
 ```ts
 const seen = readerOfSource({ source: '她说 { 然后走了。', },);
 ```
 */
export function readerOfSource({ source, }: { readonly source: string; },): string {
  /**
   The text with each comment that closes blanked, a comment that never closes left as the plain grammar reads it.
   */
  const blanked = commentRegionsOf({ text: source, },)
    .filter(function closed({ terminated, },): boolean {
      return terminated;
    },)
    .reduceRight(
      function blank(
        text,
        { startOffset, endOffset, },
      ): string {
        return `${text.slice(0, startOffset,)}${
          text.slice(
            startOffset,
            endOffset,
          ).replaceAll(
            /[^\n]/gv,
            ' ',
          )
        }${text.slice(endOffset,)}`;
      },
      source,
    );
  return accountOf({ root: parseMarkdownBody({ body: blanked, },), },);
}

//endregion Reader text of a source run
