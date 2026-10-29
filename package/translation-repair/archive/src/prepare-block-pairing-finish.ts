import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type BlockPair,
  crossingFinding,
  splitDefinitionPairs,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { PreparedBlockPairing, } from './prepare-block-pairing-model.ts';

//region Pairing handoff after acquisition and media normalization
// Both cold and warm paths preserve the same definition-order behavior and fallback semantics.

/**
 Separates definition relations before handing an explicit pairing to the production slicer.
 Empty acquired relations retain the scorer fallback rather than becoming an explicit empty map entry.

 @param pairs - acquired relations after existing media-adjacency normalization

 @param findings - observations already reported by acquisition and normalization

 @param pair - parent nodes whose definition labels are being separated

 @param pairIndex - original aligned-parent index used in findings

 @param l - caller logger retaining parent identity

 @returns Explicit slicer pairing or named unresolved fallback

 @example
 ```ts
 const result = finishPreparedBlockPairing({ pairs, findings, pair, pairIndex: 2, l });
 ```
 */
export function finishPreparedBlockPairing(
  {
    pairs,
    findings,
    pair,
    pairIndex,
    l,
  }: {
    readonly pairs: readonly BlockPair[];
    readonly findings: readonly string[];
    readonly pair: ChunkPair;
    readonly pairIndex: number;
    readonly l: Logger;
  },
): Extract<PreparedBlockPairing, { readonly kind: 'paired' | 'fallback'; }> {
  /**
   Logger distinguishing the final map handoff from acquisition.
   */
  const pl = tagged({
    tag: finishPreparedBlockPairing.name,
    l,
  },);
  if (pairs.length === 0) {
    pl.warn(`section ${String(pairIndex,)}: no agreed pairing, keeping the deterministic aligner`,);
    return {
      kind: 'fallback',
      findings,
      definitionPairs: [],
    };
  }
  /**
   Definition labels and ordinary slicer relations under the existing ordering rule.
   */
  const split = splitDefinitionPairs({
    pairs,
    sourceNodes: pair.source
      .nodes,
    targetNodes: pair.target
      .nodes,
  },);
  if (split.crossing)
    pl.warn(crossingFinding({ pairIndex, },),);
  return {
    kind: 'paired',
    pairs: split.forSlicing,
    definitionPairs: split.definitionPairs,
    findings: split.crossing ? [
      ...findings,
      crossingFinding({ pairIndex, },),
    ] : findings,
  };
}

//endregion Pairing handoff after acquisition and media normalization
