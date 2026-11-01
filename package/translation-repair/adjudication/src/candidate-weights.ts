import type {
  CandidateWeight,
  SelectionBallot,
} from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Candidate weights
// What each candidate drew, counted once and read by every exit of a round.
//
// The round tally answers how many judges answered and how many abstained. It
// cannot answer the question a decline actually raises, which is how close the
// leader came and to what: a round that tied at weight three and a round where
// one judge named one candidate both report "kept the fallback".

/**
 Counts ballots and weight per candidate.

 Every candidate gets a row, including ones nobody named, so a reader can tell
 a candidate that drew nothing from a candidate that was never offered.

 @param ballots - ballots as cast, abstentions included

 @param candidateCount - candidates the judges were shown

 @returns One row per candidate, in slate order

 @example
 ```ts
 const perCandidate = countCandidateWeights({ ballots, candidateCount, },);
 ```

 @internal
 */
export function countCandidateWeights(
  {
    ballots,
    candidateCount,
  }: {
    readonly ballots: readonly SelectionBallot[];
    readonly candidateCount: number;
  },
): readonly CandidateWeight[] {
  return Array.from(
    { length: candidateCount, },
    function toRow(
      _unused,
      position,
    ): CandidateWeight {
      /**
       One-based index this row describes.
       */
      const index = position + 1;

      /**
       Ballots that named it, abstentions and out-of-range ballots excluded
       by construction since neither carries a usable index.

       THE WEIGHT TEST IS NOT REDUNDANT WITH THAT, though today`s producer
       makes it look so: it sets `best` and weight together, so an in-range
       index always arrives above zero. It stops being so the moment
       `SELF_VOTE_WEIGHT` is tuned to zero, which is a knob rather than a
       constant, and a judge voting for its own work would otherwise be
       counted as a ballot while contributing nothing to the weight the
       minimum is compared against.
       */
      const named = ballots.filter(function namesIt(ballot,): boolean {
        return (ballot.best === index) && (ballot.weight > 0);
      },);

      /**
       Of those, the ones cast by a judge with a stake in it.
       */
      const own = named.filter(function isSelf(ballot,): boolean {
        return ballot.selfVote;
      },);

      return {
        index,
        ballots: named.length,
        fullVotes: named.length - own.length,
        selfVotes: own.length,
        weight: named.reduce(
          function addWeight(
            total,
            ballot,
          ): number {
            return total + ballot.weight;
          },
          0,
        ),
      };
    },
  );
}

//endregion Candidate weights
