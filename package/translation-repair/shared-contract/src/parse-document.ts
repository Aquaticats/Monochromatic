import type { Root, } from 'mdast';

import {
  buildDocumentNodes,
  buildFootnoteGraph,
  type ContainerSpan,
  type DocumentNode,
  flattenContainers,
  type FrontMatterBlock,
  hashContent,
  maskHtmlComments,
  maskInvisibleLines,
  parseMarkdownBody,
  parseMdxBody,
  requireMdxRefusal,
  splitFrontMatter,
  widenNodesToContainers,
  withMarkdownExhaustionRefused,
  withMdxExhaustionRefused,
  type FootnoteGraph,
  type MarkdownParseError,
  type FrontMatterParseError,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Document parsing
// Composition of the deterministic core: front matter split, tolerant parse
// (invisible-line masking, comment masking, then strict MDX, then plain-markdown
// fallback), node construction, footnote graph. Pure over its input; every later
// stage anchors claims against the result and its hashes. Tolerance is never
// silent: every blanked line, every skipped comment and every grammar downgrade
// surfaces as a parse finding.

/**
 One tolerance event from parsing:
 a blanked invisible-only line, a masked HTML comment, or a whole-document
 grammar downgrade.
 Findings are the trigger surface for any later repair-the-input stage
 (deterministic or model-driven).

 @example
 ```ts
 const finding: ParseFinding = {
   kind: 'html-comment-skipped',
   startOffset: 42,
   endOffset: 60,
   detail: 'terminated HTML comment masked before parsing',
 };
 ```
 */
export type ParseFinding = {
  /**
   Tolerance class:
   masked comment, comment that never closed, blanked invisible-only line, or
   MDX-to-markdown downgrade.
   */
  readonly kind:
    | 'html-comment-skipped'
      | 'unterminated-html-comment'
      | 'invisible-line-masked'
      | 'mdx-downgraded';

  /**
   Absolute start offset of the affected region in the full document text.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive end offset of the affected region.
   */
  readonly endOffset: number;

  /**
   Human-readable cause; downgrade findings carry the strict parser's reason.
   */
  readonly detail: string;
};

/**
 Whether a parse finding marks an HTML comment, closed or left open.

 The archive-original seal, the entry's comment notes and the footnote
 protected ranges each kept their own copy of this pair of kinds (audit area
 six, 2026-09-28); a comment kind added to one would have gone unseen by the
 others.

 @param finding - one tolerance event from parsing

 @returns True for a masked comment and for one that never closed

 @example
 ```ts
 const comments = document.parseFindings.filter(function isComment(finding,): boolean {
   return isCommentFinding({ finding, },);
 },);
 ```
 */
export function isCommentFinding({ finding, }: { readonly finding: ParseFinding; },): boolean {
  return (finding.kind === 'html-comment-skipped')
    || (finding.kind === 'unterminated-html-comment');
}

/**
 Immutable parsed form of one corpus document,
 carrying every anchor later stages validate claims against.

 @example
 ```ts
 const doc = parseDocument({ text: source, },);
 console.log(doc.nodes.length, doc.footnoteGraph.findings,);
 ```
 */
export type RepairDocument = {
  /**
   Full original source, byte-for-byte.
   */
  readonly text: string;

  /**
   SHA-256 of text; patch operations carry this to reject stale bases.
   */
  readonly documentHash: string;

  /**
   Verbatim front matter when present; never rewritten by repairs.
   */
  readonly frontMatter?: FrontMatterBlock;

  /**
   Block-level nodes in source order with absolute offsets and hashes.
   */
  readonly nodes: readonly DocumentNode[];

  /**
   Every container dissolved to produce those nodes, in source order, with
   absolute offsets for both of its tags.

   CARRIED BECAUSE THE NODES CANNOT SHOW IT. Container tags belong to none of
   the promoted children, so a reader handed only nodes sees inter-block text
   it has no reason to protect, and a slice range covering one tag and not the
   other destroys the element. Empty on a page using no containers.
   */
  readonly containers: readonly ContainerSpan[];

  /**
   Reference-to-definition graph across both footnote conventions,
   with integrity findings for human checkpoints.
   */
  readonly footnoteGraph: FootnoteGraph;

  /**
   Tolerance events from parsing, in source order;
   empty when the document parsed strictly with nothing masked.
   */
  readonly parseFindings: readonly ParseFinding[];
};

/**
 Parses body text tolerantly:
 strict MDX first, plain markdown on grammar failure.
 The downgrade is never silent; the finding carries the strict reason.

 @internal

 @param body - body text, comments already masked

 @param bodyOffset - absolute offset of body within the full document

 @returns mdast root plus downgrade findings when the strict grammar failed

 @throws {@link MarkdownParseError} when plain markdown refuses the body too,
 its nesting being past the parse bound or having exhausted the parser's stack
 (`parse-mdx.ts` `requireMarkdownRefusal`)

 @example
 ```ts
 const { root, findings, } = parseBodyTolerant({ body: masked, bodyOffset: 0, },);
 ```
 */
export function parseBodyTolerant(
  {
    body,
    bodyOffset,
  }: {
    readonly body: string;
    readonly bodyOffset: number;
  },
): {
  readonly root: Root;
  readonly findings: readonly ParseFinding[];
} {
  try {
    return {
      root: withMdxExhaustionRefused({
        parse: parseMdxBody,
        input: { body, },
      },),
      findings: [],
    };
  }
  catch (error) {
    // Only the strict grammar's own rejection downgrades; anything else
    // is an unexpected state that must keep propagating.
    /**
     The grammar's refusal.
     */
    const refusal = requireMdxRefusal({ error, },);
    return {
      root: withMarkdownExhaustionRefused({
        parse: parseMarkdownBody,
        input: { body, },
      },),
      findings: [{
        kind: 'mdx-downgraded',
        startOffset: bodyOffset,
        endOffset: bodyOffset + body.length,
        detail: `strict MDX parse failed, fell back to plain markdown: ${
          refusal.message
        }`,
      },],
    };
  }
}

/**
 Parses one corpus document into its immutable anchor-ready form.
 HTML comments are masked to whitespace before parsing (the dominant
 real-corpus failure class), and a document the MDX grammar still rejects
 reparses as plain markdown;
 both tolerances surface as parse findings, never as thrown errors.

 @param text - full document source, front matter included when present

 @returns Parsed document with nodes, hashes, footnote graph, and findings

 @throws {@link FrontMatterParseError} when fenced YAML refuses to parse

 @throws {@link MarkdownParseError} when plain markdown refuses the body too,
 its nesting being past the parse bound or having exhausted the parser's stack

 @example
 ```ts
 const doc = parseDocument({ text: '---\nname: n\n---\n\n## 简介\n\n正文[^1]\n\n[^1]: 注\n', },);
 ```
 */
export function parseDocument({ text, }: { readonly text: string; },): RepairDocument {
  /**
   Front matter split with body offset for absolute anchoring.
   */
  const split = splitFrontMatter({ text, },);

  /**
   Body with every invisible-only line blanked, plus each line blanked.

   Runs FIRST, because such a line is not blank to CommonMark and therefore
   welds the paragraphs either side of it into one block. One corpus
   translation parses to 29 blocks that way against the original's 33, and
   every block after the first weld pairs with the wrong original. Length is
   preserved, so every offset still indexes the same character.
   */
  const {
    masked: unwelded,
    regions: invisibleRegions,
  } = maskInvisibleLines({ text: split.body, },);

  /**
   Body with HTML comments blanked, plus each masked region;
   masking preserves length, so masked-parse positions index the original.
   */
  const {
    masked,
    regions,
  } = maskHtmlComments({ text: unwelded, },);

  /**
   Findings for every blanked invisible-only line, in absolute offsets.

   Emitted so the tolerance is never silent. Both parser defects this pipeline
   has hit were found by accident rather than from an artifact, and a line
   that vanishes with nothing recording it is exactly the shape that hides the
   third one.
   */
  const invisibleFindings = invisibleRegions.map(
    function toFinding(region,): ParseFinding {
      /**
       Code points the line carried, named so the detail can be read.
       */
      const { codePoints, } = region;

      return {
        kind: 'invisible-line-masked',
        startOffset: split.bodyOffset + region.startOffset,
        endOffset: split.bodyOffset + region.endOffset,
        detail: `line showing nothing yet not blank to CommonMark, welding the paragraphs either side of it; masked to spaces before parsing (${
          codePoints.join(', ',)
        })`,
      };
    },
  );

  /**
   Findings for every masked comment, in absolute document offsets.
   */
  const commentFindings = regions.map(function toFinding(region,): ParseFinding {
    return {
      kind: region.terminated
        ? 'html-comment-skipped'
        : 'unterminated-html-comment',
      startOffset: split.bodyOffset + region.startOffset,
      endOffset: split.bodyOffset + region.endOffset,
      detail: region.terminated
        ? 'HTML comment masked to whitespace before parsing'
        : 'unterminated HTML comment swallowed the rest of the body; masked to whitespace before parsing',
    };
  },);

  /**
   Tolerantly parsed body tree plus any grammar-downgrade finding.
   */
  const parsed = parseBodyTolerant({
    body: masked,
    bodyOffset: split.bodyOffset,
  },);

  /**
   Top-level blocks once disclosure containers are unwrapped.

   Computed ONCE and shared, because the node list and the footnote graph have
   to agree both about what counts as a top-level block and about what
   `block/N` names. They did not agree: the graph walked the RAW children, so
   a footnote definition sitting inside a container was invisible to it while
   being promoted for the node list, and every `nodeId` it emitted counted
   containers the node list had already unwrapped.

   One corpus translation reported all ten of its references unresolved while
   carrying all ten definitions, because every definition sat inside one.
   */
  const {
    blocks,
    containers,
  } = flattenContainers({
    children: parsed
      .root
      .children,
  },);

  /**
   Container spans shifted to absolute offsets, so every consumer reads
   container tags in the same frame as the node offsets beside them. The walk
   itself sees only body-relative positions and cannot do this.
   */
  const absoluteContainers = containers.map(function toAbsolute(container,): ContainerSpan {
    return {
      name: container.name,
      openerStartOffset: split.bodyOffset + container.openerStartOffset,
      openerEndOffset: split.bodyOffset + container.openerEndOffset,
      closerStartOffset: split.bodyOffset + container.closerStartOffset,
      closerEndOffset: split.bodyOffset + container.closerEndOffset,
    };
  },);

  return {
    text,
    documentHash: hashContent({ content: text, },),
    // Conditional spread keeps frontMatter absent (not explicitly undefined)
    // under exactOptionalPropertyTypes.
    ...(split.frontMatter === undefined
      ? {}
      : { frontMatter: split.frontMatter, }),
    // Node text stays sliced from the ORIGINAL document, keeping every quote,
    // hash, and anchor consistent with its canonical text.
    // Disclosure containers flatten first so a page nesting blocks exposes the
    // same top-level structure as a counterpart that does not; chunking and
    // alignment walk top-level blocks only.
    //
    // THEN THE EDGE BLOCKS TAKE THE TAGS THAT FLATTENING ORPHANED. Dissolving
    // a container leaves its two tags belonging to no node, and every range in
    // this package is minted from node offsets, so a boundary could fall
    // between an opening tag and its closing one and delete half a disclosure
    // block. Widening here rather than at the two sites that mint ranges is
    // what makes both of them correct at once.
    nodes: widenNodesToContainers({
      nodes: buildDocumentNodes({
        children: blocks,
        bodyText: split.body,
        bodyOffset: split.bodyOffset,
      },),
      text,
      containers: absoluteContainers,
    },),
    containers: absoluteContainers,
    // The footnote graph scans the MASKED body so commented-out marker
    // look-alikes never become phantom references or definitions.
    footnoteGraph: buildFootnoteGraph({
      children: blocks,
      bodyText: masked,
      bodyOffset: split.bodyOffset,
    },),
    // Sorted, because `parseFindings` promises source order and the three
    // sources are built in processing order instead: a downgrade finding starts
    // at the body offset, so it would otherwise sort ahead of comments that
    // appear much later in the text.
    parseFindings: [
      ...invisibleFindings,
      ...commentFindings,
      ...parsed.findings,
    ].toSorted(function byOffset(
      left,
      right,
    ) {
      return left.startOffset - right.startOffset;
    },),
  };
}

//endregion Document parsing
