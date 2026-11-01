import {
  type AdjudicatedIssue,
  type AdjudicationConfig,
  type AdjudicationStatus,
  type AggregatedClaim,
  type ClaimCluster,
  type ClaimPanelReading,
  DEFAULT_ADJUDICATION_CONFIG,
  type IssueSeverity,
  type PanelBallot,
  type ReadonlyEntry,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { panelReadingForClaim, } from './tally-claim.ts';
import {
  assembleGradedIssue,
  type GradedMember,
  partitionGradedMembers,
  severityUpperMedian,
} from './tally-issue.ts';

//region Vote tally
// Pure aggregation of panel ballots into adjudicated issues. Decision rules,
// in order per claim: fewer than two answering panelists lands needs-human; a
// protective source-defect minority blocks; a strict majority accepts or
// rejects, save that an acceptance settled at neutral severity, which asserts
// no defect, is held for a human (ledger L5); everything else needs a human.
// Merge disposition is majority same-defect weight among opining panelists,
// defaulting to distinct.
//
// THERE IS NO WEIGHT FLOOR, BY THE OWNER'S RULINGS OF 2026-10-25: "floor of 3 -
// don't set a floor." (about the minimum ballot weight of three this tally
// used to hold) and, on a claim only one judge answers, "Two ballots decide
// (Recommended)". What is left is a count, not a weight: two answering
// panelists decide, one alone never does (the older standing rule of the
// owner), and a heavy weight does not turn one answer into two. In the corpus
// pass XingZ6021 two panelists supported the claims on slices 67 to 69 at the
// default weight of one each; the electorate weight 2 sat under the floor of 3
// and a unanimous panel decided nothing.

/**
 Answering panelists a claim needs before the tally decides it: the floor of
 two ballots, the same count every other decision in the package stops at.
 */
export const MIN_PANEL_BALLOTS = 2;

/**
 Everything the panel decided over one chunk's clusters.

 @example
 ```ts
 const result: AdjudicationResult = tallyVotes({ clusters, ballots, },);
 ```
 */
export type AdjudicationResult = {
  /**
   Issues in cluster document order;
   unmerged clusters yield one issue per member claim.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   Original cluster and resulting issue identities when a same-defect merge
   is partitioned by member verdict, retaining its audit relationship.
   */
  readonly findings: readonly string[];
};

/**
 Status one tally decides under the config thresholds.

 @param reading - ballots and the weighted counts they sum to for one claim

 @param config - thresholds in force

 @returns Fate of the claim

 @example
 ```ts
 const status = decideStatus({ reading, config, },);
 ```
 */
function decideStatus(
  {
    reading,
    config,
  }: {
    readonly reading: ClaimPanelReading;
    readonly config: AdjudicationConfig;
  },
): AdjudicationStatus {
  /**
   Weighted counts for this claim.
   */
  const { tally, } = reading;
  /**
   Panelists that answered this claim with a vote that carries weight: a
   missing verdict abstains, and a panelist configured to weigh nothing is
   not a voice. The count, not the weight, is what one judge alone cannot
   exceed.
   */
  const answering = reading.ballots
    .filter(function answered(ballot,): boolean {
      return (ballot.vote !== 'abstain') && (ballot.weight > 0);
    },)
    .length;
  if (answering < MIN_PANEL_BALLOTS)
    return 'needs-human';
  /**
   Non-abstain weight forming the electorate for this claim.
   */
  const electorate = tally.supported + tally.unsupported
    + tally.ambiguous
    + tally.sourceDefect;
  if ((tally.sourceDefect / electorate) >= config.sourceDefectThreshold)
    return 'source-defect';
  if ((tally.supported / electorate) > config.decisionThreshold)
    return 'accepted';
  if ((tally.unsupported / electorate) > config.decisionThreshold)
    return 'rejected';
  return 'needs-human';
}

/**
 Status of a claim once its settled severity is known: an acceptance at
 neutral is held for a human, because neutral is the severity that asserts no
 defect (`issue-taxonomy.ts`), and an accepted issue is the editor's work
 list, its envelopes, its checkers' ballot and, for an addition, a dispute of
 the archive. LEDGER L5: over every run, 41 accepted issues had settled at
 neutral, 28 of them shipped an edit, and 8 were claims whose own summary
 called the rendering accurate or correct. A supporter who finds a real
 defect in a claim filed neutral re-grades it, and the upper median then
 leaves neutral, so a single such re-grade among supporters keeps it
 accepted.

 @param status - fate the votes decided

 @param severity - severity the claim settled at

 @returns Needs-human for an acceptance at neutral, else the voted fate

 @example
 ```ts
 heldIfNeutral({ status: 'accepted', severity: 'neutral', },); // 'needs-human'
 ```
 */
function heldIfNeutral(
  {
    status,
    severity,
  }: {
    readonly status: AdjudicationStatus;
    readonly severity: IssueSeverity;
  },
): AdjudicationStatus {
  if ((status === 'accepted') && (severity === 'neutral'))
    return 'needs-human';
  return status;
}

/**
 Final severity of one claim:
 upper median over the claimed severity plus supported ballots' re-grades.

 @param member - claim under grading

 @param ballots - resolved ballots keyed by panelist id

 @returns Final severity for the issue record

 @example
 ```ts
 const severity = finalSeverity({ member, ballots, },);
 ```
 */
function finalSeverity(
  {
    member,
    ballots,
  }: {
    readonly member: AggregatedClaim;
    readonly ballots: Readonly<Record<string, PanelBallot>>;
  },
): IssueSeverity {
  /**
   Re-grades from panelists who supported this claim and offered one.
   */
  const regrades = Object
    .values(ballots,)
    .flatMap(function toRegrade(ballot,): readonly IssueSeverity[] {
      /**
       This panelist's verdict on the claim, when cast.
       */
      const verdict = ballot.verdicts[member.claimId];
      if ((verdict === undefined) || (verdict.vote !== 'supported')
        || (verdict.severity === undefined))
        return [];
      return [verdict.severity,];
    },);

  return severityUpperMedian({ severities: [
    member.claim
      .severity,
    ...regrades,
  ], },);
}

/**
 Whether one multi-member cluster merges:
 same-defect weight must strictly exceed distinct weight among opining
 panelists; silence and ties keep claims distinct because a wrong merge
 hides a defect while a wrong split only duplicates work.

 @param cluster - cluster under disposition

 @param ballots - resolved ballots keyed by panelist id

 @param config - weight table

 @returns Whether members become one issue

 @example
 ```ts
 const merged = disposeMerge({ cluster, ballots, config, },);
 ```
 */
function disposeMerge(
  {
    cluster,
    ballots,
    config,
  }: {
    readonly cluster: ClaimCluster;
    readonly ballots: Readonly<Record<string, PanelBallot>>;
    readonly config: AdjudicationConfig;
  },
): boolean {
  if (cluster.members
    .length
    < 2)
    return false;

  /**
   Weighted same-defect and distinct opinion masses.
   */
  const {
    same,
    distinct,
  } = Object
    .entries(ballots,)
    .reduce(
    function addOpinion(
      masses: {
        readonly same: number;
        readonly distinct: number
      },
      [panelistId, ballot,]: ReadonlyEntry<string, PanelBallot>,
    ) {
        /**
         This panelist's opinion on the cluster, when given.
         */
      const opinion = ballot.mergeOpinions[cluster.clusterId];
      if (opinion === undefined)
        return masses;

        /**
         Weight of this panelist's opinion.
         */
      const weight = config.weights?.[panelistId] ?? 1;
      return opinion
        ? {
          same: masses.same + weight,
          distinct: masses.distinct,
        }
        : {
          same: masses.same,
          distinct: masses.distinct + weight,
        };
    },
    {
      same: 0,
      distinct: 0,
    },
    );

  return same > distinct;
}

/**
 Aggregates panel ballots over one chunk's clusters into adjudicated
 issues. Pure: same clusters, ballots, and config always produce the same
 issues, so checkpoints can replay adjudication without model calls.

 @param clusters - aggregation output, in document order

 @param ballots - resolved ballots keyed by panelist id; the shell owns
   panelist identity, claims never carry it

 @param configuredPanelists - panelists the run seated, heard or not, which
   the ballots cannot say: a lost voice leaves no entry while an abstention
   leaves one, and those are different evidence

 @param config - thresholds and weights; defaults to
   {@link DEFAULT_ADJUDICATION_CONFIG}

 @returns Adjudicated issues in cluster document order

 @example
 ```ts
 const { issues, } = tallyVotes({ clusters, ballots, configuredPanelists: 6, },);
 ```
 */
export function tallyVotes(
  {
    clusters,
    ballots,
    configuredPanelists,
    config = DEFAULT_ADJUDICATION_CONFIG,
  }: {
    readonly clusters: readonly ClaimCluster[];
    readonly ballots: Readonly<Record<string, PanelBallot>>;
    readonly configuredPanelists: number;
    readonly config?: AdjudicationConfig;
  },
): AdjudicationResult {
  /**
   Decisions and partition lineage accumulated in cluster document order.
   */
  const outcomes = clusters.map(function adjudicateCluster(cluster,): AdjudicationResult {
    /**
     Per-member tallies, statuses, and severities in member order.
     */
    const graded = cluster.members
      .map(function gradeMember(member,): GradedMember {
      /**
       Every ballot on this member claim, and the tally they sum to.
       */
        const reading = panelReadingForClaim({
          claimId: member.claimId,
          ballots,
          configuredPanelists,
          config,
        },);

      /**
       Severity the claim settles at.
       */
        const severity = finalSeverity({
          member,
          ballots,
        },);

        return {
          member,
          tally: reading.tally,
          reading,
          status: heldIfNeutral({
            status: decideStatus({
              reading,
              config,
            },),
            severity,
          },),
          severity,
        };
      },);

    /**
     Members a supported majority would have accepted at neutral, which the
     tally held for a human instead.
     */
    const held = graded
      .filter(function isHeldNeutral(entry,): boolean {
        return (entry.severity === 'neutral')
          && (decideStatus({
            reading: entry.reading,
            config,
          },) === 'accepted');
      },)
      .map(function heldFinding(entry,): string {
        return `neutral-held-for-human (${entry.member
          .claimId})`;
      },);

    /**
     Whether the panel relates these diagnoses as one defect.
     */
    const merged = disposeMerge({
      cluster,
      ballots,
      config,
    },);
    /**
     Related diagnoses still keep their separately decided authority.
     */
    const groups = merged
      ? partitionGradedMembers({ graded, },)
      : graded.map(function solo(entry,): readonly GradedMember[] { return [entry,]; },);
    /**
     Each record carries only its own members and evidence.
     */
    const issues = groups.map(function assemble(group,): AdjudicatedIssue {
      return assembleGradedIssue({ graded: group, },);
    },);
    return {
      issues,
      findings: [
        ...held,
        ...(merged && (groups.length > 1)
          ? [`issue-merge-partitioned (${cluster.clusterId}: ${issues.map(function partition(issue,): string {
            return `${issue.issueId}=${issue.status}`;
          },)
            .join(', ',)})`,]
          : []),
      ],
    };
  },);

  return {
    issues: outcomes.flatMap(function issuesOf(outcome,): readonly AdjudicatedIssue[] {
      return outcome.issues;
    },),
    findings: outcomes.flatMap(function findingsOf(outcome,): readonly string[] {
      return outcome.findings;
    },),
  };
}

//endregion Vote tally
