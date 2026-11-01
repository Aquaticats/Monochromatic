import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Attribution report
// Reads what critic attribution recorded and turns it into rates. Built
// alongside the writer deliberately: this pipeline's recurring failure is
// telemetry that is recorded and never read, and a data path with no reader is
// indistinguishable from one that was never built.
//
// Every count here is restricted to ELIGIBLE entries, meaning artifacts that
// carry `sliceCritics` at all. Entries settled before attribution existed carry
// none, and averaging over both populations silently mixes "this critic raised
// nothing" with "this entry could not have recorded that it did".

/**
 One critic behind one claim, as far as the report needs to read it.

 @example
 ```ts
 const proposer: ProposerView = { modelId: 'hf:openai/gpt-oss-120b', emissionCount: 1, };
 ```
 */
export type ProposerView = {
  /**
   Critic that proposed the claim.
   */
  readonly modelId: string;

  /**
   Times it emitted the claim within its own report.
   */
  readonly emissionCount: number;
};

/**
 Bumps one critic's counter.

 @param counter - counter to bump

 @param modelId - critic to credit

 @param by - amount to add

 @example
 ```ts
 bump({
          counter: heard,
          modelId,
          by: 1,
        },);
 ```
 */
function bump(
  {
    counter,
    modelId,
    by,
  }: {
    readonly counter: Map<string, number>;
    readonly modelId: string;
    readonly by: number;
  },
): void {
  counter.set(
    modelId,
    (counter.get(modelId,) ?? 0) + by,
  );
}

/**
 One chunk's calibration, as far as the report needs to read it.

 Structurally what `SliceCriticRecord` is, with model ids widened to plain
 strings. Stated independently so the reader can PARSE an artifact into this
 without asserting unknown strings into the model-id union, and a real
 `SliceCriticRecord` still satisfies it.

 @example
 ```ts
 const view: SliceCriticView = { sliceIndex: 0, heardCriticIds: [], claimAttributions: [], };
 ```
 */
export type SliceCriticView = {
  /**
   Chunk position within the document.
   */
  readonly sliceIndex: number;

  /**
   Critics that answered on this chunk.
   */
  readonly heardCriticIds: readonly string[];

  /**
   Which critics raised each surviving claim.
   */
  readonly claimAttributions: readonly {
    /**
     Deterministic identity of the claim.
     */
    readonly claimId: string;

    /**
     Critics that proposed it.
     */
    readonly proposers: readonly ProposerView[];
  }[];
};

/**
 One accepted issue, as far as attribution needs to read it.

 @example
 ```ts
 const record: AcceptedIssueView = { status: 'accepted', claimIds: ['issue/ab',], };
 ```
 */
export type AcceptedIssueView = {
  /**
   Adjudication status; only `accepted` counts toward hits.
   */
  readonly status: string;

  /**
   Deterministic ids of the claims this issue represents.
   */
  readonly claimIds: readonly string[];
};

/**
 One settled entry, as far as attribution needs to read it.

 @example
 ```ts
 const entry: AttributionEntry = { id: 'Acheron', sliceCritics, issues, };
 ```
 */
export type AttributionEntry = {
  /**
   Corpus entry identifier.
   */
  readonly id: string;

  /**
   Per-chunk calibration, absent on entries settled before attribution
   existed.
   */
  readonly sliceCritics?: readonly SliceCriticView[];

  /**
   Adjudicated issues of this entry.
   */
  readonly issues: readonly AcceptedIssueView[];
};

/**
 An entry the eligibility filter has narrowed to, carrying `sliceCritics` as a
 required field rather than an optional one.

 NAMED RATHER THAN INLINE, so the eligibility filter in
 `buildAttributionReport` can state it as a type guard.
 `Array.prototype.filter` narrows its result to this type only when the
 predicate itself reads `entry is EligibleAttributionEntry`, and a predicate
 typed to return plain `boolean` would leave every call downstream reading
 `entry.sliceCritics` as possibly absent all over again.

 @example
 ```ts
 const eligible: readonly EligibleAttributionEntry[] = entries.filter(carriesAttribution,);
 ```
 */
type EligibleAttributionEntry = AttributionEntry & {
  /**
   Per-chunk calibration, always present once an entry is eligible.
   */
  readonly sliceCritics: readonly SliceCriticView[];
};

/**
 What one critic did across the eligible population.

 @example
 ```ts
 const tally: CriticTally = { modelId, chunksHeard: 40, claimsRaised: 12, emissions: 14, acceptedHits: 9, };
 ```
 */
export type CriticTally = {
  /**
   Critic this row describes.
   */
  readonly modelId: string;

  /**
   Chunks where this critic answered; the DENOMINATOR every rate divides by.
   */
  readonly chunksHeard: number;

  /**
   Distinct claims this critic proposed.
   */
  readonly claimsRaised: number;

  /**
   Claims proposed counting repeats within one report, so
   `emissions - claimsRaised` is how often this critic said a thing twice.
   */
  readonly emissions: number;

  /**
   Accepted issues at least one of whose claims this critic proposed.
   */
  readonly acceptedHits: number;
};

/**
 Everything the attribution reader can say about a run.

 @example
 ```ts
 const report = buildAttributionReport({ entries, },);
 ```
 */
export type AttributionReport = {
  /**
   Entries carrying attribution, which every other count in this report is
   restricted to.
   */
  readonly eligibleEntries: number;

  /**
   Entries settled before attribution existed; excluded, never counted as
   silence.
   */
  readonly ineligibleEntries: number;

  /**
   Chunks across eligible entries.
   */
  readonly chunks: number;

  /**
   Per-critic rows, ordered by model id.
   */
  readonly critics: readonly CriticTally[];

  /**
   Accepted issues whose claims came from exactly one critic.
   */
  readonly soleProposerAccepted: number;

  /**
   Accepted issues whose claims drew more than one critic, which is
   independent support rather than one voice repeating.
   */
  readonly multiProposerAccepted: number;

  /**
   Accepted issues where some critic emitted the same claim more than once,
   which is self-repetition and must not read as agreement.
   */
  readonly selfRepeatedAccepted: number;

  /**
   Accepted issues carrying no attribution at all, which on an eligible entry
   means a claim id the index does not hold and is a defect worth surfacing
   rather than a zero worth averaging.
   */
  readonly unattributedAccepted: number;

  /**
   Accepted issues where SOME member claims attributed and others did not.

   Excluded from every other count here. A partial join is a defect rather
   than a measurement: the unattributed member may have come from a critic
   that gets no credit, so calling the issue sole-proposer would be a guess
   dressed as a count.
   */
  readonly partialJoinAccepted: number;
};

/**
 Indexes an entry's attributions by claim id.

 @param sliceCritics - per-chunk calibration records

 @returns Claim id to proposer list

 @example
 ```ts
 const index = indexProposers({ sliceCritics, },);
 ```
 */
function indexProposers(
  {
    sliceCritics,
  }: {
    readonly sliceCritics: readonly SliceCriticView[];
  },
): Map<string, readonly ProposerView[]> {
  /**
   Proposers per claim across every chunk of one entry.
   */
  const index = new Map<string, ProposerView[]>();
  for (const record of sliceCritics) {
    for (const attribution of record.claimAttributions) {
      // MERGED rather than overwritten. Two chunks can carry the same claim id,
      // and the writer deliberately keeps their proposers apart so neither
      // chunk inflates the other; overwriting here would make the last chunk
      // win and silently DELETE the earlier chunk's critics from every rate.
      // Deflating is no more correct than inflating. Appended in place, so a
      // claim's list is not copied at each chunk that names it (ledger B74).
      /**
       Proposers earlier chunks gave this claim, which this chunk's proposers join.
       */
      const merged = index.get(attribution.claimId,);
      if (merged === undefined) {
        index.set(
          attribution.claimId,
          [...attribution.proposers,],
        );
      }
      else
        merged.push(...attribution.proposers,);
    }
  }
  return index;
}

/**
 What one accepted issue rested on.

 Separated from the counting so each count is a `filter` over a fact already
 established, rather than a counter mutated in a loop that also decides the
 fact.

 @example
 ```ts
 const support: IssueSupport = { contributors: ['hf:openai/gpt-oss-120b',], repeated: false, };
 ```
 */
type IssueSupport = {
  /**
   Distinct critics behind any claim the issue represents.
   */
  readonly contributors: readonly string[];

  /**
   Claims of this issue the attribution index does not hold.

   Zero for a sound join. A count between zero and the issue's claim total is
   a PARTIAL join, where some support is known and some missing, and such an
   issue cannot honestly be called sole-proposer or multi-proposer: the
   missing member may have been raised by a critic nobody credited.
   */
  readonly unresolvedClaims: number;

  /**
   Whether some critic emitted one of those claims more than once, which is
   self-repetition and must never read as agreement.
   */
  readonly repeated: boolean;
};

/**
 Reads what one accepted issue rested on.

 @param issue - accepted issue

 @param proposersOf - proposers by claim id, for the issue's own entry

 @returns Support behind it

 @example
 ```ts
 const support = readIssueSupport({ issue, proposersOf, },);
 ```
 */
function readIssueSupport(
  {
    issue,
    proposersOf,
  }: {
    readonly issue: AcceptedIssueView;
    readonly proposersOf: Map<string, readonly ProposerView[]>;
  },
): IssueSupport {
  /**
   Every proposer behind any claim this issue represents.
   */
  const proposers = issue.claimIds
    .flatMap(function toProposers(claimId,): readonly ProposerView[] {
      return proposersOf.get(claimId,) ?? [];
    },);

  return {
    contributors: [...new Set(proposers.map(function toModelId(proposer,): string {
      return proposer.modelId;
    },),),],
    unresolvedClaims: issue.claimIds
      .filter(function isUnresolved(claimId,): boolean {
        return !proposersOf.has(claimId,);
      },)
      .length,
    repeated: proposers.some(function isRepeat(proposer,): boolean {
      return proposer.emissionCount > 1;
    },),
  };
}

/**
 Turns recorded attribution into per-critic rates and support counts.

 Restricted to entries carrying `sliceCritics`, because an entry settled
 before attribution existed records no proposer for a claim its critics did
 raise, and counting it would understate every critic at once.

 @param entries - settled entries in any order

 @returns Report over the eligible population only

 @example
 ```ts
 const report = buildAttributionReport({ entries, },);
 ```
 */
export function buildAttributionReport(
  {
    entries,
  }: {
    readonly entries: readonly AttributionEntry[];
  },
): AttributionReport {
  /**
   Entries that could record attribution at all.

   TYPED AS A GUARD rather than a plain predicate, so `eligible` carries
   `sliceCritics` as a required field for the rest of this function: every
   `?? []` that follows would otherwise be load-bearing rather than defensive,
   masking an entry this filter already proved cannot occur.
   */
  const eligible = entries.filter(function carriesAttribution(
    entry,
  ): entry is EligibleAttributionEntry {
    return entry.sliceCritics !== undefined;
  },);

  /**
   Chunks where each critic answered.
   */
  const heard = new Map<string, number>();

  /**
   Distinct claims each critic proposed.
   */
  const raised = new Map<string, number>();

  /**
   Claims each critic proposed, repeats included.
   */
  const emitted = new Map<string, number>();

  /**
   Accepted issues each critic contributed a claim to.
   */
  const hits = new Map<string, number>();

  /**
   Chunks across the eligible population, the denominator every rate divides
   by.
   */
  const chunks = eligible.reduce(
    function addChunks(
      total,
      { sliceCritics, },
    ): number {
      return total + sliceCritics.length;
    },
    0,
  );

  for (const entry of eligible) {
    for (const record of entry.sliceCritics) {
      for (const modelId of record.heardCriticIds)
        bump({
          counter: heard,
          modelId,
          by: 1,
        },);
      for (const attribution of record.claimAttributions) {
        for (const proposer of attribution.proposers) {
          bump({
            counter: raised,
            modelId: proposer.modelId,
            by: 1,
          },);
          bump({
            counter: emitted,
            modelId: proposer.modelId,
            by: proposer.emissionCount,
          },);
        }
      }
    }
  }

  /**
   Support behind every accepted issue of the eligible population.
   */
  const supports = eligible.flatMap(function toSupports(entry,): readonly IssueSupport[] {
    /**
     Proposers of this entry's claims, by claim id.
     */
    const proposersOf = indexProposers({ sliceCritics: entry.sliceCritics, },);

    return entry.issues
      .filter(function isAccepted(issue,): boolean {
        return issue.status === 'accepted';
      },)
      .map(function toSupport(issue,): IssueSupport {
        return readIssueSupport({
          issue,
          proposersOf,
        },);
      },);
  },);

  /**
   Accepted issues whose join is PARTIAL: some claims attributed, some not.

   Held out of every other count in this report rather than counted anywhere. An issue like
   this is a defect in the join, not a datum about critics, and averaging it
   in would let a broken join read as a confident calibration.
   */
  const partialJoinAccepted = supports.filter(function isPartial(support,): boolean {
    return (support.unresolvedClaims > 0) && (support.contributors
      .length
      > 0);
  },)
    .length;

  /**
   Supports whose join is sound, which alone are calibration.
   */
  const sound = supports.filter(function isSound(support,): boolean {
    return (support.unresolvedClaims === 0) || (support.contributors
      .length
      === 0);
  },);

  for (const support of sound) {
    for (const modelId of support.contributors)
      bump({
        counter: hits,
        modelId,
        by: 1,
      },);
  }

  /**
   Accepted issues carrying no attribution at all.
   */
  const unattributedAccepted = sound.filter(function isUnattributed(support,): boolean {
    return support.contributors
      .length
      === 0;
  },)
    .length;

  /**
   Accepted issues resting on exactly one critic.
   */
  const soleProposerAccepted = sound.filter(function isSole(support,): boolean {
    return support.contributors
      .length
      === 1;
  },)
    .length;

  /**
   Accepted issues drawing more than one critic.
   */
  const multiProposerAccepted = sound.filter(function isMulti(support,): boolean {
    return support.contributors
      .length
      > 1;
  },)
    .length;

  /**
   Accepted issues where some critic repeated itself. Restricted to attributed
   issues, so it stays a subset of what the sole and multi counts cover.
   */
  const selfRepeatedAccepted = sound.filter(function isRepeated(support,): boolean {
    return support.repeated && (support.contributors
      .length
      > 0);
  },)
    .length;

  /**
   Every critic's own heard count, in code-point order of model id, and the
   one place `chunksHeard` is read: each row comes from one of these entries
   directly rather than from a separately built id list looked back up in
   `heard`.

   THE WHOLE ROSTER, because `attribution-decode.ts` refuses a claim
   attribution whose proposer its own chunk's `heardCriticIds` does not name:
   every model id `raised` or `hits` can hold was bumped into `heard` on that
   same chunk first. A critic heard but silent still gets a row, and no critic
   that raised a claim can be missing one.
   */
  const heardSorted = [...heard.entries(),].toSorted(function byModelId(
    [left,]: ReadonlyEntry<string, number>,
    [right,]: ReadonlyEntry<string, number>,
  ): number {
    return compareCodePoints({
      left,
      right,
    },);
  },);

  return {
    eligibleEntries: eligible.length,
    ineligibleEntries: entries.length - eligible.length,
    chunks,
    critics: heardSorted.map(function toTally([modelId, chunksHeard,]: ReadonlyEntry<string, number>,): CriticTally {
      return {
        modelId,
        chunksHeard,
        claimsRaised: raised.get(modelId,) ?? 0,
        emissions: emitted.get(modelId,) ?? 0,
        acceptedHits: hits.get(modelId,) ?? 0,
      };
    },),
    soleProposerAccepted,
    multiProposerAccepted,
    selfRepeatedAccepted,
    unattributedAccepted,
    partialJoinAccepted,
  };
}

//endregion Attribution report
