import type {
  BlockPair,
  DefinitionLabelPair,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region One section's pairing result
// What one aligned section's pairing hands document preparation. Whether the
// relations came from the cache or from the roster is not carried: the
// provider-free preparation layer that read it was removed in `cbedea357`, and
// nothing in a pass reads it (ledger D20).

/**
 Observations shared by paired and deliberately unpaired section outcomes.

 @example
 ```ts
 const details = { findings: [], definitionPairs: [] };
 ```
 */
type PreparedBlockDetails = {
  /**
   Stage, cache and structural-normalization findings in the order they arose.
   */
  readonly findings: readonly string[];
  /**
   Definition label relations kept apart from ordinary body ordering.
   */
  readonly definitionPairs: readonly DefinitionLabelPair[];
};

/**
 One aligned section's pairing result, without inventing a correspondence on fallback.
 Only `paired` contributes a map entry to document preparation.
 The other kinds keep apart a structural singleton, an empty side and a
 question the roster or the cache could not settle.

 @example
 ```ts
 if (result.kind === 'paired') blockPairings.set(pairIndex, result.pairs);
 ```
 */
export type PreparedBlockPairing = PreparedBlockDetails & (
  | {
    /**
     Relations survived preparation's media and definition normalization.
     */
    readonly kind: 'paired';
    /**
     Map value handed to the slicer, even when empty after definition separation.
     */
    readonly pairs: readonly BlockPair[];
  }
    | {
    /**
     The section was asked about, or found in the cache, and settled on no relation.
     */
      readonly kind: 'fallback';
    }
    | {
    /**
     A structural singleton or empty side needed no model question or cache lookup.
     */
      readonly kind: 'implicit' | 'empty';
    }
);

//endregion One section's pairing result
