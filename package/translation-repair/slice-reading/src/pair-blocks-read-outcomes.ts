import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  agreePairs,
  type BlockPair,
  type BlockPairingWire,
  countPairedBlocks,
  type FreeOrderBlocks,
  readBlockPairing,
  requireBlockPairingRefusal,
  type RoundOutcome,
  type StageVoice,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { assertPairingSeats, } from './pair-blocks-evidence-identity.ts';
import type { BlockPairingOutcome, } from './pair-blocks-stage.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Pairing outcomes read without transport
// The stage's replies read apart from the round that gathered them, so the reader and the
// agreement rule are tested without a client.

/**
 Voices that must name a relation before it is kept, distinct from the stage's roster quorum.
 */
const AGREEMENT_NEEDED = 2;

/**
 Asked-seat outcome whose reply arrived and validated in shape.
 */
type HeardPairingOutcome = RoundOutcome<BlockPairingWire> & {
  /**
   Reply that arrived, with its validated value.
   */
  readonly voice: Extract<StageVoice<BlockPairingWire>, { readonly heard: true; }>;
};

/**
 Reads final asked-seat outcomes through the production pairing reader and agreement rule.
 A schema-valid heard reply may still be unusable because its indexes or order are invalid.
 This operation buys no calls and does not turn missing seats into ballots.

 @param outcomes - final outcomes of exactly the seats the stage asked

 @param modelIds - configured electorate supplying the reported denominator

 @param sourceCount - source block bound used by the original question

 @param targetCount - target block bound used by the original question

 @param freeOrder - definition indexes exempt from ordinary block ordering

 @param l - caller logger retaining preparation identity

 @returns Agreed relations, heard and usable counts, cache eligibility and findings

 @throws {@link import('./pair-blocks-evidence-identity.ts').PairingEvidenceError} when seat identities cannot represent the configured electorate

 @throws Error when an unexpected reader failure occurs

 @example
 ```ts
 const read = readBlockPairingOutcomes({ outcomes, modelIds, sourceCount: 2, targetCount: 3, l });
 ```
 */
export function readBlockPairingOutcomes(
  {
    outcomes,
    modelIds,
    sourceCount,
    targetCount,
    freeOrder,
    l,
  }: ForeignBorrowed<{
    readonly outcomes: readonly RoundOutcome<BlockPairingWire>[];
    readonly modelIds: readonly RosterModelId[];
    readonly sourceCount: number;
    readonly targetCount: number;
    readonly freeOrder?: FreeOrderBlocks;
    readonly l: Logger;
  }>,
): BlockPairingOutcome {
  /**
   Logger tagged with this reader, apart from the stage that gathered the replies.
   */
  const pl = tagged({
    tag: readBlockPairingOutcomes.name,
    l,
  },);
  pl.debug(
    `reading ${String(outcomes.length,)} asked-seat outcomes over ${String(modelIds.length,)} configured ${
      wordForCount({
        count: modelIds.length,
        one: 'seat',
        many: 'seats',
      },)
    }`,
  );
  assertPairingSeats({
    modelIds,
    askedModelIds: outcomes.map(function modelOf(outcome,): RosterModelId {
      return outcome.modelId;
    },),
    l: pl,
  },);
  /**
   Replies that arrived and validated in shape.
   */
  const heardVoices = outcomes.filter(function wasHeard(outcome,): outcome is HeardPairingOutcome {
    return outcome.voice
      .heard;
  },);
  /**
   Findings accumulated while reading replies.
   */
  const findings: string[] = [];
  /**
   Pairings that survived the reader, one per usable voice.
   */
  const pairings: (readonly BlockPair[])[] = [];
  for (const outcome of heardVoices) {
    /**
     This seat's reply, heard by the filter's narrowing.
     */
    const { voice, } = outcome;
    try {
      pairings.push(readBlockPairing({
        value: voice.value,
        sourceCount,
        targetCount,
        ...((freeOrder === undefined) ? {} : { freeOrder, }),
      },),);
    }
    catch (error) {
      /**
       Why the reply cannot be used; anything else propagates.
       */
      const refusal = requireBlockPairingRefusal({ error, },);
      findings.push(`block-pairing unusable (${outcome.modelId}: ${refusal.message})`,);
      pl.warn(`${outcome.modelId} returned an unusable pairing: ${refusal.message}`,);
    }
  }
  if (pairings.length === 0) {
    findings.push(`block-pairing no-usable-voice (${String(heardVoices.length,)} heard of ${String(modelIds.length,)})`,);
    return {
      pairs: [],
      heard: heardVoices.length,
      usable: 0,
      cacheEligible: false,
      findings,
    };
  }
  /**
   Existing many-to-many agreement and monotone-order resolution.
   */
  const agreement = agreePairs({
    pairings,
    needed: AGREEMENT_NEEDED,
    pairingShape: 'many-to-many',
  },);
  /**
   Relations the agreement rule withheld.
   */
  const { findings: dropped, } = agreement;
  findings.push(...dropped.map(function prefix(finding,): string {
    return `block-pairing ${finding}`;
  },),);
  /**
   Relations preserved by endorsement and ordering.
   */
  const agreed = agreement.pairs;
  /**
   Unique block reach beside many-to-many relation count.
   */
  const counts = countPairedBlocks({ pairs: agreed, },);
  pl.info(
    `paired ${String(counts.source,)} of ${String(sourceCount,)} original and ${
      String(counts.target,)
    } of ${String(targetCount,)} translation ${
      wordForCount({
        count: targetCount,
        one: 'block',
        many: 'blocks',
      },)
    } across ${String(counts.relations,)} ${
      wordForCount({
        count: counts.relations,
        one: 'relation',
        many: 'relations',
      },)
    }, from ${
      String(pairings.length,)
    } usable ${
      wordForCount({
        count: pairings.length,
        one: 'voice',
        many: 'voices',
      },)
    } of ${String(heardVoices.length,)} heard`,
  );
  return {
    pairs: agreed,
    heard: heardVoices.length,
    usable: pairings.length,
    cacheEligible: dropped.length === 0,
    findings,
  };
}

//endregion Pairing outcomes read without transport
