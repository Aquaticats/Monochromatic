import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { Root, } from 'mdast';

import { definitionStartsOf, } from './footnote-definition-starts.ts';
import {
  isAngleAutolink,
  isAutolinkLiteral,
  NO_NODE_BOUNDS,
  nodeBounds,
  type TreeNode,
  unpositionedRuns,
} from './footnote-unpositioned-runs.ts';
import { gfmMarkerSpans, } from './gfm-marker-spans.ts';
import { disjointSpans, } from './footnote-span-union.ts';
import { maskHtmlComments, } from './mask-html-comments.ts';
import { maskInvisibleLines, } from './mask-invisible-lines.ts';
import {
  parseMarkdownBody,
  parseMdxBody,
  requireMarkdownRefusal,
  requireMdxRefusal,
  withMarkdownExhaustionRefused,
  withMdxExhaustionRefused,
} from './parse-mdx.ts';

//region Footnote parsed spans
// Where the parse of a fragment reads a GFM marker shape as no footnote
// reference (ledger B212): inside a URL, an inline code span, an image, a link
// reference definition, a masked comment, or the markup around a link's or a
// reference's label. The footnote
// graph answers the same question from the parse of a whole page, and a scan
// of raw fragments (`footnote-mentions.ts`) asks this module instead of
// restating the grammar. An earlier lexical reader of URLs disagreed with the
// parse in both directions: it dropped a reference the parse reads (a URL
// scheme with no domain after it, a literal inside a link's label) and
// counted a shape the parse reads as URL (a literal after a Han character, a
// digit or a full stop, a destination after whitespace).
//
// THE FRAGMENT IS PARSED AS MARKDOWN AND NOT AS A DOCUMENT. No front matter
// is split, which a fragment opening on a thematic break would misread.
// Invisible-only lines and HTML comments are masked as `parseDocument` masks
// them, since a comment hides the backtick that closes a code span and the
// end of a URL, and the page reads the masked text. Both masks are
// context-free scans from the left, so a region the fragment masks the page
// masks too, and a marker the fragment leaves unmasked may lie in a comment
// the page opened earlier, which only ever counts one mention too many.
// Slices are cut at the offsets of a page's top-level blocks, so a fragment
// starts where a block starts and its inline readings agree with the page's.
//
// BOTH GRAMMARS MUST AGREE. The page is read as strict MDX where that
// accepts it and as plain markdown where it does not, which a fragment cannot
// know. They differ in block structure (a fence indented four spaces is code
// to the first and a paragraph line to the second, and so decides whether a
// backtick pair closes a code span), so a region is skipped only where both
// read it, and a marker one grammar reads as text stays counted.
//
// ONLY REGIONS THE FRAGMENT'S OWN TEXT SETTLES ARE READ. A code block, a raw
// HTML node and a JSX attribute are left to the scan, which counts a marker
// there: a fragment that opens or closes a fence for the page cannot know
// whether the page reads the shape as code, and the guards cannot afford a
// dropped reference.

/**
 Raw span of text the parse reads no footnote reference in, the end exclusive.

 @example
 ```ts
 const span: ParsedSpan = { start: 8, end: 31, };
 ```
 */
export type ParsedSpan = {
  /**
   Offset of the span's first character.
   */
  readonly start: number;

  /**
   Offset after the span's last character.
   */
  readonly end: number;
};

/**
 Spans of a link that stand outside the link's label: the markup before the
 label's first child, between its children, and after its last, which is
 where the destination, the title and the label of a reference stand.

 @param node - link or reference link whose own span the caller found present

 @param bounds - link's own span

 @returns Spans in source order, the whole link when its label is empty

 @throws {@link Error} when a label child carries no span of its own, which
 no parse builds: only the autolink-literal transform builds a node without
 one, and it never splits text inside a link or a link reference

 @example
 ```ts
 const spans = outsideLabelSpans({ node, bounds, },);
 ```
 */
function outsideLabelSpans(
  {
    node,
    bounds,
  }: {
    readonly node: Extract<TreeNode, { readonly type: 'link' | 'linkReference'; }>;
    readonly bounds: ParsedSpan;
  },
): readonly ParsedSpan[] {
  /**
   Owned result list.
   */
  const spans: ParsedSpan[] = [];
  /**
   Where the markup being collected starts.
   */
  let from = bounds.start;
  for (const child of node.children) {
    /**
     The label child's own span.
     */
    const childBounds = nodeBounds(child,);
    if (childBounds === NO_NODE_BOUNDS)
      throw new Error(
        'unreachable: a label child of a positioned link carries no position, though only the autolink-literal '
          + 'transform builds a node without one, and it never splits text inside a link or a link reference',
      );
    spans.push({
      start: from,
      end: childBounds.start,
    },);
    from = childBounds.end;
  }
  spans.push({
    start: from,
    end: bounds.end,
  },);
  return spans;
}

/**
 Spans of the marker shapes an unpositioned run's raw region holds outside
 the run's text: a URL or an address the autolink-literal transform built a
 link from, which the footnote graph reads as URL (ledger B123).

 @param text - fragment the parser read

 @param parent - node whose child list the runs stand in

 @returns Marker spans in source order, offsets of the fragment

 @throws FootnoteRewriteError of the `position` kind when a run cannot be
 placed in the raw text

 @example
 ```ts
 const spans = runUrlSpans({ text, parent, },);
 ```
 */
function runUrlSpans(
  {
    text,
    parent,
  }: {
    readonly text: string;
    readonly parent: Extract<TreeNode, { readonly children: unknown; }>;
  },
): readonly ParsedSpan[] {
  return unpositionedRuns({
    parent,
    text,
  },)
    .flatMap(function urlMarkers(run,): ParsedSpan[] {
      return gfmMarkerSpans({
        text: text.slice(
          run.regionStart,
          run.regionEnd,
        ),
      },)
        .filter(function outsideRunText(marker,): boolean {
          return !run.spans
            .some(function same(kept,): boolean {
              return kept.startOffset === marker.startOffset;
            },);
        },)
        .map(function shifted(marker,): ParsedSpan {
          return {
            start: run.regionStart + marker.startOffset,
            end: run.regionStart + marker.endOffset,
          };
        },);
    },);
}

/**
 Spans one node adds, by what the node is.

 @param node - node the walk holds

 @param text - fragment the parser read

 @returns Spans of this node itself, none for a node whose children carry
 the text

 @example
 ```ts
 const spans = ownSpans({ node, text, },);
 ```
 */
function ownSpans(
  {
    node,
    text,
  }: {
    readonly node: TreeNode;
    readonly text: string;
  },
): readonly ParsedSpan[] {
  /**
   The node's own span, absent on a node the transform rebuilt.
   */
  const bounds = nodeBounds(node,);
  if (bounds === NO_NODE_BOUNDS)
    return [];
  if ((node.type === 'inlineCode') || (node.type === 'image')
    || (node.type === 'imageReference')
    || (node.type === 'definition'))
    return [bounds,];
  if (node.type === 'linkReference')
    return outsideLabelSpans({
      node,
      bounds,
    },);
  if (node.type !== 'link')
    return [];
  if (isAutolinkLiteral(node,) || isAngleAutolink({
    node,
    text,
  },))
    return [bounds,];
  return outsideLabelSpans({
    node,
    bounds,
  },);
}

/**
 Reads the spans one parse of the masked text gives, a masked comment's among
 them.

 @param root - tree one grammar read the masked text into

 @param masked - text the parser read

 @param comments - spans of the comments masked out of it

 @returns Disjoint spans in source order

 @throws FootnoteRewriteError of the `position` kind when an unpositioned run
 cannot be placed in the raw text, a tree shape no input is known to build

 @example
 ```ts
 const spans = spansOfTree({ root: parseMarkdownBody({ body: masked, },), masked, comments: [], },);
 ```
 */
function spansOfTree(
  {
    root,
    masked,
    comments,
  }: {
    readonly root: TreeNode;
    readonly masked: string;
    readonly comments: readonly ParsedSpan[];
  },
): readonly ParsedSpan[] {
  /**
   Spans found so far, in walk order.
   */
  const spans: ParsedSpan[] = [...comments,];
  /**
   Owned structural work-stack, avoiding recursion through nested blocks.
   */
  const work: TreeNode[] = [root,];
  for (let node = work.pop(); node !== undefined; node = work.pop()) {
    for (const own of ownSpans({
      node,
      text: masked,
    },))
      spans.push(own,);
    /**
     Whether this node is a URL `ownSpans` read whole, whose text is never
     searched for runs.
     */
    const isUrl = isAutolinkLiteral(node,) || isAngleAutolink({
      node,
      text: masked,
    },);
    if ((!('children' in node)) || isUrl)
      continue;
    for (const span of runUrlSpans({
      text: masked,
      parent: node,
    },))
      spans.push(span,);
    for (const child of node.children
      .toReversed())
      work.push(child,);
  }
  return disjointSpans({ spans, },);
}

/**
 What one grammar's parse of a fragment settles: where it reads no footnote
 reference, and where it opens a footnote definition.

 @example
 ```ts
 const reading: GrammarReading = { spans: [], definitionStarts: new Set([0,],), };
 ```
 */
type GrammarReading = {
  /**
   Disjoint spans in source order where the parse reads no reference.
   */
  readonly spans: readonly ParsedSpan[];

  /**
   Offsets of the `[` of every footnote definition the parse reads as a
   top-level block, containers dissolved, which is where the footnote graph
   reads a definition.
   */
  readonly definitionStarts: ReadonlySet<number>;
};

/**
 Reads what one parse of the masked text settles.

 @param root - tree one grammar read the masked text into

 @param masked - text the parser read

 @param comments - spans of the comments masked out of it

 @returns Spans and definition starts

 @throws FootnoteRewriteError of the `position` kind when an unpositioned run
 cannot be placed in the raw text, a tree shape no input is known to build

 @throws {@link Error} when a parsed footnote definition carries no offset,
 though the parser sets one on every node it builds

 @example
 ```ts
 const reading = readTree({ root: parseMarkdownBody({ body: masked, },), masked, comments: [], },);
 ```
 */
function readTree(
  {
    root,
    masked,
    comments,
  }: {
    readonly root: Root;
    readonly masked: string;
    readonly comments: readonly ParsedSpan[];
  },
): GrammarReading {
  return {
    spans: spansOfTree({
      root,
      masked,
      comments,
    },),
    definitionStarts: definitionStartsOf({ root, },),
  };
}

/**
 Plain markdown could not read the text, distinct from a text it reads with no
 span in it.

 @example
 ```ts
 if (plain === NO_PLAIN_READING) return [];
 ```
 */
const NO_PLAIN_READING: unique symbol = Symbol('plain markdown refused the text for its nesting',);

/**
 Reads what plain markdown settles.

 @param masked - text the parser reads

 @param comments - spans of the comments masked out of it

 @returns Reading, or {@link NO_PLAIN_READING} when the parser's stack is exhausted

 @throws FootnoteRewriteError of the `position` kind when an unpositioned run
 cannot be placed in the raw text, a tree shape no input is known to build

 @example
 ```ts
 const plain = plainReading({ masked: 'A `nap [^6]`.', comments: [], },);
 ```
 */
function plainReading(
  {
    masked,
    comments,
  }: {
    readonly masked: string;
    readonly comments: readonly ParsedSpan[];
  },
): GrammarReading | typeof NO_PLAIN_READING {
  try {
    return readTree({
      root: withMarkdownExhaustionRefused({
        parse: parseMarkdownBody,
        input: { body: masked, },
      },),
      masked,
      comments,
    },);
  }
  catch (error) {
    // Only the plain grammar's own refusal, a nesting past the parse bound
    // or one that exhausts its stack, reads as nothing known; anything else is an unexpected state
    // that must keep propagating.
    requireMarkdownRefusal({ error, },);
    return NO_PLAIN_READING;
  }
}

/**
 Intersects two lists of disjoint spans in source order.

 @param left - disjoint spans in source order

 @param right - disjoint spans in source order

 @returns Stretches both lists cover, disjoint and in source order

 @example
 ```ts
 const both = intersectSpans({ left: [{ start: 0, end: 5, },], right: [{ start: 3, end: 9, },], },);
 ```
 */
function intersectSpans(
  {
    left,
    right,
  }: {
    readonly left: readonly ParsedSpan[];
    readonly right: readonly ParsedSpan[];
  },
): readonly ParsedSpan[] {
  /**
   Owned result list.
   */
  const both: ParsedSpan[] = [];
  for (let leftAt = 0, rightAt = 0; (leftAt < left.length) && (rightAt < right.length);) {
    /**
     The left list's span under its cursor, present because both cursors lie inside their lists.
     */
    const leftSpan = nonNullishOrThrow(left[leftAt],);
    /**
     The right list's span under its cursor.
     */
    const rightSpan = nonNullishOrThrow(right[rightAt],);
    /**
     Start of the stretch both spans cover, past its end when they do not meet.
     */
    const start = Math.max(
      leftSpan.start,
      rightSpan.start,
    );
    /**
     End of that stretch.
     */
    const end = Math.min(
      leftSpan.end,
      rightSpan.end,
    );
    if (start < end)
      both.push({
        start,
        end,
      },);
    // The span that ends first cannot meet anything further on.
    if (leftSpan.end < rightSpan.end)
      leftAt += 1;
    else
      rightAt += 1;
  }
  return both;
}

/**
 Where a fragment's footnote definitions open, as far as the parse settles it.

 @example
 ```ts
 const definitions: DefinitionReading = { settled: true, starts: new Set([0,],), };
 ```
 */
export type DefinitionReading =
  | {
    /**
     Plain markdown could not read the fragment, so no offset is known.
     */
    readonly settled: false;
  }
    | {
    /**
     The parse of the fragment settled where definitions open.
     */
      readonly settled: true;

    /**
     Offsets of the `[` of each footnote definition the page's own grammar
     reads as a top-level block: the strict grammar's where it accepts the
     fragment, plain markdown's where it does not.
     */
      readonly starts: ReadonlySet<number>;
    };

/**
 What the parse of a fragment settles: where it finds no footnote reference,
 and where it opens a footnote definition.

 @example
 ```ts
 const reading: FragmentReading = { spans: [], definitions: { settled: true, starts: new Set(), }, };
 ```
 */
export type FragmentReading = {
  /**
   Disjoint spans in source order where the parse finds no reference.
   */
  readonly spans: readonly ParsedSpan[];

  /**
   Where the parse opens a definition.
   */
  readonly definitions: DefinitionReading;
};

/**
 Reads a fragment under both grammars: the spans in which the parse finds no
 footnote reference, in source order and disjoint, and the offsets at which
 it opens a footnote definition.

 THE PAGE MAY BE READ UNDER EITHER GRAMMAR (strict MDX where it accepts the
 page, plain markdown where it does not), and a fragment cannot know which.
 The two differ in block structure (an indented fence is code to one and a
 line of a paragraph to the other), so a region is read only where BOTH
 grammars read it, which can only leave a marker counted. Where the strict
 grammar refuses the fragment, plain markdown alone decides.

 A DEFINITION IS ONE THE PAGE'S OWN GRAMMAR OPENS, read the way `parseDocument`
 chooses it: strict MDX where it accepts the fragment, plain markdown where it
 does not. A label four spaces in or more is a definition to the strict
 grammar, which has no indented code, and a code block or a line of the
 paragraph to plain markdown, and the footnote graph reads whichever grammar
 the page was parsed under. Skipping a region needs both grammars because a
 dropped reference is the costly error; no role is the safe side, so the role
 follows the grammar the page most likely has, which is the strict one for
 a corpus that compiles as MDX upstream.

 @internal

 @param text - fragment to read, whose front matter is not split and whose
 comments are masked here

 @returns Spans of URLs, inline code spans, images, link reference
 definitions, the markup around link and reference labels, and masked
 comments; none when the fragment holds no marker shape, or when the plain
 parser refuses it for its nesting, so that every marker there stays counted.
 The definition offsets, unsettled in that same refusal.

 @throws FootnoteRewriteError of the `position` kind when an unpositioned run
 cannot be placed in the raw text, a tree shape no input is known to build

 @throws {@link Error} when a label child of a positioned link carries no
 position, which no parse builds

 @example
 ```ts
 const { spans, definitions, } = fragmentReadingOf({ text: 'See https://cat.example/[^9]x [^1].', },);
 ```
 */
export function fragmentReadingOf({ text, }: { readonly text: string; },): FragmentReading {
  if (!text.includes('[^',))
    return {
      spans: [],
      definitions: {
        settled: true,
        starts: new Set(),
      },
    };
  /**
   The text as the page's parse reads it: no invisible-only line, no comment.
   */
  const { masked: unwelded, } = maskInvisibleLines({ text, },);
  /**
   The invisible-line mask with its comments masked too, and each comment's region.
   */
  const {
    masked,
    regions,
  } = maskHtmlComments({ text: unwelded, },);
  /**
   Spans of the comments masked out.
   */
  const comments = regions.map(function toSpan(region,): ParsedSpan {
    return {
      start: region.startOffset,
      end: region.endOffset,
    };
  },);
  /**
   What plain markdown settles, absent where its parser refuses the nesting.
   */
  const plain = plainReading({
    masked,
    comments,
  },);
  if (plain === NO_PLAIN_READING)
    return {
      spans: [],
      definitions: { settled: false, },
    };
  try {
    /**
     What the strict grammar settles.
     */
    const strict = readTree({
      root: withMdxExhaustionRefused({
        parse: parseMdxBody,
        input: { body: masked, },
      },),
      masked,
      comments,
    },);
    return {
      spans: intersectSpans({
        left: plain.spans,
        right: strict.spans,
      },),
      definitions: {
        settled: true,
        starts: strict.definitionStarts,
      },
    };
  }
  catch (error) {
    // The strict grammar refused the fragment, so only plain markdown reads
    // it; anything but its refusal is an unexpected state that must keep
    // propagating.
    requireMdxRefusal({ error, },);
    return {
      spans: plain.spans,
      definitions: {
        settled: true,
        starts: plain.definitionStarts,
      },
    };
  }
}

/**
 Whether an offset lies inside one of the spans, by binary search over their
 order.

 @param spans - disjoint spans in source order

 @param offset - offset to place

 @returns Whether a span holds the offset

 @example
 ```ts
 const inside = insideParsedSpan({ spans: fragmentReadingOf({ text, },).spans, offset: 20, },);
 ```
 */
export function insideParsedSpan(
  {
    spans,
    offset,
  }: {
    readonly spans: readonly ParsedSpan[];
    readonly offset: number;
  },
): boolean {
  for (let high = spans.length, low = 0; low < high;) {
    /**
     Middle of the indices in play.
     */
    const middle = Math.floor((low + high) / 2,);
    /**
     Span at that index, present because the middle lies inside the range.
     */
    const span = spans[middle];
    if (span === undefined)
      throw new Error(`unreachable: no span at index ${String(middle,)} of the spans a search was given`,);
    if (offset < span.start)
      high = middle;
    else if (offset >= span.end)
      low = middle + 1;
    else
      return true;
  }
  return false;
}

//endregion Footnote parsed spans
