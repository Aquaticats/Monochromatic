import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { blockPairingProtocol, } from './block-pairing-protocol.ts';
import {
  type BlockPair,
  type FanOutMode,
  type FreeOrderBlocks,
  isBlockPairingWire,
  type NumberedBlock,
  runWindowedRounds,
  shortBenchStageFinding,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { readBlockPairingOutcomes, } from './pair-blocks-read-outcomes.ts';
import { assertPairingSeats, } from './pair-blocks-evidence-identity.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Block pairing stage
// ASKS THE ROSTER WHICH PARAGRAPH RENDERS WHICH, and keeps only what enough of
// them agree on.
//
// NO SINGLE MODEL DECIDES, which is this package's rule everywhere a model is
// asked anything, and it matters more here than usual: a pairing is the input
// every later stage reasons from, so one model's mistake is not one bad claim
// but a document's worth of claims about passages that were never about the
// same thing.
//
// AGREEMENT IS PER PAIR rather than per reply. Two models can agree on nine
// correspondences and differ on the tenth, and discarding both replies over the
// tenth throws away the nine. Each `source,target` pair is counted on its own
// and kept when enough voices named it.

/**
 What the roster settled on for one document pair.

 @example
 ```ts
 const outcome: BlockPairingOutcome = { pairs: [], heard: 0, usable: 0, cacheEligible: false, findings: [], };
 ```
 */
export type BlockPairingOutcome = {
  /**
   Correspondences enough voices named, in document order.
   */
  readonly pairs: readonly BlockPair[];

  /**
   Voices that answered at all.
   */
  readonly heard: number;

  /**
   Voices whose answer survived the reader.
   */
  readonly usable: number;

  /**
   Whether result is terminal enough for cross-run cache.

   False when agreed correspondences were dropped as contested or non-monotone,
   because another roster round can settle them differently.
   */
  readonly cacheEligible: boolean;

  /**
   What went wrong, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 One section's settled pairing as the cache stores it.

 THE FINDINGS ARE HALF THE RECORD, not decoration beside the pairs. A resumed
 run makes no calls for a cached section, so anything the section reported the
 first time is reported by nothing on the second unless it was stored. Before
 this type the cache held a bare `BlockPair[]`, and a resumed entry lost the
 per-section pairing counts, the fallback notice, and every voice-level
 finding the round produced.

 ROSTER REACHABILITY IS STORED ON PURPOSE, including `block-pairing unusable`
 naming a voice that failed. It reads as a claim about a call this run never
 made, and it is kept anyway: the findings say what buying this pairing cost,
 and a resume that dropped them would report a healthier roster than the one
 that produced the stored pairs. `RefinedSliceSettlement` stores its
 `refine-candidates (N/M heard)` line for the same reason.

 `usable` and `heard` ARE DELIBERATELY ABSENT. They decide whether this round
 may be cached at all, which is a question about the run that asked rather
 than about these blocks, and the finding wording already carries both counts
 for any reader that wants them.

 @example
 ```ts
 const settled: PairedSectionRecord = { pairs: [], findings: [], };
 ```
 */
export type PairedSectionRecord = {
  /**
   Correspondences the roster agreed on, in document order.
   */
  readonly pairs: readonly BlockPair[];

  /**
   Findings this section contributed, in the order a cold run emitted them.
   */
  readonly findings: readonly string[];
};

/**
 Asks the roster to pair two block lists and keeps what enough voices agree on.

 REFUSES RATHER THAN GUESSES. When no voice answers usably the outcome carries
 no pairs and says why, and the caller falls back to the deterministic aligner
 rather than proceeding on one model's word. It follows the section aligner's
 rule: a wrong pairing is worse than no pairing, because it manufactures
 issues rather than skipping work.

 @param client - injected model client

 @param modelIds - roster to ask

 @param sourceBlocks - original blocks in document order

 @param targetBlocks - translation blocks in document order

 @param pictureContext - transcripts of the section's pictures for the sheet, absent when it shows none

 @param freeOrder - chunk-local indices of the footnote definitions on each
 side, exempt from the order rule

 @param signal - caller's steering

 @param exchangeTimeoutMs - per-call bound

 @param l - stage logger

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns What the roster agreed on, with what it lost

 @throws {@link import('./pair-blocks-evidence-identity.ts').PairingEvidenceError} before calls when the configured electorate is empty or duplicated

 @example
 ```ts
 const outcome = await pairBlocksWithRoster({ client, modelIds, sourceBlocks, targetBlocks, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function pairBlocksWithRoster(
  {
    client,
    modelIds,
    sourceBlocks,
    targetBlocks,
    pictureContext,
    freeOrder,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly sourceBlocks: readonly NumberedBlock[];
    readonly targetBlocks: readonly NumberedBlock[];
    readonly pictureContext?: string;
    readonly freeOrder?: FreeOrderBlocks;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<BlockPairingOutcome> {
  /**
   Logger tagged with this stage.
   */
  const pl = tagged({
    tag: pairBlocksWithRoster.name,
    l,
  },);
  assertPairingSeats({
    modelIds,
    l: pl,
  },);

  /**
   Messages and response format the roster is asked with.
   */
  const {
    messages,
    responseFormat,
  } = blockPairingProtocol({
    sourceBlocks,
    targetBlocks,
    ...((pictureContext === undefined) ? {} : { pictureContext, }),
  },);
  /**
   Every voice's reply, heard or lost.
   */
  const {
    outcomes,
    quorum,
  } = await runWindowedRounds({
    client,
    modelIds,
    messages,
    signal,
    exchangeTimeoutMs,
    responseFormat,
    validate: isBlockPairingWire,
    stage: 'block-pairing',
    l: pl,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  /**
   What the replies came to.
   */
  const read = readBlockPairingOutcomes({
    outcomes,
    modelIds,
    sourceCount: sourceBlocks.length,
    targetCount: targetBlocks.length,
    ...((freeOrder === undefined) ? {} : { freeOrder, }),
    l: pl,
  },);
  // A SHORT BENCH IS SAID IN THE FINDINGS, as the gathers say it (ledger X8).
  return quorum.short
    ? {
      ...read,
      findings: [
        ...read.findings,
        shortBenchStageFinding({
          stage: 'block-pairing',
          quorum,
          benchSize: modelIds.length,
        },),
      ],
    }
    : read;
}

//endregion Block pairing stage
