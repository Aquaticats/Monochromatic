import {
  commentRegionsOf,
  maskHtmlComments,
  parseGfmMathBody,
  parseMdxBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { blanking, } from './source-ships-readable.ts';
import type { TextRange, } from './source-ships-ranges.ts';
import { UNTRANSLATED_PASSAGE_NOTE, } from './source-ships.ts';

//region Reading account of a source run
// WHAT A READER OF A PAGE SEES OF A TEXT, put side by side for the text as the artifact records it and the text as the
// page carries it. A page is read the way the package admits it (HTML comments masked, then the strict MDX grammar),
// the source the way any Markdown reader would read it in the flavour the package parses pages in (CommonMark with GFM
// and the site's math, no MDX; `parseGfmMathBody` in `document-structure`), and both are reduced to the
// same account: the text of every leaf, with a link's target marked where the link stands and the formula of a math
// node as it is. The account of a page that holds a construct the strict grammar reads as syntax (an expression, a tag)
// is refused rather than guessed at, since such a page does not show the source text. It moved out of the test fixture
// `source-reader-text.test-fixture.ts` on 2026-10-25, when the page check began to compare the two readings.

/**
 The note a run of untranslated slices carries, which the shipped page puts before the original.
 */
const NOTE = UNTRANSLATED_PASSAGE_NOTE;

/**
 Refusal when the page holds a construct the strict grammar reads as syntax, which shows a reader other than the text.

 @example
 ```ts
 throw new PageSyntaxError({ type: 'mdxTextExpression', },);
 ```
 */
export class PageSyntaxError extends Error {
  /**
   Message names the node type and nothing of the page.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param type - type of the node the strict grammar built
   */
  public constructor({ type, }: { readonly type: string; },) {
    super(`the page holds a ${type} node, which the strict grammar reads as syntax`,);
    this.name = 'PageSyntaxError';
  }
}

/**
 The byte order mark a text may open with, which a parser drops from the start of its input.
 */
const BYTE_ORDER_MARK = '\uFEFF';

/**
 Node types whose value is what a reader sees.
 */
const VALUE_TYPES: ReadonlySet<string> = new Set([
  'text',
  'inlineCode',
  'code',
  'html',
],);

/**
 Node types the strict grammar builds for syntax, which show the reader something other than the source text.
 */
const SYNTAX_TYPES: ReadonlySet<string> = new Set([
  'mdxFlowExpression',
  'mdxTextExpression',
  'mdxJsxFlowElement',
  'mdxJsxTextElement',
  'mdxjsEsm',
],);

/**
 Node types whose children are blocks, which a reader sees on lines of their own whether the block is a paragraph or
 raw HTML.
 */
const BLOCK_PARENTS: ReadonlySet<string> = new Set([
  'root',
  'blockquote',
  'list',
  'listItem',
  'footnoteDefinition',
],);

/**
 The text a reader sees in the children of a node.

 @param node - node with children

 @returns Leaves of the children, in order

 @example
 ```ts
 const seen = childrenLeaves({ node: root, },);
 ```
 */
function childrenLeaves({ node, }: { readonly node: TreeNode; },): string {
  if (!('children' in node))
    return '';
  return node.children
    .map(function leaves(child,): string {
      return leavesOf({ node: child, },);
    },)
    .join(BLOCK_PARENTS.has(node.type,) ? '\n' : '',);
}

/**
 The text a reader sees at one node and in its descendants.

 @param node - node of a tree read by either grammar

 @returns Text of the leaves, a link's target in marks, a footnote's label in brackets

 @throws {@link PageSyntaxError} for a node the strict grammar reads as syntax

 @example
 ```ts
 const seen = leavesOf({ node: root, },);
 ```
 */
function leavesOf({ node, }: { readonly node: TreeNode; },): string {
  /**
   The node's type as a plain string, since the math nodes are not in the type vocabulary.
   */
  const type: string = node.type;
  if (SYNTAX_TYPES.has(type,))
    throw new PageSyntaxError({ type, },);
  if (VALUE_TYPES.has(type,) && ('value' in node))
    return node.value;
  if (((type === 'inlineMath') || (type === 'math')) && ('value' in node))
    return node.value;
  if (node.type === 'image')
    return node.alt ?? '';
  if (node.type === 'break')
    return '\n';
  if (node.type === 'footnoteReference')
    return `[^${node.identifier}]`;
  if (node.type === 'link')
    return `⟦${decodeURI(node.url,)}⟧${childrenLeaves({ node, },)}`;
  if (node.type === 'footnoteDefinition')
    return `[^${node.identifier}]: ${childrenLeaves({ node, },)}\n`;
  return `${childrenLeaves({ node, },)}\n`;
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
function accountOf({ root, }: { readonly root: TreeNode; },): string {
  return leavesOf({ node: root, },)
    .split('',)
    .map(function spaced(character,): string {
      return (character.trim() === '') ? ' ' : character;
    },)
    .join('',)
    .split(' ',)
    .filter(function isWord(word,): boolean {
      return word !== '';
    },)
    .join(' ',);
}

/**
 The tree of the page that carries the text after the note, less the note.

 @param carried - text as the page carries it, encoded or as written

 @returns Root holding everything the page has after the note

 @throws Whatever the strict grammar raises when the page does not parse

 @example
 ```ts
 const root = pageRootOf({ carried: String.raw`她说 \{ 然后走了。`, },);
 ```
 */
function pageRootOf({ carried, }: { readonly carried: string; },): TreeNode {
  /**
   The page as the package reads it, comments masked.
   */
  const { masked, } = maskHtmlComments({ text: `${NOTE}\n\n${carried}\n`, },);
  /**
   Tree of the strict grammar.
   */
  const root = parseMdxBody({ body: masked, },);
  /**
   Everything the page holds after the note.
   */
  const [, ...afterNote] = root.children;
  return {
    ...root,
    children: afterNote,
  };
}

/**
 What a reader sees of the page that carries the text after the note.

 @param carried - text as the page carries it, encoded or as written

 @returns Account of everything after the note

 @throws {@link PageSyntaxError} when the page holds syntax the reader would not see as text, and whatever the strict
 grammar raises when the page does not parse

 @example
 ```ts
 const seen = readerOfPage({ carried: String.raw`她说 \{ 然后走了。`, },);
 ```
 */
export function readerOfPage({ carried, }: { readonly carried: string; },): string {
  return accountOf({ root: pageRootOf({ carried, },), },);
}

/**
 What a reader sees of each top-level block of the page that carries the text after the note.

 @param carried - text as the page carries it, encoded or as written

 @returns Account of each block after the note, in order

 @throws {@link PageSyntaxError} when the page holds syntax the reader would not see as text, and whatever the strict
 grammar raises when the page does not parse

 @example
 ```ts
 const seen = blocksOfPage({ carried: String.raw`她说 \{ 然后走了。`, },);
 ```
 */
export function blocksOfPage({ carried, }: { readonly carried: string; },): readonly string[] {
  /**
   The tree after the note.
   */
  const root = pageRootOf({ carried, },);
  if (!('children' in root))
    return [];
  return root.children
    .map(function accountOfBlock(child,): string {
      return accountOf({ root: child, },);
    },);
}

/**
 The tree a reader of plain Markdown builds of the text as the artifact records it, and how far its offsets are from the
 text's.

 @param source - text as the artifact records it

 @returns Root of the tree, and the number of characters its offsets run ahead of the text's: none, except for a text
 that opens with a byte order mark, which is read after the note, as the page carries it, since a parser drops a mark that
 opens its input and reads the first line as the start of the document, where the page has it in the middle

 @example
 ```ts
 const { root, shift, } = sourceRootOf({ source: '她说 { 然后走了。', },);
 ```
 */
function sourceRootOf({ source, }: { readonly source: string; },): {
  readonly root: TreeNode;
  readonly shift: number
} {
  /**
   The comments of the text.
   */
  const regions = commentRegionsOf({ text: source, },);
  /**
   Whether every comment of the text closes, which the page's reading masks.
   */
  const allClose = regions.every(function closes({ terminated, },): boolean {
    return terminated;
  },);
  /**
   The text with its comments masked.
   */
  const { masked, } = maskHtmlComments({ text: source, },);
  /**
   The text the plain grammar is given: masked as the page's reading masks it when every comment closes; where one does
   not, the comments that close are blanked and that one is left as it stands, since the plain grammar reads it as raw HTML
   to the end of the text.
   */
  const readable = allClose
    ? masked
    : blanking({
      text: source,
      ranges: regions
        .filter(function closes({ terminated, },): boolean {
          return terminated;
        },)
        .map(function toRange(region,): TextRange {
          return {
            start: region.startOffset,
            end: region.endOffset,
          };
        },),
    },);
  if (readable.startsWith(BYTE_ORDER_MARK,)) {
    /**
     The note and the blank line after it, which the page holds before the text.
     */
    const before = `${NOTE}\n\n`;
    /**
     The note and the text, read as the page holds them.
     */
    const {
      children,
      ...page
    } = parseGfmMathBody({ body: `${before}${readable}`, },);
    return {
      root: {
        ...page,
        children: children.slice(1,),
      },
      shift: before.length,
    };
  }
  return {
    root: parseGfmMathBody({ body: readable, },),
    shift: 0,
  };
}

/**
 What a reader of plain Markdown sees of the text as the artifact records it.

 @param source - text as the artifact records it

 @returns Account of the text, its closed HTML comments blanked the way the page's reading blanks them; a text that
 opens with a byte order mark is read after the note, as the page carries it

 @example
 ```ts
 const seen = readerOfSource({ source: '她说 { 然后走了。', },);
 ```
 */
export function readerOfSource({ source, }: { readonly source: string; },): string {
  return accountOf({ root: sourceRootOf({ source, },)
    .root, },);
}

/**
 One top-level block of a text as a reader of plain Markdown sees it: what the block shows, and where it stands.
 */
export type SourceBlock = {
  /**
   What the block shows, in the form the two readings are compared by.
   */
  readonly account: string;

  /**
   Offset in the text of the block's first character.
   */
  readonly start: number;

  /**
   Offset in the text just past the block's last character.
   */
  readonly end: number;
};

/**
 What a reader of plain Markdown sees of each top-level block of the text as the artifact records it.

 @param source - text as the artifact records it

 @returns Blocks in order, each with the offsets it stands at in the text; a block the parse gave no position stands
 at the start of the text and ends there

 @example
 ```ts
 const blocks = blocksOfSource({ source: '她说 { 然后走了。', },);
 ```
 */
export function blocksOfSource({ source, }: { readonly source: string; },): readonly SourceBlock[] {
  /**
   The tree, and how far its offsets run ahead of the text's.
   */
  const {
    root,
    shift,
  } = sourceRootOf({ source, },);
  if (!('children' in root))
    return [];
  return root.children
    .map(function blockOf(child,): SourceBlock {
      return {
        account: accountOf({ root: child, },),
        start: Math.max(
          0,
          (child.position
            ?.start
            .offset
            ?? shift) - shift,
        ),
        end: Math.max(
          0,
          (child.position
            ?.end
            .offset
            ?? shift) - shift,
        ),
      };
    },);
}

/**
 An account with the marks of the links' targets taken out, so that two accounts compare by their text alone.

 A raw HTML block and indented code are text to the strict grammar, whose autolink literals turn an address in it into a
 link where a reader of plain Markdown sees the raw characters; the text they show is the same.

 @param account - account of a page or of a source

 @returns Account without the mark and the target of each link

 @example
 ```ts
 const text = withoutLinkTargets({ account: '⟦u⟧见', },);
 ```
 */
export function withoutLinkTargets({ account, }: { readonly account: string; },): string {
  /**
   Characters kept, and whether the one read is inside a target's marks.
   */
  const kept: string[] = [];
  /**
   Whether the character read stands inside a target's marks.
   */
  const target = { inside: false, };
  for (const character of account) {
    if (character === '⟦') {
      target.inside = true;
      continue;
    }
    if (character === '⟧') {
      target.inside = false;
      continue;
    }
    if (!target.inside)
      kept.push(character,);
  }
  return kept.join('',);
}

//endregion Reading account of a source run
