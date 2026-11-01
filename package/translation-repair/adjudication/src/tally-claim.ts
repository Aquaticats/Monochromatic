import type {
  AdjudicationConfig,
  ClaimPanelReading,
  PanelBallot,
  PanelClaimBallot,
  PanelVoteState,
  ReadonlyEntry,
  VoteTally,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Tally claim
// One claim's arithmetic, split out of `tally-votes.ts` so the reading that
// records the ballots has somewhere to live beside the sums it produces.
//
// `tally-votes.ts` decides; this file counts. The split follows the line cap
// rather than preceding it, but the boundary is the honest one: nothing here
// reads a cluster, a severity or a status.

/**
 Weight one panelist's vote on one claim carries.

 @param panelistId - panelist whose weight is read

 @param config - weight table

 @returns Configured weight, defaulting to one

 @example
 ```ts
 const weight = panelistWeight({ panelistId, config, },);
 ```
 */
function panelistWeight(
  {
    panelistId,
    config,
  }: {
    readonly panelistId: string;
    readonly config: AdjudicationConfig;
  },
): number {
  return config.weights?.[panelistId] ?? 1;
}

/**
 Vote one panelist cast on one claim, with a missing verdict abstaining.

 @param claimId - claim under tally

 @param ballot - that panelist's whole ballot

 @returns Vote state to count

 @example
 ```ts
 const vote = castVote({ claimId, ballot, },);
 ```
 */
function castVote(
  {
    claimId,
    ballot,
  }: {
    readonly claimId: string;
    readonly ballot: PanelBallot;
  },
): PanelVoteState {
  return ballot.verdicts[claimId]
    ?.vote
    ?? 'abstain';
}

/**
 Weight behind one vote state on one claim across every ballot;
 a missing verdict abstains.

 @param claimId - claim under tally

 @param ballots - resolved ballots keyed by panelist id

 @param config - weight table

 @param state - vote state whose mass is summed

 @returns Weighted vote mass for the state

 @example
 ```ts
 const mass = voteWeight({ claimId, ballots, config, state: 'supported', },);
 ```
 */
function voteWeight(
  {
    claimId,
    ballots,
    config,
    state,
  }: {
    readonly claimId: string;
    readonly ballots: Readonly<Record<string, PanelBallot>>;
    readonly config: AdjudicationConfig;
    readonly state: PanelVoteState;
  },
): number {
  return Object
    .entries(ballots,)
    .reduce(
    function addBallot(
      mass: number,
      [panelistId, ballot,]: ReadonlyEntry<string, PanelBallot>,
    ): number {
      if (castVote({
        claimId,
        ballot,
      },) !== state)
        return mass;
      return mass + panelistWeight({
        panelistId,
        config,
      },);
    },
    0,
    );
}

/**
 Weighted tally of one claim across every ballot.

 @param claimId - claim under tally

 @param ballots - resolved ballots keyed by panelist id

 @param config - weight table and thresholds

 @returns Weighted counts per vote state

 @example
 ```ts
 const tally = tallyClaim({ claimId, ballots, config, },);
 ```
 */
function tallyClaim(
  {
    claimId,
    ballots,
    config,
  }: {
    readonly claimId: string;
    readonly ballots: Readonly<Record<string, PanelBallot>>;
    readonly config: AdjudicationConfig;
  },
): VoteTally {
  /**
   Shared facets of every per-state sum.
   */
  const facets = {
    claimId,
    ballots,
    config,
  };

  return {
    supported: voteWeight({
      ...facets,
      state: 'supported',
    },),
    unsupported: voteWeight({
      ...facets,
      state: 'unsupported',
    },),
    ambiguous: voteWeight({
      ...facets,
      state: 'ambiguous',
    },),
    sourceDefect: voteWeight({
      ...facets,
      state: 'source-defect',
    },),
    abstain: voteWeight({
      ...facets,
      state: 'abstain',
    },),
  };
}

/**
 Everything the panel said about one claim, ballots included.

 ABSTENTIONS ARE RECORDED AS BALLOTS, because a panelist that answered the
 sheet and declined this claim is different evidence from one whose reply
 never arrived, and only the first leaves an entry in `ballots`. Together with
 `configuredPanelists` that separates all three states a run can be in.

 @param claimId - claim under tally

 @param ballots - resolved ballots keyed by panelist id

 @param configuredPanelists - panelists the run seated, heard or not

 @param config - weight table and thresholds

 @returns Ballots, seated count, and the tally they sum to

 @example
 ```ts
 const reading = panelReadingForClaim({ claimId, ballots, configuredPanelists: 6, config, },);
 ```
 */
export function panelReadingForClaim(
  {
    claimId,
    ballots,
    configuredPanelists,
    config,
  }: {
    readonly claimId: string;
    readonly ballots: Readonly<Record<string, PanelBallot>>;
    readonly configuredPanelists: number;
    readonly config: AdjudicationConfig;
  },
): ClaimPanelReading {
  return {
    ballots: Object
      .entries(ballots,)
      .map(function toBallot([panelistId, ballot,]: ReadonlyEntry<string, PanelBallot>,): PanelClaimBallot {
        /**
         Reason this panelist gave for this claim, absent where it gave none.
         */
        const reason = ballot.verdicts[claimId]
          ?.reason;
        return {
          panelistId,
          vote: castVote({
            claimId,
            ballot,
          },),
          weight: panelistWeight({
            panelistId,
            config,
          },),
          ...((reason === undefined) ? {} : { reason, }),
        };
      },),
    configuredPanelists,
    tally: tallyClaim({
      claimId,
      ballots,
      config,
    },),
  };
}

//endregion Tally claim
