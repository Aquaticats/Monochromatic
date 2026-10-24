import {
  maskHtmlComments,
  parseMdxBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure';

//region Live constructs of a page
// WHAT THE STRICT MDX GRAMMAR TAKES FOR CODE IN A PAGE: an expression (`{b}`), an ES module (`import a from 'b'`), and a
// JSX element with its attributes (`<a href="x">`). A page that carries an original as literal text must hold none of
// them where the original had none, since the site compiles the page and runs them. The original is read as the
// package reads Markdown pages (CommonMark with GFM and the site's math, no MDX), which builds none of these nodes, so
// every one the strict grammar builds in the text of an encoded original is one the original lacks.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 The note a run of untranslated slices carries, which the shipped page puts before the original.
 */
const NOTE = '> This passage has not been translated yet. The original text follows.';

/**
 Node types the strict grammar builds for code: expressions, JSX elements and ES modules. An attribute is a part of its
 element and is not a node of the tree the walk reads.
 */
const LIVE_TYPES: ReadonlySet<string> = new Set([
  'mdxFlowExpression',
  'mdxTextExpression',
  'mdxJsxFlowElement',
  'mdxJsxTextElement',
  'mdxjsEsm',
],);

/**
 A construct of a page the strict grammar takes for code.
 */
type LiveConstruct = {
  /**
   Type of the node the grammar built.
   */
  readonly type: string;

  /**
   Index of its first character in the page text, in UTF-16 units.
   */
  readonly start: number;

  /**
   Index after its last character.
   */
  readonly end: number;
};

/**
 Finds every construct of a page the strict grammar takes for code.

 @param pageText - page text, read as the package reads a page: comments masked, then the strict grammar

 @returns Constructs in text order, none for a page whose text is all text

 @throws {@link Error} whatever the strict grammar raises for a page it refuses

 @example
 ```ts
 const live = liveConstructsOf({ pageText: 'A cat {b}.\n', },);
 ```
 */
export function liveConstructsOf({ pageText, }: { readonly pageText: string; },): readonly LiveConstruct[] {
  /**
   The page as the package reads it.
   */
  const { masked, } = maskHtmlComments({ text: pageText, },);
  /**
   Constructs found.
   */
  const found: LiveConstruct[] = [];
  /**
   Nodes still to read, the next one on top.
   */
  const pending: TreeNode[] = [parseMdxBody({ body: masked, },),];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (LIVE_TYPES.has(node.type,) && (node.position !== undefined)) {
      /**
       Where the node starts and ends.
       */
      const {
        start,
        end,
      } = node.position;
      found.push({
        type: node.type,
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
 Finds every construct the strict grammar takes for code in a text the page carries after the note.

 @param carried - text as the page carries it, encoded or as written

 @returns Constructs in the page text, which holds the note and a blank line before the text

 @throws {@link Error} whatever the strict grammar raises for a page it refuses

 @example
 ```ts
 const live = liveConstructsOfCarried({ carried: String.raw`她说 \{ 然后走了。`, },);
 ```
 */
export function liveConstructsOfCarried({ carried, }: { readonly carried: string; },): readonly LiveConstruct[] {
  return liveConstructsOf({ pageText: `${NOTE}\n\n${carried}\n`, },);
}

//endregion Live constructs of a page
