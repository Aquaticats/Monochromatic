import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type AdjudicatedIssue,
  type AdjudicationStatus,
  type AggregatedClaim,
  type ClaimPanelReading,
  ISSUE_SEVERITIES,
  type IssueSeverity,
  type VoteTally,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Graded issue assembly
// Authority is decided before grouping. A same-defect vote relates diagnoses;
// it cannot promote a rejected or unresolved diagnosis into repair authority.

/**
 One member after voting under the caller's actual thresholds and weights.

 @example
 ```ts
 const grades: readonly GradedMember[] = [];
 ```
 */
export type GradedMember = {
  /**
   Original atomic claim retained for attribution.
   */
  readonly member: AggregatedClaim;
  /**
   Weighted decisions belonging only to this claim.
   */
  readonly tally: VoteTally;
  /**
   Ballots and configured electorate behind the tally.
   */
  readonly reading: ClaimPanelReading;
  /**
   Effective decision, never re-derived under default thresholds.
   */
  readonly status: AdjudicationStatus;
  /**
   Effective severity after supported re-grades.
   */
  readonly severity: IssueSeverity;
};

/**
 Upper median under the taxonomy's least-to-most severity order.
 Even splits retain the more severe grade, matching existing adjudication.

 @param severities - nonempty severity opinions

 @returns Upper-median grade

 @example
 ```ts
 severityUpperMedian({ severities: ['minor', 'major'] });
 ```
 */
export function severityUpperMedian(
  { severities, }: { readonly severities: readonly IssueSeverity[]; },
): IssueSeverity {
  /**
   Opinions ordered for median selection.
   */
  const sorted = [...severities,].toSorted(function bySeverityRank(
    left,
    right,
  ): number {
    return ISSUE_SEVERITIES.indexOf(left,) - ISSUE_SEVERITIES.indexOf(right,);
  },);
  return nonNullishOrThrow(sorted[Math.floor(sorted.length / 2,)],);
}

/**
 Partitions a panel-approved merge by already-decided member status.
 First occurrence determines partition order. A source-defect status keeps
 the entire merged cluster blocked, preserving protective-minority policy.

 @param graded - members in stable cluster order

 @returns Uniform-status groups, or one protected heterogeneous group

 @example
 ```ts
 const groups = partitionGradedMembers({ graded });
 ```
 */
export function partitionGradedMembers(
  { graded, }: { readonly graded: readonly GradedMember[]; },
): readonly (readonly GradedMember[])[] {
  if (graded.some(function sourceBlocked(entry,): boolean {
    return entry.status === 'source-defect';
  },))
    return [graded,];
  /**
   Status order follows first occurrence rather than a new priority rule.
   */
  const statuses = [...new Set(graded.map(function statusOf(entry,): AdjudicationStatus {
    return entry.status;
  },),),];
  return statuses.map(function membersWithStatus(status,): readonly GradedMember[] {
    return graded.filter(function matches(entry,): boolean {
      return entry.status === status;
    },);
  },);
}

/**
 Assembles one nonempty partition produced by {@link partitionGradedMembers},
 or one unmerged member. Only source-defect-protected groups mix statuses.
 Per-member evidence keysets exactly follow the retained claims.

 @param graded - uniform-status or source-defect-protected group

 @returns Issue whose repair authority cannot exceed its member decisions

 @example
 ```ts
 const issue = assembleGradedIssue({ graded: group });
 ```
 */
export function assembleGradedIssue(
  { graded, }: { readonly graded: readonly GradedMember[]; },
): AdjudicatedIssue {
  /**
   Accepted members still carry severity in a source-defect-blocked group.
   */
  const severityCarriers = graded.filter(function accepted(entry,): boolean {
    return entry.status === 'accepted';
  },);
  /**
   Membership determines identity exactly as before partitioning existed.
   */
  const ids = textsInCodePointOrder({ texts: graded.map(function claimId(entry,): string {
    return entry.member
      .claimId;
  },), },);
  return {
    issueId: `adjudicated/${hashContent({ content: JSON.stringify(ids,), },)}`,
    status: graded.some(function blocked(entry,): boolean {
      return entry.status === 'source-defect';
    },) ? 'source-defect' : nonNullishOrThrow(graded[0],)
      .status,
    severity: severityUpperMedian({
      severities: (severityCarriers.length > 0 ? severityCarriers : graded)
        .map(function severityOf(entry,): IssueSeverity {
          return entry.severity;
        },),
    },),
    claims: graded.map(function claimOf(entry,): AggregatedClaim {
      return entry.member;
    },),
    tallies: Object.fromEntries(graded.map(function tallyOf(entry,): readonly [
      string,
      VoteTally
    ] {
      return [
        entry.member
          .claimId,
        entry.tally,
      ];
    },),),
    readings: Object.fromEntries(graded.map(function readingOf(entry,): readonly [
      string,
      ClaimPanelReading
    ] {
      return [
        entry.member
          .claimId,
        entry.reading,
      ];
    },),),
  };
}

//endregion Graded issue assembly
