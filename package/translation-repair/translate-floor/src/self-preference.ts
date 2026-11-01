import {
  type CandidateProducer,
  producerModelIds,
  type SelectionBallot,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Self-preference
// Whether a producer judging its own work actually favours it, measured rather
// than assumed.
//
// WHY THIS EXISTS. Selection discounts a producer's ballot for its own
// candidate to half. The half was CHOSEN, on consistency with nothing, and
// question 4 was answered knowing that: `doc/decision/translation-repair-
// question-answers.md` records it as a stated preference open to revision by
// exactly this measurement.
//
// THE COMPARISON HAS TO BE PAIRED, and that is the whole design. Counting how
// often a producer votes for its own candidate says nothing on its own: a model
// whose translations are genuinely better would do that without any
// self-preference at all. So each candidate is scored against ITSELF: the
// producer's own vote on one side, and the rate among judges with no stake in
// that same candidate on the other. Same text, same slice, same sheet. What
// remains after subtracting is preference for one's own work rather than
// agreement about its quality.
//
// WHAT IT STILL CANNOT SEE: a producer that writes to its own taste rather than
// to the criteria would show up here as self-preference, and so would a roster
// whose members simply agree with themselves across runs. Neither is
// distinguishable from favouritism by ballots alone.

/**
 One candidate on one slice, with the ballots cast over that slate.

 @example
 ```ts
 const round: SelectionRound = { producers: [producer,], ballots, };
 ```
 */
export type SelectionRound = {
  /**
   Each candidate's provenance, in slate order, so a one-based ballot index
   names `producers[best - 1]`.
   */
  readonly producers: readonly CandidateProducer[];

  /**
   Every ballot cast over that slate.
   */
  readonly ballots: readonly SelectionBallot[];
};

/**
 Ballot counts every outcome carries, whether or not a rate could be taken.

 @example
 ```ts
 const counts: SelfPreferenceCounts = { opportunities: 4, ownVotes: 3, otherBallots: 12, otherVotes: 5, };
 ```
 */
export type SelfPreferenceCounts = {
  /**
   Occasions a model held a stake in a candidate AND cast a ballot on that
   slate, which is the only situation where self-preference can be expressed.
   */
  readonly opportunities: number;

  /**
   Occasions it named its own candidate.
   */
  readonly ownVotes: number;

  /**
   Ballots cast over those same candidates by judges holding no stake.
   */
  readonly otherBallots: number;

  /**
   How many of those named the candidate.
   */
  readonly otherVotes: number;
};

/**
 What the paired comparison found.

 THREE OUTCOMES RATHER THAN OPTIONAL NUMBERS, because the two ways this fails
 to produce a rate are different facts and a reader has to tell them apart. No
 stakeholder ever voting means the question was never put; every judge holding
 a stake means it was put with no one left to answer it, which is a roster
 shape rather than missing data. Both carry their counts, so a caller can
 report what was seen either way.

 @example
 ```ts
 const measured: SelfPreference = selfPreference({ rounds, },);
 ```
 */
export type SelfPreference =
  | ({
    readonly kind: 'measured';

    /**
     Share of stakeholder ballots naming their own candidate.
     */
    readonly ownRate: number;

    /**
     Share of the SAME candidates taken by judges holding no stake in them,
     measured on the same texts rather than assumed from the slate size.
     */
    readonly disinterestedRate: number;

    /**
     Own rate minus disinterested rate.

     Positive means producers favoured their own work beyond what judges with
     no stake in it thought of the same text. Zero means the half-weight
     discount is correcting nothing this measurement can see.
     */
    readonly excess: number;
  } & SelfPreferenceCounts)
    | ({
    /**
     No model both held a stake in a candidate and cast a ballot on its slate,
     so no self-vote was ever possible.
     */
      readonly kind: 'no-stakeholder-ballots';
    } & SelfPreferenceCounts)
    | ({
    /**
     Every judge over these candidates held a stake in them, so nothing
     disinterested is left to compare against. A roster where producers judge
     their own work and nobody else does cannot be measured this way.
     */
      readonly kind: 'no-disinterested-ballots';
    } & SelfPreferenceCounts);

/**
 Ballots a candidate drew, split by whether the judge had a stake in it.

 @example
 ```ts
 const drawn: CandidateDraw = { ownVotes: 1, opportunities: 1, otherVotes: 2, otherBallots: 5, };
 ```
 */
type CandidateDraw = {
  /**
   Stakeholder ballots naming this candidate.
   */
  readonly ownVotes: number;

  /**
   Stakeholder ballots cast at all, whatever they named.
   */
  readonly opportunities: number;

  /**
   Disinterested ballots naming this candidate.
   */
  readonly otherVotes: number;

  /**
   Disinterested ballots cast at all.
   */
  readonly otherBallots: number;
};

/**
 Scores one candidate against the slate it stood on.

 @param producer - that candidate's provenance

 @param candidateIndex - one-based slate position ballots name it by

 @param ballots - every ballot cast over the slate

 @returns How the candidate fared with stakeholders and with everyone else

 @example
 ```ts
 const drawn = drawForCandidate({ producer, candidateIndex: 1, ballots, },);
 ```
 */
function drawForCandidate(
  {
    producer,
    candidateIndex,
    ballots,
  }: {
    readonly producer: CandidateProducer;
    readonly candidateIndex: number;
    readonly ballots: readonly SelectionBallot[];
  },
): CandidateDraw {
  /**
   Models whose ballot for this candidate would be a self-vote.
   */
  const stakeholders: readonly RosterModelId[] = producerModelIds(producer,);

  /**
   Ballots from judges who helped write this candidate.
   */
  const held = ballots.filter(function holdsStake(ballot,): boolean {
    return stakeholders.includes(ballot.modelId,);
  },);

  /**
   Ballots from judges with no stake in it, which is the baseline population.
   */
  const disinterested = ballots.filter(function holdsNoStake(ballot,): boolean {
    return !stakeholders.includes(ballot.modelId,);
  },);

  /**
   Whether one ballot named the candidate being scored.

   @param ballot - cast over this slate

   @returns True when it names this candidate's slate position

   @example
   ```ts
   const chose = namedIt(ballot,);
   ```
   */
  function namedIt(ballot: SelectionBallot,): boolean {
    return ballot.best === candidateIndex;
  }

  return {
    ownVotes: held.filter(namedIt,)
      .length,
    opportunities: held.length,
    otherVotes: disinterested.filter(namedIt,)
      .length,
    otherBallots: disinterested.length,
  };
}

/**
 Measures self-preference over a set of selection rounds.

 ONLY CANDIDATES WITH A STAKEHOLDER WHO VOTED CONTRIBUTE, on either side. A
 candidate nobody had a stake in cannot express self-preference, and counting
 its disinterested ballots into the baseline would dilute the comparison with
 texts the measurement is not about. That restriction is what keeps the two
 rates paired on the same candidates.

 @param rounds - one entry per judged slice, with its slate and its ballots

 @returns Own rate, the disinterested rate over the same candidates, and the
 difference

 @example
 ```ts
 const measured = selfPreference({ rounds, },);
 ```
 */
export function selfPreference(
  { rounds, }: { readonly rounds: readonly SelectionRound[]; },
): SelfPreference {
  /**
   Every candidate that had at least one stakeholder ballot, scored.
   */
  const draws = rounds.flatMap(function roundDraws(round,): readonly CandidateDraw[] {
    return round.producers
      .map(function toDraw(
        producer,
        slateIndex,
      ): CandidateDraw {
        return drawForCandidate({
          producer,
          candidateIndex: slateIndex + 1,
          ballots: round.ballots,
        },);
      },)
      .filter(function expressible(drawn,): boolean {
        return drawn.opportunities > 0;
      },);
  },);

  /**
   Stakeholder ballots across every contributing candidate.
   */
  const opportunities = draws.reduce(
    function sumOpportunities(
      total,
      drawn,
    ): number {
      return total + drawn.opportunities;
    },
    0,
  );

  /**
   Stakeholder ballots that named their own candidate.
   */
  const ownVotes = draws.reduce(
    function sumOwn(
      total,
      drawn,
    ): number {
      return total + drawn.ownVotes;
    },
    0,
  );

  /**
   Disinterested ballots over those same candidates.
   */
  const otherBallots = draws.reduce(
    function sumOtherBallots(
      total,
      drawn,
    ): number {
      return total + drawn.otherBallots;
    },
    0,
  );

  /**
   Disinterested ballots naming them.
   */
  const otherVotes = draws.reduce(
    function sumOtherVotes(
      total,
      drawn,
    ): number {
      return total + drawn.otherVotes;
    },
    0,
  );

  /**
   Ballot counts every outcome reports.
   */
  const counts: SelfPreferenceCounts = {
    opportunities,
    ownVotes,
    otherBallots,
    otherVotes,
  };
  if (opportunities === 0) {
    return {
      kind: 'no-stakeholder-ballots',
      ...counts,
    };
  }
  if (otherBallots === 0) {
    return {
      kind: 'no-disinterested-ballots',
      ...counts,
    };
  }

  /**
   Share of stakeholder ballots naming their own candidate.
   */
  const ownRate = ownVotes / opportunities;

  /**
   Share of disinterested ballots naming the same candidates.
   */
  const disinterestedRate = otherVotes / otherBallots;

  return {
    kind: 'measured',
    ...counts,
    ownRate,
    disinterestedRate,
    excess: ownRate - disinterestedRate,
  };
}

//endregion Self-preference
