import type { AuditRepeatPair, } from './rendering-audit-settled-repeat.ts';

//region Settled audit band
// Turns pairs of audits of one text into the spread the instrument moves
// through on unchanged input.
//
// THIS IS THE NUMBER EVERY OTHER NUMBER HAS TO CLEAR. `QNB` says a comparison
// resolves nothing smaller than the run-to-run spread, and the headline of
// the settled audit is a comparison. A difference between archive text and fresh text that
// is narrower than the band is a difference this instrument cannot see, and
// quoting it would credit noise to the thing under test.
//
// REPORTED ON CLAIM COUNT, which is the quantity a reader quotes. The tiers are
// carried alongside because a band that is wide on raw claims and narrow on
// corroborated ones says something specific and useful: that the spread lives
// in what single voices raise rather than in what the roster agrees on.

/**
 How far apart two audits of one text landed, over every pair found.

 @example
 ```ts
 const band: AuditRepeatBand = { pairs: 6, agreedExactly: 2, widest: 4, ... };
 ```
 */
export type AuditRepeatBand = {
  /**
   Texts audited twice.
   */
  readonly pairs: number;

  /**
   Pairs where both audits claimed the same number of defects.

   NOT THE SAME DEFECTS. This counts agreement of the headline number, which
   is weaker than agreement about the text: two audits can claim three each
   and share none of them.
   */
  readonly agreedExactly: number;

  /**
   Largest gap between two audits of one text.
   */
  readonly widest: number;

  /**
   Every gap, summed, so a mean is derivable without keeping the pairs.
   */
  readonly totalGap: number;

  /**
   Pairs where the two audits disagreed about whether there was ANYTHING here,
   one claiming nothing and the other claiming something.

   The sharpest form of the spread, and the one that matters most to any
   future gate: a threshold reading "claimed at least one" would have flipped
   on these subjects for no reason in the text.
   */
  readonly silentOnOneSide: number;

  /**
   Claims the first audits made in total.
   */
  readonly leftClaimed: number;

  /**
   Claims the second audits made in total.
   */
  readonly rightClaimed: number;

  /**
   Corroborated defects the first audits reached.
   */
  readonly leftCorroborated: number;

  /**
   Corroborated defects the second audits reached.
   */
  readonly rightCorroborated: number;
};

/**
 Adds a list of numbers.

 @param values - numbers to add

 @returns Total

 @example
 ```ts
 const total = sumOf({ values: [1, 2,], },);
 ```
 */
function sumOf(
  { values, }: { readonly values: readonly number[]; },
): number {
  return values.reduce(
    function add(
      total,
      value,
    ): number {
      return total + value;
    },
    0,
  );
}

/**
 Reads the spread over a set of repeat pairs.

 @param pairs - texts audited twice

 @returns Spread, with zeroes throughout when nothing was paired

 @example
 ```ts
 const band = repeatBandOf({ pairs, },);
 ```
 */
export function repeatBandOf(
  { pairs, }: { readonly pairs: readonly AuditRepeatPair[]; },
): AuditRepeatBand {
  /**
   Gap between the two sides of each pair.

   ABSOLUTE, because neither side is the reference. Both are single readings
   of one text, and calling either of them correct is the assumption this
   measurement exists to avoid.
   */
  const gaps = pairs.map(function gapOf({
    left,
    right,
  },): number {
    return Math.abs(left.claimed - right.claimed,);
  },);

  /**
   Pairs where both audits landed on the same count.
   */
  const exact = gaps.filter(function isZero(gap,): boolean {
    return gap === 0;
  },);

  /**
   Pairs where one audit found something and the other found nothing.
   */
  const lopsided = pairs.filter(function onlyOneSpoke({
    left,
    right,
  },): boolean {
    return (left.claimed === 0) !== (right.claimed === 0);
  },);

  return {
    pairs: pairs.length,
    agreedExactly: exact.length,
    widest: gaps.reduce(
      function larger(
        worst,
        gap,
      ): number {
        return Math.max(
          worst,
          gap,
        );
      },
      0,
    ),
    totalGap: sumOf({ values: gaps, },),
    silentOnOneSide: lopsided.length,
    leftClaimed: sumOf({
      values: pairs.map(function claimed({ left, },): number {
        return left.claimed;
      },),
    },),
    rightClaimed: sumOf({
      values: pairs.map(function claimed({ right, },): number {
        return right.claimed;
      },),
    },),
    leftCorroborated: sumOf({
      values: pairs.map(function strict({ left, },): number {
        return left.corroborated;
      },),
    },),
    rightCorroborated: sumOf({
      values: pairs.map(function strict({ right, },): number {
        return right.corroborated;
      },),
    },),
  };
}

//endregion Settled audit band
