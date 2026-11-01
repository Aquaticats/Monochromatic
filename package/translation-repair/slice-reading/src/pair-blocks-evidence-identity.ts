import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Pairing electorate integrity
// An ordered subsequence proves that every recorded outcome belongs to one distinct configured seat.

/**
 Closed identity failures whose diagnostics cannot carry model prose.

 @example
 ```ts
 const failure: PairingEvidenceFailure = { kind: 'empty-electorate' };
 ```
 */
export type PairingEvidenceFailure =
  | {
    /**
     Configured roster cannot identify independent seats.
     */
    readonly kind: 'empty-electorate' | 'duplicate-electorate';
  }
    | {
    /**
     Recorded identity is missing, repeated or out of configured order.
     */
      readonly kind: 'outcome-order';
    /**
     Position in the recorded outcome sequence, not the configured roster.
     */
      readonly index: number;
    };

/**
 Configuration or recorded-seat identity cannot represent independent pairing evidence.
 Unlike a model's unusable block indexes, this failure aborts interpretation or dispatch.

 @example
 ```ts
 throw new PairingEvidenceError({ kind: 'empty-electorate' });
 ```
 */
export class PairingEvidenceError extends Error {
  /**
   Stable operation name for safe diagnostics.
   */
  public override readonly name = 'PairingEvidenceError';
  /**
   Only a closed failure kind and numeric input index enter the message.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the input that cannot establish independent seats.

   @param failure - closed failure details containing no response wording

   @example
   ```ts
   const error = new PairingEvidenceError({ kind: 'duplicate-electorate' });
   ```
   */
  public constructor(failure: PairingEvidenceFailure,) {
    super(failure.kind === 'outcome-order'
      ? `pairing evidence: askedModelIds[${String(failure.index,)}] is not a unique ordered member of configured modelIds; retain only actual final seat outcomes in configured order`
      : failure.kind === 'empty-electorate'
        ? 'pairing evidence: configured modelIds is empty; retain the electorate used by the pairing stage'
        : 'pairing evidence: configured modelIds repeats an identity; each model must hold exactly one configured seat',);
  }
}

/**
 Refuses invalid electorates and recorded identities before they become calls or endorsements.
 Empty recorded outcomes and sparse ordered subsets are valid; skipped seats are not invented.
 A single configured identity is allowed but cannot satisfy the separate two-voice relation rule.

 @param modelIds - configured electorate in its original order

 @param askedModelIds - final recorded seat identities, omitted before live dispatch

 @param l - caller logger preserving operation identity

 @throws PairingEvidenceError when the electorate is empty or duplicated, or outcomes are not an ordered subset

 @example
 ```ts
 assertPairingSeats({ modelIds, askedModelIds: outcomes.map(outcome => outcome.modelId), l });
 ```
 */
export function assertPairingSeats(
  {
    modelIds,
    askedModelIds = [],
    l,
  }: {
    readonly modelIds: readonly RosterModelId[];
    readonly askedModelIds?: readonly RosterModelId[];
    readonly l: Logger;
  },
): void {
  /**
   Logger for identity validation rather than wire interpretation.
   */
  const pl = tagged({
    tag: assertPairingSeats.name,
    l,
  },);
  pl.debug(`validating ${String(modelIds.length,)} configured and ${String(askedModelIds.length,)} recorded pairing identities`,);
  if (modelIds.length === 0)
    throw new PairingEvidenceError({ kind: 'empty-electorate', },);
  /**
   Configured positions also expose duplicated electorate identities through cardinality.
   */
  const positions = new Map(modelIds.map(function indexed(
    modelId,
    index
  ): readonly [
    RosterModelId,
    number
  ] {
    return [
      modelId,
      index,
    ];
  },),);
  if (positions.size !== modelIds.length)
    throw new PairingEvidenceError({ kind: 'duplicate-electorate', },);
  // Carry the preceding configured position only through this ordered-subsequence check.
  askedModelIds.reduce(
    function advanceSeat(
      previous,
      modelId,
      index,
    ): number {
      /**
       Missing members cannot advance from the initial or any later configured position.
       */
      const position = positions.get(modelId,) ?? (-1);
      if (position <= previous)
        throw new PairingEvidenceError({
          kind: 'outcome-order',
          index,
        },);
      return position;
    },
    -1,
  );
}

//endregion Pairing electorate integrity
