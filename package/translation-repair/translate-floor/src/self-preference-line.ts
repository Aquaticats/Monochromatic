import type { SelfPreference, } from './self-preference.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Self-preference line
// How a self-preference result is stated to a reader.
//
// SEPARATE FROM THE MEASUREMENT because the two answer to different rules. The
// measurement must not decide how many digits matter; the wording must not
// decide what was measured. Keeping them apart also lets the wording be tested
// against the outcomes it has to distinguish, which is the part a reader
// actually acts on.

/**
 Digits kept on a rate, enough to separate a bench-scale difference without
 implying precision a few dozen ballots cannot carry.
 */
const RATE_DIGITS = 2;

/**
 States a self-preference result in one line.

 EVERY OUTCOME SAYS WHAT IT MEANS rather than printing a bare number, because
 the three are acted on differently: a measured excess is evidence about the
 discount, no stakeholder ballots means the question was never put, and no
 disinterested ballots means the roster left nobody able to answer it. A
 reader who saw `0.00` for all three would take the last two for evidence of
 no favouritism.

 @param preference - what the paired comparison found

 @returns One line naming the outcome and the counts behind it

 @example
 ```ts
 const line = describeSelfPreference({ preference, },);
 ```
 */
export function describeSelfPreference(
  { preference, }: { readonly preference: SelfPreference; },
): string {
  if (preference.kind === 'no-stakeholder-ballots') {
    return 'self-preference not put (no producer judged its own candidate)';
  }
  if (preference.kind === 'no-disinterested-ballots') {
    return `self-preference unanswerable (every judge held a stake in all `
      + `${String(preference.opportunities,)} ${
        wordForCount({
          count: preference.opportunities,
          one: 'ballot',
          many: 'ballots',
        },)
      })`;
  }

  /**
   Difference between the two rates, which is the headline.
   */
  const excess = preference.excess
    .toFixed(RATE_DIGITS,);

  /**
   Share of stakeholder ballots naming their own candidate.
   */
  const own = preference.ownRate
    .toFixed(RATE_DIGITS,);

  /**
   Share among judges with no stake in the same candidates.
   */
  const others = preference.disinterestedRate
    .toFixed(RATE_DIGITS,);

  return `self-preference ${excess} (own ${own} of `
    + `${String(preference.opportunities,)}, others ${others} of `
    + `${String(preference.otherBallots,)}, same candidates)`;
}

//endregion Self-preference line
