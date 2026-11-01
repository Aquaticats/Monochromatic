import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import type {
  ReadonlyEntry,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Critic attribution
// Which critic raised which claim, kept OUTSIDE the claim because adjudication
// is provenance-blind and must stay that way: a real defect can arrive with
// exactly one proposer, so proposer counts may calibrate critics but must never
// influence acceptance. Keyed by the deterministic claim id, which claims
// already carry, so nothing here has to travel inside `IssueClaim`.

/**
 One critic's contribution to one claim.

 @example
 ```ts
 const proposer: ClaimProposer = { modelId: 'hf:openai/gpt-oss-120b', emissionCount: 2, };
 ```
 */
export type ClaimProposer = {
  /**
   Critic that emitted this claim.
   */
  readonly modelId: RosterModelId;

  /**
   How many times this critic emitted it within its own report;
   separated from proposer count because one critic repeating itself is
   self-repetition while two critics agreeing is independent support, and a
   flat model list conflates exactly those two.
   */
  readonly emissionCount: number;
};

/**
 Every critic behind one deduplicated claim.

 @example
 ```ts
 const attribution: ClaimAttribution = { claimId, proposers, };
 ```
 */
export type ClaimAttribution = {
  /**
   Deterministic `issue/<hash>` identity from `computeIssueClaimId`.
   */
  readonly claimId: string;

  /**
   Critics that emitted it, ordered by model id so identical inputs serialize
   identically into a cached outcome.
   */
  readonly proposers: readonly ClaimProposer[];
};

/**
 One critic emitting one resolved claim, before deduplication.

 @example
 ```ts
 const emission: ClaimEmission = { claimId, modelId, };
 ```
 */
export type ClaimEmission = {
  /**
   Deterministic identity of the claim emitted.
   */
  readonly claimId: string;

  /**
   Critic that emitted it.
   */
  readonly modelId: RosterModelId;
};

/**
 Folds raw emissions into per-claim, per-critic counts.

 Must run BEFORE `aggregateClaims` deduplicates: structurally identical claims
 collapse to one id there, so afterwards there is no longer anything to
 attribute a second emitter to.

 @param emissions - every emission in critic then report order

 @returns Attribution per claim, both claims and proposers sorted by id so
 identical evidence serializes identically

 @example
 ```ts
 const attributions = collectClaimAttributions({ emissions, },);
 ```
 */
export function collectClaimAttributions(
  {
    emissions,
  }: {
    readonly emissions: readonly ClaimEmission[];
  },
): readonly ClaimAttribution[] {
  /**
   Per-claim tally of how often each critic emitted it. Mutable while folding
   and never returned, because a `Map` would not survive `JSON.stringify`
   into a cached outcome.
   */
  const tally = new Map<string, Map<RosterModelId, number>>();

  for (
    const {
      claimId,
      modelId,
    } of emissions
  ) {
    /**
     Critics seen for this claim so far.
     */
    const byModel = tally.get(claimId,) ?? new Map<RosterModelId, number>();
    byModel.set(
      modelId,
      (byModel.get(modelId,) ?? 0) + 1,
    );
    tally.set(
      claimId,
      byModel,
    );
  }

  return [...tally.entries(),]
    // Sorted by claim id, NOT left in insertion order. Insertion order follows
    // voice ARRIVAL, and `gatherStageVoices` orders voices by retry round then
    // roster position, so a run that heard one critic on the first round and
    // another on a retry would order these differently from a run that heard
    // them the other way around, even with identical evidence. That reaches a
    // cached outcome compared across runs.
    .toSorted(function byClaimId(
      [left,],
      [right,],
    ): number {
      return compareCodePoints({
        left,
        right,
      },);
    },)
    .map(function toAttribution(
      [
        claimId,
        byModel,
      ],
    ): ClaimAttribution {
      return {
        claimId,
        proposers: [...byModel.entries(),]
          .map(function toProposer([modelId, emissionCount,]: ReadonlyEntry<RosterModelId, number>,): ClaimProposer {
            return {
              modelId,
              emissionCount,
            };
          },)
        // Code-point order rather than localeCompare: this value is serialized
        // into a cached outcome and compared across runs, so a comparison that
        // depends on the machine's default locale could order the same
        // proposers differently on two hosts.
          .toSorted(function byModelId(
            left,
            right,
          ): number {
            return compareCodePoints({
              left: left.modelId,
              right: right.modelId,
            },);
          },),
      };
    },);
}

/**
 Drops attribution for claims that did not survive a later screen.

 Screening removes claims after attribution is built, and an entry left
 pointing at a discarded claim would inflate a critic's recorded hits with
 claims the pipeline threw away.

 @param attributions - attribution built at emission time

 @param claimIds - identities still standing

 @returns Attribution restricted to surviving claims, order preserved

 @example
 ```ts
 const kept = retainAttributions({ attributions, claimIds, },);
 ```
 */
export function retainAttributions(
  {
    attributions,
    claimIds,
  }: {
    readonly attributions: readonly ClaimAttribution[];
    readonly claimIds: ReadonlySet<string>;
  },
): readonly ClaimAttribution[] {
  return attributions.filter(function survives(attribution,): boolean {
    return claimIds.has(attribution.claimId,);
  },);
}

/**
 One chunk's calibration record, as the run artifact carries it.

 @example
 ```ts
 const record: SliceCriticRecord = { sliceIndex: 0, heardCriticIds, claimAttributions, };
 ```
 */
export type SliceCriticRecord = {
  /**
   Chunk position within the document.
   */
  readonly sliceIndex: number;

  /**
   Critics that answered on this chunk, sorted by model id.
   */
  readonly heardCriticIds: readonly RosterModelId[];

  /**
   Which critics raised each surviving claim of this chunk.
   */
  readonly claimAttributions: readonly ClaimAttribution[];
};

/**
 Collects each chunk's calibration record for the run artifact.

 PER CHUNK rather than folded into the issue list, because a chunk whose
 critics raised nothing produces no issue record at all, and dropping it would
 discard exactly the denominator that makes a rate computable. Keeping the
 chunks separate also stops two chunks that happened to produce an identical
 claim from merging their proposers into one inflated entry.

 @param outcomes - settled chunk outcomes in any order

 Does NOT reject a repeated chunk index, deliberately, though the READER
 throws on one. The proportion matters: an artifact's primary value is the
 repaired text, and attribution is telemetry beside it. Failing here would
 abort an entry and discard hours of repair over a calibration invariant,
 while failing at read time costs only the report. The reader is the right
 place for that guard.

 @returns One record per chunk, ordered by chunk index

 @example
 ```ts
 const sliceCritics = buildSliceCriticRecords({ outcomes, },);
 ```
 */
export function buildSliceCriticRecords(
  {
    outcomes,
  }: {
    readonly outcomes: readonly SliceCriticRecord[];
  },
): readonly SliceCriticRecord[] {
  return outcomes
    .map(function toRecord(outcome,): SliceCriticRecord {
      return {
        sliceIndex: outcome.sliceIndex,
      // CANONICALIZED here rather than trusted from the caller. Every producer
      // upstream already sorts, so this changes nothing today; what it adds is
      // that the ARTIFACT BOUNDARY guarantees the order rather than inheriting
      // it. A permuted array serializes to different bytes for identical
      // evidence, and this value goes into a cached outcome.
      //
      // Code-point order throughout (`compareCodePoints`), never
      // `localeCompare`, which is locale-dependent and would order the same
      // critics differently on two machines.
        heardCriticIds: outcome.heardCriticIds
          .toSorted(function byCriticId(
            left,
            right,
          ): number {
            return compareCodePoints({
              left,
              right,
            },);
          },),
        claimAttributions: outcome.claimAttributions
          .map(function canonical(attribution,): ClaimAttribution {
            return {
              claimId: attribution.claimId,
              proposers: attribution.proposers
                .toSorted(function byModelId(
                  left,
                  right,
                ): number {
                  return compareCodePoints({
                    left: left.modelId,
                    right: right.modelId,
                  },);
                },),
            };
          },)
          .toSorted(function byClaimId(
            left,
            right,
          ): number {
            return compareCodePoints({
              left: left.claimId,
              right: right.claimId,
            },);
          },),
      };
    },)
    .toSorted(function bySliceIndex(
      left,
      right,
    ): number {
      return left.sliceIndex - right.sliceIndex;
    },);
}

//endregion Critic attribution
