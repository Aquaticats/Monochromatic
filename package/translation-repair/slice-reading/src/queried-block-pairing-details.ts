import type { ChunkPair, } from './chunk-document.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { declinedTargetIdsOfPairing, } from './declined-target-runs.ts';
import {
  type BlockPair,
  claimMediaAdjacentTargets,
  countPairedBlocks,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { BlockPairingOutcome, } from './pair-blocks-stage.ts';
import type { ContainerSpan, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Queried preparation before persistence
// Findings and the cache gate for a section the roster was asked about, worked out before
// definition separation so the cache stores the relations whole.

/**
 Existing queried-parent normalization and persistence eligibility.

 @example
 ```ts
 const details = queriedBlockPairingDetails({ outcome, pair, pairIndex, targetContainers });
 ```
 */
export type QueriedBlockPairingDetails = {
  /**
   Normalized relations before definition-order separation.
   */
  readonly pairs: readonly BlockPair[];
  /**
   Exact stage, media, coverage and fallback findings in production order.
   */
  readonly findings: readonly string[];
  /**
   Whether the pairing may be cached: nothing was dropped as contested or out of order,
   and every archive block is claimed.
   */
  readonly canPersistPairing: boolean;
};

/**
 Derives the current queried handoff without persisting or changing its ordering.

 @param outcome - current pairing stage result

 @param pair - current complete parent

 @param pairIndex - original alignment position used in findings

 @param targetContainers - complete target parser's media ownership spans

 @returns Production normalization and unchanged cache gate

 @throws Error when existing structural media validation fails

 @example
 ```ts
 const { pairs, findings, canPersistPairing, } = queriedBlockPairingDetails({ outcome, pair, pairIndex, targetContainers });
 ```
 */
export function queriedBlockPairingDetails({
  outcome,
  pair,
  pairIndex,
  targetContainers,
}: {
  readonly outcome: BlockPairingOutcome;
  readonly pair: ChunkPair;
  readonly pairIndex: number;
  readonly targetContainers: readonly ContainerSpan[];
},): QueriedBlockPairingDetails {
  /**
   Source blocks retain their current local indexes.
   */
  const sourceNodes = pair.source
    .nodes;
  /**
   Target blocks retain the same current-question convention.
   */
  const targetNodes = pair.target
    .nodes;
  /**
   Media ownership joined to the relations the roster actually supplied.
   */
  const media = claimMediaAdjacentTargets({
    pairs: outcome.pairs,
    sourceBlocks: sourceNodes,
    targetBlocks: targetNodes,
    targetContainers,
  },);
  /**
   Normalized relations the cache retains before definition-order separation.
   */
  const { pairs, } = media;
  /**
   Archive blocks still outside a source claim prevent terminal caching.
   */
  const unclaimed = declinedTargetIdsOfPairing({
    pairs,
    sourceNodes,
    targetNodes,
  },);
  /**
   Findings retain exactly the cold path's stage-then-media order.
   */
  const findings: string[] = [
    ...outcome.findings,
    ...media.findings
      .map(function prefix(finding,): string { return `block-pairing ${finding}`; },),
  ];
  if (outcome.usable > 0) {
    /**
     Counts describe roster relations, not additional structural media claims.
     */
    const counts = countPairedBlocks({ pairs: outcome.pairs, },);
    findings.push(
      `block-pairing section ${String(pairIndex,)} paired ${String(counts.source,)} of ${String(sourceNodes.length,)} original and ${String(counts.target,)} of ${String(targetNodes.length,)} translation ${
        wordForCount({
          count: targetNodes.length,
          one: 'block',
          many: 'blocks',
        },)
      } across ${String(counts.relations,)} ${
        wordForCount({
          count: counts.relations,
          one: 'relation',
          many: 'relations',
        },)
      }, from ${String(outcome.usable,)} usable ${
        wordForCount({
          count: outcome.usable,
          one: 'voice',
          many: 'voices',
        },)
      } of ${String(outcome.heard,)} heard`,
    );
  }
  /**
   Existing cache gate excludes unresolved agreement and unclaimed archive blocks.
   */
  const canPersistPairing = outcome.cacheEligible ? unclaimed.size === 0 : false;
  if ((!canPersistPairing) && (outcome.usable > 0))
    findings.push(`block-pairing section ${String(pairIndex,)} unresolved, not cached`,);
  if (pairs.length === 0)
    findings.push(`block-pairing section ${String(pairIndex,)} fell back to scoring`,);
  return {
    pairs,
    findings,
    canPersistPairing,
  };
}

//endregion Queried preparation before persistence
