import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  type CandidateWeight,
  FULL_VOTE_WEIGHT,
  type SelectionBallot,
  SELF_VOTE_WEIGHT,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  CANDIDATE_NONE,
  type CandidateBallotAsSent,
  readCandidateBallotWire,
} from './candidate-select-wire.ts';
import { countCandidateWeights, } from './candidate-weights.ts';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';
import type {
  ReadonlyEntry,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Ballot counting
// The weighing and tallying half of the select stage, apart from
// `candidate-select.ts` at the line cap on 2026-09-09 when the short-bench
// minimum landed. What a ballot weighs and what each candidate drew are
// decided here; what the leader needs is decided there.

/**
 Everything one round's ballots add up to.

 @example
 ```ts
 const { ranked, abstained, } = countBallots({ voices, stakesByIndex, candidateCount: 2, l, },);
 ```
 */
export type CountedBallots = {
  /**
   Every ballot as cast and weighed, in the order judges answered.
   */
  readonly ballots: readonly SelectionBallot[];

  /**
   What every candidate drew, whether or not it led.
   */
  readonly perCandidate: readonly CandidateWeight[];

  /**
   One-based candidate index beside its drawn weight, most first; empty
   when every judge abstained.
   */
  readonly ranked: readonly (readonly [
    number,
    number,
  ])[];

  /**
   Ballots that named no usable candidate, kept so a selection that failed
   for want of agreement is distinguishable from one nobody voted in.
   */
  readonly abstained: number;

  /**
   Ballots a judge cast for its own work.
   */
  readonly self: number;
};

/**
 Weighs every heard ballot and tallies what each candidate drew.

 Self-votes are counted rather than prevented: the reason for seating
 producers is that their judgement carries value, and the reason for
 weighing and recording is that self-preference is a known failure of
 exactly this arrangement. A rate nobody can read is an assumption.

 @param voices - ballots the gather heard, already validated

 @param stakesByIndex - models with a stake in each one-based candidate
 index, for telling a self-vote from an ordinary one

 @param candidateCount - how many candidates the judges were shown, so an
 index past the end is an abstention rather than a vote

 @param l - logger of the calling round

 @returns Ballots, per-candidate weights, the ranking and the counts

 @example
 ```ts
 const counted = countBallots({ voices: gather.voices, stakesByIndex, candidateCount: 2, l, },);
 ```
 */
export function countBallots(
  {
    voices,
    stakesByIndex,
    candidateCount,
    l,
  }: {
    readonly voices: readonly HeardVoice<CandidateBallotAsSent>[];
    readonly stakesByIndex: ReadonlyMap<number, ReadonlySet<RosterModelId>>;
    readonly candidateCount: number;
    readonly l: Logger;
  },
): CountedBallots {
  /**
   Every ballot as cast, weighed, and carried out of this function rather
   than left in a log line.
   */
  const ballots: readonly SelectionBallot[] = voices.map(function toBallot(voice,): SelectionBallot {
    /**
     This judge's chosen index, read as a number whichever way it was sent.
     */
    const {
      best,
      assessment: reason,
    } = readCandidateBallotWire({ sent: voice.value, },);

    /**
     Whether this judge named text it has a stake in.
     */
    const ownWork = stakesByIndex.get(best,)
      ?.has(voice.modelId,)
      === true;
    /**
     Whether this ballot names a candidate at all.
     */
    const usable = (best !== CANDIDATE_NONE) && (best <= candidateCount);
    return {
      modelId: voice.modelId,
      best,
      reason,
      weight: usable
        ? (ownWork ? SELF_VOTE_WEIGHT : FULL_VOTE_WEIGHT)
        : 0,
      selfVote: usable && ownWork,
    };
  },);

  /**
   What each candidate drew, kept per index so a decline says by how much the
   leader fell short and against what.
   */
  const perCandidate = countCandidateWeights({
    ballots,
    candidateCount,
  },);

  /**
   Ballot weight per one-based candidate index; out-of-range ballots and
   explicit declines are counted as abstentions rather than discarded
   silently.
   */
  const tally = new Map<number, number>();

  /**
   Ballots that named no usable candidate, and ballots a judge cast for its
   own work.
   */
  const counters = {
    abstained: 0,
    self: 0,
  };
  for (const ballot of ballots) {
    if (ballot.weight === 0) {
      counters.abstained += 1;
      // AN ABSTENTION IS LOGGED WITH ITS REASON like a choice is. mikaela4
      // (2026-09-19) lost a run-off to two abstentions whose grounds were
      // readable only off the cached replies; a judge that declines has said
      // something about the slate, and the record has to carry it.
      l.info(
        (ballot.best === CANDIDATE_NONE)
          ? `${ballot.modelId} declined every candidate: ${ballot.reason}`
          : `${ballot.modelId} named candidate ${String(ballot.best,)}, which is not on the slate of `
            + `${String(candidateCount,)}: ${ballot.reason}`,
      );
      continue;
    }
    if (ballot.weight === SELF_VOTE_WEIGHT)
      counters.self += 1;
    tally.set(
      ballot.best,
      (tally.get(ballot.best,) ?? 0) + ballot.weight,
    );
    l.info(
      `${ballot.modelId} chose candidate ${String(ballot.best,)} at weight ${
        String(ballot.weight,)
      }: ${ballot.reason}`,
    );
  }

  /**
   Candidate indexes ordered by drawn weight, most first.
   */
  const ranked = [...tally.entries(),].toSorted(function byWeight(
    a: ReadonlyEntry<number, number>,
    b: ReadonlyEntry<number, number>,
  ): number {
    return b[1] - a[1];
  },);
  return {
    ballots,
    perCandidate,
    ranked,
    abstained: counters.abstained,
    self: counters.self,
  };
}

//endregion Ballot counting
