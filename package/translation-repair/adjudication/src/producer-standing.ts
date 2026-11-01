import {
  type CandidateProducer,
  producerModelIds,
  type SelectionBallot,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Producer standing
// HOW OFTEN EACH MODEL'S WRITING IS PREFERRED, counted from the rounds the
// selection already recorded rather than from a fresh contest.
//
// WHAT THIS IS FOR. The roster carries nine models (eight when this was
// written) and the writers stay at three, chosen by measurement. `roster-bench.ts` answers a question about
// WIDTH; this answers the question about WHO, off the same rounds, because a
// round already names who wrote each candidate and carries every ballot cast
// over that slate.
//
// ONLY DISINTERESTED BALLOTS COUNT, and that is the whole design. A producer
// judging its own candidate is measured to favour it, which is why selection
// discounts that vote at all. Counting self-votes into a standing would rank
// the most self-confident model first rather than the best-written one, and
// the calibration would then seat exactly the models least able to tell.
// `self-preference.ts` makes the same argument for the same reason and shares
// the stakeholder reading, so neither file invents its own idea of a stake.
//
// `SelectionBallot.selfVote` DOES NOT ANSWER THIS, though it looks as if it
// might. That flag says whether a judge named text it produced; the question
// here is whether a judge holds a stake in ONE PARTICULAR candidate, whichever
// way it voted. A judge with a stake in candidate 1 that votes for candidate 2
// carries `selfVote: false` and must still be excluded from candidate 1's
// tally, because its opinion of its own rival is not disinterested.
//
// COUNTS, NOT RATES. A rate hides its own evidence: three candidates winning
// three times reads identically to forty winning forty, and only one of those
// should decide a roster. Whoever reports divides, and has the denominator in
// hand to say whether the division means anything.
//
// A COMPOSITE CREDITS EVERY CONTRIBUTOR, since the candidate is their joint
// work. An incumbent credits the models collapsed into it and nobody else: the
// archive is not a roster member, so an incumbent standing alone credits no
// one, which is correct rather than a gap.

/**
 What the disinterested judges made of one model's writing.

 @example
 ```ts
 const standing: ProducerStanding = {
   modelId: 'minimax-m3',
   candidates: 12,
   disinterestedBallots: 48,
   disinterestedVotes: 19,
 };
 ```
 */
export type ProducerStanding = {
  /**
   Model whose writing was judged.
   */
  readonly modelId: RosterModelId;

  /**
   Slates carrying a candidate this model helped write.

   THE EVIDENCE COUNT, not a score. A model seated on few rounds can lead on
   rate and mean nothing.
   */
  readonly candidates: number;

  /**
   Ballots cast over those candidates by judges holding no stake in them.
   */
  readonly disinterestedBallots: number;

  /**
   How many of those ballots named this model's candidate.
   */
  readonly disinterestedVotes: number;
};

/**
 Adds one candidate's disinterested ballots into a running tally.

 @param tally - per-model counts, mutated in place

 @param producer - who wrote this candidate

 @param candidateIndex - one-based slate position a ballot names

 @param ballots - every ballot cast over the whole slate

 @example
 ```ts
 foldCandidate({ tally, producer, candidateIndex: 1, ballots, },);
 ```
 */
function foldCandidate(
  {
    tally,
    producer,
    candidateIndex,
    ballots,
  }: {
    readonly tally: Map<RosterModelId, ProducerStanding>;
    readonly producer: CandidateProducer;
    readonly candidateIndex: number;
    readonly ballots: readonly SelectionBallot[];
  },
): void {
  /**
   Models whose ballot for this candidate would be a vote for their own work.
   */
  const stakeholders = producerModelIds(producer,);

  if (stakeholders.length === 0)
    return;

  /**
   Ballots from judges with no stake in this particular candidate.
   */
  const disinterested = ballots.filter(function noStake(ballot,): boolean {
    return !stakeholders.includes(ballot.modelId,);
  },);

  /**
   Those that named it.
   */
  const naming = disinterested.filter(function named(ballot,): boolean {
    return ballot.best === candidateIndex;
  },);

  /**
   How many that came to.
   */
  const votes = naming.length;

  for (const modelId of stakeholders) {
    /**
     This model's counts so far, empty on first sight.
     */
    const standing = tally.get(modelId,) ?? {
      modelId,
      candidates: 0,
      disinterestedBallots: 0,
      disinterestedVotes: 0,
    };

    tally.set(
      modelId,
      {
        modelId,
        candidates: standing.candidates + 1,
        disinterestedBallots: standing.disinterestedBallots + disinterested.length,
        disinterestedVotes: standing.disinterestedVotes + votes,
      },
    );
  }
}

/**
 Counts how often each model's writing was preferred by judges with no stake.

 @param rounds - selection rounds, each a slate plus the ballots over it

 @returns One standing per model that wrote at least one candidate

 @example
 ```ts
 const standings = producerStandings({ rounds, },);
 ```
 */
export function producerStandings(
  { rounds, }: { readonly rounds: readonly SelectionRound[]; },
): readonly ProducerStanding[] {
  /**
   Running counts, keyed by model.
   */
  const tally = new Map<RosterModelId, ProducerStanding>();

  for (const round of rounds) {
    /**
     Slate and ballots of this round.
     */
    const {
      producers,
      ballots,
    } = round;

    for (const [position, producer,] of producers.entries()) {
      foldCandidate({
        tally,
        producer,
        // Ballots name a ONE-BASED index, matching `producers[best - 1]`.
        candidateIndex: position + 1,
        ballots,
      },);
    }
  }

  return [...tally.values(),];
}

/**
 Share of disinterested ballots that named one model's writing, where the
 share can be taken at all.

 @example
 ```ts
 const rate: PreferenceRate = { measured: true, share: 0.4, };
 ```
 */
export type PreferenceRate =
  | {
    /**
     Discriminator marking a standing something was actually cast on.
     */
    readonly measured: true;

    /**
     Share between zero and one.
     */
    readonly share: number;
  }
    | {
    /**
     Discriminator marking a standing no disinterested judge voted on.
     */
      readonly measured: false;
    };

/**
 Share of disinterested ballots that named one model's writing.

 A DISCRIMINATED ABSENCE rather than a zero, because a model nobody
 disinterested voted on has not been measured to be bad. Returning zero would
 sort it below every measured model and read as the strongest possible
 evidence against it, which is the opposite of what an empty denominator
 means.

 @param standing - one model's counts

 @returns Share, or that nothing disinterested was cast on its work

 @example
 ```ts
 const rate = preferenceRate({ standing, },);
 ```
 */
export function preferenceRate(
  { standing, }: { readonly standing: ProducerStanding; },
): PreferenceRate {
  /**
   Ballots the rate would divide by.
   */
  const { disinterestedBallots, } = standing;

  if (disinterestedBallots === 0)
    return { measured: false, };
  return {
    measured: true,
    share: standing.disinterestedVotes / disinterestedBallots,
  };
}

//endregion Producer standing
