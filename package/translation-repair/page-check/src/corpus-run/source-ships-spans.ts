import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  footnoteBodyStart,
  isIndentedCode,
  opensFootnote,
  withoutIndentation,
} from './source-ships-footnote.ts';
import { linkInsteadOfAutolink, } from './source-ships-links.ts';
import { startsEsModule, } from './source-ships-module-line.ts';
import {
  closingBracket,
  FENCE_MARKS,
  indicesOf,
  originalStretches,
  rangeOf,
  secondLabel,
  startsAny,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

export type {
  TextEdit,
  TextRange,
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
// of its own, which a link cannot hold: it is written as text there, and the reader sees the address as typed.

/**
 Characters a backslash escapes in Markdown: the ASCII punctuation.
 */
export const ESCAPABLE = '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~';

/**
 How many characters `![` takes.
 */
const IMAGE_OPENER_LENGTH = 2;

/**
 What a parse says of a text.
 */
export type SourceSpans = {
  /**
   Stretches a reader sees as text, in which a brace or a less-than sign is syntax for the strict grammar.
   */
  readonly text: readonly TextRange[];

  /**
   Stretches rewritten whole, in text order and not overlapping.
   */
  readonly edits: readonly TextEdit[];

  /**
   Links and link references, each from its first bracket to its last character, inside whose text GFM reads no bare
   address.
   */
  readonly insideLinks: readonly TextRange[];
};
/**
 The characters of a text at the indices.

 @param text - the text

 @param indices - indices of the characters, in order

 @returns The characters joined

 @example
 ```ts
 const body = charactersAt({ text: 'abc', indices: [0, 2,], },);
 ```
 */
function charactersAt(
  {
    text,
    indices,
  }: {
    readonly text: string;
    readonly indices: readonly number[];
  },
): string {
  return indices
    .map(function characterAt(at,): string {
      return text.charAt(at,);
    },)
    .join('',);
}

/**
 Reads a block nested in another as a text of its own: the body of a footnote definition, or a block indented under one.

 @param source - the whole text

 @param readable - the whole text with its comments blanked, the parse's own input

 @param indices - the indices of the whole text that make up the block, in order

 @returns The spans of the block at the offsets of the whole text

 @throws {@link Error} when a node of the block has no position, which the reading used never builds

 @example
 ```ts
 const spans = nestedSpans({ source, readable, indices: [6, 7, 8,], });
 ```
 */
function nestedSpans(
  {
    source,
    readable,
    indices,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly indices: readonly number[];
  },
): SourceSpans {
  /**
   The block as a text of its own.
   */
  const body = charactersAt({
    text: source,
    indices,
  },);
  /**
   The same with its comments blanked.
   */
  const bodyReadable = charactersAt({
    text: readable,
    indices,
  },);
  /**
   What the parse of the block says of it.
   */
  const inner = spansOf({
    source: body,
    readable: bodyReadable,
    root: parseMarkdownMathBody({ body: bodyReadable, },),
    isNested: true,
  },);
  return {
    text: inner
      .text
      .flatMap(function original(range,): readonly TextRange[] {
        return originalStretches({
          range,
          indices,
        },);
      },),
    edits: inner
      .edits
      .map(function original(edit,): TextEdit {
        return {
          ...edit,
          start: nonNullishOrThrow(indices[edit.start],),
          end: nonNullishOrThrow(indices[edit.end - 1],) + 1,
        };
      },),
    insideLinks: inner
      .insideLinks
      .flatMap(function original(range,): readonly TextRange[] {
        return originalStretches({
          range,
          indices,
        },);
      },),
  };
}

/**
 A node still to read, and where it stands.
 */
type Pending = {
  /**
   The node.
   */
  readonly node: TreeNode;

  /**
   Whether its parent is the root of the text being read.
   */
  readonly isRootChild: boolean;

  /**
   Whether it stands inside the text of a link, where a link cannot be written.
   */
  readonly insideLink: boolean;
};

/**
 Reads where a text holds syntax, from the parse of it.

 @param source - the text, which edits and rewrites are made from

 @param root - parse of the text by the plain reading with the site's math

 @param readable - the text as the parse read it, with its comments blanked, the source itself where none is; a block
 nested in another is parsed from this

 @param isNested - whether the text is a block nested in another, none of whose nodes MDX reads as an ES module

 @returns The stretches a reader sees as text, the stretches rewritten whole and the links

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
    isNested = false,
  }: {
    readonly source: string;
    readonly root: TreeNode;
    readonly readable?: string;
    readonly isNested?: boolean;
  },
): SourceSpans {
  /**
   Stretches read as text.
   */
  const text: TextRange[] = [];
  /**
   Stretches rewritten.
   */
  const edits: TextEdit[] = [];
  /**
   Links.
   */
  const insideLinks: TextRange[] = [];

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
    edits.push(...nested.edits,);
    insideLinks.push(...nested.insideLinks,);
  }

  /**
   Whether the block read last at the top is part of a footnote definition, which blocks indented under it continue.
   */
  const footnote = { open: false, };
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
    } = next;
    /**
     Where its characters are.
     */
    const range = rangeOf({ node, },);
    /**
     Whether the node is indented code under a footnote definition, which the strict grammar reads as the blocks of the
     definition.
     */
    const continuesFootnote = isRootChild
      && footnote.open
      && isIndentedCode({
        source,
        node,
        range,
      },);
    if (isRootChild) {
      footnote.open = continuesFootnote || opensFootnote({
        source,
        node,
        range,
      },);
    }
    // A TEXT AND A RAW HTML NODE are what MDX reads a brace and a less-than sign in.
    if ((node.type === 'text') || (node.type === 'html')) {
      text.push(range,);
      continue;
    }
    // THE ALTERNATIVE TEXT OF AN IMAGE is text too; the destination and the title after it are not.
    if ((node.type === 'image') || (node.type === 'imageReference')) {
      text.push({
        start: range.start + IMAGE_OPENER_LENGTH,
        end: closingBracket({
          source,
          from: range.start + IMAGE_OPENER_LENGTH,
          limit: range.end,
        },),
      },);
      if ((node.type === 'imageReference') && (node.referenceType === 'full')) {
        text.push(secondLabel({
          source,
          range,
          textFrom: range.start + IMAGE_OPENER_LENGTH,
        },),);
      }
      continue;
    }
    // BLOCKS INDENTED UNDER A FOOTNOTE DEFINITION are read by their own structure, since GFM reads them as the blocks
    // of the footnote and a fence among them is code.
    if (continuesFootnote) {
      absorb({
        nested: nestedSpans({
          source,
          readable,
          indices: withoutIndentation({
            source,
            range,
          },),
        },),
      },);
      continue;
    }
    // CODE IS COPIED, EXCEPT THE INDENTED KIND, which MDX does not have: it reads such lines as a paragraph.
    if (node.type === 'code') {
      if (!startsAny({
        text: source,
        at: range.start,
        candidates: FENCE_MARKS,
      },))
        text.push(range,);
      continue;
    }
    // A FOOTNOTE DEFINITION CommonMark READS AS A LINK REFERENCE DEFINITION is a footnote to the strict grammar, whose
    // body is blocks; any other definition keeps its destination and title and has a label read as text.
    if (node.type === 'definition') {
      if (node
        .identifier
        .startsWith('^',)) {
        absorb({
          nested: nestedSpans({
            source,
            readable,
            indices: indicesOf({
              range: {
                start: footnoteBodyStart({
                  source,
                  range,
                },),
                end: range.end,
              },
            },),
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
    // it is written as text there. Any other link keeps everything but its text.
    if ((node.type === 'link') && (source.charAt(range.start,) === '<')) {
      if (insideLink)
        text.push(range,);
      else {
        edits.push({
          ...range,
          replacement: linkInsteadOfAutolink({
            source,
            url: node.url,
            range,
          },),
        },);
      }
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
    // A PARAGRAPH OR SETEXT HEADING STARTING AT THE FIRST COLUMN WITH `import ` OR `export `, which MDX reads as an ES
    // module and no backslash makes plain; a character reference for the first letter does.
    if (startsEsModule({
      source,
      node,
      isTopLevel: isRootChild && (!isNested),
      range,
    },)) {
      edits.push({
        start: range.start,
        end: range.start + 1,
        replacement: `&#${String(source.codePointAt(range.start,),)};`,
      },);
    }
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
        },);
    }
  }
  return {
    text,
    edits: edits.toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },),
    insideLinks,
  };
}

//endregion Source ships spans

//endregion Source ships spans
