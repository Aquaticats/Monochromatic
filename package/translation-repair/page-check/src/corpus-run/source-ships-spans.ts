import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isLazyDefinition, } from './source-ships-footnote.ts';
import { continuesTheFootnote, } from './source-ships-footnote-container.ts';
import { lineJoinsOf, } from './source-ships-lines.ts';
import { autolinkReading, } from './source-ships-autolink.ts';
import { readsAsDisplayMath, } from './source-ships-display-math.ts';
import { footnoteRead, } from './source-ships-footnote-spans.ts';
import { rawHtmlBlockReading, } from './source-ships-footnote-html-region.ts';
import { imageAltReading, } from './source-ships-image-alt.ts';
import {
  isIndentedCode,
  markerEscapeEdits,
} from './source-ships-list-marker.ts';
import { literalBlockReading, } from './source-ships-literal-block.ts';
import {
  footnoteReadingsOf,
  NO_PAGE_ONLY_DEFINITION,
  pageOnlyDefinitionWritten,
} from './source-ships-footnote-ranges.ts';
import {
  type FootnoteState,
  footnoteStateOf,
  stateAfter,
  topLevelPending,
  withEndLine,
} from './source-ships-pending.ts';
import {
  closingBracket,
  rangeOf,
  secondLabel,
  type SourceSpans,
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
// WHAT THE PLAIN PARSE READS DIFFERENTLY FROM THE STRICT GRAMMAR, and how each is handled. The plain parse has the
// tables and the strikethrough of GFM and no other part of it (its autolink-literal transform builds nodes without a
// position, and its footnotes read a definition the plain reading takes for a paragraph), so a footnote definition
// made of one destination is a link reference definition there and a footnote to the strict grammar: its body is read
// as a text of its own (`nestedSpans`), as are the blocks indented under a footnote definition, which GFM takes for
// the blocks of the footnote and the plain parse for indented code. A footnote ends where a parse with GFM ends it
// (`document-structure/src/gfm-block-ranges.ts`), and the lines the plain parse goes on to read after that are a text
// of their own. A paragraph in which a footnote starts on a later line is cut there, a heading made of a footnote's
// lines opens it, and a lazy line of equals signs or hyphens in it is paragraph text (`source-ships-footnote-spans.ts`,
// `source-ships-lazy.ts`). A label of a reference definition and of the references to it are written alike, so the
// reference still resolves. An angle autolink inside the text of a link cannot become a link of its own, which a link
// cannot hold: its address is written as text there, without the brackets, and the reader sees the address as the
// source shows it.

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
   Where the strict grammar reads a table and a footnote definition, where the page starts a footnote definition and a
   reader of the source does not, and where the lines of a block of raw HTML that a definition opens with stand.
   */
  const {
    tables,
    pageFootnoteRanges,
    footnoteRanges,
    rawHtmlRegions,
    pageOnlyStarts,
  } = footnoteReadingsOf({ readable, },);
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
   Where each link reference definition read so far ends.
   */
  const definitionEnds: number[] = [];

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
   For each container, whether the block read last in it is indented code, which a line that looks like a list marker
   right after it is text to.
   */
  const endsInIndentedCode = new Map<TreeNode, boolean>();
  /**
   Nodes still to read, the last of the root's children on top so they come out in order.
   */
  const pending = topLevelPending({ root, },);
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
     Whether the block read right before it in its parent is indented code.
     */
    const afterIndentedCode = endsInIndentedCode.get(parent,) ?? false;
    endsInIndentedCode.set(
      parent,
      isIndentedCode({
        source,
        node,
      },),
    );
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
      source,
      node,
      footnoteOpen: footnote.open || footnote.endsInLiteral,
      previousEndLine: footnote.endLine,
    },)) {
      text.push(range,);
      footnotes.set(
        parent,
        withEndLine({
          state: footnote,
          node,
        },),
      );
      continue;
    }
    // A FOOTNOTE LABEL AT A START ONLY THE PAGE READS, on a line right after a link reference definition, is a second
    // definition to a reader of the source and shows nothing: its caret is written with a backslash, so that the page
    // reads a definition of another label (`source-ships-footnote-ranges.ts`, `pageOnlyDefinitionLabelAt`).
    /**
     What writing the node as that definition comes to, when it is one.
     */
    const pageOnly = pageOnlyDefinitionWritten({
      source,
      node,
      starts: pageOnlyStarts,
      endLine: footnote.endLine,
    },);
    if (pageOnly !== NO_PAGE_ONLY_DEFINITION) {
      definitionEnds.push(range.end,);
      edits.push(pageOnly.edit,);
      text.push(pageOnly.label,);
      footnotes.set(
        parent,
        pageOnly.state,
      );
      continue;
    }
    /**
     Whether the node is indented code under a footnote definition, which the
     strict grammar reads as the blocks of the definition.
     */
    const continuesFootnote = continuesTheFootnote({
      source,
      node,
      range,
      state: footnote,
      isRootChild,
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
        isRootChild,
      },)
      : footnote;
    footnotes.set(
      parent,
      after,
    );
    // A BLOCK THAT BELONGS TO A FOOTNOTE DEFINITION is read as the body of the footnote is, which
    // `source-ships-footnote-spans.ts` says how.
    /**
     What reading the block as part of a footnote says of it.
     */
    const read = footnoteRead({
      source,
      readable,
      readSpans: spansOf,
      footnotes: footnoteRanges,
      pageFootnotes: pageFootnoteRanges,
      pageOnlyStarts,
      pending,
      node,
      range,
      isRootChild,
      continuesFootnote,
      state: after,
    },);
    if (read.kind === 'read') {
      footnotes.set(
        parent,
        read.state,
      );
      absorb({ nested: read.spans, },);
      continue;
    }
    // THE LINES OF A BLOCK OF RAW HTML A FOOTNOTE DEFINITION IN A CONTAINER OPENS WITH are text on the page, whatever the plain
    // reading makes of them (`source-ships-footnote-html-region.ts`).
    /**
     Their content, when the block is one of them.
     */
    const html = rawHtmlBlockReading({
      source,
      readable,
      regions: rawHtmlRegions,
      range,
      inContainer: (!isRootChild) && BLOCK_PARENTS.has(parentType,),
    },);
    if (html.inRegion) {
      absorb({ nested: html, },);
      continue;
    }
    // A definition, and display math the strict grammar reads, end their block at the line, which no fence mark after it joins.
    if ((node.type === 'definition') || ((node.type === 'inlineMath') && readsAsDisplayMath({
      source,
      range,
    },)))
      definitionEnds.push(range.end,);
    edits.push(...lineJoinsOf({
      source,
      node,
      range,
      definitionEnds,
      footnoteStarts: pageOnlyStarts,
    },),);
    // A LINE THAT LOOKS LIKE A LIST MARKER RIGHT AFTER INDENTED CODE is text to the parse and a list on the page, which has
    // no indented code, so its marker is written with a backslash (`source-ships-list-marker.ts`).
    if (afterIndentedCode && (node.type === 'paragraph')) {
      edits.push(...markerEscapeEdits({
        source,
        start: range.start,
      },),);
    }
    // A TEXT, AND RAW HTML AMONG INLINE CONTENT, are what MDX reads a brace and a less-than sign in.
    if ((node.type === 'text') || ((node.type === 'html') && (!BLOCK_PARENTS.has(parentType,)))) {
      text.push(range,);
      continue;
    }
    // A RAW HTML BLOCK, AND CODE, EXCEPT THE FENCED KIND, which MDX does not have: it reads such lines as a paragraph,
    // and the reader sees them as they stand. A fence is lengthened where its code holds a mark MDX would close it at.
    if ((node.type === 'html') || (node.type === 'code')) {
      /**
       What the block comes to for the page.
       */
      const block = literalBlockReading({
        source,
        readable,
        node,
        range,
        isRootChild,
      },);
      literal.push(...block.literal,);
      edits.push(...block.edits,);
      continue;
    }
    // THE ALTERNATIVE TEXT OF AN IMAGE is text too; the destination and the title after it are not.
    if ((node.type === 'image') || (node.type === 'imageReference')) {
      absorb({
        nested: imageAltReading({
          source,
          node,
          range,
        },),
      },);
      continue;
    }
    // A DEFINITION that is no footnote keeps its destination and title and has a label read as text.
    if (node.type === 'definition') {
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
    // its address is written as text there, without the brackets, and where a table cuts it at a pipe, which leaves no
    // autolink in the source, so it is text as it stands. Any other link keeps everything but its text.
    if ((node.type === 'link') && (source.charAt(range.start,) === '<')) {
      absorb({
        nested: autolinkReading({
          source,
          url: node.url,
          range,
          tables,
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
