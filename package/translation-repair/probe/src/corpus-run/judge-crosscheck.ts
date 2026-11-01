import type {
  AdjudicationStatus,
  ReadonlyEntry,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { AttributionEntry, } from './attribution-report.ts';
import { seatJudges, } from './judge-independence.ts';

//region Judge crosscheck census
// Which claims a disinterested judge may re-examine, and what the reading is
// allowed to conclude from the answer.
//
// A CENSUS, NOT A SAMPLE. Every attributed claim in every attributed entry is
// enumerated, because the eligible population is small enough to judge whole
// and a sampler would add a seed, a bias question and a reproducibility burden
// for nothing. If the population outgrows the budget, cap it here and say so;
// do not quietly sample.
//
// TWO ARMS, and the control is the point. Judging only accepted claims yields
// an agreement rate with nothing to compare against: a roster that says
// `supported` to everything scores identically to one that reads carefully.
// The control arm re-asks the same judges about claims the panel did NOT
// accept. A crosscheck worth citing agrees with the panel more on the accepted
// arm than on the control arm, and the gap between the two is the finding.
// Equal rates mean the judges are not discriminating and the accepted-arm
// number means nothing on its own.

/**
 Which side of the crosscheck a claim sits on.

 `undecided` is separated from `control` rather than folded into it, and the
 distinction is the difference between a rate and an average over two
 incommensurable things. `rejected` means the panel DECIDED AGAINST the
 claim, so there is a verdict for a judge to agree or disagree with.
 `needs-human` means the panel DECLINED TO DECIDE, so agreement is undefined:
 there is no verdict to survive re-asking. Measured on this run those claims
 lean supported 228 to 23, so filing them as control would fill the control
 arm with claims the panel mostly believed.

 From ledger L5 on, `needs-human` also holds a claim a supported majority
 settled at neutral, the severity that asserts no defect, so there is no
 defect for a judge to confirm. Such claims leave the accepted arm for the
 undecided one, and on runs from `SLICE_CACHE_VERSION` 34 the accepted arm's
 precision reads higher by construction against earlier runs.

 `source-defect` sits in `control` because it IS a verdict, the panel ruling
 the original text wrong at the claimed spot rather than the translation.
 */
export type CrosscheckArm = 'accepted' | 'control' | 'undecided';

/**
 One claim queued for re-examination, with its seating already worked out.

 @example
 ```ts
 const item: CrosscheckItem = { entryId: 'Whiskers', claimId, arm: 'accepted', status: 'accepted', proposers, judges, barred, };
 ```
 */
export type CrosscheckItem = {
  /**
   Corpus entry this claim came from.
   */
  readonly entryId: string;

  /**
   Deterministic claim identity, the join key between attribution and issues.
   */
  readonly claimId: string;

  /**
   Which arm this claim serves.
   */
  readonly arm: CrosscheckArm;

  /**
   Adjudication status verbatim, kept beside {@link CrosscheckItem.arm} so a
   control-arm result can be broken down by reason without re-reading
   artifacts.
   */
  readonly status: string;

  /**
   Models that proposed this claim and therefore may not judge it.
   */
  readonly proposers: readonly string[];

  /**
   Models seated to judge it.
   */
  readonly judges: readonly RosterModelId[];

  /**
   Models barred as authors.
   */
  readonly barred: readonly RosterModelId[];
};

/**
 The enumerated population, with everything excluded from it counted.

 @example
 ```ts
 const census = buildCrosscheckCensus({ entries, roster, },);
 ```
 */
export type CrosscheckCensus = {
  /**
   Claims a disinterested judge may rule on, in entry then claim order.
   */
  readonly items: readonly CrosscheckItem[];

  /**
   Claims every roster model proposed, so nobody may judge them.

   Carried out rather than dropped. A claim the whole roster authored is the
   most corroborated claim in the run, and silently removing it would lift
   every rate by hiding the strongest agreement in the population.
   */
  readonly unjudgeable: readonly CrosscheckItem[];

  /**
   Claims on entries that predate attribution entirely.

   Expected, not a defect. An entry settled before attribution was recorded
   names claims no proposer was ever written for. Counted so a reading can
   see how much of the run the census covers.
   */
  readonly unattributedLegacyClaims: number;

  /**
   Claims on entries that DO carry attribution, yet whose id no attribution
   record holds.

   A DEFECT IN THE JOIN, and held apart from the legacy count for that
   reason. On an entry whose critics were attributed, every surviving claim
   should have a proposer; one that does not means the two records disagree
   about claim identity, and folding it in with the legacy claims would hide
   a broken join inside an expected number.
   */
  readonly unattributedJoinFailures: number;

  /**
   Entries carrying no attribution at all, settled before it was recorded.
   */
  readonly entriesWithoutAttribution: number;

  /**
   Entries the census draws from.
   */
  readonly entriesCovered: number;
};

/**
 The arm each adjudication status sits in.

 A table rather than a chain, so adding a status is a data edit. BUILT FROM A
 RECORD KEYED BY `AdjudicationStatus`, so a status the panel gains cannot be
 left without an arm: the compiler refuses the record until it has one. A
 status a stored artifact carries that is not one of these is not in the map,
 which `armOf` answers as `undecided`.
 A map rather than a plain object, since the status is the artifact's text:
 a plain object answered a status spelled `constructor` or `__proto__` with
 what every object inherits, and filed the claim under that as an arm
 (ledger B77).
 */
const ARM_OF_STATUS: ReadonlyMap<string, CrosscheckArm> = new Map(Object.entries({
  accepted: 'accepted',
  rejected: 'control',
  'source-defect': 'control',
  'needs-human': 'undecided',
} satisfies Readonly<Record<AdjudicationStatus, CrosscheckArm>>,),);

/**
 Places one claim in an arm by the verdict its issue carries.

 An unrecognized status lands in `undecided` rather than `control`, which is
 the conservative direction: a status this code has never seen is one whose
 meaning it cannot assert, and `undecided` is reported apart from every rate
 instead of quietly becoming a denominator.

 @param status - adjudication status verbatim from the artifact

 @returns Arm the claim belongs to

 @example
 ```ts
 const arm = armOf({ status: 'rejected', },);
 ```
 */
function armOf({ status, }: { readonly status: string; },): CrosscheckArm {
  return ARM_OF_STATUS.get(status,) ?? 'undecided';
}

/**
 Enumerates every claim a disinterested judge may re-examine, both arms.

 @param entries - settled entries as the attribution reader returns them

 @param roster - models available to judge

 @returns Census with both arms populated and every exclusion counted

 @example
 ```ts
 const { items, unjudgeable, } = buildCrosscheckCensus({ entries, roster: RUN_ROSTER, },);
 ```
 */
export function buildCrosscheckCensus(
  {
    entries,
    roster,
  }: {
    readonly entries: readonly AttributionEntry[];
    readonly roster: readonly RosterModelId[];
  },
): CrosscheckCensus {
  /**
   Every claim of every entry, paired with its seating.
   */
  const enumerated = entries.flatMap(function toItems(entry,): readonly CrosscheckItem[] {
    /**
     Proposers of each attributed claim in this entry, across every chunk.

     MERGED, each model once, in the order first met. Two chunks can carry one
     claim id, and the writer keeps their proposers apart
     (`critic-attribution.ts`), so nothing makes a claim id unique across an
     entry's chunks: setting the claim's list at each chunk let the last
     chunk replace the others, which seated an earlier chunk's author as a
     judge of its own claim. A set, since the seating asks who authored the
     claim and never how often.
     */
    const proposersOf = new Map<string, Set<string>>();
    for (const chunk of entry.sliceCritics ?? []) {
      for (const attribution of chunk.claimAttributions) {
        /**
         Authors earlier chunks gave this claim, which this chunk's authors join.
         */
        const merged = proposersOf.get(attribution.claimId,) ?? new Set<string>();
        for (const proposer of attribution.proposers)
          merged.add(proposer.modelId,);
        proposersOf.set(
          attribution.claimId,
          merged,
        );
      }
    }

    return entry.issues
      .flatMap(function toClaims(issue,): readonly CrosscheckItem[] {
        return issue.claimIds
          .filter(function isAttributed(claimId,): boolean {
            return proposersOf.has(claimId,);
          },)
          .map(function toItem(claimId,): CrosscheckItem {
          /**
           Set of authors of this claim, known present by `isAttributed`.
           */
            const authors = proposersOf.get(claimId,);
            if (authors === undefined)
              throw new Error(
                'unreachable: a claim `isAttributed` kept has no authors in the map it asked, which '
                  + 'holds a claim id for every attribution it read',
              );

          /**
           Authors of this claim, in the order first met.
           */
            const proposers = [...authors,];

          /**
           Who may re-examine this claim and who authored it.
           */
            const {
              judges,
              barred,
            } = seatJudges({
              proposers,
              roster,
            },);

            return {
              entryId: entry.id,
              claimId,
              arm: armOf({ status: issue.status, },),
              status: issue.status,
              proposers,
              judges,
              barred,
            };
          },);
      },);
  },);

  /**
   Claims an issue names that no attribution record covers, split by whether
   the entry carries attribution at all.
   */
  const unattributed = entries.reduce(
    function countMissing(
      total: {
        readonly legacy: number;
        readonly joinFailures: number;
      },
      entry,
    ) {
      /**
       Claim ids this entry attributed.
       */
      const attributed = new Set(
        (entry.sliceCritics ?? []).flatMap(function toIds(chunk,): readonly string[] {
          return chunk.claimAttributions
            .map(function toId(attribution,): string {
              return attribution.claimId;
            },);
        },),
      );

      /**
       Claims of this entry with no proposer recorded.
       */
      const missing = entry.issues
        .reduce(
        function countEntryMissing(
          running: number,
          { claimIds, },
        ): number {
            /**
             Claims of this issue with no proposer recorded.
             */
          const unrecorded = claimIds
            .filter(function isMissing(claimId,): boolean {
              return !attributed.has(claimId,);
            },)
            .length;

          return running + unrecorded;
        },
        0,
        );

      // An entry attributing nothing is legacy; one attributing something and
      // still missing a claim has a join that disagrees with itself.
      return (attributed.size === 0)
        ? {
          legacy: total.legacy + missing,
          joinFailures: total.joinFailures,
        }
        : {
          legacy: total.legacy,
          joinFailures: total.joinFailures + missing,
        };
    },
    {
      legacy: 0,
      joinFailures: 0,
    },
  );

  return {
    items: enumerated
      .filter(function isJudgeable({ judges, },): boolean {
        return judges.length > 0;
      },),
    unjudgeable: enumerated
      .filter(function isUnjudgeable({ judges, },): boolean {
        return judges.length === 0;
      },),
    unattributedLegacyClaims: unattributed.legacy,
    unattributedJoinFailures: unattributed.joinFailures,
    entriesWithoutAttribution: entries
      .filter(function isBare(entry,): boolean {
        return (entry.sliceCritics ?? [])
          .every(function empty({ claimAttributions, },): boolean {
            return claimAttributions.length === 0;
          },);
      },)
      .length,
    entriesCovered: entries.length,
  };
}

/**
 Claims counted by the status their issue carries, as the crosscheck prints
 them beside the arms.

 @param claims - claims outside the accepted arm, each with its issue's status
 verbatim from the artifact

 @returns Each status with its count, `status=count`, in the order first met,
 joined by spaces; empty when there are no claims

 @example
 ```ts
 const line = statusBreakdown({ claims: [...control, ...undecided,], },); // 'rejected=2 needs-human=1'
 ```
 */
export function statusBreakdown(
  { claims, }: { readonly claims: readonly { readonly status: string; }[]; },
): string {
  /**
   Count per status, in the order first met; a map, since the statuses are
   the artifact's text (ledger B77).
   */
  const byStatus = new Map<string, number>();
  for (const { status, } of claims) {
    byStatus.set(
      status,
      (byStatus.get(status,) ?? 0) + 1,
    );
  }
  return [...byStatus,]
    .map(function toPair([status, count,]: ReadonlyEntry<string, number>,): string {
      return `${status}=${String(count,)}`;
    },)
    .join(' ',);
}

//endregion Judge crosscheck census
