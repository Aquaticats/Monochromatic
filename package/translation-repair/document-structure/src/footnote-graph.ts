import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { RootContent, } from 'mdast';

import { normalizeFootnoteIdentifier, } from './footnote-identifier.ts';
import { footnoteGraphFindings, } from './footnote-graph-findings.ts';
import {
  isAngleAutolink,
  isAutolinkLiteral,
  NO_NODE_BOUNDS,
  nodeBounds,
  type TreeNode,
  type UnpositionedRun,
  unpositionedRuns,
} from './footnote-unpositioned-runs.ts';
import {
  gfmMarkerSpans,
  type GfmMarkerSpan,
} from './gfm-marker-spans.ts';
import type {
  FootnoteDefinitionHit,
  FootnoteGraph,
  FootnoteReferenceHit,
} from './footnote-model.ts';

//region Text marker scanning
// 〔N〕 markers are plain text, not markdown syntax, so they are found by scanning
// source slices of text nodes. Scanning source (not decoded mdast values) keeps
// offsets faithful when values and source diverge through escapes.

/**
 Opening bracket of archive-convention footnote markers.
 */
const FULLWIDTH_OPEN = '〔';

/**
 Closing bracket of archive-convention footnote markers.
 */
const FULLWIDTH_CLOSE = '〕';

/**
 Digit characters accepted inside markers:
 ASCII first, full-width second, index modulo base yields digit value.
 */
const DIGIT_CHARS = '0123456789０１２３４５６７８９';

/**
 Numeric base folding full-width digit indexes onto ASCII digit values.
 */
const DECIMAL_BASE = 10;

/**
 One raw full-width marker found in a source slice.

 @example
 ```ts
 const hit: TextMarkerHit = { identifier: '1', localOffset: 2, };
 ```
 */
export type TextMarkerHit = {
  /**
   Marker number normalized to ASCII digits.
   */
  readonly identifier: string;

  /**
   Offset of opening bracket within scanned slice.
   */
  readonly localOffset: number;
};

/**
 Scans one source slice for `〔N〕` markers in a single linear pass.
 Accepts ASCII and full-width digits;
 brackets without digits between them are ordinary text, not markers.

 @param slice - exact source text of one mdast text node

 @returns Hits in source order with slice-local offsets

 @example
 ```ts
 scanFullwidthMarkers({ slice: '文学上的折扣〔1〕', },);
 ```
 */
export function scanFullwidthMarkers(
  { slice, }: { readonly slice: string; },
): readonly TextMarkerHit[] {
  /**
   Accumulated hits in source order.
   */
  const hits: TextMarkerHit[] = [];

  /**
   Scan cursor advanced past each examined opening bracket.
   */
  let cursor = slice.indexOf(FULLWIDTH_OPEN,);

  while (cursor !== (-1)) {
    /**
     Digits collected between brackets, normalized to ASCII, joined once.
     */
    const digitParts: string[] = [];

    /**
     Cursor walking characters after opening bracket.
     */
    let probe = cursor + 1;

    while (probe < slice.length) {
      /**
       Digit-table index of probed character; -1 ends digit collection.
       */
      const digitIndex = DIGIT_CHARS.indexOf(nonNullishOrThrow(slice[probe],),);
      if (digitIndex === (-1))
        break;

      digitParts.push(String(digitIndex % DECIMAL_BASE,),);
      probe += 1;
    }

    /**
     Identifier the digits spell.
     */
    const digits = digitParts.join('',);
    if ((digits !== '') && (slice[probe] === FULLWIDTH_CLOSE))
      hits.push({
        identifier: digits,
        localOffset: cursor,
      },);

    cursor = slice.indexOf(
      FULLWIDTH_OPEN,
      cursor + 1,
    );
  }

  return hits;
}

/**
 Scans one source slice for literal `[^identifier]` sequences.

 micromark consumes every `[^identifier]` whose definition exists into a
 footnoteReference node, so a literal surviving inside a text node is an
 unresolved reference by construction:
 scanning literals is exactly how dropped or mistranslated definitions surface.

 @param slice - exact source text of one mdast text node

 @returns Hits in source order with slice-local offsets

 @example
 ```ts
 scanGfmReferenceLiterals({ slice: '引用[^7]没有定义。', },);
 ```
 */
export function scanGfmReferenceLiterals(
  { slice, }: { readonly slice: string; },
): readonly TextMarkerHit[] {
  return gfmMarkerSpans({ text: slice, },)
    .map(function hit(marker,): TextMarkerHit {
      return {
        identifier: marker.rawLabel,
        localOffset: marker.startOffset,
      };
    },);
}

//endregion Text marker scanning

//region Graph construction

/**
 Mutable accumulator threaded through one document walk.

 @example
 ```ts
 const acc: GraphAccumulator = { references: [], definitions: [], };
 ```
 */
type GraphAccumulator = {
  /**
   References collected so far in source order.
   */
  readonly references: FootnoteReferenceHit[];

  /**
   Definitions collected so far in source order.
   */
  readonly definitions: FootnoteDefinitionHit[];
};

/**
 Records the markers one raw region of text holds: each full-width marker,
 a definition where only whitespace precedes it in its block and a reference
 elsewhere, then each GFM literal the caller found there.

 @param regionStart - body-relative start of the region

 @param region - raw text of the region, from the masked body

 @param literals - GFM literal lexemes of the region that are references,
 their offsets relative to it

 @param blockStart - body-relative start of the block holding the region

 @param bodyText - body source for faithful slice scanning

 @param bodyOffset - absolute offset of body start in full document source

 @param nodeId - structural identifier of the block

 @param acc - accumulator receiving hits

 @example
 ```ts
 collectRegionHits({ regionStart: 0, region: '猫〔1〕', literals: [], blockStart: 0, bodyText, bodyOffset: 0, nodeId: 'block/0', acc, },);
 ```
 */
function collectRegionHits(
  {
    regionStart,
    region,
    literals,
    blockStart,
    bodyText,
    bodyOffset,
    nodeId,
    acc,
  }: {
    readonly regionStart: number;
    readonly region: string;
    readonly literals: readonly GfmMarkerSpan[];
    readonly blockStart: number;
    readonly bodyText: string;
    readonly bodyOffset: number;
    readonly nodeId: string;
    readonly acc: GraphAccumulator;
  },
): void {
  for (const hit of scanFullwidthMarkers({ slice: region, },)) {
    /**
     Text between block start and marker;
     all-whitespace prefix means marker opens its block,
     which is how archive-convention definitions are written.
     */
    const prefix = bodyText.slice(
      blockStart,
      regionStart + hit.localOffset,
    );

    if (prefix.trim() === '') {
      acc.definitions
        .push({
          convention: 'fullwidth-bracket',
          identifier: hit.identifier,
          nodeId,
        },);
    }
    else {
      acc.references
        .push({
          convention: 'fullwidth-bracket',
          identifier: hit.identifier,
          nodeId,
          offset: bodyOffset + regionStart
            + hit.localOffset,
        },);
    }
  }

  // Literal [^id] sequences survive parsing only when their definition is
  // missing, so every hit here is an unresolved GFM reference.
  //
  // Folded on the way in, because every other identifier in this graph
  // arrives from an mdast node already folded. An unresolved reference
  // spelled `[^Note]` and an orphan definition spelled `[^note]:` are one
  // footnote, and reporting them under two names hides that they are.
  for (const literal of literals) {
    acc.references
      .push({
        convention: 'gfm',
        identifier: normalizeFootnoteIdentifier({ identifier: literal.rawLabel, },),
        nodeId,
        offset: bodyOffset + regionStart
          + literal.startOffset,
      },);
  }
}

/**
 Walks one top-level block with an explicit work-stack,
 collecting GFM footnote references and full-width markers from text.
 Code and inline-code nodes never enter text scanning because only text is
 scanned, which is what makes marker look-alikes inside code harmless.

 TEXT IS READ RAW, in three shapes. A positioned `text` node gives its own
 span. A run of nodes the autolink-literal transform rebuilt without
 positions gives the raw between its positioned neighbours, read through
 `footnote-unpositioned-runs.ts` as the footnote relabel reads it (ledger
 B123), at the run's first node. The text of an autolink literal micromark
 tokenized is its URL: GFM shapes there are no references, as they are none
 in a rebuilt literal either. A full-width marker is read wherever text
 shows it, a literal's link text included, since no grammar reads that
 convention and both micromark and the transform take a glued `〔N〕` into
 the link.

 @param block - top-level mdast block to walk

 @param blockIndex - index of block among top-level children

 @param blockStart - body-relative start offset of block

 @param bodyText - body source for faithful slice scanning

 @param bodyOffset - absolute offset of body start in full document source

 @param acc - accumulator receiving hits

 @throws FootnoteRewriteError of the `position` kind when an unpositioned run
 cannot be placed in the raw text, a tree shape no input is known to build

 @example
 ```ts
 collectBlockHits({ block, blockIndex: 0, blockStart: 0, bodyText, bodyOffset: 0, acc, },);
 ```
 */
function collectBlockHits(
  {
    block,
    blockIndex,
    blockStart,
    bodyText,
    bodyOffset,
    acc,
  }: {
    readonly block: RootContent;
    readonly blockIndex: number;
    readonly blockStart: number;
    readonly bodyText: string;
    readonly bodyOffset: number;
    readonly acc: GraphAccumulator;
  },
): void {
  /**
   Structural identifier shared by every hit inside this block.
   */
  const nodeId = `block/${String(blockIndex,)}`;

  /**
   Unpositioned runs of the parents walked so far, keyed by each run's first
   node, where the walk records the run's markers in source order.
   */
  const runs = new Map<TreeNode, UnpositionedRun>();

  /**
   Explicit work-stack replacing recursion for this bounded structural walk.
   */
  const stack: TreeNode[] = [block,];

  while (stack.length > 0) {
    /**
     Node under examination, proven present by loop condition.
     */
    const node = nonNullishOrThrow(stack.pop(),);

    /**
     Raw span of this node, absent on a node the transform rebuilt.
     */
    const bounds = nodeBounds(node,);

    if (node.type === 'footnoteReference') {
      acc.references
        .push({
          convention: 'gfm',
          identifier: node.identifier,
          nodeId,
          offset: bodyOffset + nonNullishOrThrow(node.position
            ?.start
            .offset,),
        },);
    }
    else if ((node.type === 'text') && (bounds !== NO_NODE_BOUNDS)) {
      /**
       Raw text of this node.
       */
      const region = bodyText.slice(
        bounds.start,
        bounds.end,
      );
      collectRegionHits({
        regionStart: bounds.start,
        region,
        literals: gfmMarkerSpans({ text: region, },),
        blockStart,
        bodyText,
        bodyOffset,
        nodeId,
        acc,
      },);
    }

    /**
     Unpositioned run this node opens, absent for every other node.
     */
    const run = runs.get(node,);
    if (run !== undefined) {
      collectRegionHits({
        regionStart: run.regionStart,
        region: bodyText.slice(
          run.regionStart,
          run.regionEnd,
        ),
        literals: run.spans,
        blockStart,
        bodyText,
        bodyOffset,
        nodeId,
        acc,
      },);
    }

    if (isAutolinkLiteral(node,) || isAngleAutolink({
      node,
      text: bodyText,
    },)) {
      // A tokenized literal's or an angle autolink's text is its URL:
      // full-width markers only. A rebuilt literal carries no span, its text
      // read with its run.
      if (bounds !== NO_NODE_BOUNDS) {
        collectRegionHits({
          regionStart: bounds.start,
          region: bodyText.slice(
            bounds.start,
            bounds.end,
          ),
          literals: [],
          blockStart,
          bodyText,
          bodyOffset,
          nodeId,
          acc,
        },);
      }
    }
    else if ('children' in node) {
      for (const opened of unpositionedRuns({
        parent: node,
        text: bodyText,
      },))
        runs.set(
          opened.opener,
          opened,
        );
      // Reverse push keeps source order once the LIFO stack pops.
      for (const child of [...node.children,].toReversed())
        stack.push(child,);
    }
  }
}

/**
 Builds complete footnote graph of one parsed document:
 GFM reference and definition nodes plus archive-convention `〔N〕` text markers,
 validated as a reference-to-definition graph rather than by marker counting.

 @param children - top-level mdast blocks in source order

 @param bodyText - body source the blocks were parsed from

 @param bodyOffset - absolute offset of body start in full document source

 @returns Graph with references, definitions, and integrity findings

 @throws FootnoteRewriteError of the `position` kind when an unpositioned run
 cannot be placed in the raw text, a tree shape no input is known to build

 @example
 ```ts
 const graph = buildFootnoteGraph({ children: root.children, bodyText: body, bodyOffset, },);
 ```
 */
export function buildFootnoteGraph(
  {
    children,
    bodyText,
    bodyOffset,
  }: {
    readonly children: ForeignBorrowed<readonly RootContent[]>;
    readonly bodyText: string;
    readonly bodyOffset: number;
  },
): FootnoteGraph {
  /**
   Accumulator receiving hits from every block walk.
   */
  const acc: GraphAccumulator = {
    references: [],
    definitions: [],
  };

  children.forEach(function walkBlock(
    block,
    blockIndex,
  ): void {
    if (block.type === 'footnoteDefinition') {
      acc.definitions
        .push({
          convention: 'gfm',
          identifier: block.identifier,
          nodeId: `block/${String(blockIndex,)}`,
        },);
    }

    collectBlockHits({
      block,
      blockIndex,
      blockStart: nonNullishOrThrow(block.position
        ?.start
        .offset,),
      bodyText,
      bodyOffset,
      acc,
    },);
  },);

  return {
    references: acc.references,
    definitions: acc.definitions,
    findings: footnoteGraphFindings(acc,),
  };
}

//endregion Graph construction
