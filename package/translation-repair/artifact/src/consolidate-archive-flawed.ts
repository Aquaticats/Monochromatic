import type {
  ArtifactContestVerdict,
  LaneContestBallot,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Consolidate archive flawed
// When a consolidation slate over an ELIGIBLE standing is run off rather than
// left to its single round (class one hundred six, zheermao8 slice 9,
// 2026-09-24). The contest split 2 to 2 with every ballot calling the archive
// flawed, the slate split 1/1/1 with both lane texts on offer, and the
// archive's own wording shipped: an eligible standing keeps its single round
// (the ineligible-standing decision's addenda). The owner's answer of
// 2026-09-24: "Run-off only when every contest ballot called the archive
// flawed". A tie among texts that all correct the archive is then decided
// rather than defaulted, and the standing still ships if the run-off ends
// undecided.
//
// The owner's answers of 2026-10-11 moved both readings of the contest to a
// strict majority of the usable ballots: the run-off ("Majority flawed yes",
// after TianqiChen66622 slice 4 kept an archive four of five ballots called
// flawed) and the forfeit of an eligible standing past an undecided gate
// ("Majority, like the run-off", which had read every ballot since 2026-09-27,
// "Slate's choice"). One predicate answers both.

/**
 Whether a strict majority of the usable contest ballots call the archive
 flawed on a contest that settled on neither lane. It runs off a tied slate
 over an eligible standing, and forfeits that standing past an undecided gate
 (owner answers 2026-10-11, "Majority flawed yes" and "Majority, like the
 run-off").

 A STRICT MAJORITY OF EVERY USABLE BALLOT, NOT OF THE ANSWERS. A ballot that
 did not answer about the archive is not a flawed one, and it still counts in
 the denominator, so five ballots need three flawed voices and four need three
 as well: two of four is a tie and does neither.

 @param verdict - what the contest recorded for this slice

 @param ballots - every usable ballot of that contest

 @returns Whether a tied slate over this slice's standing is run off, and an undecided gate forfeits it

 @example
 ```ts
 const runoffOverStanding = archiveFlawedByMajority({ verdict: contest.verdict, ballots: contest.ballots, },);
 ```
 */
export function archiveFlawedByMajority(
  {
    verdict,
    ballots,
  }: {
    readonly verdict: ArtifactContestVerdict;
    readonly ballots: readonly LaneContestBallot[];
  },
): boolean {
  if (verdict.kind !== 'settled-neither')
    return false;
  /**
   Usable ballots that called the archive flawed.
   */
  const flawed = ballots.filter(function calledFlawed(ballot,): boolean {
    return ballot.archive === 'flawed';
  },);
  return (flawed.length * 2) > ballots.length;
}

//endregion Consolidate archive flawed
