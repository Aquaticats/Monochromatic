import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  alignHeadingsForced,
  type DocumentNode,
  leadingMarkWidth,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type ContentChunk,
  describePlacement,
  type DocumentChunk,
  type InsertionPlacement,
  makeInsertionChunk,
  placeInsertions,
  type RepairDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type SectionPair,
  sectionPairingToSteps,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Section chunking
// Some documents send thinking models into token-ceiling spirals (run 4,
// live), while section-scale units complete in ~30 s with quality output, so
// sections bound each call's blast radius. Memorial pages open blocks with
// `##` headings on both sides; a chunk is one heading plus every node until
// the next heading, and nodes before the first heading form a preamble
// chunk. Alignment is automatic and PARTIAL: sides of equal shape pair by
// index, and anything else goes to the forced heading aligner, which pairs
// what the headings support and REFUSES the rest. A refused section has no
// pair, and is named in a finding.
//
// IT USED TO BE TOTAL, by degrading to proportional monotone merging over
// cumulative character fractions, and that is exactly why it is not any
// more: merging produced a confident WRONG pairing that slid a whole
// document by two sections, so every critic call read the wrong original and
// every issue filed on that entry was noise. Leaving a section unpaired
// cannot damage it; guessing its pair damages text that was correct.

// Re-exported rather than defined here, so every consumer that already reads a
// chunk from this file keeps its import while the shapes themselves live in
// `chunk-placement.ts`. A contiguous run of nodes is what this file BUILDS; a
// place where nodes are missing is not, and that distinction is the whole
// reason the two kinds are declared apart from the carving that produces one
// of them.

/**
 Syntax roles requiring rules beyond ordinary Markdown prose.

 @example
 ```ts
 const syntax: SliceSyntax = 'front-matter';
 ```
 */
export type SliceSyntax = 'front-matter';

/**
 One source chunk paired with its translation chunk.

 Each side is exactly one chunk. Merging several sections into one side was
 the proportional fallback's doing, and that fallback is gone.

 @example
 ```ts
 const [pair,] = alignDocumentSections({ source, target, },).pairs;
 ```
 */
export type ChunkPair = {
  /**
   Syntax role requiring rules beyond ordinary Markdown prose.

   Absent for body slices so existing structural consumers stay narrow.
   */
  readonly syntax?: SliceSyntax;

  /**
   Original-side chunk, which is always existing text: a pair exists
   because a source section exists.
   */
  readonly source: ContentChunk;

  /**
   Translation-side chunk, which is content when this section is
   translated and an insertion anchor when it is not.
   */
  readonly target: DocumentChunk;
};

/**
 Where an alignment observation attaches, naming the numbering it counts in.

 A UNION RATHER THAN A NUMBER, because the three cases count in three
 different spaces. A refused chunk has only the index it holds on its OWN
 side, and that side's numbering need not line up with the pairs a run
 produced. A whole-document observation has no index at all and used to
 borrow zero. Rendered together as `pair N`, all three read as one space,
 and a reader meeting `pair 0` could not tell which one it lived in.

 @example
 ```ts
 const attachedTo: AlignmentAttachment = { kind: 'source-section', index: 2, };
 ```
 */
export type AlignmentAttachment =
  | {
    /**
     Observation about the two documents rather than about any one section.
     */
    readonly kind: 'whole-document';
  }
    | {
    /**
     Section of the ORIGINAL side, counted in that side's own numbering.
     */
      readonly kind: 'source-section';

    /**
     Position on that side, which no pair index need match.
     */
      readonly index: number;
    }
    | {
    /**
     Section of the TRANSLATED side, counted in that side's own numbering.
     */
      readonly kind: 'target-section';

    /**
     Position on that side, which no pair index need match.
     */
      readonly index: number;
    };

/**
 Renders where an observation attaches, in wording naming its numbering.

 SPELLS THE SPACE OUT rather than leaving a bare number, so a refusal
 reporting a side index can no longer be read as a pair index. That misread
 is the one that sends a reader to the wrong section of the document.

 @param attachedTo - place this observation hangs from

 @returns Phrase naming the numbering, and the index where there is one

 @example
 ```ts
 const where = describeAlignmentAttachment({ attachedTo: { kind: 'whole-document', }, },);
 ```
 */
export function describeAlignmentAttachment(
  { attachedTo, }: { readonly attachedTo: AlignmentAttachment; },
): string {
  if (attachedTo.kind === 'whole-document')
    return 'whole document';

  return `${(attachedTo.kind === 'source-section') ? 'source' : 'target'} section ${
    String(attachedTo.index,)
  }`;
}

/**
 One structural observation from automatic alignment.

 @example
 ```ts
 const finding: AlignmentFinding = {
   kind: 'structure-mismatch',
   attachedTo: { kind: 'source-section', index: 2, },
   detail: 'source-only (no target heading scored above the floor); has no translation to repair',
 };
 ```
 */
export type AlignmentFinding = {
  /**
   Observation class, and there is one: the sides do not correspond.

   A `sections-merged` kind existed while the proportional fallback did, and
   went with it. Artifacts settled before 2026-08-15 carry the string in their
   findings, which is prose to every reader here; nothing in this package
   matches on it.
   */
  readonly kind: 'structure-mismatch';

  /**
   Where this observation attaches, carrying the numbering it counts in.
   */
  readonly attachedTo: AlignmentAttachment;

  /**
   Concrete structural description.
   */
  readonly detail: string;
};

/**
 PARTIAL alignment outcome: the pairs the aligner committed to, and what it
 refused.

 NOT A COVER. A refused chunk appears in no pair, on either side, which is the
 whole point of refusing it. A consumer counting coverage has to compare the
 pairs against the chunk counts rather than assume them equal.

 @example
 ```ts
 const { pairs, findings, } = alignDocumentSections({ source, target, },);
 ```
 */
export type SectionAlignment = {
  /**
   Monotone chunk pairs in document order, one chunk per side.
   */
  readonly pairs: readonly ChunkPair[];

  /**
   Sections the aligner refused, and whole-document observations.

   EMPTY DOES NOT MEAN VERIFIED. Two sides of equal shape pair by index
   without the aligner being consulted at all, so a document that dropped one
   section and gained an unrelated one later has equal counts, pairs straight
   through, and reports nothing. A roster reading both documents agreed with
   index order on all 56 equal-count entries of the pinned corpus, so the fast
   path stays: refusing those pairings would discard real repair coverage to
   catch a case measured absent there.
   */
  readonly findings: readonly AlignmentFinding[];
};

/**
 Splits a parsed document into heading-bounded chunks.
 Each `heading` node starts a fresh chunk;
 nodes before the first heading form a preamble chunk.

 @param document - parsed document whose nodes carry absolute offsets

 @returns Chunks partitioning the document's nodes in order

 @example
 ```ts
 const chunks = chunkByHeadings({ document: parseDocument({ text, },), },);
 ```
 */
export function chunkByHeadings(
  { document, }: { readonly document: RepairDocument; },
): readonly ContentChunk[] {
  /**
   Node groups partitioned at heading boundaries;
   built by appending in one pass so grouping stays linear.
   */
  const groups: DocumentNode[][] = [];
  for (const node of document.nodes) {
    /**
     Currently open group, when any nodes were grouped already.
     */
    const open = groups.at(-1,);
    if ((node.kind === 'heading') || (open === undefined)) {
      groups.push([node,],);
      continue;
    }
    open.push(node,);
  }

  return groups.map(function toChunk(
    nodes: readonly DocumentNode[],
    sliceIndex,
  ): ContentChunk {
    /**
     First node of the group. ALWAYS PRESENT: every pushed group starts with
     `[node,]`, a single-element array, so `groups` never holds an empty one.
     */
    const first = nonNullishOrThrow(nodes[0],);
    /**
     Last node of the group, present by the same construction as `first`.
     */
    const last = nonNullishOrThrow(nodes.at(-1,),);
    return {
      sliceIndex,
      nodes,
      startOffset: first.startOffset,
      endOffset: last.endOffset,
      text: document.text
        .slice(
        first.startOffset,
        last.endOffset,
        ),
    };
  },);
}

/**
 Reads the label the aligner reasons over for one chunk: its heading, or
 empty for a preamble. The coverage candidates and the section round kept
 their own copies (audit area six, 2026-09-28); this one serves all three.

 A heading chunk carries its heading text; a preamble chunk carries an EMPTY
 label. Building units this way makes UNIT INDEX EQUAL CHUNK INDEX by
 construction, since `chunkByHeadings` emits the preamble as chunk 0 and then
 one chunk per heading. No offset arithmetic remains to get wrong, and offset
 arithmetic is what an earlier adapter got wrong.

 @param chunk - chunk to label

 @returns Heading text, or empty for a preamble

 @example
 ```ts
 const label = chunkLabel(chunk,);
 ```
 */
export function chunkLabel(chunk: ContentChunk,): string {
  /**
   Leading node, which is the heading when the chunk has one.
   */
  const [first,] = chunk.nodes;
  return ((first !== undefined) && (first.kind === 'heading')) ? first.text : '';
}

/**
 Says what became of a section the translation does not carry.

 @param placements - every decision this document produced

 @param sourceIndex - section to report on

 @returns Sentence for a finding's detail

 @throws Error when no decision names that section, since both lists are built
 from the same steps and a missing one means the two passes disagree about
 which sections are unpaired. A detail invented to cover that would hide it.

 @example
 ```ts
 const detail = describeSourceOnly({ placements, sourceIndex: 3, },);
 ```
 */
function describeSourceOnly(
  {
    placements,
    sourceIndex,
  }: {
    readonly placements: readonly InsertionPlacement[];
    readonly sourceIndex: number;
  },
): string {
  /**
   Decision for that section. ALWAYS FOUND: `placeInsertions` returns one
   placement per `source-only` step, keyed by that step's own `sourceIndex`,
   and `alignDocumentSections` builds the placements and the findings from
   the same steps.
   */
  const found = nonNullishOrThrow(placements.find(function atIndex(placement,): boolean {
    return placement.sourceIndex === sourceIndex;
  },),);

  return describePlacement(found,);
}

/**
 Aligns two parsed documents into critic-sized section pairs, automatically
 and PARTIALLY.

 TWO PATHS. Sides of equal shape, meaning equal chunk counts with matching
 leading node kinds, pair by index and report nothing. Everything else goes to
 the forced heading aligner, and only what it pairs becomes a pair: a section
 it refuses is named in a finding and has no pair at all. When the original has
 content and the translation has none (empty, whitespace, a byte order mark or
 front matter alone), every original section is paired with an insertion anchor
 at the one offset after the mark and the front matter, and the finding says
 so. When the original has none, there are no pairs and the finding says so;
 the pipeline decides what a content-free original means.

 WHAT EQUAL SHAPE DOES NOT PROVE. For a document of ordinary heading sections
 every leading kind is `heading`, so that half of the test holds by
 construction and the path reduces to "the counts match". A document that
 dropped one section and gained an unrelated one later has matching counts and
 pairs straight through with no finding. A roster reading both documents agreed
 with index order on all 56 equal-count entries of the pinned corpus, and
 refusing every equal-count pairing would discard real repair coverage to catch a case nothing here can detect.

 @param source - parsed original document

 @param target - parsed translation document

 @param sectionPairing - correspondences a roster agreed on, when one was
 bought. Supplied, it REPLACES the deterministic decision entirely rather than
 supplementing it: a reading of both documents outranks a token-overlap score
 that reads 0.00 across this language boundary, and mixing the two would let
 an index match the roster rejected survive as a pair.

 @returns Pairs the aligner committed to, plus what it refused

 @example
 ```ts
 const { pairs, findings, } = alignDocumentSections({
   source: parseDocument({ text: zh, },),
   target: parseDocument({ text: en, },),
 },);
 ```
 */
export function alignDocumentSections(
  {
    source,
    target,
    sectionPairing,
  }: {
    readonly source: RepairDocument;
    readonly target: RepairDocument;
    readonly sectionPairing?: readonly SectionPair[];
  },
): SectionAlignment {
  /**
   Original-side chunks.
   */
  const sourceChunks = chunkByHeadings({ document: source, },);

  /**
   Translation-side chunks.
   */
  const targetChunks = chunkByHeadings({ document: target, },);

  if ((sourceChunks.length === 0) || (targetChunks.length === 0)) {
    if ((sourceChunks.length === 0) && (targetChunks.length === 0)) {
      return {
        pairs: [],
        findings: [],
      };
    }

    // A TRANSLATION WITH NO BODY IS AN ANCHOR FOR EVERY ORIGINAL SECTION, not a document with nothing to
    // place. An empty archive, one holding whitespace, a byte order mark or front matter alone, passes intake
    // like any other, and pairing nothing here left `prepareDocumentPair` with no slice and the publisher with
    // a 0-byte page, where `doc/design-commitments-shipping-and-the-archive.md` ships the original under a
    // note. Every anchor takes the one offset the translation has to give, and several at one offset is
    // a layout `assertPlacementLayout` allows. `placeInsertions` is not asked: it reads the translation's
    // sections, and this one has none.
    /**
     Where the translation's front matter stands and what it spans, an empty span at the byte order mark (or at
     zero) when it carries none.
     */
    const {
      startOffset: openingStart,
      raw: openingRaw,
    } = target.frontMatter ?? {
      startOffset: leadingMarkWidth({ text: target.text, },),
      raw: '',
    };
    /**
     Where the original's sections are written when the translation has no body: after the byte order mark
     and the front matter it opens with, whichever it carries.
     */
    const insertionOffset = openingStart + openingRaw.length;
    /**
     Whether the translation is the side with no content, which is what earns its sections' anchors.
     */
    const anchorsEverySection = (targetChunks.length === 0);
    return {
      pairs: anchorsEverySection
        ? sourceChunks.map(function toAnchor(sourceChunk,): ChunkPair {
          return {
            source: sourceChunk,
            target: makeInsertionChunk({
              sliceIndex: sourceChunk.sliceIndex,
              offset: insertionOffset,
            },),
          };
        },)
        : [],
      findings: [{
        kind: 'structure-mismatch',
        attachedTo: { kind: 'whole-document', },
        detail: `one side has no content: source ${String(sourceChunks.length,)} ${
          wordForCount({
            count: sourceChunks.length,
            one: 'chunk',
            many: 'chunks',
          },)
        }, target ${
          String(targetChunks.length,)
        } ${
          wordForCount({
            count: targetChunks.length,
            one: 'chunk',
            many: 'chunks',
          },)
        }${
          anchorsEverySection
            ? `; every original section is anchored for insertion at offset ${String(insertionOffset,)}`
            : ''
        }`,
      },],
    };
  }

  /**
   Whether both sides have equal SHAPE: equal counts, and matching leading
   node kinds per index.

   Not "mirrored", which is what this was called and what it cannot check. It
   says nothing about whether section 3 on one side is section 3 on the other.
   */
  const equalShape = (sourceChunks.length === targetChunks.length)
    && sourceChunks.every(function leadingKindMatches(
      chunk,
      index,
    ) {
      return chunk.nodes[0]
        ?.kind
        === targetChunks[index]
          ?.nodes[0]
          ?.kind;
    },);

  // A ROSTER PAIRING SUPPRESSES THE FAST PATH. Equal shape pairs by index
  // without anything being consulted, which is a known blind
  // spot; a roster was shown both documents and its answer is the better one
  // wherever the two disagree.
  if (equalShape && (sectionPairing === undefined)) {
    return {
      pairs: sourceChunks.map(function toPair(
        sourceChunk,
        index,
      ): ChunkPair {
        /**
         Target chunk at the same index, always present: `equalShape` has
         already checked the two counts equal.
         */
        const targetChunk = nonNullishOrThrow(targetChunks[index],);
        return {
          source: sourceChunk,
          target: targetChunk,
        };
      },),
      findings: [],
    };
  }

  /**
   Original section labels, which are all either decider has to go on.
   */
  const sourceHeadings = sourceChunks.map(chunkLabel,);

  /**
   Translation section labels.
   */
  const targetHeadings = targetChunks.map(chunkLabel,);

  /**
   Pairings committed to, plus the refusals, from whichever decider ran.
   */
  const steps = (sectionPairing === undefined)
    ? alignHeadingsForced({
      sourceHeadings,
      targetHeadings,
    },)
    : sectionPairingToSteps({
      pairs: sectionPairing,
      sourceHeadings,
      targetHeadings,
    },);

  // ONLY forced pairings become pairs, and a refusal is never a block. The
  // document still settles with its own text, per
  // `doc/decision/translation-repair-always-yields-output.md`. Leaving text
  // alone cannot damage it; a guessed pairing feeds critics the wrong original
  // and the repairs that follow damage text that was correct.
  //
  // THE TWO REFUSALS ARE NOT THE SAME EVENT, and the detail says which. A
  // target-only section is English nobody will look at, so it passes through
  // unrepaired. A source-only section is Chinese with no English beside it,
  // and section-scale insertion asks whether it can be given a place to be WRITTEN
  // at rather than only reported. It can when the aligner proves every optimal
  // alignment skips it at the same boundary AND the page is measurably too
  // short to hold it, per
  // `doc/decision/translation-repair-absence-verdict.md`. Anything less stays
  // reported and unwritten, and the finding names which signature refused.

  /**
   What was decided about each section the translation does not carry.
   */
  const placements = placeInsertions({
    steps,
    sourceChunks,
    targetChunks,
    sourceText: source.text,
    targetText: target.text,
  },);

  /**
   Those decisions by source index, so the pair and finding builders read one
   answer rather than each deriving its own.
   */
  const placementBySource = new Map(placements.map(
    function keyed(placement,): readonly [
      number,
      InsertionPlacement,
    ] {
      return [
        placement.sourceIndex,
        placement,
      ];
    },
  ),);

  return {
    pairs: steps.flatMap(function toPair(step,): readonly ChunkPair[] {
      if (step.kind === 'source-only') {
        /**
         Whether this section earned a place to be written at. ALWAYS FOUND:
         `placeInsertions` returns one placement per `source-only` step, keyed
         by its `sourceIndex`.
         */
        const placement = nonNullishOrThrow(placementBySource.get(step.sourceIndex,),);
        if (placement.kind !== 'placed')
          return [];

        return [{
          // Present: a step's `sourceIndex` is a section both deciders scan,
          // one per original section (`alignHeadingsForced`'s rows,
          // `sectionPairingToSteps`'s `targets`).
          source: nonNullishOrThrow(sourceChunks[step.sourceIndex],),
          target: makeInsertionChunk({
            sliceIndex: step.sourceIndex,
            offset: placement.offset,
          },),
        },];
      }

      if (step.kind !== 'paired')
        return [];

      // BOTH CHUNKS PRESENT: `alignHeadingsForced` pairs only a row and a
      // column it scanned, and `sectionPairingToSteps` only a pair the stage
      // read against both sides' counts (`readSectionPairing`,
      // `assertIndicesExist`); the headings both read are one per chunk.
      return [{
        source: nonNullishOrThrow(sourceChunks[step.sourceIndex],),
        target: nonNullishOrThrow(targetChunks[step.targetIndex],),
      },];
    },),
    findings: steps.flatMap(function toFinding(step,): readonly AlignmentFinding[] {
      if (step.kind === 'paired')
        return [];

      return [{
        kind: 'structure-mismatch',
        attachedTo: (step.kind === 'source-only')
          ? {
            kind: 'source-section',
            index: step.sourceIndex,
          }
          : {
            kind: 'target-section',
            index: step.targetIndex,
          },
        detail: `${step.kind} (${step.reason}); ${
          (step.kind === 'source-only')
            ? describeSourceOnly({
              placements,
              sourceIndex: step.sourceIndex,
            },)
            : 'passes through unrepaired'
        }`,
      },];
    },),
  };
}

//endregion Section chunking
