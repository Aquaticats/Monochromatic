import type { AggregatedClaim, } from './aggregate-claims.ts';
import type { IssueSeverity, } from './issue-taxonomy.ts';
import type { RosterModelId, } from './roster-id.ts';
// TYPE-ONLY AND DELIBERATELY CIRCULAR: `panel-reading.ts` names two types from
// here. Both directions are erased before anything runs, so the cycle costs
// nothing, and the alternative is a third file holding two type aliases.
import type { ClaimPanelReading, } from './panel-reading.ts';

//region Adjudication model
// Vote vocabulary and result types for the provenance-blind panel. Panelists
// judge each claim strictly on document evidence; they never learn which
// model proposed what, and the electorate is fixed up front (settled
// architecture: a variable electorate of non-proposers shrinks consensus).
// A claim's fate never depends on how many critics proposed it: the
// reference run's sole-proposer seed hit is the standing counterexample.

/**
 Every vote a panelist may cast on one claim, closed vocabulary.
 `source-defect` asserts the original text itself is wrong at the claimed
 spot, which must block "corrections" toward corruption;
 `abstain` withdraws from the electorate for that claim.

 @example
 ```ts
 PANEL_VOTE_STATES.includes('supported',);
 ```
 */
export const PANEL_VOTE_STATES = [
  'supported',
  'unsupported',
  'ambiguous',
  'source-defect',
  'abstain',
] as const;

/**
 One panelist's judgment of one claim.

 @example
 ```ts
 const vote: PanelVoteState = 'supported';
 ```
 */
export type PanelVoteState = typeof PANEL_VOTE_STATES[number];

/**
 Guards untrusted vote strings from model JSON.

 @param value - candidate from unvalidated model output

 @returns Whether value names one listed vote state

 @example
 ```ts
 isPanelVoteState('supported',);
 ```
 */
export function isPanelVoteState(value: unknown,): value is PanelVoteState {
  if ((typeof value) !== 'string')
    return false;

  return (PANEL_VOTE_STATES as readonly string[]).includes(value,);
}

/**
 One resolved verdict inside a ballot:
 the vote, the optional severity re-grade the panel may apply, and the
 reason given before the vote.

 @example
 ```ts
 const verdict: BallotVerdict = { vote: 'supported', severity: 'major', reason: 'The quote shows the wrong name.', };
 ```
 */
export type BallotVerdict = {
  /**
   Vote cast on this claim.
   */
  readonly vote: PanelVoteState;

  /**
   Re-graded severity, when the panelist disagrees with the claimed one.
   */
  readonly severity?: IssueSeverity;

  /**
   What the panelist said decides the claim, written before its vote (owner,
   2026-09-27, "Reason before vote"). Absent where it gave none, which the
   ballot's findings record, and on every ballot resolved before the rule.
   */
  readonly reason?: string;
};

/**
 One panelist's complete resolved ballot over one chunk's clusters.
 Missing claims count as abstentions at tally time, so a panelist who
 answers half the sheet weakens only its own influence.

 @example
 ```ts
 const ballot: PanelBallot = {
   verdicts: { 'issue/abc': { vote: 'supported', }, },
   mergeOpinions: { 'cluster/def': true, },
   findings: [],
 };
 ```
 */
export type PanelBallot = {
  /**
   Verdicts keyed by claim id.
   */
  readonly verdicts: Readonly<Record<string, BallotVerdict>>;

  /**
   Same-defect opinions keyed by cluster id,
   only meaningful for multi-member clusters.
   */
  readonly mergeOpinions: Readonly<Record<string, boolean>>;

  /**
   Wire irregularities in scorecard-stable wording
   (duplicate verdicts, out-of-range indices, unknown vote strings).
   */
  readonly findings: readonly string[];
};

/**
 Fate of one adjudicated issue.
 `source-defect` outranks acceptance because correcting toward a corrupted
 original is worse than leaving the translation alone.

 @example
 ```ts
 const status: AdjudicationStatus = 'accepted';
 ```
 */
export type AdjudicationStatus =
  | 'accepted'
    | 'rejected'
    | 'needs-human'
    | 'source-defect';

/**
 Weighted vote counts over one claim, kept on the issue for calibration
 and steering; weights default to one per panelist until canary
 calibration supplies better ones.

 @example
 ```ts
 const tally: VoteTally = {
   supported: 3, unsupported: 1, ambiguous: 0, sourceDefect: 0, abstain: 1,
 };
 ```
 */
export type VoteTally = {
  /**
   Weight behind supported votes.
   */
  readonly supported: number;

  /**
   Weight behind unsupported votes.
   */
  readonly unsupported: number;

  /**
   Weight behind ambiguous votes.
   */
  readonly ambiguous: number;

  /**
   Weight behind source-defect votes.
   */
  readonly sourceDefect: number;

  /**
   Weight behind abstentions, explicit or from missing verdicts.
   */
  readonly abstain: number;
};

/**
 One issue after the panel spoke.
 Panel-merged diagnoses are partitioned by their effective member status,
 so accepted issues contain only accepted claims. A source-defect decision
 still blocks the entire merged cluster, whose members may have mixed statuses.
 Per-claim tallies stay attached for calibration.

 @example
 ```ts
 const issue: AdjudicatedIssue = {
   issueId: 'adjudicated/abc',
   status: 'accepted',
   severity: 'major',
   claims: [member,],
   tallies: { 'issue/abc': tally, },
 };
 ```
 */
export type AdjudicatedIssue = {
  /**
   Initial deterministic `adjudicated/<hash>` identity over sorted member claim ids.
   Emission deduplication retains its first representative's identity and severity.
   */
  readonly issueId: string;

  /**
   Fate decided by the tally rules.
   */
  readonly status: AdjudicationStatus;

  /**
   Final severity: upper median over member claim severities plus
   supported ballots' re-grades.
   */
  readonly severity: IssueSeverity;

  /**
   Member claims, atomic as proposed. Fresh accepted issues contain only
   accepted members; rejected and unresolved diagnoses remain separate records.
   */
  readonly claims: readonly AggregatedClaim[];

  /**
   Per-claim weighted tallies keyed by claim id.
   */
  readonly tallies: Readonly<Record<string, VoteTally>>;

  /**
   Per-claim BALLOTS behind those tallies, keyed by claim id.

   BESIDE {@link AdjudicatedIssue.tallies} RATHER THAN INSIDE IT, because the
   tally is the decision and the ballots are the evidence for it. Keeping them
   in one record is what made the evidence easy to drop: five weighted numbers
   look complete on their own, and cannot say whether an acceptance was
   unanimous or one weighted vote wide, cannot name a dissenter, and cannot be
   re-tallied under a different weight table.

   OPTIONAL, AND ITS ABSENCE HAS ONE MEANING: this issue was not built by
   `tallyVotes`. A deduplicated issue may have readings for only its known
   members when it combines current records with older records.
   That covers records written before this field existed, and
   the readers and tools that rebuild an issue from a settled artifact rather
   than adjudicating one. Requiring it would make those write an empty record,
   which would say "no panel voted" where the truth is "I am not the panel".
   */
  readonly readings?: Readonly<Record<string, ClaimPanelReading>>;

  /**
   Which critics FILED each member claim, keyed by claim id, sorted by model
   id (owner, 2026-09-24: "we're not logging enough then. Refine how we
   log."). Attached AFTER the panel spoke, from the critic phase's attribution
   record, so the panel stays provenance-blind and the artifact's reader no
   longer joins `sliceCritics[].claimAttributions` by hand to learn who
   filed a claim the panel accepted.

   OPTIONAL WITH ONE MEANING: absent where no filer is on record, which is an
   issue rebuilt from an older artifact, a probe fixture, or a claim the
   reference screen filed itself. A claim on record always names at least
   one filer; an empty list is never written.
   */
  readonly filedBy?: Readonly<Record<string, readonly RosterModelId[]>>;
};

/**
 Tally rules; every knob is calibratable by the scorecard later.

 @example
 ```ts
 const config: AdjudicationConfig = DEFAULT_ADJUDICATION_CONFIG;
 ```
 */
export type AdjudicationConfig = {
  /**
   Fraction of non-abstain weight supported votes must strictly exceed
   for acceptance; the same fraction gates rejection symmetrically.
   */
  readonly decisionThreshold: number;

  /**
   Fraction of non-abstain weight at which source-defect votes block the
   issue; deliberately lower than the decision threshold because a
   protective minority suffices against correcting toward corruption.
   */
  readonly sourceDefectThreshold: number;

  /**
   Vote weight per panelist id; absent panelists weigh one.
   */
  readonly weights?: Readonly<Record<string, number>>;
};

/**
 Denominator of the protective minority fraction: one third of the
 electorate flagging a source defect suffices to block, because
 correcting toward corruption is the costlier mistake.
 */
const SOURCE_DEFECT_BLOCK_DENOMINATOR = 3;

/**
 Defaults until canary calibration supplies weights:
 strict majority of the weight that answered decides, one-third blocks on
 suspected source defects. THERE IS NO WEIGHT FLOOR: the owner's rulings of
 2026-10-25, "floor of 3 - don't set a floor." and "Two ballots decide
 (Recommended)", removed the minimum ballot weight of three. The tally decides
 with two or more answering panelists and leaves a claim that only one
 panelist answers to a human, whatever weight that panelist carries
 (`MIN_PANEL_BALLOTS` in the adjudication package).
 */
export const DEFAULT_ADJUDICATION_CONFIG: AdjudicationConfig = {
  decisionThreshold: 1 / 2,
  sourceDefectThreshold: 1 / SOURCE_DEFECT_BLOCK_DENOMINATOR,
};

//endregion Adjudication model
