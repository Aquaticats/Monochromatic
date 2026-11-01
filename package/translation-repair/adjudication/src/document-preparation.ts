import {
  alignDocumentSections,
  assertContainerIntegrity,
  assertSliceCoverage,
  assertSliceIndexing,
  assertSpanContiguity,
  type ChunkGovernance,
  type ChunkPair,
  type ChunkSlice,
  describeAlignmentAttachment,
  frontMatterSlice,
  governedSliceIndices,
  reindexSlicePair,
  sectionPairingsOf,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  archiveContributorNameForms,
  declaredNameForms,
  declaredNamePairs,
  extractDeclaredIdentity,
  lifeEndedOf,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { pageIdentityLines, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import { declaredSubjectNames, } from './subject-names.ts';
import type {
  BlockPair,
  SectionPair,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { AttestedDetail, } from '@monochromatic-dev/translation-repair-reference/ts';
import { leadingMarkWidth, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  archiveOriginalSpansOf,
  assertPlacementLayout,
  chunkSealedTargets,
  declinedLessSealed,
  type FrontMatterAuthority,
  type PreparedDocumentPair,
  sealedFinding,
  unclaimedFinding,
  unclaimedInAlignedPair,
  unclaimedOutsideAlignment,
  type UnclaimedTargetBlock,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  SLICE_CHAR_BUDGET,
  subdivideSealedChunkPair,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { sealedNodeIds, } from './sealed-node-ids.ts';

//region Document preparation
// Everything a lane needs to know about a document PAIR before any model is
// asked anything: how it parses, how its sections pair up, where the slice
// boundaries fall, which slices inherit the line-structure rule, and what names
// both sides declare.
//
// Shared because it is lane-neutral and must be IDENTICAL across lanes. Two
// lanes preparing separately would drift the moment either changed a budget or
// a governance rule, and the drift would be invisible: each lane would report
// slices that look right on their own, while the same document produced two
// different slicings and no artifact recorded which one a result came from.
//
// Spends no quota and takes no client, roster, config, signal or cache. That
// exclusion is the boundary: anything needing one of those is lane work.

/**
 Parses, aligns and subdivides a document pair.

 @param sourceText - whole original document

 @param targetText - whole translation as it stands

 @param sliceCharBudget - target characters a slice may carry; defaults to
 {@link SLICE_CHAR_BUDGET}

 @param includeFrontMatter - whether visible metadata becomes explicit slice;
 false only when rebuilding pre-generation-5 artifacts

 @param frontMatterAuthority - whose front matter the page carries: under
 `archive` no metadata slice is made and the archive's bytes ship as they
 stand (the owner's rule of 2026-09-08); under `rendered` slice zero is the
 lanes' to write

 @param blockPairings - correspondences a roster agreed on WITHIN each aligned
 section, keyed by section index

 @param sectionPairing - correspondences a roster agreed on BETWEEN the two
 sides' sections, which decides what the aligned sections are in the first
 place. Kept apart from `blockPairings` because the two answer different
 questions and are bought in that order: which sections correspond, and then
 which blocks within one do.

 @param contextLines - evidence lines a caller bought outside preparation
 (web lookups of the works the original names), appended to the identity
 context after the notes both documents carry

 @param referenceContext - what the pages the original links say, passed
 through untouched to the prepared pair for the critic and panel sheets
 (class thirty-five); empty means the original links nowhere

 @param attestedDetails - archive details a cited reference states, passed
 through untouched for the repair lane's claim screen (class thirty-seven);
 none when nothing was attested

 @param sealArchiveOriginal - whether a span the archive's translators' note
 calls the English original is sealed out of every slice so it ships as it
 stands (the owner's rule of 2026-09-08, `archive-original-note.ts`); false
 when rebuilding an artifact written before generation twelve, whose slicing
 sealed nothing

 @param sourceEntryInfo - parsed `info.yml` of the original's corpus entry,
 whose recorded death joins the identity block and sets `lifeEnded`; absent
 when the caller reads no entry, which records no death

 @returns Slices, governance, declared names and alignment findings

 @example
 ```ts
 const { slices, lineStructuredSliceIndices, } = prepareDocumentPair({ sourceText, targetText, },);
 ```
 */
export function prepareDocumentPair(
  {
    sourceText,
    targetText,
    sliceCharBudget = SLICE_CHAR_BUDGET,
    includeFrontMatter = true,
    frontMatterAuthority = 'rendered',
    blockPairings,
    sectionPairing,
    contextLines = [],
    referenceContext = '',
    attestedDetails = [],
    sealArchiveOriginal = false,
    sourceEntryInfo,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly sliceCharBudget?: number;
    readonly includeFrontMatter?: boolean;
    readonly frontMatterAuthority?: FrontMatterAuthority;
    readonly blockPairings?: ReadonlyMap<number, readonly BlockPair[]>;
    readonly sectionPairing?: readonly SectionPair[];
    readonly contextLines?: readonly string[];
    readonly referenceContext?: string;
    readonly attestedDetails?: readonly AttestedDetail[];
    readonly sealArchiveOriginal?: boolean;
    readonly sourceEntryInfo?: unknown;
  },
): PreparedDocumentPair {
  /**
   Whole original document, parsed once and reused for both alignment and the
   identity block chunk text cannot supply.
   */
  const sourceDocument = parseDocument({ text: sourceText, },);

  /**
   Whole translation document, parsed once for the same two uses.
   */
  const targetDocument = parseDocument({ text: targetText, },);

  /**
   Spans of the archive that ship as they stand, EMPTY when no note seals
   one or the caller asked for no seal (`preparation-seal.ts`).
   */
  const archiveOriginalSpans = archiveOriginalSpansOf({
    document: targetDocument,
    seal: sealArchiveOriginal,
  },);

  /**
   Ids of every translation block a seal covers, across the whole archive.
   */
  const sealedTargetIds = sealedNodeIds({
    nodes: targetDocument.nodes,
    spans: archiveOriginalSpans,
  },);

  /**
   Declared names and handles from both sides' front matter. Front matter is
   document-level while stages see slice text, so this is the only path by
   which a declared correspondence reaches them. Empty when neither side
   declares anything.
   */
  const frontMatterData = {
    sourceData: sourceDocument.frontMatter
      ?.data,
    targetData: targetDocument.frontMatter
      ?.data,
  };
  /**
   Declared name pairs, source form beside declared rendering, for the
   glossary note (class eighty-six) and the publication rule's floor (class
   one hundred fourteen).
   */
  const declaredPairs = declaredNamePairs(frontMatterData,);

  /**
   Contributor public handles existing English attribution establishes.
   */
  const contributorNames = archiveContributorNameForms({ text: targetText, });

  /**
   Prompt lines for every sheet: declared names, pronoun, contributors, notes,
   community words, the page's own names and the caller's context lines,
   assembled where the page title lexicon reads them too (ledger B28).
   */
  const identityContextLines = pageIdentityLines({
    sourceDocument,
    targetDocument,
    sourceText,
    targetText,
    contextLines,
    sourceEntryInfo,
  },);

  /**
   Whether the original's entry records a death, which the house tense rule
   reads for the sheets that judge tense (`presentTenseCandidates`).
   */
  const lifeEnded = lifeEndedOf({ entryInfo: sourceEntryInfo, },);

  /**
   Names the translation declares for its subject, its display name and its
   aliases, which the tense candidates read as named subjects beside the
   pronouns. Empty when it declares none.
   */
  const subjectNames = declaredSubjectNames({
    data: targetDocument.frontMatter
      ?.data,
  },);

  /**
   Forms the translation's front matter declares for its subject, the display
   name and the aliases, which are one person: a guard may read one of them
   standing in for another, and may not do so for a contributor handle.
   */
  const declaredSubjectForms = declaredNameForms({
    identity: extractDeclaredIdentity({
      data: targetDocument.frontMatter
        ?.data,
    },),
  },);

  /**
   Target-authoritative identity forms guards preserve wherever archive body
   already carries them.
   */
  const declaredNames = [
    ...declaredSubjectForms,
    ...contributorNames,
  ]
    .filter(function firstOccurrence(
      form,
      at,
      forms: readonly string[],
    ): boolean {
      return forms.indexOf(form,) === at;
    },)
    .toSorted(function longestFirst(
      left,
      right,
    ): number {
      return right.length - left.length;
    },);

  /**
   Aligned chunk pairs covering both documents totally.
   */
  const alignment = alignDocumentSections({
    source: sourceDocument,
    target: targetDocument,
    ...((sectionPairing === undefined) ? {} : { sectionPairing, }),
  },);

  /**
   Alignment findings in scorecard-stable wording.
   */
  const sectionFindings = alignment.findings
    .map(function toText(finding,): string {
      return `alignment ${finding.kind} (${
        describeAlignmentAttachment({ attachedTo: finding.attachedTo, },)
      }: ${finding.detail})`;
    },);

  /**
   One finding per chunk whose pairing accounted for translation blocks
   nowhere, so the decision is legible from the artifact alone.

   `Zha_Ke` settled with this list EMPTY while two of its six English blocks,
   2943 characters of dense text between them, had been declined.
   Reconstructing that took the pairing cache and the parser. Recording it
   here puts it at the one place that already knows both the
   pairing and the blocks.

   The finding itself counts OFFSET SPANS, which run wider than dense text
   because they carry the markdown a block is written in.
   */
  const declinedFindings: string[] = [];

  /**
   Target blocks outside every aligned section, which no slice can review,
   less the sealed ones (`preparation-unclaimed.ts`).
   */
  const unclaimedTargetBlocks: UnclaimedTargetBlock[] = [
    ...unclaimedOutsideAlignment({
      alignment,
      targetDocument,
      sealedTargetIds,
    },),
  ];

  /**
   Slice pairs accumulated across front matter and aligned body sections.
   */
  const slices: ChunkPair[] = [];
  /**
   Visible localized metadata excluded from Markdown nodes.
   */
  const metadataSlice = (includeFrontMatter && (frontMatterAuthority === 'rendered'))
    ? frontMatterSlice({
      ...(sourceDocument.frontMatter === undefined ? {} : { source: sourceDocument.frontMatter, }),
      ...(targetDocument.frontMatter === undefined ? {} : { target: targetDocument.frontMatter, }),
      // A translation opening with a byte order mark and carrying no metadata
      // takes the metadata after the mark, so the mark stays the page's first
      // character.
      targetInsertionOffset: leadingMarkWidth({ text: targetText, },),
    },)
    : { kind: 'none' as const, };
  if (metadataSlice.kind === 'paired')
    slices.push(metadataSlice.slice,);

  /**
   Slices whose enclosing CHUNK's original is line-structured.

   Decided on the chunk and inherited by its slices, because the predicate
   needs at least five blocks and subdivision routinely leaves fewer. Measured
   on `Toka_ls`: the verse chunk trips at 21 blocks, median 22, then
   subdivides into seven slices of which one still trips, while four more sit
   at medians 20, 22, 23 and 29 and fail only for want of a fifth block.
   Deciding per slice therefore dropped the instruction on most of the verse
   it exists for.
   */
  const governance: ChunkGovernance[] = [];
  for (
    const [pairIndex, pair,] of alignment
      .pairs
      .entries()
  ) {
    /**
     Correspondences the roster agreed for this chunk, ABSENT when it agreed
     none.

     ABSENCE HAS TO STAY ABSENCE all the way to subdivision. A section the
     roster could not pair is left OUT of the map, and `prepare-with-pairing`
     says what it means by that: it records `fell back to scoring` and logs
     "keeping the deterministic aligner". Reading the miss as an EMPTY pairing
     says something else entirely, because `blockPairingToSteps` reads zero
     pairs as every block unpartnered on both sides, every run then comes out
     one-sided, `mergeOneSidedRuns` folds them all together, and the section
     becomes ONE slice. Measured on `Zha_Ke`: four slices become one, 262
     characters of original against 4340 of translation, which is the largest
     slice in the document at exactly the section nobody could pair.
     */
    const blockPairing = blockPairings?.get(pairIndex,);

    /**
     Translation blocks of this chunk the archive's note seals.
     */
    const sealedTargets = chunkSealedTargets({
      pair,
      sealedTargetIds,
    },);

    /**
     Slices carved from this chunk, the originals the seal took, and the
     translation blocks no slice could take.
     */
    const {
      slices: carved,
      sealedSourceIds,
      unplacedTargetIds,
    } = subdivideSealedChunkPair({
      pair,
      sourceText,
      targetText,
      baseIndex: slices.length,
      budget: sliceCharBudget,
      ...((blockPairing === undefined) ? {} : { blockPairing, }),
      sealed: sealedTargets,
    },);
    if (sealedTargets.size > 0)
      declinedFindings.push(sealedFinding({
        pairIndex,
        sealedTargets,
        sealedSourceIds,
      },),);

    /**
     Translation blocks this chunk's pairing accounted for nowhere, less the
     sealed ones (`preparation-seal.ts`).
     */
    const declinedByPairing = declinedLessSealed({
      pair,
      ...((blockPairing === undefined) ? {} : { blockPairing, }),
      sealedTargets,
    },);

    // A BLOCK NO RUN COULD TAKE SHIPS AS THE ARCHIVE HAS IT, like a declined
    // one. It is not declined by the pairing: the grouping had no run to put
    // it in (an archive block ahead of the sealing note, with no paired run
    // before it), and refusing the page for that stopped an entry whose bytes
    // could be kept.
    /**
     Every block of this chunk that reaches no slice by decision.
     */
    const declined = new Set([
      ...declinedByPairing,
      ...unplacedTargetIds,
    ],);
    if (declined.size > 0) {
      /**
       Blocks of this chunk that reach no slice, in document order.
       */
      const blocks = pair.target
        .nodes
        .filter(function isDeclined(node,): boolean {
          return declined.has(node.id,);
        },);
      unclaimedTargetBlocks.push(...unclaimedInAlignedPair({
        pairIndex,
        blocks,
      },),);

      /**
       Blocks the pairing accounted for nowhere.
       */
      const declinedBlocks = blocks.filter(function isByPairing(node,): boolean {
        return declinedByPairing.has(node.id,);
      },);
      if (declinedBlocks.length > 0)
        declinedFindings.push(unclaimedFinding({
          label: 'target-unclaimed',
          pairIndex,
          blocks: declinedBlocks,
          reason: 'no original claims',
        },),);

      /**
       Blocks no run could take.
       */
      const unplacedBlocks = blocks.filter(function isUnplaced(node,): boolean {
        return unplacedTargetIds.has(node.id,);
      },);
      if (unplacedBlocks.length > 0)
        declinedFindings.push(unclaimedFinding({
          label: 'target-unplaced',
          pairIndex,
          blocks: unplacedBlocks,
          reason: 'no run could take',
        },),);
    }

    // BEFORE ANYTHING READS THEM. A block that reached no slice leaves the
    // document silently, and every later check works from the slices, so this
    // is the last point where the blocks it was given are still in hand.
    // A DECLINED block is the one exception, and it is passed in rather than
    // inferred so the assertion still fails on every block that went missing
    // without a decision behind it.
    assertSliceCoverage({
      pair,
      carved,
      declined,
      sealed: {
        source: sealedSourceIds,
        target: sealedTargets,
      },
    },);

    /**
     Those slices renamed by where they actually landed.

     Subdivision was handed a base index and added its own offset, which is
     the same answer this produces until an insertion precedes. It stops being the same answer the
     moment a section contributes a slice the base index did not count, which
     is exactly what an insertion slice does, so the preparation stamps the
     final name itself rather than trusting arithmetic it handed out.
     */
    const stamped = carved.map(function toStamped(
      carvedSlice,
      offset,
    ): ChunkPair {
      return reindexSlicePair({
        slice: carvedSlice,
        slicePosition: slices.length + offset,
      },);
    },);
    governance.push({
      sourceText: pair.source
        .text,
      slices: stamped.map(function toSlice(stampedSlice,): ChunkSlice {
        return {
          index: stampedSlice.target
            .sliceIndex,
          sourceText: stampedSlice.source
            .text,
        };
      },),
    },);
    slices.push(...stamped,);
  }

  // BELT OVER BRACES, and worth the line. `reindexSlicePair`'s restamp already makes this
  // true from this path, so it can only fail if that restamp is changed or
  // removed. What it pins is the property everything downstream reads: the
  // lanes, the assembly and the cross-lane comparison are all further from the
  // stamping than this, and none of them could tell a mis-stamped preparation
  // from a strange document.
  assertSliceIndexing({ slices, },);

  // AND WHERE THEY POINT, which indexing says nothing about. A content span comes
  // from a disjoint run of nodes, so for it this holds by construction; an
  // insertion anchor covers no nodes at all, so for it nothing does.
  assertPlacementLayout({
    slices,
    targetText,
  },);

  // AND WHAT THEY COVER, which the layout says nothing about either. A span's
  // text is sliced from its own offsets, so a block lying between two of its
  // nodes but missing from the run is inside the range, agrees with the text
  // byte for byte, and is replaced at assembly by a decision that never saw it.
  // Only the document's whole node sequence can see that, and this is the last
  // place holding it.
  assertSpanContiguity({
    slices,
    targetNodes: targetDocument.nodes,
  },);

  // AND THAT EVERY CONTAINER TAG IS STILL OWNED BY A BLOCK, which none of
  // `assertSliceIndexing`, `assertPlacementLayout` and `assertSpanContiguity`
  // can reach. Dissolving a container leaves its opening and
  // closing tags belonging to no block, and `container-extents.ts` hands each of
  // them to the block beside it so that ranges minted from block offsets cannot
  // cut one. This asks whether that still happened.
  assertContainerIntegrity({
    slices,
    containers: targetDocument.containers,
    blocks: targetDocument.nodes,
  },);

  return {
    ...(!includeFrontMatter ? { legacyIdentity: true as const, } : {}),
    ...((frontMatterAuthority === 'archive') ? { frontMatterAuthority: 'archive' as const, } : {}),
    // OMITTED WHEN NOTHING WAS SEALED, so a preparation that read no note and
    // one that was asked not to seal both read as the absence they are.
    ...((archiveOriginalSpans.length === 0) ? {} : { archiveOriginalSpans, }),
    sourceText,
    targetText,
    // TRUE EXACTLY WHEN THE ORIGINAL'S ENTRY RECORDS A DEATH, carried as a
    // typed flag so no sheet has to read it back out of the identity block.
    lifeEnded,
    ...((lifeEnded && (subjectNames.length > 0)) ? { subjectNames, } : {}),
    slices,
    lineStructuredSliceIndices: governedSliceIndices({ chunks: governance, },),
    // Omitted rather than empty, so spreading this into a prompt cannot emit a
    // heading with nothing under it.
    ...(identityContextLines.length === 0
      ? {}
      : { identityContext: identityContextLines.join('\n',), }),
    // Omitted when the original links nowhere, for the same reason.
    ...((referenceContext === '') ? {} : { referenceContext, }),
    // Omitted when nothing was attested, for the same reason.
    ...((attestedDetails.length === 0) ? {} : { attestedDetails, }),
    declaredNames,
    // OMITTED WHEN THE FRONT MATTER DECLARES NO FORM, so a pair built before the
    // identity was named, or by a caller with no front matter, reads as naming
    // nobody and every guard keeps the plain rule.
    ...((declaredSubjectForms.length === 0) ? {} : { declaredSubjectForms, }),
    ...((declaredPairs.length === 0) ? {} : { declaredNamePairs: declaredPairs, }),
    alignmentFindings: [
      ...sectionFindings,
      ...declinedFindings,
    ],
    unclaimedTargetBlocks,
    alignmentPairCount: alignment.pairs
      .length,

    // OMITTED WHEN NO PAIRING WAS SUPPLIED, because a caller that asked nobody
    // and a roster that agreed nothing are different facts, and both would read
    // as an empty list if absence were spelled that way.
    ...((blockPairings === undefined) ? {} : { blockPairing: sectionPairingsOf({ blockPairings, },), }),

    // ECHOED AS CONSUMED, for the rebuild an artifact reader performs: with
    // this and the block pairing, `prepareDocumentPair` over the same two
    // texts reproduces this exact slicing, and without it a reader can only
    // guess whether the aligner or a roster chose the sections.
    ...((sectionPairing === undefined) ? {} : { sectionPairing, }),
  };
}

//endregion Document preparation
