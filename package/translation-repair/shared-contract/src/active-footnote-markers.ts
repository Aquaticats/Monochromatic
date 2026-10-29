import type { RootContent, } from 'mdast';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  type DeepReadonlyData,
  FootnoteRewriteError,
  gfmMarkerAt,
  type GfmMarkerSpan,
  gfmMarkerSpans,
  isAutolinkLiteral,
  maskHtmlComments,
  maskInvisibleLines,
  NO_NODE_BOUNDS,
  nodeBounds,
  normalizeFootnoteIdentifier,
  parseMdxBody,
  requireMdxRefusal,
  splitFrontMatter,
  type TreeNode,
  unpositionedRuns,
  withMdxExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Syntax-authorized footnote markers
// AST nodes supply context; masked raw text supplies unresolved references and exact encoded spans.

/**
 One active marker in the original document coordinate space.

 @example
 ```ts
 const references = activeFootnoteMarkers({ text }).filter(marker => marker.kind === 'reference');
 ```
 */
export type ActiveFootnoteMarker = GfmMarkerSpan & {
  /**
   Definition opener or reference, never code or metadata.
   */
  readonly kind: 'definition' | 'reference';
  /**
   Parser-equivalent key for comparison, separate from raw replacement syntax.
   */
  readonly identifier: string;
};

/**
 Reads raw and normalized identities from a positioned syntax node.

 @param node - parser-authorized footnote node

 @param text - exact masked body supplied to the parser

 @param bodyOffset - body origin in the complete document

 @returns Matching raw marker in document coordinates

 @throws FootnoteRewriteError when raw syntax reads no marker where the parser
 placed one, or reads a different identity

 @example
 ```ts
 const marker = positionedFootnote({ node, text, bodyOffset });
 ```
 */
function positionedFootnote(
  {
    node,
    text,
    bodyOffset,
  }: {
    readonly node: DeepReadonlyData<Extract<RootContent, { type: 'footnoteDefinition' | 'footnoteReference'; }>>;
    readonly text: string;
    readonly bodyOffset: number;
  },
): ActiveFootnoteMarker {
  /**
   Parser-authorized opening position, always present: both footnote node
   types are built through `mdast-util-from-markdown`'s `enter`
   (`mdast-util-gfm-footnote`), which sets every node's start from its token,
   and no transform the grammar runs builds a footnote node.
   */
  const offset = nonNullishOrThrow(node.position
    ?.start
    .offset,);
  /**
   Exact lexical end, independent of decoded label length.
   */
  const marker = gfmMarkerAt({
    text,
    offset,
  },);
  // NO MARKER WHERE THE PARSER PLACED ONE is the disagreement an input reaches:
  // micromark also forms a call from an image label (`![^a ]`, its
  // `tokenizePotentialGfmFootnoteCall`), whose label may hold the whitespace
  // the raw grammar refuses. The colon and end checks guard the parser
  // boundary against a grammar change: `gfmMarkerAt` reads the label as the
  // installed `micromark-extension-gfm-footnote` does, and no input found
  // either disagreeing (ledger T8).
  if (((typeof marker) === 'symbol') || ((node.type === 'footnoteDefinition') && (text[marker.endOffset] !== ':'))
    || ((node.type === 'footnoteReference') && (node.position
      ?.end
      .offset
      !== marker.endOffset)))
    throw new FootnoteRewriteError({ kind: 'position', },);
  /**
   Normalized raw spelling must agree with the parser's association.
   */
  const identifier = normalizeFootnoteIdentifier({ identifier: marker.rawLabel, },);
  if (identifier !== normalizeFootnoteIdentifier({ identifier: node.identifier, },))
    throw new FootnoteRewriteError({ kind: 'position', },);
  return {
    ...marker,
    identifier,
    kind: node.type === 'footnoteDefinition' ? 'definition' : 'reference',
    startOffset: bodyOffset + marker.startOffset,
    endOffset: bodyOffset + marker.endOffset,
  };
}

/**
 Places the marker lexemes read off one syntax-authorized raw region as
 literal-looking references in document coordinates.

 @param spans - lexemes of the region, their offsets relative to it

 @param regionStart - region origin in the masked parser input

 @param bodyOffset - body origin in the complete document

 @returns Reference markers in document coordinates, source order

 @example
 ```ts
 const placed = placedReferences({ spans: gfmMarkerSpans({ text: 'Missing[^9].', },), regionStart: 0, bodyOffset: 0, },);
 ```
 */
function placedReferences(
  {
    spans,
    regionStart,
    bodyOffset,
  }: {
    readonly spans: readonly GfmMarkerSpan[];
    readonly regionStart: number;
    readonly bodyOffset: number;
  },
): ActiveFootnoteMarker[] {
  return spans.map(function place(marker,): ActiveFootnoteMarker {
    return {
      ...marker,
      kind: 'reference',
      identifier: normalizeFootnoteIdentifier({ identifier: marker.rawLabel, },),
      startOffset: bodyOffset + regionStart
        + marker.startOffset,
      endOffset: bodyOffset + regionStart
        + marker.endOffset,
    };
  },);
}

/**
 Inventories active markers under strict document grammar without hiding unmatched container tags.
 Literal-looking references are recovered only from text in the masked parser input: the raw span of a
 positioned text node, or the raw behind the text nodes of an unpositioned run (ledger B123). The text
 of an autolink literal is its URL and is never read.

 @internal

 @param text - whole document or structural slice with canonical offsets

 @returns Active references and definition openers in source order

 @throws FootnoteRewriteError when syntax or positioned marker identity cannot be established, or an
 unpositioned run cannot be placed in the raw text

 @example
 ```ts
 const markers = activeFootnoteMarkers({ text: 'Real[^1].\n\n[^1]: Note.' });
 ```
 */
export function activeFootnoteMarkers({ text, }: { readonly text: string; },): readonly ActiveFootnoteMarker[] {
  if (!text.includes('[^',))
    return [];
  /**
   Front matter is not Markdown content and cannot supply active markers.
   */
  const {
    body,
    bodyOffset,
  } = splitFrontMatter({ text, },);
  /**
   Match document preparation's invisible-line boundaries without changing offsets.
   */
  const { masked, } = maskInvisibleLines({ text: body, },);
  try {
    /**
     Exact parser input, without rescuing unmatched or malformed JSX as a slice atom.
     */
    const { masked: parsedText, } = maskHtmlComments({ text: masked, },);
    /**
     A whole-document syntax failure cannot authorize raw-text marker edits.
     */
    const root = withMdxExhaustionRefused({
      parse: parseMdxBody,
      input: { body: parsedText, },
    },);
    /**
     References of each unpositioned run, keyed by the run's first member,
     where the walk emits them in source order.
     */
    const runReferences = new Map<TreeNode, readonly ActiveFootnoteMarker[]>();
    /**
     Owned structural work-stack, avoiding recursion through container spines.
     */
    const work: TreeNode[] = [root,];
    /**
     Positioned markers collected in source-order preorder.
     */
    const markers: ActiveFootnoteMarker[] = [];
    for (let node = work.pop(); node !== undefined; node = work.pop()) {
      if ((node.type === 'footnoteDefinition') || (node.type === 'footnoteReference'))
        markers.push(positionedFootnote({
          node,
          text: parsedText,
          bodyOffset,
        },),);
      else if (node.type === 'text') {
        /**
         Raw span of this text node, absent on a member of an unpositioned
         run, whose references the run's first member carries.
         */
        const bounds = nodeBounds(node,);
        if (bounds !== NO_NODE_BOUNDS) {
          for (const marker of placedReferences({
            spans: gfmMarkerSpans({ text: parsedText.slice(
              bounds.start,
              bounds.end,
            ), },),
            regionStart: bounds.start,
            bodyOffset,
          },))
            markers.push(marker,);
        }
      }
      /**
       References of the unpositioned run this node opens, absent for
       every other node.
       */
      const opened = runReferences.get(node,);
      if (opened !== undefined) {
        for (const marker of opened)
          markers.push(marker,);
      }
      // An autolink literal's text is its URL: neither read nor searched
      // for runs, a link the transform built being a member of its
      // parent's run already.
      if (('children' in node) && (!isAutolinkLiteral(node,))) {
        for (const run of unpositionedRuns({
          parent: node,
          text: parsedText,
        },))
          runReferences.set(
            run.opener,
            placedReferences({
              spans: run.spans,
              regionStart: run.regionStart,
              bodyOffset,
            },),
          );
        for (const child of node.children
          .toReversed())
          work.push(child,);
      }
    }
    return markers;
  }
  catch (error) {
    // A refusal of the grammar becomes a syntax rewrite error; anything
    // else, the position refusal thrown in the walk among it, propagates.
    throw new FootnoteRewriteError({
      kind: 'syntax',
      cause: requireMdxRefusal({ error, },),
    },);
  }
}

/**
 Keeps the first raw spelling of each logical marker identity.

 @param markers - positioned markers already restricted to the desired roles

 @returns Distinct raw labels in encounter order

 @example
 ```ts
 const labels = footnoteMarkerLabels({ markers: activeFootnoteMarkers({ text }) });
 ```
 */
export function footnoteMarkerLabels({ markers, }: { readonly markers: readonly ActiveFootnoteMarker[]; },): readonly string[] {
  /**
   First spelling per logical identifier.
   */
  const labels = new Map<string, string>();
  for (const marker of markers) {
    if (!labels.has(marker.identifier,))
      labels.set(
        marker.identifier,
        marker.rawLabel,
      );
  }
  return [...labels.values(),];
}

//endregion Syntax-authorized footnote markers
