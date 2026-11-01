import type {
  PanelVoteState,
  VoteTally,
} from './adjudicate-model.ts';

//region Panel reading
// What the adjudication panel said about one claim, kept in full rather than
// summed away.
//
// WHY THE TALLY WAS NOT ENOUGH, and why this gate matters more than the two
// that already record their ballots. `VoteTally` is five weighted numbers, and
// the status derived from them decides whether a claim becomes an issue the
// pipeline repairs AT ALL. Everything downstream is conditioned on that
// decision. Without the ballots a settled run cannot say whether an acceptance
// was unanimous or one weighted vote wide, cannot name the panelist that
// dissented, and cannot be re-tallied under a different weight table, because
// `voteWeight` folds `AdjudicationConfig.weights` into the sums before they are
// stored.
//
// The resolution checkers and the introduced-defect probe both keep their
// ballots and their seated roster per issue. The stage that decides what is
// worth repairing kept less.

/**
 One panelist's vote on one claim, beside the weight it carried.

 @example
 ```ts
 const ballot: PanelClaimBallot = {
   panelistId: 'hf:zai-org/GLM-5.3-Flash',
   vote: 'supported',
   weight: 1,
 };
 ```
 */
export type PanelClaimBallot = {
  /**
   Panelist that voted, as the shell named it when resolving the ballot.

   A plain string rather than a catalog id because the panel roster is the
   shell's to name: `tallyVotes` receives ballots keyed by whatever the caller
   used, and claims never carry panelist identity themselves.
   */
  readonly panelistId: string;

  /**
   What it said about this claim, with a missing verdict recorded as the
   abstention the tally already treats it as.
   */
  readonly vote: PanelVoteState;

  /**
   Weight this vote carried, read off the config in force for that run.

   STORED PER BALLOT rather than left to a reader to look up, because the
   table lives in `AdjudicationConfig` and no settled artifact records it. A
   reader holding the votes alone could not reproduce the tally on a run whose
   config was not the default.
   */
  readonly weight: number;

  /**
   Why it voted so, as it wrote before the vote (owner, 2026-09-27, "Reason
   before vote").

   ABSENT WITH TWO MEANINGS a reader can tell apart: a panelist that gave none
   left a `missing-reason` finding on its packet, and a ballot settled before
   the rule has neither, since those slices were cached under repair slice
   version 32 or earlier.
   */
  readonly reason?: string;
};

/**
 Everything the panel decided about one claim.

 @example
 ```ts
 const reading: ClaimPanelReading = { ballots, configuredPanelists: 6, tally, };
 ```
 */
export type ClaimPanelReading = {
  /**
   Every ballot cast on this claim, one per panelist heard.
   */
  readonly ballots: readonly PanelClaimBallot[];

  /**
   Panelists the run seated, which is not recoverable from the ballots.

   A lost voice leaves no ballot at all while an abstention leaves one, so
   three ballots of six seated and three of three are very different evidence
   and are indistinguishable without this. Same reasoning as
   `IssueCheckerReading.configuredCheckers` and
   `IssueProbeReading.configuredProbers`.
   */
  readonly configuredPanelists: number;

  /**
   Weighted mass behind each vote state, as this run summed it.

   STORED RATHER THAN LEFT DERIVABLE so a reader can check its own arithmetic
   against the run's. The ballots and their weights are enough to recompute
   it, and a recomputation that disagrees means the weighting changed under a
   settled artifact, which is a thing worth being able to notice.
   */
  readonly tally: VoteTally;
};

//endregion Panel reading
