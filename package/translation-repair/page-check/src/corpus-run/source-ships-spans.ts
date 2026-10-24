import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  endLineOf,
  footnoteBodyStart,
  isIndentedCode,
  isLazyDefinition,
  opensFootnote,
} from './source-ships-footnote.ts';
import {
  contentStretches,
  lineJoinsOf,
} from './source-ships-lines.ts';
import { autolinkWritten, } from './source-ships-links.ts';
import { footnoteBodySpans, } from './source-ships-nested.ts';
import {
  type FootnoteState,
  footnoteStateOf,
  type Pending,
  stateAfter,
  takeContinuation,
} from './source-ships-pending.ts';
import { fenceLengthEdits, } from './source-ships-fenced.ts';
import {
  closingBracket,
  FENCE_MARKS,
  imageTextRanges,
  indicesOf,
  rangeOf,
  secondLabel,
  type SourceSpans,
  startsAny,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

export {
  ESCAPABLE,
  type SourceSpans,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships spans
// WHERE IN A TEXT THE STRICT MDX GRAMMAR READS SYNTAX, read off a CommonMark parse of the same text
// (`document-structure/src/parse-markdown-math.ts`) by node positions. The parse is of the text itself and not of a
// guess at it: a node says whether the characters it spans are text a reader sees (a `text` node, a raw `html` node,
// indented code, which MDX has no such thing as, the alternative text of an image) or something the grammar reads as
// its own syntax (a fence, a code span, math, a link destination), and the second kind is never touched.
//
// WHAT THE PLAIN PARSE READS DIFFERENTLY FROM THE STRICT GRAMMAR, and how each is handled. GFM is not on in the plain
// parse (its autolink-literal transform builds nodes without a position), so a footnote definition made of one
// destination is a link reference definition there and a footnote to the strict grammar: its body is read as a text of
// its own (`nestedSpans`), as are the blocks indented under a footnote definition, which GFM takes for the blocks of
// the footnote and the plain parse for indented code. A label of a reference definition and of the references to it
// are written alike, so the reference still resolves. An angle autolink inside the text of a link cannot become a link
// of its own, which a link cannot hold: its address is written as text there, without the brackets, and the reader
// sees the address as the source shows it.

/**
 Node types whose children are blocks, among which a raw HTML node is a block of its own.
 */
const BLOCK_PARENTS: ReadonlySet<string> = new Set([
  'root',
  'blockquote',
  'listItem',
  'footnoteDefinition',
],);

/**
 Node types whose children are inline content, a run of which a label opened in one does not reach into the next.
 */
const INLINE_CONTENT: ReadonlySet<string> = new Set([
  'paragraph',
  'heading',
  'tableCell',
],);

/**
 Reads where a text holds syntax, from the parse of it.

 @param source - text, which edits and rewrites are made from

 @param root - parse of the text by the plain reading with the site's math

 @param readable - text as the parse read it, with its comments blanked, the source itself where none is; a block
 nested in another is parsed from this

 @returns Stretches a reader sees as text, the stretches rewritten whole and the links

 @throws {@link Error} when a node has no position, which the reading used never builds

 @example
 ```ts
 const spans = spansOf({ source, root: parseMarkdownMathBody({ body: source, },), },);
 ```
 */
export function spansOf(
  {
    source,
    root,
    readable = source,
  }: {
    readonly source: string;
    readonly root: TreeNode;
    readonly readable?: string;
  },
): SourceSpans {
  /**
   Stretches read as text.
   */
  const text: TextRange[] = [];
  /**
   Stretches of raw HTML blocks and indented code.
   */
  const literal: TextRange[] = [];
  /**
   Stretches rewritten.
   */
  const edits: TextEdit[] = [];
  /**
   Links.
   */
  const insideLinks: TextRange[] = [];
  /**
   Where each run of inline content starts.
   */
  const streamStarts: number[] = [];

  /**
   Adds what the parse of a nested block says of it.

   @param nested - spans of the block at the offsets of the whole text

   @example
   ```ts
   absorb({ nested: nestedSpans({ source, readable, indices, },), },);
   ```
   */
  function absorb({ nested, }: { readonly nested: SourceSpans; },): void {
    text.push(...nested.text,);
    literal.push(...nested.literal,);
    edits.push(...nested.edits,);
    insideLinks.push(...nested.insideLinks,);
    streamStarts.push(...nested.streamStarts,);
  }

  /**
   For each container, whether the block read last in it is part of a footnote definition, which blocks indented under
   it continue at the top, and the line it ends on.
   */
  const footnotes = new Map<TreeNode, FootnoteState>();
  /**
   Nodes still to read, the last of the root's children on top so they come out in order.
   */
  const pending: Pending[] = ('children' in root)
    ? root.children
      .map(function topLevel(node,): Pending {
        return {
          node,
          isRootChild: true,
          insideLink: false,
          parentType: 'root',
          parent: root,
        };
      },)
      .toReversed()
    : [];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    /**
     The node and where it stands.
     */
    const {
      node,
      isRootChild,
      insideLink,
      parentType,
      parent,
    } = next;
    /**
     Where its characters are.
     */
    const range = rangeOf({ node, },);
    /**
     What the block read last in its parent was.
     */
    const footnote = footnoteStateOf({
      footnotes,
      parent,
    },);
    // A DEFINITION LINE RIGHT AFTER A FOOTNOTE DEFINITION, OR A RAW HTML BLOCK OR INDENTED CODE, is a line of a
    // paragraph to the strict grammar, and a text, at the top and in a block quote or a list item.
    if (BLOCK_PARENTS.has(parentType,) && isLazyDefinition({
      node,
      footnoteOpen: footnote.open || footnote.endsInLiteral,
      previousEndLine: footnote.endLine,
    },)) {
      text.push(range,);
      footnotes.set(
        parent,
        {
          ...footnote,
          endLine: endLineOf({
            node,
            fallback: footnote.endLine,
          },),
        },
      );
      continue;
    }
    /**
     Whether the node is indented code under a footnote definition, which the
     strict grammar reads as the blocks of the definition.
     */
    const continuesFootnote = isRootChild
      && footnote.open
      && isIndentedCode({
        source,
        node,
        range,
      },);
    /**
     What the block read last in the parent is once this one is read.
     */
    const after = BLOCK_PARENTS.has(parentType,)
      ? stateAfter({
        before: footnote,
        source,
        node,
        range,
        continuesFootnote,
      },)
      : footnote;
    footnotes.set(
      parent,
      after,
    );
    // A PARAGRAPH THAT OPENS A FOOTNOTE DEFINITION AT THE TOP, AND BLOCKS INDENTED UNDER A DEFINITION, are read as the
    // body of the footnote is: the strict grammar reads the lines indented under it as its blocks, so a fence mark among
    // them opens a fence of the footnote, which the plain reading takes for a line of the paragraph.
    /**
     Whether the node is a paragraph at the top whose first line opens a footnote definition.
     */
    const opensTopFootnote = isRootChild
      && (node.type === 'paragraph')
      && opensFootnote({
        source,
        node,
        range,
      },);
    if (opensTopFootnote || continuesFootnote) {
      absorb({
        nested: footnoteBodySpans({
          source,
          readable,
          readSpans: spansOf,
          range: opensTopFootnote
            ? {
              start: footnoteBodyStart({
                source,
                range,
              },),
              end: range.end,
            }
            : range,
        },),
      },);
      continue;
    }
    edits.push(...lineJoinsOf({
      source,
      node,
      range,
    },),);
    // A TEXT, AND RAW HTML AMONG INLINE CONTENT, are what MDX reads a brace and a less-than sign in.
    if ((node.type === 'text') || ((node.type === 'html') && (!BLOCK_PARENTS.has(parentType,)))) {
      text.push(range,);
      continue;
    }
    // A RAW HTML BLOCK, which MDX does not have: it reads such lines as a
    // paragraph, and the reader sees them as they stand.
    if (node.type === 'html') {
      literal.push(...contentStretches({
        source,
        node,
        range,
      },),);
      continue;
    }
    // THE ALTERNATIVE TEXT OF AN IMAGE is text too; the destination and the title after it are not.
    if ((node.type === 'image') || (node.type === 'imageReference')) {
      text.push(...imageTextRanges({
        source,
        node,
        range,
      },),);
      continue;
    }
    // CODE IS COPIED, EXCEPT THE INDENTED KIND, which MDX does not have: it reads such lines as a paragraph, and the
    // reader sees them as they stand. A fence is lengthened where its code holds a mark MDX would close it at.
    if (node.type === 'code') {
      if (startsAny({
        text: source,
        at: range.start,
        candidates: FENCE_MARKS,
      },))
        edits.push(...fenceLengthEdits({
          source,
          range,
        },),);
      else {
        literal.push(...contentStretches({
          source,
          node,
          range,
        },),);
      }
      continue;
    }
    // A FOOTNOTE DEFINITION CommonMark READS AS A LINK REFERENCE DEFINITION is a footnote to the strict grammar, whose
    // body is blocks; any other definition keeps its destination and title and has a label read as text.
    if (node.type === 'definition') {
      if (node
        .identifier
        .startsWith('^',)) {
        /**
         Where the footnote's lines end: with the paragraph on the line right after the definition, whose lines the
         footnote's paragraph continues and which is read with it, else with the definition itself.
         */
        const { end, } = rangeOf({
          node: isRootChild
            ? takeContinuation({
              pending,
              definition: node,
            },)
            : node,
        },);
        absorb({
          nested: footnoteBodySpans({
            source,
            readable,
            readSpans: spansOf,
            range: {
              start: footnoteBodyStart({
                source,
                range,
              },),
              end,
            },
          },),
        },);
        continue;
      }
      text.push({
        start: range.start + 1,
        end: closingBracket({
          source,
          from: range.start + 1,
          limit: range.end,
        },),
      },);
      continue;
    }
    // AN ANGLE AUTOLINK, which MDX reads as a tag, becomes a link, except inside the text of a link, which holds none:
    // its address is written as text there, without the brackets. Any other link keeps everything but its text.
    if ((node.type === 'link') && (source.charAt(range.start,) === '<')) {
      edits.push({
        ...range,
        replacement: autolinkWritten({
          source,
          url: node.url,
          range,
          insideLink,
        },),
      },);
      continue;
    }
    if ((node.type === 'link') || (node.type === 'linkReference')) {
      insideLinks.push(range,);
      if ((node.type === 'linkReference') && (node.referenceType === 'full')) {
        text.push(secondLabel({
          source,
          range,
          textFrom: range.start + 1,
        },),);
      }
    }
    if (INLINE_CONTENT.has(node.type,))
      streamStarts.push(range.start,);
    if ('children' in node) {
      /**
       The node's children, the last first so they come out in order.
       */
      const children = node
        .children
        .toReversed();
      for (const child of children)
        pending.push({
          node: child,
          isRootChild: false,
          insideLink: insideLink
            || (node.type === 'link')
            || (node.type === 'linkReference'),
          parentType: node.type,
          parent: node,
        },);
    }
  }
  return {
    text,
    literal,
    edits: edits.toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },),
    insideLinks,
    streamStarts,
  };
}

//endregion Source ships spans
