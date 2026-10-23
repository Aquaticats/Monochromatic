import {
  commentRegionsOf,
  maskHtmlComments,
  parseMarkdownBody,
  parseMdxBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure';

//region Reader text of a source run
// WHAT A READER OF A PAGE SEES OF A TEXT, put side by side for the text as the artifact records it and the text as the
// page carries it. A page is read the way the package admits it (HTML comments masked, then the strict MDX grammar),
// the source the way any Markdown reader would read it (the plain grammar), and both are reduced to the same account:
// the text of every leaf, with a link's target marked where the link stands and math between its dollar signs. The
// account of a page that holds a construct the strict grammar reads as syntax (an expression, a tag) is refused rather
// than guessed at, since such a page does not show the source text.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 The note a run of untranslated slices carries, which the shipped page puts before the original.
 */
const NOTE = '> This passage has not been translated yet. The original text follows.';

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

 @returns The leaves of the children, in order

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

 @throws {@link Error} for a node the strict grammar reads as syntax

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
    throw new Error(`the page holds a ${type} node, which the strict grammar reads as syntax`,);
  if (VALUE_TYPES.has(type,) && ('value' in node))
    return node.value;
  if ((type === 'inlineMath') && ('value' in node))
    return `$${node.value}$`;
  if ((type === 'math') && ('value' in node))
    return `$$\n${node.value}\n$$`;
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
  /**
   Everything the page holds after the note.
   */
  const [, ...afterNote] = root.children;
  return accountOf({
    root: {
      ...root,
      children: afterNote,
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
   Whether every comment of the text closes, which the page's reading masks.
   */
  const allClose = commentRegionsOf({ text: source, },)
    .every(function closes({ terminated, },): boolean {
      return terminated;
    },);
  /**
   The text with its comments masked.
   */
  const { masked, } = maskHtmlComments({ text: source, },);
  /**
   The text the plain grammar is given: masked as the page's reading masks it when every comment closes, and untouched
   when one does not, since the plain grammar reads that comment as it stands.
   */
  const readable = allClose ? masked : source;
  return accountOf({ root: parseMarkdownBody({ body: readable, },), },);
}

//endregion Reader text of a source run
