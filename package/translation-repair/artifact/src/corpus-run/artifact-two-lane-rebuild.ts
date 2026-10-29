import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type {
  CorpusPin,
  PreparedDocumentPair,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  type BlockPair,
  requireBlockPairingRefusal,
  type SectionPair,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  ARTIFACT_SCHEMA_VERSION_V12,
  ARTIFACT_SCHEMA_VERSION_V5,
  artifactGenerationRecordsTense,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import type { ParsedTwoLaneArtifact, } from './artifact-two-lane-read-contract.ts';
import { carveDivergence, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type EntryInfoRecord,
  readEntryInfoRecord,
} from '@monochromatic-dev/translation-repair-insertion/ts';

//region Preparation rebuilt from a settled artifact
// Carving a document pair the way the run that settled it carved it.
//
// WHY A REBUILD EXISTS. The pass carves through the roster shell, whose section
// round and block rounds move slices the deterministic aligner would place
// elsewhere, and the caches that held those answers retire at settlement. An
// instrument that calls the bare `prepareDocumentPair` over the corpus is
// therefore measuring a pipeline that no longer runs. The artifact records
// both halves of the pairing recipe, and this is the one place they are
// turned back into the map and list `prepareDocumentPair` consumed.
//
// A GAP IS NAMED, NOT PAPERED OVER. An artifact written before a recipe half
// existed rebuilds with the deterministic default for that half, and says so.
// Whether that default was what the run actually used is then a question the
// identity check answers: a match proves the slicing is reproduced, and a
// mismatch beside a named gap is not evidence that the slicing moved.
//
// A COMPLETE RECIPE IS NOT A REPRODUCTION (ledger A12). The slicer that reads
// the recipe changes too: class one hundred twelve (`a43c5d88d`) reads an
// interior gap unplaced on both sides as one merge, and mikaela15, settled at
// the commit before it, rebuilt to 32 of its recorded 34 slices with an empty
// gap list. And the artifact records the carve after the pass folded carried
// passages into their carriers, while nothing records the fold (TianqiChen66614
// rebuilds with slices 13 and 14 moved). So the rebuild compares its carve with
// the rows the run recorded and says where it departs
// (`artifact-two-lane-rebuild-rows.ts` says why rows rather than identity).
//
// A RECIPE THAT DOES NOT FIT ITS TEXT IS A MOVED CARVE, NOT A CRASH. The
// recorded block pairing names blocks by index, and the text the rebuild
// carves can parse into fewer blocks than the run's did: an artifact that
// predates storing its archive is carved over the corpus copy, and a parser
// change moves block boundaries. `blockPairingToSteps` refuses a pair naming a
// block its section lacks, and the rebuild answers `moved` in that refusal's
// words, so a republish leaves the page and says why, and the instruments
// that walk every settled artifact (`carveSettled`, the rendering audit) go on
// to the next one instead of stopping on this one.

/**
 One half of the pairing recipe a settled artifact may fail to record.

 @example
 ```ts
 const gap: RecipeHalf = 'sectionPairing';
 ```
 */
export type RecipeHalf = 'sectionPairing' | 'blockPairing';

/**
 The pairing recipe an artifact records, as the inputs `prepareDocumentPair`
 consumes, beside the halves the file does not record.

 @example
 ```ts
 const { sectionPairing, blockPairings, unrecorded, } = recipeOf({ artifact, },);
 ```
 */
export type PairingRecipe = {
  /**
   Section pairing to supply, present only when the file records one as
   supplied.
   */
  readonly sectionPairing?: readonly SectionPair[];

  /**
   Block pairings keyed by aligned section index, present only when the file
   records them.
   */
  readonly blockPairings?: ReadonlyMap<number, readonly BlockPair[]>;

  /**
   Recipe halves the artifact does not record, each to be rebuilt as the
   deterministic default; empty when the recipe is complete.
   */
  readonly unrecorded: readonly RecipeHalf[];
};

/**
 A preparation carved from an artifact's recorded recipe, beside what the
 recipe was missing.

 @example
 ```ts
 const { prepared, unrecorded, } = rebuildPreparation({ artifact, sourceText, targetText, },);
 ```
 */
export type RebuiltPreparation = {
  /**
   Slicing carved over the two texts with every recorded recipe half applied;
   where the recorded block pairing names a block the carved text lacks, with
   the deterministic aligner's blocks in every section instead, the way an
   artifact that records no block pairing is carved.
   */
  readonly prepared: PreparedDocumentPair;

  /**
   Recipe halves the artifact does not record, each rebuilt as the
   deterministic default; empty when the recipe is complete, which alone
   does not make the rebuild the run's carve.
   */
  readonly unrecorded: readonly RecipeHalf[];

  /**
   Whether the rebuild is the run's own carve, read off the rows the run
   recorded.
   */
  readonly reproduction: RebuildReproduction;
};

/**
 Whether a rebuild is the run's carve, and where it departs when it is not.

 @example
 ```ts
 const reproduction: RebuildReproduction = { kind: 'moved', detail: '32 slices rebuilt where the run recorded 34', };
 ```
 */
export type RebuildReproduction =
  | {
    readonly kind: 'reproduced';
  }
    | {
      readonly kind: 'moved';

    /**
     First departure from the recorded rows, or the refusal of a recorded
     block pairing that does not fit the text carved.
     */
      readonly detail: string;
    };

/**
 Turns a recorded section decider back into the list preparation consumed.

 @param artifact - parsed artifact

 @returns Pairs to supply, in a one-element list, or nothing when the
 deterministic aligner decided or the file does not say

 @example
 ```ts
 const [sectionPairing,] = sectionPairingOf({ artifact, },);
 ```
 */
function sectionPairingOf(
  { artifact, }: { readonly artifact: ParsedTwoLaneArtifact; },
): readonly (readonly SectionPair[])[] {
  /**
   What the artifact says about its sections.
   */
  const { sectionPairing, } = artifact.preparation;
  if (sectionPairing.kind !== 'supplied')
    return [];
  return [
    sectionPairing.pairs
      .map(function live(pair,): SectionPair {
        return {
          source: pair.source,
          target: pair.target,
        };
      },),
  ];
}

/**
 Turns a recorded block pairing back into the map preparation consumed.

 @param artifact - parsed artifact

 @returns Map keyed by aligned section index, in a one-element list, or
 nothing when the file records no block pairing

 @example
 ```ts
 const [blockPairings,] = blockPairingsOf({ artifact, },);
 ```
 */
function blockPairingsOf(
  { artifact, }: { readonly artifact: ParsedTwoLaneArtifact; },
): readonly ReadonlyMap<number, readonly BlockPair[]>[] {
  /**
   What the artifact says about its blocks.
   */
  const { blockPairing, } = artifact.preparation;
  if (blockPairing.kind !== 'stored')
    return [];
  return [
    new Map(blockPairing.sections
      .map(function entry(section,): readonly [
        number,
        readonly BlockPair[],
      ] {
        return [
          section.sectionIndex,
          section.pairs
            .map(function live(pair,): BlockPair {
              return {
                source: pair.source,
                target: pair.target,
              };
            },),
        ];
      },),),
  ];
}

/**
 Reads the pairing recipe an artifact records, as preparation inputs.

 SEPARATE FROM THE REBUILD because the slice census walks alignment and
 subdivision itself, for its section-level accounting, and needs the recipe
 pieces rather than a finished preparation.

 @param artifact - parsed artifact naming the recipe

 @returns Recipe halves the file records, and the names of those it lacks

 @example
 ```ts
 const recipe = recipeOf({ artifact, },);
 ```
 */
export function recipeOf(
  { artifact, }: { readonly artifact: ParsedTwoLaneArtifact; },
): PairingRecipe {
  /**
   Section pairing to supply, when one was recorded as supplied.
   */
  const [sectionPairing,] = sectionPairingOf({ artifact, },);

  /**
   Block pairings to supply, when the file records them.
   */
  const [blockPairings,] = blockPairingsOf({ artifact, },);

  /**
   Halves the file does not say anything about.
   */
  const unrecorded: RecipeHalf[] = [];

  /**
   What the artifact says about both halves, read once for the gap list.
   */
  const {
    sectionPairing: recordedSections,
    blockPairing: recordedBlocks,
  } = artifact.preparation;
  if (recordedSections.kind === 'unrecorded')
    unrecorded.push('sectionPairing',);
  if (recordedBlocks.kind === 'unrecorded')
    unrecorded.push('blockPairing',);
  return {
    ...((sectionPairing === undefined) ? {} : { sectionPairing, }),
    ...((blockPairings === undefined) ? {} : { blockPairings, }),
    unrecorded,
  };
}

/**
 Says why a recorded block pairing was set aside and what carved the blocks
 instead, in the one wording the rebuild and the slice census both give, so
 an artifact read by either answers with the same sentence.

 @param refusal - why the recorded pairing does not fit the text carved

 @returns Sentence a `moved` rebuild and a `settled-moved` census row carry

 @example
 ```ts
 const detail = pairingSetAsideSentence({ refusal: 'pairing names translation block 5, and there are 4', },);
 ```
 */
export function pairingSetAsideSentence(
  { refusal, }: { readonly refusal: string; },
): string {
  return `the recorded block pairing does not fit the text carved (${refusal}), so every section's `
    + 'blocks were carved by the deterministic aligner';
}

/**
 Carves a document pair with the recipe a settled artifact records.

 CARVES OVER THE ARCHIVE THE ARTIFACT STORED (ledger A18). A pass reshapes
 the archive before it carves (`passArchiveText`, a heading relabel,
 `repairArchiveBlocks`) and the artifact stores the text it carved, so the
 corpus copy at the artifact's commit is carved only for an artifact written
 before that text was stored. Carving the corpus copy read 23 of 77 stored
 carves as moved that the stored archive reproduces, and left shihai4h's
 republished page 9 characters short.

 A RECORDED BLOCK PAIRING THAT NAMES A BLOCK THE TEXT LACKS is refused while
 the carve converts it, and the artifact is then carved again as one that
 records no block pairing: every section's blocks by the deterministic
 aligner, not only the refused section's. The refusal names the block and the
 count, not the section, and a pairing that does not fit one section's blocks
 says the text parsed otherwise than the run's, which makes the pairings that
 still fit by count no better evidence of the run's blocks.

 @param artifact - parsed artifact naming the recipe

 @param sourceText - whole original, as read at the artifact's own commit

 @param targetText - archive English at the artifact's own commit, carved
 only where the artifact predates storing the text it carved

 @param sourceEntryInfo - parsed `info.yml` of the original's entry as read at
 the artifact's own commit, absent where the entry has none or the caller reads
 none; its recorded death joins the identity block only for an artifact of a
 generation whose runs told their models the death, since the models of an
 earlier run were never shown it

 @returns Preparation and the recipe halves that had to be defaulted, and
 `moved` in the refusal's words where the recorded block pairing names a
 block the carved text lacks

 @throws Error when the carve refuses a block pairing for an artifact that
 records none, which no carve does: only a recorded pairing names blocks by
 index

 @example
 ```ts
 const rebuilt = rebuildPreparation({ artifact, sourceText, targetText, },);
 ```
 */
export function rebuildPreparation(
  {
    artifact,
    sourceText,
    targetText: corpusTarget,
    sourceEntryInfo,
  }: {
    readonly artifact: ParsedTwoLaneArtifact;
    readonly sourceText: string;
    readonly targetText: string;
    readonly sourceEntryInfo?: unknown;
  },
): RebuiltPreparation {
  /**
   Recipe the file records, and what it lacks.
   */
  const {
    sectionPairing,
    blockPairings,
    unrecorded,
  } = recipeOf({ artifact, },);
  /**
   Whose front matter the recorded slicing carried, read off the file and
   never recomputed: a later change to the rule must not re-slice an older
   file; and the archive the run carved, where the file kept it.
   */
  const {
    frontMatterAuthority,
    archiveText: storedArchive,
  } = artifact.preparation;
  /**
   Archive text the carve runs over.
   */
  const targetText = (storedArchive.kind === 'stored') ? storedArchive.text : corpusTarget;
  try {
    /**
     Slicing carved from the recipe.
     */
    const prepared = prepareDocumentPair({
      sourceText,
      targetText,
      includeFrontMatter: artifact.artifactSchemaVersion >= ARTIFACT_SCHEMA_VERSION_V5,
      ...((frontMatterAuthority === undefined) ? {} : { frontMatterAuthority, }),
      // SEALED AGAIN ONLY FROM THE GENERATION THAT SEALED, read off the
      // version rather than the record: the spans are recomputed from the
      // archive text because the block correction round moves offsets, and an
      // older file's slicing never sealed anything.
      sealArchiveOriginal: artifact.artifactSchemaVersion >= ARTIFACT_SCHEMA_VERSION_V12,
      ...((sectionPairing === undefined) ? {} : { sectionPairing, }),
      ...((blockPairings === undefined) ? {} : { blockPairings, }),
      // THE DEATH IS TOLD ONLY WHERE THE RUN TOLD IT, read off the generation
      // as the front matter and sealing flags of this call are: a run of an
      // earlier generation read no `info.yml`, and its models saw no date.
      ...(((sourceEntryInfo === undefined)
        || (!artifactGenerationRecordsTense({ generation: artifact.artifactSchemaVersion, },)))
        ? {}
        : { sourceEntryInfo, }),
    },);
    /**
     First departure from the run's recorded carve, empty for none. The repair
     lane's ledger records it, a row for every slice the run carved.
     */
    const divergence = carveDivergence({
      rows: artifact
        .lanes
        .repair
        .delivery,
      slices: prepared.slices,
    },);
    return {
      prepared,
      unrecorded,
      reproduction: (divergence === '')
        ? { kind: 'reproduced', }
        : {
          kind: 'moved',
          detail: divergence,
        },
    };
  }
  catch (error) {
    // Only the carve throws a pairing refusal; anything else propagates.
    /**
     Why the recorded block pairing does not fit the text carved.
     */
    const { message: refusal, } = requireBlockPairingRefusal({ error, },);
    // ONE CARVE AGAIN, NEVER MORE: the artifact read as one that records no
    // block pairing hands the carve no pairing to refuse.
    if (blockPairings === undefined)
      throw new Error(
        'unreachable: the carve refused a block pairing, though this artifact records none and only a recorded '
          + 'pairing names blocks by index',
        { cause: error, },
      );
    /**
     The same artifact carved as one that records no block pairing.
     */
    const { prepared, } = rebuildPreparation({
      artifact: {
        ...artifact,
        preparation: {
          ...artifact.preparation,
          blockPairing: { kind: 'unrecorded', },
        },
      },
      sourceText,
      targetText: corpusTarget,
      ...((sourceEntryInfo === undefined) ? {} : { sourceEntryInfo, }),
    },);
    return {
      prepared,
      unrecorded,
      reproduction: {
        kind: 'moved',
        detail: pairingSetAsideSentence({ refusal, },),
      },
    };
  }
}

/**
 Reads the `info.yml` of an artifact's entry at the artifact's own commit, only
 for a generation whose runs told their models the recorded death. An artifact
 of an earlier generation reads nothing, so a damaged `info.yml` at an old
 commit refuses no replay that never needed it.

 @param generation - generation of the artifact whose entry is read

 @param entryId - directory name of the artifact's entry

 @param pin - corpus checkout and the artifact's own commit

 @param readFile - document reader at the pin

 @param l - logger the absence and a file that refuses to parse are named on, left out by a command whose terminal output is pinned, which then records both in silence

 @returns Parsed record, or the absence for an entry with no `info.yml` and for an earlier generation, which is not read at all, or the unreadable record for a file that refuses to parse; the last two record no death, as the run that settled the artifact recorded none for a file it could not read

 @throws {@link Error} when the file exists but cannot be read, for a generation that reads it

 @example
 ```ts
 const entryInfo = await readSettledEntryInfo({ generation: artifact.artifactSchemaVersion, entryId: artifact.id, pin, readFile: readCorpusFile, },);
 ```
 */
export async function readSettledEntryInfo(
  {
    generation,
    entryId,
    pin,
    readFile,
    l,
  }: {
    readonly generation: TwoLaneArtifactGeneration;
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly readFile: typeof readCorpusFile;
    readonly l?: Logger;
  },
): Promise<EntryInfoRecord> {
  if (!artifactGenerationRecordsTense({ generation, },))
    return { kind: 'absent', };
  return await readEntryInfoRecord({
    pin,
    entryId,
    readFile,
    ...((l === undefined) ? {} : { l, }),
  },);
}

//endregion Preparation rebuilt from a settled artifact
