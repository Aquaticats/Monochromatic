/**
 Adversarial tests for deterministic anchor validation:
 every way an unreliable critic misquotes, drifts, or fabricates anchors must be
 rejected as data.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  type DocumentSide,
  type IssueClaim,
  parseDocument,
  type SpanAnchor,
  validateIssueClaim,
} from '../dist/final/node/index.mjs';

/**
 Invented zh source with heading, footnoted paragraph, and definition.
 */
const SOURCE_TEXT =
  '---\nname: 小猫-whiskers\n---\n\n## 简介\n\n猫猫喜欢晒太阳，也喜欢追蝴蝶。[^1]\n\n[^1]:[关于猫猫习性的说明。](https://example.org/cat)\n';

/**
 Invented en translation missing the butterfly clause,
 the shape omission claims anchor against.
 */
const TARGET_TEXT =
  '---\nname: 小猫-whiskers\n---\n\n## Introduction\n\nThe cat likes to nap in the sun.\n';

/**
 Parsed pair every test validates anchors against.
 */
const DOCUMENTS = {
  source: parseDocument({ text: SOURCE_TEXT, },),
  target: parseDocument({ text: TARGET_TEXT, },),
} as const;

/**
 Builds correctly anchored span for one needle by measuring the parsed pair,
 so tests never hand-count offsets.

 @param side - document of the pair the needle lives in

 @param needle - exact text to anchor

 @returns Span whose offsets, node, hash, and quote all hold

 @throws {@link Error} when needle is absent or spans no single node

 @example
 ```ts
 const span = anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },);
 ```
 */
function anchorFor(
  {
    side,
    needle,
  }: {
    readonly side: DocumentSide;
    readonly needle: string;
  },
): SpanAnchor {
  /**
   Document the needle is measured against.
   */
  const document = DOCUMENTS[side];

  /**
   Absolute start of needle within document source.
   */
  const startOffset = document.text.indexOf(needle,);
  if (startOffset === (-1)) {
    throw new Error(
      `fixture needle ${JSON.stringify(needle,)} missing from ${side} document`,
    );
  }

  /**
   Absolute end (exclusive) of needle.
   */
  const endOffset = startOffset + needle.length;

  /**
   Node containing the whole needle, proven present for fixture needles.
   */
  const node = nonNullishOrThrow(document.nodes.find(function containing(candidate,) {
    return (startOffset >= candidate.startOffset) && (endOffset <= candidate.endOffset);
  },),);

  return {
    side,
    nodeId: node.id,
    nodeHash: node.contentHash,
    startOffset,
    endOffset,
    quotedText: needle,
  };
}

/**
 Builds zero-width insertion anchor immediately after one needle,
 the shape omission claims use to name where missing content belongs.

 @param side - document of the pair the needle lives in

 @param needle - exact text the insertion point follows

 @returns Zero-width span with empty quote at needle end

 @example
 ```ts
 const anchor = insertionAnchorAfter({ side: 'target', needle: 'in the sun.', },);
 ```
 */
function insertionAnchorAfter(
  {
    side,
    needle,
  }: {
    readonly side: DocumentSide;
    readonly needle: string;
  },
): SpanAnchor {
  /**
   Anchored needle whose end names the insertion point.
   */
  const base = anchorFor({
    side,
    needle,
  },);

  return {
    ...base,
    startOffset: base.endOffset,
    endOffset: base.endOffset,
    quotedText: '',
  };
}

/**
 Wraps spans in an omission claim so tests vary anchors only.

 @param spans - anchors under test

 @returns Claim carrying given spans

 @example
 ```ts
 const claim = omissionClaim({ spans: [span,], },);
 ```
 */
function omissionClaim(
  { spans, }: { readonly spans: readonly SpanAnchor[]; },
): IssueClaim {
  return {
    category: 'accuracy/omission',
    severity: 'major',
    summary: '追蝴蝶那句没有翻译。',
    spans,
  };
}

/**
 Fractional offset for malformed-offset rejection tests;
 named because it sits outside the exempt literal range.
 */
const FRACTIONAL_OFFSET = 1.5;

await describe({
  name: validateIssueClaim.name,
  children: [
    it({
      name: 'admits a fully anchored multi-span omission claim',
      fn: async () => {
        expect(validateIssueClaim({
          claim: omissionClaim({
            spans: [
              anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },),
              insertionAnchorAfter({ side: 'target', needle: 'in the sun.', },),
            ],
          },),
          documents: DOCUMENTS,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'rejects non-empty quotes on zero-width spans as quote mismatches',
      fn: async () => {
        /** Insertion anchor corrupted with a phantom quote. */
        const phantom: SpanAnchor = {
          ...insertionAnchorAfter({ side: 'target', needle: 'in the sun.', },),
          quotedText: 'and chases butterflies',
        };
        /** Rejections for the phantom quote. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [phantom,], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(1,);
        expect(rejections[0]?.kind,).toBe('quote-mismatch',);
      },
    },),

    it({
      name: 'rejects paraphrased quotes at correct offsets, naming both texts',
      fn: async () => {
        /** Correct offsets carrying a paraphrase instead of the exact slice. */
        const paraphrased: SpanAnchor = {
          ...anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },),
          quotedText: '也喜欢晒月亮',
        };
        /** Rejections for the paraphrase. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [paraphrased,], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(1,);
        expect(rejections[0]?.kind,).toBe('quote-mismatch',);
        expect(rejections[0]?.detail,).toContain('也喜欢晒月亮',);
        expect(rejections[0]?.detail,).toContain('也喜欢追蝴蝶',);
      },
    },),

    it({
      name: 'rejects offsets shifted off the quoted text by one',
      fn: async () => {
        /** Correct quote whose offsets drifted right by one character. */
        const drifted: SpanAnchor = {
          ...anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },),
          startOffset: anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },).startOffset + 1,
          endOffset: anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },).endOffset + 1,
        };
        /** Rejections for the drifted offsets. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [drifted,], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(1,);
        expect(rejections[0]?.kind,).toBe('quote-mismatch',);
      },
    },),

    it({
      name: 'rejects anchors built against a drifted base as stale before quote checks',
      fn: async () => {
        /** Same-length edit so offsets still hold and only the hash differs. */
        const editedSource = parseDocument({
          text: SOURCE_TEXT.replace(
            '晒太阳',
            '晒月亮',
          ),
        },);
        /** Node the edited-base anchor was built against. */
        const editedNode = nonNullishOrThrow(editedSource.nodes.find(function byKind(candidate,) {
          return candidate.kind === 'paragraph';
        },),);
        /** Anchor valid against the edited base, stale against the original. */
        const staleAnchor: SpanAnchor = {
          ...anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },),
          nodeHash: editedNode.contentHash,
        };
        /** Rejections for the stale base. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [staleAnchor,], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(1,);
        expect(rejections[0]?.kind,).toBe('stale-node-hash',);
      },
    },),

    it({
      name: 'rejects spans naming nodes that do not exist',
      fn: async () => {
        /** Valid anchor re-pointed at a fabricated node id. */
        const fabricated: SpanAnchor = {
          ...anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },),
          nodeId: 'block/99',
        };
        /** Rejections for the fabricated node. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [fabricated,], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(1,);
        expect(rejections[0]?.kind,).toBe('unknown-node',);
      },
    },),

    it({
      name: 'rejects real text claimed under the wrong node',
      fn: async () => {
        /** Heading node whose identity gets misapplied to paragraph offsets. */
        const heading = nonNullishOrThrow(DOCUMENTS.target.nodes.find(function byKind(candidate,) {
          return candidate.kind === 'heading';
        },),);
        /** Paragraph offsets under heading identity: text exists, node claim lies. */
        const misfiled: SpanAnchor = {
          ...anchorFor({ side: 'target', needle: 'The cat likes', },),
          nodeId: heading.id,
          nodeHash: heading.contentHash,
        };
        /** Rejections for the misfiled span. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [misfiled,], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(1,);
        expect(rejections[0]?.kind,).toBe('span-outside-node',);
      },
    },),

    it({
      name: 'rejects inverted spans',
      fn: async () => {
        /** Anchor whose offsets swapped ends. */
        const inverted: SpanAnchor = {
          ...anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },),
          startOffset: anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },).endOffset,
          endOffset: anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },).startOffset,
        };
        /** Rejections for the inverted span. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [inverted,], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(1,);
        expect(rejections[0]?.kind,).toBe('inverted-span',);
      },
    },),

    it({
      name: 'rejects fractional and negative offsets as malformed',
      fn: async () => {
        /** Anchor with fractional start. */
        const fractional: SpanAnchor = {
          ...anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },),
          startOffset: FRACTIONAL_OFFSET,
        };
        /** Anchor with negative end. */
        const negative: SpanAnchor = {
          ...anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },),
          startOffset: -1,
          endOffset: -1,
        };
        /** Rejections across both malformed spans. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [fractional, negative,], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(2,);
        expect(rejections[0]?.kind,).toBe('malformed-offset',);
        expect(rejections[0]?.spanIndex,).toBe(0,);
        expect(rejections[1]?.kind,).toBe('malformed-offset',);
        expect(rejections[1]?.spanIndex,).toBe(1,);
      },
    },),

    it({
      name: 'rejects claims without any span as anchorless',
      fn: async () => {
        /** Rejections for the anchorless claim. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({ spans: [], },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(1,);
        expect(rejections[0]?.kind,).toBe('anchorless-issue',);
        expect(rejections[0]?.spanIndex,).toBe(undefined,);
      },
    },),

    it({
      name: 'REJECTS a span whose node id the document lists more than once, naming that defect of the '
        + 'document rather than calling the span stale against the first of them, and ADMITS the same '
        + 'span against the document as parsed',
      fn: async () => {
        /**
         Span anchored in the translation's paragraph, correct in every field.
         */
        const span = anchorFor({ side: 'target', needle: 'The cat likes', },);

        /**
         The translation with every block renamed to the paragraph's id, so
         the heading is the first node a search by that id meets.
         */
        const repeating = {
          text: DOCUMENTS.target.text,
          nodes: DOCUMENTS.target.nodes.map(function renamed(node,) {
            return {
              ...node,
              id: span.nodeId,
            };
          },),
        };

        expect(validateIssueClaim({
          claim: omissionClaim({ spans: [span,], },),
          documents: {
            source: DOCUMENTS.source,
            target: repeating,
          },
        },),).toEqual([{
          kind: 'repeated-node-id',
          spanIndex: 0,
          detail: `span 0 (target) names node ${span.nodeId}, which the target document lists more than `
            + 'once, so the span cannot be checked against the node it was built on.',
        },],);
        // The same span against the document as parsed holds.
        expect(validateIssueClaim({
          claim: omissionClaim({ spans: [span,], },),
          documents: DOCUMENTS,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'reports every defective span independently',
      fn: async () => {
        /** Valid anchor sandwiched between two defective ones. */
        const rejections = validateIssueClaim({
          claim: omissionClaim({
            spans: [
              { ...anchorFor({ side: 'source', needle: '也喜欢追蝴蝶', },), nodeId: 'block/99', },
              anchorFor({ side: 'target', needle: 'The cat likes', },),
              {
                ...anchorFor({ side: 'target', needle: 'The cat likes', },),
                quotedText: 'The dog likes',
              },
            ],
          },),
          documents: DOCUMENTS,
        },);
        expect(rejections,).toHaveLength(2,);
        expect(rejections[0]?.kind,).toBe('unknown-node',);
        expect(rejections[0]?.spanIndex,).toBe(0,);
        expect(rejections[1]?.kind,).toBe('quote-mismatch',);
        expect(rejections[1]?.spanIndex,).toBe(2,);
      },
    },),
  ],
},);
