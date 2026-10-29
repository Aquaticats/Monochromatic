import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  alignBlocks,
  type AlignmentStep,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  alignDocumentSections,
  chunkByHeadings,
  type ChunkPair,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type CorpusPin,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type BlockPair,
  blockPairingToSteps,
  requireBlockPairingRefusal,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  SLICE_CHAR_BUDGET,
  subdivideChunkPair,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  type PairingRecipe,
  pairingSetAsideSentence,
} from '@monochromatic-dev/translation-repair-artifact/ts';

/**
 Logger root for the slice census.
 */
const l = contextRoot({ tag: 'translation-repair', },);

//region Slice census entry
// One corpus entry, measured after slicing.
//
// Split from the reporting driver at the line budget, and the boundary is a
// real one: this file answers what an entry IS, and `slice-census.ts` answers
// what the corpus looks like across all of them.


/**
 One entry's measured shape.

 @example
 ```ts
 const row: EntryCensus = { entryId, sliceCount: 12, ... };
 ```
 */
export type EntryCensus = {
  /**
   Corpus id.
   */
  readonly entryId: string;

  /**
   Source characters of every slice, in document order.
   */
  readonly sliceSourceChars: readonly number[];

  /**
   Target characters of every slice.
   */
  readonly sliceTargetChars: readonly number[];

  /**
   Sections the aligner REFUSED to pair, which therefore reach no slice.

   Counted from the aligner's own output rather than from the pairs it
   produced. Only a forced pairing becomes a pair, so a refused section is
   absent from `alignment.pairs` entirely rather than present with an empty
   side, and a counter that walks the pairs can only ever report zero. That is
   what the earlier `onesidedSections` did, and it read as an answer.
   */
  readonly unpairedSourceSections: number;

  /**
   Characters of source in those sections, which is what translating them
   would carry and what no lane spends today.
   */
  readonly unpairedSourceChars: number;

  /**
   Translation sections no source section partnered, which reach no slice for
   the same reason from the other side.
   */
  readonly unpairedTargetSections: number;

  /**
   Characters of translation in those sections.
   */
  readonly unpairedTargetChars: number;

  /**
   Blocks the translation carries that no source block partnered, read off
   the same steps that sliced the section: the settled artifact's recorded
   pairing where the recipe carries one, the deterministic aligner otherwise.
   A block a recorded pairing renders for an original, even beside another
   block, is a rendering and not counted.
   */
  readonly targetOnlyBlocks: number;

  /**
   Characters in those blocks.
   */
  readonly targetOnlyChars: number;

  /**
   Size of every target-only block, so a transcription can be told from an
   ordinary paragraph split.

   The transcribed-image class is the case where a Chinese page holds a letter
   as a picture and the English page transcribes and translates it. MEASURED
   2026-08-15: that picture is nowhere in the markdown this pipeline reads.
   Only 2 of 92 source pages mention `img` at all, and the entry with the most
   target-only text mentions none, so no image-adjacency test can find the
   class. Size is the signal that remains: a transcription runs long and a
   split paragraph does not.
   */
  readonly targetOnlyBlockChars: readonly number[];

  /**
   Which carve the slice sizes describe: the settled artifact's recipe, whole
   or with a defaulted half, the recipe with its block pairing set aside, or
   the deterministic baseline where no artifact records this entry.
   */
  readonly carve: CensusCarve;

  /**
   Why the recorded block pairing was set aside, in the sentence the rebuild
   of the same artifact gives (`rebuildPreparation`); empty where the pairing
   was used or the recipe records none.
   */
  readonly pairingRefusal: string;
};

/**
 Which slicing a census row measured. `settled-moved` is a recipe whose
 recorded block pairing names a block the carved text lacks, so every
 section's blocks were carved by the deterministic aligner, as
 `rebuildPreparation` answers `moved` for the same artifact.

 @example
 ```ts
 const carve: CensusCarve = 'deterministic';
 ```
 */
export type CensusCarve = 'settled-complete' | 'settled-partial' | 'settled-moved' | 'deterministic';

/**
 Sizes measured over every aligned section of one entry.

 @example
 ```ts
 const measures: SectionMeasures = { sliceSourceChars: [15,], sliceTargetChars: [47,], targetOnlyBlockChars: [], };
 ```
 */
type SectionMeasures = {
  /**
   Source characters of every slice, in document order.
   */
  readonly sliceSourceChars: readonly number[];

  /**
   Target characters of every slice.
   */
  readonly sliceTargetChars: readonly number[];

  /**
   Size of every target-only block.
   */
  readonly targetOnlyBlockChars: readonly number[];
};

/**
 Steps one section is carved by: the recorded pairing's where the recipe
 carries one for it, the deterministic aligner's otherwise, which is the
 choice `subdivideChunkPair` makes for the same section.

 @param pair - aligned section pair

 @param blockPairing - recorded pairing for this section, absent where the
 recipe has none for it

 @returns Steps covering both sides' blocks once each

 @throws {@link BlockPairingError} when the recorded pairing names a block
 the section lacks

 @example
 ```ts
 const steps = stepsOfSection({ pair, },);
 ```
 */
function stepsOfSection(
  {
    pair,
    blockPairing,
  }: {
    readonly pair: ChunkPair;
    readonly blockPairing?: readonly BlockPair[];
  },
): readonly AlignmentStep[] {
  if (blockPairing === undefined)
    return alignBlocks({
      sourceNodes: pair.source
        .nodes,
      targetNodes: pair.target
        .nodes,
    },);
  return blockPairingToSteps({
    pairs: blockPairing,
    sourceCount: pair.source
      .nodes
      .length,
    targetCount: pair.target
      .nodes
      .length,
  },);
}

/**
 Measures every section's slices and its target-only blocks, over one choice
 of block pairings.

 @param pairs - aligned section pairs, in document order

 @param sourceText - whole original

 @param targetText - whole translation

 @param blockPairings - recorded block pairings by section pair index,
 absent for the deterministic aligner

 @returns Slice sizes and the size of every block no original partnered

 @throws {@link BlockPairingError} when a recorded pairing names a block its
 section lacks

 @example
 ```ts
 const measures = measureSections({ pairs: alignment.pairs, sourceText, targetText, },);
 ```
 */
function measureSections(
  {
    pairs,
    sourceText,
    targetText,
    blockPairings,
  }: {
    readonly pairs: readonly ChunkPair[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly blockPairings?: NonNullable<PairingRecipe['blockPairings']>;
  },
): SectionMeasures {
  /**
   Size of every target-only block this entry carries.
   */
  const targetOnlyBlockChars: number[] = [];

  /**
   Source characters of every slice.
   */
  const sliceSourceChars: number[] = [];

  /**
   Target characters of every slice.
   */
  const sliceTargetChars: number[] = [];
  for (
    const [pairIndex, pair,] of pairs
      .entries()
  ) {
    /**
     Block pairing the recipe supplies for this section, absent under the
     deterministic carve and where the roster agreed nothing.
     */
    const blockPairing = blockPairings
      ?.get(pairIndex,);

    /**
     Translation blocks, which may be none.
     */
    const targetNodes = pair.target
      .nodes;
    for (
      const step of stepsOfSection({
        pair,
        ...((blockPairing === undefined) ? {} : { blockPairing, }),
      },)
    ) {
      // A BLOCK THAT RENDERS AN ORIGINAL BESIDE ANOTHER IS NOT ADDED: only a
      // recorded pairing sets the continuation, and it names the blocks that
      // belong to a rendering.
      if ((step.kind !== 'target-only') || (step.continuesPairing === true))
        continue;
      targetOnlyBlockChars.push(nonNullishOrThrow(targetNodes[step.targetIndex],)
        .text
        .length,);
    }
    for (
      const slice of subdivideChunkPair({
        pair,
        sourceText,
        targetText,
        baseIndex: sliceSourceChars.length,
        budget: SLICE_CHAR_BUDGET,
        ...((blockPairing === undefined) ? {} : { blockPairing, }),
      },)
    ) {
      sliceSourceChars.push(slice.source
        .text
        .length,);
      sliceTargetChars.push(slice.target
        .text
        .length,);
    }
  }
  return {
    sliceSourceChars,
    sliceTargetChars,
    targetOnlyBlockChars,
  };
}

/**
 Measures an entry by its recipe's block pairings, or by the deterministic
 aligner for every section where a recorded pairing names a block its
 section lacks.

 THE SAME ANSWER `rebuildPreparation` GIVES FOR THE SAME ARTIFACT: the refusal
 names no section, and a pairing that does not fit one section says the text
 parsed otherwise than the run's, so every section's pairing is set aside
 and the census goes on to the next entry with the row saying why.

 @param pairs - aligned section pairs, in document order

 @param sourceText - whole original

 @param targetText - whole translation

 @param blockPairings - recorded block pairings by section pair index,
 absent where the recipe records none

 @returns Measures, and the sentence saying why the pairings were set aside,
 empty where they were used or none was recorded

 @throws Error when the carve refuses a block pairing for a recipe that
 records none, which no carve does: only a recorded pairing names blocks by
 index

 @example
 ```ts
 const { measures, pairingRefusal, } = measureCarve({ pairs, sourceText, targetText, },);
 ```
 */
function measureCarve(
  {
    pairs,
    sourceText,
    targetText,
    blockPairings,
  }: {
    readonly pairs: readonly ChunkPair[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly blockPairings?: NonNullable<PairingRecipe['blockPairings']>;
  },
): {
  readonly measures: SectionMeasures;
  readonly pairingRefusal: string;
} {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: measureCarve.name,
    l,
  },);
  try {
    return {
      measures: measureSections({
        pairs,
        sourceText,
        targetText,
        ...((blockPairings === undefined) ? {} : { blockPairings, }),
      },),
      pairingRefusal: '',
    };
  }
  catch (error) {
    // Only a recorded pairing is refused; anything else propagates.
    /**
     Why the recorded block pairing does not fit the text carved.
     */
    const { message: refusal, } = requireBlockPairingRefusal({ error, },);
    if (blockPairings === undefined)
      throw new Error(
        'unreachable: the carve refused a block pairing, though this recipe records none and only a recorded '
          + 'pairing names blocks by index',
        { cause: error, },
      );
    rl.warn(`recorded block pairing set aside: ${refusal}`,);
    return {
      measures: measureSections({
        pairs,
        sourceText,
        targetText,
      },),
      pairingRefusal: pairingSetAsideSentence({ refusal, },),
    };
  }
}

/**
 Measures one corpus entry.

 @param entryId - corpus id

 @returns That entry's shape after slicing

 @throws {@link CorpusReadError} when either side is absent

 @param pin - corpus clone and commit to read: the pin its entry reads in a run, a
 throwaway clone in a test, since the real one is unlicensed. REQUIRED: a
 default read the clone for any caller that left it out (ledger M43, X24)

 @param recipe - pairing recipe the entry's settled artifact records, which
 makes the slice sizes those of the slicing the lanes judged; absent, the
 deterministic aligner carves and the row says so

 @example
 ```ts
 const row = await censusEntry({ entryId: 'Toka_ls', pin, },);
 ```
 */
export async function censusEntry(
  {
    entryId,
    pin,
    recipe,
  }: {
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly recipe?: PairingRecipe;
  },
): Promise<EntryCensus> {
  /**
   Original document at the pin.
   */
  const sourceText = await readCorpusFile({
    pin,
    relPath: `people/${entryId}/page.md`,
  },);

  /**
   Translation at the same commit.
   */
  const targetText = await readCorpusFile({
    pin,
    relPath: `people/${entryId}/page.en.md`,
  },);

  /**
   Original, parsed once and kept so its sections can be counted against the
   pairs the aligner committed to.
   */
  const sourceDocument = parseDocument({ text: sourceText, },);

  /**
   Translation, parsed for the same two uses.
   */
  const targetDocument = parseDocument({ text: targetText, },);

  /**
   Section pairing the recipe supplies, absent under the deterministic carve.
   */
  const sectionPairing = recipe?.sectionPairing;

  /**
   Aligned section pairs, as the pass cut them when a recipe is supplied and
   as the deterministic aligner cuts them otherwise.
   */
  const alignment = alignDocumentSections({
    source: sourceDocument,
    target: targetDocument,
    ...((sectionPairing === undefined) ? {} : { sectionPairing, }),
  },);

  /**
   Every section of the original, paired or not.
   */
  const sourceSections = chunkByHeadings({ document: sourceDocument, },);

  /**
   Every section of the translation.
   */
  const targetSections = chunkByHeadings({ document: targetDocument, },);

  /**
   Sizes read section by section, with the recorded block pairing set aside
   where it does not fit the text.
   */
  const {
    measures,
    pairingRefusal,
  } = measureCarve({
    pairs: alignment.pairs,
    sourceText,
    targetText,
    ...((recipe?.blockPairings === undefined) ? {} : { blockPairings: recipe.blockPairings, }),
  },);

  /**
   Source characters the pairs cover, which is every character a slice can
   come from.
   */
  const pairedSourceChars = alignment.pairs
    .reduce(
    function addSource(
      running,
      pair,
    ): number {
      /**
       Characters this pair's original carries.
       */
      const sectionChars = pair.source
        .text
        .length;
      return running + sectionChars;
    },
    0,
    );

  /**
   Translation characters the pairs cover.
   */
  const pairedTargetChars = alignment.pairs
    .reduce(
    function addTarget(
      running,
      pair,
    ): number {
      /**
       Characters this pair's translation carries.
       */
      const sectionChars = pair.target
        .text
        .length;
      return running + sectionChars;
    },
    0,
    );

  /**
   Every source character the document holds inside a section.
   */
  const allSourceChars = sourceSections.reduce(
    function addSection(
      running,
      section,
    ): number {
    /**
     Characters this section carries.
     */
      const sectionChars = section.text
        .length;
      return running + sectionChars;
    },
    0,
  );

  /**
   Every translation character the document holds inside a section.
   */
  const allTargetChars = targetSections.reduce(
    function addSection(
      running,
      section,
    ): number {
    /**
     Characters this section carries.
     */
      const sectionChars = section.text
        .length;
      return running + sectionChars;
    },
    0,
  );

  // One pair consumes one section on each side, so the shortfall IS the refusal
  // count. Read from the sections rather than from the aligner's findings,
  // which name a section without saying how much text it holds.
  /**
   Sections the aligner committed to, counted once for both sides.
   */
  const pairedSections = alignment.pairs
    .length;

  /**
   Halves the recipe lacks, none at all under the deterministic carve.
   */
  const unrecorded = recipe?.unrecorded ?? [];

  /**
   Which carve these sizes describe.
   */
  const carve: CensusCarve = (recipe === undefined)
    ? 'deterministic'
    : ((pairingRefusal !== '')
      ? 'settled-moved'
      : ((unrecorded.length === 0) ? 'settled-complete' : 'settled-partial'));

  return {
    entryId,
    carve,
    pairingRefusal,
    sliceSourceChars: measures.sliceSourceChars,
    sliceTargetChars: measures.sliceTargetChars,
    unpairedSourceSections: sourceSections.length - pairedSections,
    unpairedSourceChars: allSourceChars - pairedSourceChars,
    unpairedTargetSections: targetSections.length - pairedSections,
    unpairedTargetChars: allTargetChars - pairedTargetChars,
    targetOnlyBlocks: measures.targetOnlyBlockChars
      .length,
    targetOnlyChars: measures.targetOnlyBlockChars
      .reduce(
      function addBlock(
        running,
        chars,
      ): number {
        return running + chars;
      },
      0,
      ),
    targetOnlyBlockChars: measures.targetOnlyBlockChars,
  };
}

//endregion Slice census entry
