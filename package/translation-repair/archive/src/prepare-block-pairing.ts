import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  blockPairingQuestion,
  type ChunkPair,
  pairBlocksWithRoster,
  type PairedReading,
  type PairedSectionRecord,
  pairingPictureContext,
  queriedBlockPairingDetails,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type BlockPairingError,
  claimMediaAdjacentTargets,
  type FanOutMode,
  type FreeOrderBlocks,
  readBlockPairing,
  requireBlockPairingRefusal,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { finishPreparedBlockPairing, } from './prepare-block-pairing-finish.ts';
import type { PreparedBlockPairing, } from './prepare-block-pairing-model.ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ContainerSpan, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region One indexed parent through existing block preparation
// One aligned section's pairing: settled by structure, resumed from the cache, or asked of the roster.
//
// A CACHED PAIRING IS READ AGAINST THE BLOCKS BEFORE ANYTHING USES IT. The
// cache reader checks a record's shape only (`isPairList`), and the key names
// the blocks' text, so a record that names a block its section lacks, moves
// backwards, repeats a correspondence or pairs a definition with a body block
// was not written by this pipeline for this question: a damaged or hand-edited
// file. It is a miss, said on a warning, and the section is bought again. It
// is read through `readBlockPairing`, the whole reader a fresh reply goes
// through, with the question's own footnote definitions as the blocks whose
// order is free. Used as it stood, a record naming a block the section lacks
// stopped the run with a missing-value error naming nothing, a backwards one
// stopped the slicer with a coverage error far from the cache, and a repeated
// or mixed one was sliced without a word.

/**
 The refusal of a cached pairing read against the section's blocks, or
 nothing when every pair names a block the section carries.

 @param pairs - pairs the cached record holds

 @param sourceCount - original blocks of the section

 @param targetCount - translation blocks of the section

 @param freeOrder - section's footnote definitions, whose order the
 reader leaves free as it does for a fresh reply

 @returns Refusal in a one-element list, or an empty list when the
 record fits

 @throws The caught value unchanged when the check fails other than by
 refusing the pairing

 @example
 ```ts
 const [misfit,] = cachedPairingMisfit({ pairs: cached.pairs, sourceCount: 2, targetCount: 2, freeOrder, },);
 ```
 */
function cachedPairingMisfit(
  {
    pairs,
    sourceCount,
    targetCount,
    freeOrder,
  }: {
    readonly pairs: PairedSectionRecord['pairs'];
    readonly sourceCount: number;
    readonly targetCount: number;
    readonly freeOrder: FreeOrderBlocks;
  },
): readonly BlockPairingError[] {
  try {
    readBlockPairing({
      value: { pairs, },
      sourceCount,
      targetCount,
      freeOrder,
    },);
    return [];
  }
  catch (error) {
    return [requireBlockPairingRefusal({ error, },),];
  }
}

/**
 Prepares one already-aligned parent without buying unrelated section or block questions.
 Singletons and empty sides retain their zero-call paths.
 A cached section reuses its stored relations and findings without asking again.
 A cached record the whole block reader refuses (a block the section lacks,
 a move backwards, a repeated correspondence, a definition paired with a body
 block) is a miss: a warning says so in the refusal's words, and the roster is
 asked as for a section never cached.

 @param client - existing production model client

 @param modelIds - configured preparation electorate, unchanged by this operation

 @param pair - complete aligned source and target parent

 @param pairIndex - original alignment index used for findings and map identity

 @param targetContainers - complete target parser containers for media-adjacency ownership

 @param signal - caller cancellation

 @param exchangeTimeoutMs - existing per-call ceiling

 @param l - caller logger preserving entry identity

 @param pairingCache - existing versioned block cache, absent for an uncached probe

 @param fanOut - seats a pairing round asks: the window of quorum plus one
 by default, or the whole bench a fixture scripting every seat asks for,
 since the window is rotated by the sheet text and a sheet edit elsewhere
 moves which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Explicit slicer pairing or the existing implicit, empty or fallback state

 @throws Error when acquisition, cancellation or cache persistence fails unexpectedly

 @example
 ```ts
 const result = await prepareBlockPairing({ client, modelIds, pair, pairIndex: 2, targetContainers, signal, exchangeTimeoutMs, l, stageSettings, });
 ```
 */
export async function prepareBlockPairing(
  {
    client,
    modelIds,
    pair,
    pairIndex,
    targetContainers,
    signal,
    exchangeTimeoutMs,
    l,
    pairingCache,
    pictureReadings,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly pair: ChunkPair;
    readonly pairIndex: number;
    readonly targetContainers: readonly ContainerSpan[];
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly pairingCache?: SliceCache<PairedSectionRecord>;
    readonly pictureReadings?: ReadonlyMap<string, PairedReading>;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<PreparedBlockPairing> {
  /**
   Logger naming this parent's shared preparation operation.
   */
  const pl = tagged({
    tag: prepareBlockPairing.name,
    l,
  },);
  /**
   Parsed source blocks whose local indexes the question uses.
   */
  const sourceNodes = pair.source
    .nodes;
  /**
   Parsed incumbent blocks under the same local-index convention.
   */
  const targetNodes = pair.target
    .nodes;
  if ((sourceNodes.length === 0) || (targetNodes.length === 0)) {
    pl.debug(`section ${String(pairIndex,)} has an empty side; no pairing question`,);
    return {
      kind: 'empty',
      findings: [],
      definitionPairs: [],
    };
  }
  if ((sourceNodes.length < 2) && (targetNodes.length < 2)) {
    pl.debug(`section ${String(pairIndex,)} is a structural singleton; no pairing question`,);
    return {
      kind: 'implicit',
      findings: [],
      definitionPairs: [],
    };
  }
  /**
   What the sheet is shown about this section's pictures, empty when nobody
   read any (class thirty-four).
   */
  const pictureContext = (pictureReadings === undefined)
    ? ''
    : pairingPictureContext({
      pair,
      pictureReadings,
    },);
  if (pictureContext !== '')
    pl.info(`section ${String(pairIndex,)} pairs with ${String(pictureContext.length,)} ${
      wordForCount({
        count: pictureContext.length,
        one: 'character',
        many: 'characters',
      },)
    } of picture transcript in the sheet`,);
  /**
   Shared current numbering, definition exemptions and unchanged cache identity.
   */
  const {
    sourceBlocks,
    targetBlocks,
    freeOrder,
    key,
  } = blockPairingQuestion({
    pair,
    pictureContext,
    modelIds,
  },);
  /**
   Relations and findings a past round stored under this question's key.
   */
  const cached = pairingCache?.resumed
    .get(key,);
  /**
   Why the cached record does not fit this section's blocks, absent when it
   fits or nothing was cached.
   */
  const [misfit,] = (cached === undefined)
    ? []
    : cachedPairingMisfit({
      pairs: cached.pairs,
      sourceCount: sourceNodes.length,
      targetCount: targetNodes.length,
      freeOrder,
    },);
  if (misfit !== undefined)
    pl.warn(`section ${String(pairIndex,)} misses the block-pairing cache: the record under ${key} does not fit its `
      + `blocks (${misfit.message}), so the roster is asked again`,);
  if ((cached !== undefined) && (misfit === undefined)) {
    pl.debug(`section ${String(pairIndex,)} resumes block pairing ${key}`,);
    /**
     Current media ownership applied to the cached relations, as on the cold path.
     */
    const media = claimMediaAdjacentTargets({
      pairs: cached.pairs,
      sourceBlocks: sourceNodes,
      targetBlocks: targetNodes,
      targetContainers,
    },);
    return finishPreparedBlockPairing({
      pairs: media.pairs,
      findings: [
        ...cached.findings,
        ...media.findings
          .map(function prefix(finding,): string {
            return `block-pairing ${finding}`;
          },),
      ],
      pair,
      pairIndex,
      l: pl,
    },);
  }
  /**
   What the roster agreed on when asked now.
   */
  const outcome = await pairBlocksWithRoster({
    client,
    modelIds,
    sourceBlocks,
    targetBlocks,
    pictureContext,
    freeOrder,
    signal,
    exchangeTimeoutMs,
    l: pl,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    stageSettings,
  },);
  /**
   Shared normalization retains the exact findings and cache gate before persistence.
   */
  const {
    pairs,
    findings,
    canPersistPairing,
  } = queriedBlockPairingDetails({
    outcome,
    pair,
    pairIndex,
    targetContainers,
  },);
  // Persistence precedes definition separation, preserving the historical cache's relabel evidence.
  if ((outcome.usable > 0) && canPersistPairing)
    await pairingCache?.persist({
      key,
      serialized: JSON.stringify({
        pairs,
        findings,
      },),
    },);
  return finishPreparedBlockPairing({
    pairs,
    findings,
    pair,
    pairIndex,
    l: pl,
  },);
}

//endregion One indexed parent through existing block preparation
