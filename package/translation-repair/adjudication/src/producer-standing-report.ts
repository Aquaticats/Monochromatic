import {
  preferenceRate,
  type ProducerStanding,
} from './producer-standing.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Producer standing report
// Renders and orders standings, shared by every calibration that produces one.
//
// SPLIT OUT WHEN THE SECOND CALLER ARRIVED. `producer-calibrate.ts` ranks
// writers and `editor-calibrate.ts` ranks editors, and a standing means the
// same thing in both: the share of ballots cast by judges holding no stake.
// Two copies of that rendering would be two places for the denominator to
// drift, and the denominator is the part a reader has to trust.

/**
 Percent, for rendering a share as one.
 */
const AS_PERCENT = 100;

/**
 Decimal places a reported share carries.
 */
const SHARE_PLACES = 1;

/**
 Renders one model's standing as a report line.

 CARRIES ITS OWN DENOMINATOR. A share with no count beside it cannot be told
 apart from a share one ballot wide, and a lead smaller than its denominator
 supports is not a lead.

 @param standing - counts for one model

 @returns Line naming the share and the evidence behind it

 @example
 ```ts
 console.log(standingLine({ standing, },),);
 ```
 */
export function standingLine(
  { standing, }: { readonly standing: ProducerStanding; },
): string {
  /**
   Share of disinterested ballots, where anything was cast.
   */
  const rate = preferenceRate({ standing, },);

  /**
   That share rendered, or a mark saying nothing was cast.
   */
  const share = rate.measured
    ? `${(rate.share * AS_PERCENT).toFixed(SHARE_PLACES,)}%`
    : 'UNJUDGED';

  return `${standing.modelId}: ${share} (${String(standing.disinterestedVotes,)}`
    + ` of ${String(standing.disinterestedBallots,)} disinterested ${
      wordForCount({
        count: standing.disinterestedBallots,
        one: 'ballot',
        many: 'ballots',
      },)
    },`
    + ` over ${String(standing.candidates,)} ${
      wordForCount({
        count: standing.candidates,
        one: 'candidate',
        many: 'candidates',
      },)
    })`;
}

/**
 Orders standings best first, with unjudged models last.

 AN UNJUDGED MODEL IS NOT A ZERO. It wrote candidates no disinterested judge
 ever voted on, which is absence of evidence rather than evidence of a poor
 showing, so it sorts to the end instead of to the bottom.

 @param standings - what the tally produced

 @returns Same standings, sorted

 @example
 ```ts
 const ranked = rankStandings({ standings, },);
 ```
 */
export function rankStandings(
  { standings, }: { readonly standings: readonly ProducerStanding[]; },
): readonly ProducerStanding[] {
  return standings.toSorted(function byShare(
    left,
    right,
  ): number {
    /**
     Left-hand share, read so an unjudged model sorts last.
     */
    const leftRate = preferenceRate({ standing: left, },);

    /**
     Right-hand share, read the same way.
     */
    const rightRate = preferenceRate({ standing: right, },);

    if (!leftRate.measured)
      return rightRate.measured ? 1 : 0;

    if (!rightRate.measured)
      return -1;

    return rightRate.share - leftRate.share;
  },);
}

//endregion Producer standing report
