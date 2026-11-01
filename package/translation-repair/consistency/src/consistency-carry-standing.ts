import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Nodes, } from 'mdast';

import { treeNodes, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  parseBodyTolerant,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Consistency carry standing
// WHERE A SPAN OF THE ENGLISH STANDS AMONG ITS BLOCKS, decided without a model.
// A rewrite of a receiving place's paragraph needs the paragraph or heading
// that holds the place's words, whole. Words in a table (a Markdown table or
// HTML table markup the site renders as cells), a fenced code block, a setext
// heading or a pre element the site renders without a paragraph have no such
// paragraph: a rewrite of "the paragraph" would have to write the cells, the
// fence, the underline or the preformatted layout too, so only the code splice
// can carry words there, and the standing names which of the four kinds holds
// them. A paragraph the site renders as a paragraph stays writable, also inside
// an HTML container: the content of a pre element or of a table cell that
// stands apart by blank lines is a paragraph of its own. Words in no body
// block, in a footnote definition, across blocks, or in a container (a block
// quote, a list, an HTML or JSX element) without lying in one paragraph or
// heading of it have no block and no kind, at every depth of nesting: a block is
// never handed out around the blocks of other words of the same English, since
// two replacements of one group over nested blocks would address the same words.

/**
 A body block of an English: the paragraph a receiving place's span lies in.

 @example
 ```ts
 const block: CarryBlock = { start: 0, end: 13, text: 'The cat naps.', };
 ```
 */
export type CarryBlock = {
  /**
   Offset in the English where the block starts.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends.
   */
  readonly end: number;

  /**
   The block's text, cut from the English as written. For a paragraph inside a
   block quote that runs over several lines, the first line carries no `> `
   marker, since the parser puts the paragraph's start after the marker that
   opens it, while every later line keeps its own marker, since they lie
   inside the cut.
   */
  readonly text: string;
};

/**
 Kinds of node a writer rewrites: the leaf blocks that hold running text. A
 list or a block quote around one is only where it stands.
 */
const RUNNING_TEXT_BLOCKS: ReadonlySet<string> = new Set([
  'paragraph',
  'heading',
],);

/**
 Kinds of top-level node that are containers of other blocks whatever they hold:
 a block quote and a list (an item is never a top-level node of its own).
 */
const CONTAINER_BLOCKS: ReadonlySet<string> = new Set([
  'blockquote',
  'list',
],);

/**
 Where a parsed node stands in the text it was parsed from.

 @example
 ```ts
 const offsets: LeafOffsets = { from: 4, to: 28, };
 ```
 */
type LeafOffsets = {
  /**
   Offset where the node starts.
   */
  readonly from: number;

  /**
   Exclusive offset where it ends.
   */
  readonly to: number;
};

/**
 The offsets a parsed node stands at in the text it was parsed from.

 @param leaf - a node of the parsed tree

 @returns The start and the exclusive end as a list of zero or one; none for a
 node that the parser gave no position

 @example
 ```ts
 const [offsets,] = leafOffsetsOf({ leaf, },);
 ```
 */
function leafOffsetsOf(
  { leaf, }: { readonly leaf: ForeignBorrowed<Nodes>; },
): readonly LeafOffsets[] {
  /**
   Where the parser put the node.
   */
  const { position, } = leaf;
  if (position === undefined)
    return [];
  /**
   Offset where the node starts.
   */
  const { offset: from, } = position.start;
  /**
   Offset where the node ends.
   */
  const { offset: to, } = position.end;
  return ((from === undefined) || (to === undefined))
    ? []
    : [
      {
        from,
        to,
      },
    ];
}

/**
 A kind of block a span can stand in where no paragraph rewrite can carry words
 to it, named as a reader of a finding says it. A table is a Markdown table or
 HTML table markup (a table, row or cell element) that the site renders as
 cells; a preformatted block is a pre element that the site renders without a
 paragraph inside it.
 */
export type SpliceOnlyKind = 'table' | 'code block' | 'setext heading' | 'preformatted block';

/**
 Where a span of the English stands among its blocks.

 @example
 ```ts
 const standing: BlockStanding = { block: [], spliceOnlyIn: ['table',], };
 ```
 */
export type BlockStanding = {
  /**
   The block a writer is given, as a list of zero or one: none where the span
   lies in no body block, in a footnote definition, in a table, a code block,
   a setext heading or a preformatted block, across blocks, or in a container
   without lying in one paragraph or heading of it.
   */
  readonly block: readonly CarryBlock[];

  /**
   The kind of block the span stands in where only a splice can carry words,
   as a list of zero or one.
   */
  readonly spliceOnlyIn: readonly SpliceOnlyKind[];
};

/**
 A node of the parsed tree that holds a span whole, with where it stands in the
 English.

 @example
 ```ts
 const holder: SpanHolder = { type: 'paragraph', start: 4, end: 28, };
 ```
 */
type SpanHolder = {
  /**
   The mdast type of the node.
   */
  readonly type: string;

  /**
   Offset in the English where the node starts.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends.
   */
  readonly end: number;

  /**
   The names of the inline elements a paragraph holds at any depth, empty for any
   other node; a table cell or a pre element that shares its line with its
   content is one of them, also inside emphasis or a link.
   */
  readonly inlineTags: readonly string[];
};

/**
 The names of HTML elements that make a table, which the site renders as rows
 and cells.
 */
const TABLE_TAG_NAMES: ReadonlySet<string> = new Set([
  'table',
  'thead',
  'tbody',
  'tfoot',
  'tr',
  'th',
  'td',
  'caption',
  'colgroup',
  'col',
],);

/**
 The characters that end the name of an HTML element in its opening tag.
 */
const TAG_NAME_DELIMITERS: readonly string[] = [
  ' ',
  '\t',
  '\n',
  '\r',
  '\f',
  '/',
  '>',
];

/**
 The kind an HTML element of this name is where it holds a span.

 The name is read in lower case, whatever its case in the English. HTML names are not case sensitive, and plain markdown
 reads `<TABLE>` as the table element, yet the strict grammar (MDX) reads a capitalised name such as `<TABLE>` or
 `<Table>` as a JSX component, so reading the case as written would give two answers for one text depending on which
 reader accepted it. The site these pages are for renders MDX, where a capitalised name is a component the site would
 have to define, and nothing in this package names such a component for the names this reads (`table`, `td`, `pre` and
 the rest); a table written in capitals is far likelier HTML typed in capitals. So both readings take such a name as the
 element, and a paragraph holding it is no paragraph a writer can rewrite.

 @param name - the element's name as written, in any case

 @returns The kind as a list of zero or one; none for an element a paragraph rewrite can carry words through

 @example
 ```ts
 const [kind,] = spliceOnlyKindOfTag({ name: 'TD', },); // 'table'
 ```
 */
function spliceOnlyKindOfTag({ name, }: { readonly name: string; },): readonly SpliceOnlyKind[] {
  /**
   The name with its case folded, the one spelling of the element in both readings.
   */
  const folded = name.toLowerCase();
  if (TABLE_TAG_NAMES.has(folded,))
    return ['table',];
  if (folded === 'pre')
    return ['preformatted block',];
  return [];
}

/**
 The name of the HTML element a raw HTML node opens with, read from the first
 characters only.

 @param text - the node's text in the English

 @returns The name in lower case, since HTML names are not case sensitive; empty where the opening bracket is followed by no name, as in a comment

 @throws {@link Error} when the text, leading whitespace aside, does not open with an angle bracket, which no raw HTML node of the parsed tree does

 @example
 ```ts
 const name = openingTagNameOf({ text: '<TABLE><tr>', },); // 'table'
 ```
 */
function openingTagNameOf({ text, }: { readonly text: string; },): string {
  /**
   The text's first characters, which hold the whole name of any element this reads; a longer name is cut short and
   still differs from the names this reads.
   */
  const opening = text
    .trimStart()
    .slice(
    0,
    16,
    );
  if (!opening.startsWith('<',)) {
    throw new Error(
      'unreachable: a raw HTML node of the parsed tree does not open with an angle bracket, though the parser reads '
        + 'HTML only from the bracket that opens it',
    );
  }
  /**
   The characters after the angle bracket.
   */
  const rest = opening.slice(1,);
  /**
   How many characters come before each delimiter, all of them where it is absent.
   */
  const lengthsBefore = TAG_NAME_DELIMITERS.map(function lengthBefore(delimiter,): number {
    /**
     The characters before the first delimiter.
     */
    const [before = '',] = rest.split(
      delimiter,
      1,
    );
    return before.length;
  },);
  return rest
    .slice(
    0,
    Math.min(...lengthsBefore,),
    )
    .toLowerCase();
}

/**
 The name of the inline HTML element a node of a paragraph opens, in either reading of the text: a JSX element of the
 strict grammar, or a raw HTML node of the plain markdown fallback.

 @param node - a node inside a paragraph of the parsed tree

 @returns The name as a list of zero or one, as written for a JSX element and in lower case for a raw HTML node; none
 for a node that opens no element and for a fragment. A raw HTML node that is a closing tag or a comment gives an empty
 name, which names no kind of block.

 @throws {@link Error} when a raw HTML node's text does not open with an angle bracket, which no such node of the parsed tree does

 @example
 ```ts
 const [name,] = inlineTagNameOf({ node, },); // ['td',]
 ```
 */
function inlineTagNameOf({ node, }: { readonly node: ForeignBorrowed<Nodes>; },): readonly string[] {
  if ((node.type === 'mdxJsxTextElement') && (node.name !== null))
    return [node.name,];
  return (node.type === 'html') ? [openingTagNameOf({ text: node.value, },),] : [];
}

/**
 The names of the inline elements a paragraph holds at any depth: directly, or inside emphasis, a link, a deletion or
 an element that holds them.

 The tree is a bounded structural walk of one pass, with a work stack and no recursion, since a paragraph is as deep as
 the nesting the page writes. The children of each node go onto the stack in reverse, so the names come in the order
 the elements stand in the text, which decides the kind when a paragraph holds a table element and a pre element.

 @param leaf - a node of the parsed tree

 @returns The names, in the order of the text, an empty name standing for a closing tag or a comment; none for a node
 that is no paragraph, and none for an element without a name

 @throws {@link Error} when a raw HTML node inside the paragraph does not open with an angle bracket, which no such node of the parsed tree does

 @example
 ```ts
 const tags = inlineTagNamesOf({ leaf, },); // ['td',]
 ```
 */
function inlineTagNamesOf({ leaf, }: { readonly leaf: ForeignBorrowed<Nodes>; },): readonly string[] {
  if (leaf.type !== 'paragraph')
    return [];
  /**
   The names met so far, which is what the function returns.
   */
  const names: string[] = [];
  /**
   The nodes still to visit, the next one last.
   */
  const pending: ForeignBorrowed<Nodes>[] = leaf.children
    .toReversed();
  // Each node in turn, the first child of a node before its later siblings, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    names.push(...inlineTagNameOf({ node, },),);
    if ('children' in node)
      pending.push(...node.children
        .toReversed(),);
  }
  return names;
}

/**
 The kind a node of the parsed tree is where no paragraph rewrite can carry
 words into it.

 A setext heading is a heading of more than one line: an ATX heading is one
 line, and a setext heading always includes its underline line, so a rewrite of
 "the paragraph" would have to write the underline too. A paragraph that holds
 a table cell or a pre element sharing its line with its content (also inside
 emphasis or a link), and a raw HTML block that opens with one, would have a
 rewrite write that markup too.

 @param type - the mdast type of the node

 @param text - the node's text in the English

 @param inlineTags - the names of the inline elements a paragraph holds at any depth

 @returns The kind as a list of zero or one; none for a node a writer can rewrite or that holds no running text

 @throws {@link Error} when a raw HTML node's text does not open with an angle bracket, which no such node of the parsed tree does

 @example
 ```ts
 const [kind,] = spliceOnlyKindOf({ type: 'table', text: '| a |', inlineTags: [], },); // 'table'
 ```
 */
function spliceOnlyKindOf(
  {
    type,
    text,
    inlineTags,
  }: {
    readonly type: string;
    readonly text: string;
    readonly inlineTags: readonly string[];
  },
): readonly SpliceOnlyKind[] {
  if (type === 'table')
    return ['table',];
  if (type === 'code')
    return ['code block',];
  if (type === 'html') {
    return spliceOnlyKindOfTag({
      name: openingTagNameOf({ text, },),
    },);
  }
  if (type === 'paragraph') {
    return inlineTags.flatMap(function kindOfInlineTag(name,): readonly SpliceOnlyKind[] {
      return spliceOnlyKindOfTag({ name, },);
    },)
      .slice(
      0,
      1,
      );
  }
  if (type !== 'heading')
    return [];
  return (text.includes('\n',) || text.includes('\r',)) ? ['setext heading',] : [];
}

/**
 Where a span stands inside a top-level block that holds it whole: in the
 innermost paragraph or ATX heading around it, as a block of the English; in a
 table, a code block, a setext heading or a pre element, which no paragraph
 rewrite can carry words into; or, when the block holds no such leaf around the
 span, in the top-level block itself where that block is no container, and
 nowhere (no block and no kind) where it is one. A container is a block quote
 or a list, whatever it holds, or a block that holds a paragraph or heading other
 than itself at any depth, such as an HTML or JSX element around paragraphs (the
 document reader types that block as a paragraph). The span has no leaf there
 when it crosses a paragraph boundary, an item boundary or a nesting level, or
 lies in an HTML comment or a JSX attribute, at whatever depth of nesting. The
 container's block would nest around the block of every paragraph in it that
 another span of the English stands in, so the answer is the one the top level
 gives a span across blocks. A top-level block that is no container (a thematic
 break, a link definition, an expression, a paragraph) holds no other block, and
 is the block.

 Every top-level block but a table or a code block is read again, a heading
 included. The document reader joins the text of an HTML container's content
 with the container's tags into one node, so a heading that opens or closes the
 container carries a tag in its text, which is neither one line nor free of
 line breaks; the reading again finds the heading on its own, an ATX heading
 being one line of running text, and the writer is to be given it alone.

 @param node - the top-level block of the English that holds the span

 @param english - the slice's English

 @param start - offset where the span starts

 @param end - exclusive offset where it ends

 @returns Where the span stands; no block and no kind for a span in a container that no single paragraph or heading of it holds

 @throws Whatever `parseBodyTolerant` throws for text plain markdown refuses, and the error of `spliceOnlyKindOf` for a raw HTML node that does not open with an angle bracket

 @example
 ```ts
 const standing = standingIn({ node, english, start: 4, end: 7, },);
 ```
 */
function standingIn(
  {
    node,
    english,
    start,
    end,
  }: {
    readonly node: {
      readonly kind: string;
      readonly text: string;
      readonly startOffset: number;
      readonly endOffset: number;
    };
    readonly english: string;
    readonly start: number;
    readonly end: number;
  },
): BlockStanding {
  /**
   The top-level block as the block of the English it is.
   */
  const whole: CarryBlock = {
    start: node.startOffset,
    end: node.endOffset,
    text: node.text,
  };
  /**
   The kind the top-level node is itself, where it is one a rewrite cannot carry words into. A heading is left to
   the reading again, since its text may carry the tags of a container.
   */
  const topKind = (node.kind === 'heading')
    ? []
    : spliceOnlyKindOf({
      type: node.kind,
      text: node.text,
      inlineTags: [],
    },);
  if (topKind.length > 0) {
    return {
      block: [],
      spliceOnlyIn: topKind,
    };
  }
  /**
   The block read as markdown, its offsets counted from its own start.
   */
  const { root, } = parseBodyTolerant({
    body: node.text,
    bodyOffset: node.startOffset,
  },);
  /**
   Every node of the block that holds the span whole, outermost first.
   */
  const holders = treeNodes({ root, },)
    .flatMap(function holderOf(leaf: ForeignBorrowed<Nodes>,): readonly SpanHolder[] {
      return leafOffsetsOf({ leaf, },)
        .flatMap(function holdingSpan(
          {
            from,
            to,
          },
        ): readonly SpanHolder[] {
          return (((node.startOffset + from) <= start) && (end <= (node.startOffset + to)))
            ? [
              {
                type: leaf.type,
                start: node.startOffset + from,
                end: node.startOffset + to,
                inlineTags: inlineTagNamesOf({ leaf, },),
              },
            ]
            : [];
        },);
    },);
  /**
   The first holder that a rewrite cannot carry words into, none when there is no such holder.
   */
  const [spliceOnly,] = holders.flatMap(function kindOfHolder(holder,): readonly SpliceOnlyKind[] {
    return spliceOnlyKindOf({
      type: holder.type,
      text: english.slice(
        holder.start,
        holder.end,
      ),
      inlineTags: holder.inlineTags,
    },);
  },);
  if (spliceOnly !== undefined) {
    return {
      block: [],
      spliceOnlyIn: [spliceOnly,],
    };
  }
  /**
   The paragraphs and headings that hold the span, as blocks of the English: at most one, since a paragraph or a heading holds no other.
   */
  const [leaf,] = holders.filter(function isRunningText({ type, },): boolean {
    return RUNNING_TEXT_BLOCKS.has(type,);
  },);
  if (leaf !== undefined) {
    return {
      block: [
        {
          start: leaf.start,
          end: leaf.end,
          text: english.slice(
            leaf.start,
            leaf.end,
          ),
        },
      ],
      spliceOnlyIn: [],
    };
  }
  /**
   Where every paragraph and heading of the block stands, counted from the block's own start.
   */
  const runningTextOffsets = treeNodes({ root, },)
    .filter(function isRunningTextNode(inner: ForeignBorrowed<Nodes>,): boolean {
      return RUNNING_TEXT_BLOCKS.has(inner.type,);
    },)
    .flatMap(function offsetsOfRunningText(inner: ForeignBorrowed<Nodes>,): readonly LeafOffsets[] {
      return leafOffsetsOf({ leaf: inner, },);
    },);
  /**
   Whether the top-level block holds a paragraph or heading other than itself at any depth (the document reader gives
   an HTML or JSX element around paragraphs as one block, typed as a paragraph). It is a container then, as a block
   quote or a list is whatever it holds. A span in a container that no single paragraph or heading holds (it crosses a
   paragraph or item boundary, or lies in an HTML comment or a JSX attribute) has the whole container as its only
   holder, and the container's block would nest around the block of every paragraph in it, so it gets no block.
   */
  const holdsOtherBlock = runningTextOffsets
    .some(function differsFromWhole(
      {
        from,
        to,
      },
    ): boolean {
      return ((node.startOffset + from) !== whole.start) || ((node.startOffset + to) !== whole.end);
    },);
  return {
    block: (CONTAINER_BLOCKS.has(node.kind,) || holdsOtherBlock) ? [] : [whole,],
    spliceOnlyIn: [],
  };
}

/**
 Where a span of an English stands among its body blocks.

 @param english - the slice's English

 @param start - offset where the span starts

 @param end - exclusive offset where it ends

 @returns The innermost block that holds the span whole, the paragraph or the
 ATX heading itself, also when a list item or a block quote holds it (the text
 of a multi-line paragraph in a block quote has no `> ` marker on its first
 line, since the paragraph starts after the marker, and keeps the marker of
 every later line); the kind of block where a table, a code block, a setext
 heading or a pre element holds it, and no block then; no block and no kind where
 the span lies in no body block, in a footnote definition, or across blocks. The
 last holds at every depth: a span that crosses a paragraph boundary inside a
 block quote or a list item, that crosses two list items, or that lies in an HTML
 comment or a JSX attribute of a container, has no block, as it has none at the
 top level, while a span wholly inside one paragraph of the container keeps that
 paragraph. So the blocks this names for the spans of one English never nest.

 @throws Whatever `parseDocument` throws for text the shared grammar and the front-matter reader refuse, and whatever the plain markdown reader throws for a container it cannot read, and the error of `spliceOnlyKindOf` for a raw HTML node that does not open with an angle bracket, all of which the pass contains

 @example
 ```ts
 const { block, spliceOnlyIn, } = blockStandingOf({ english: 'The cat naps.\n\nThe fish swims.', start: 4, end: 7, },);
 ```
 */
export function blockStandingOf(
  {
    english,
    start,
    end,
  }: {
    readonly english: string;
    readonly start: number;
    readonly end: number;
  },
): BlockStanding {
  /**
   The top-level body block that holds the span, as a list of zero or one.
   */
  const standings = parseDocument({ text: english, },)
    .nodes
    .filter(function holdsSpan(node,): boolean {
      return (node.zone === 'body') && (node.startOffset <= start)
        && (end <= node.endOffset);
    },)
    .slice(
    0,
    1,
    )
    .map(function standingOf(node,): BlockStanding {
      return standingIn({
        node,
        english,
        start,
        end,
      },);
    },);
  return standings[0] ?? {
    block: [],
    spliceOnlyIn: [],
  };
}

/**
 The innermost body block of an English that holds a span whole: the paragraph
 or heading itself, also when a list item or a block quote holds it.

 @param english - the slice's English

 @param start - offset where the span starts

 @param end - exclusive offset where it ends

 @returns The block as a list of zero or one; none where the span lies in no
 body block, in a footnote definition, in a table, a code block, a setext
 heading or a pre element, across blocks at any depth of nesting, or in a
 container without lying in one paragraph or heading of it

 @throws Whatever `parseDocument` throws for text the shared grammar and the front-matter reader refuse, and whatever the plain markdown reader throws for a container it cannot read, and the error of `spliceOnlyKindOf` for a raw HTML node that does not open with an angle bracket, all of which the pass contains

 @example
 ```ts
 const [block,] = blockHolding({ english: 'The cat naps.\n\nThe fish swims.', start: 4, end: 7, },);
 ```
 */
export function blockHolding(
  {
    english,
    start,
    end,
  }: {
    readonly english: string;
    readonly start: number;
    readonly end: number;
  },
): readonly CarryBlock[] {
  /**
   Where the span stands.
   */
  const { block, } = blockStandingOf({
    english,
    start,
    end,
  },);
  return block;
}

//endregion Consistency carry standing
