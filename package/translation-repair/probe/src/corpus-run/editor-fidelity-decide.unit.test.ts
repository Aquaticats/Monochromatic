/**
 Tests for the rule the editor fidelity bench is read by: which effort it picks from the clean fix counts of the four
 arms, and which clause decided. The rule is code, and the line printed before any call is made from the same source.

 Counts are written as a pool of twenty-two insertion trials and eighteen alteration trials, the frozen pool's split.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  type ArmCount,
  decideEffort,
  decisionRuleText,
  KIND_TOLERANCE_TRIALS,
  POOLED_FLOOR_TRIALS,
} from '../../dist/final/node/index.mjs';

/**
 The clean fix counts of an arm over the frozen pool's twenty-two insertion and eighteen alteration trials.

 @param insertion - insertion trials fixed cleanly
 @param alteration - alteration trials fixed cleanly

 @returns The arm's counts

 @example
 ```ts
 const arm = armOf({ insertion: 11, alteration: 9, },);
 ```
 */
function armOf(
  {
    insertion,
    alteration,
  }: {
    readonly insertion: number;
    readonly alteration: number;
  },
): ArmCount {
  return {
    insertion: { trials: 22, fixed: insertion, },
    alteration: { trials: 18, fixed: alteration, },
  };
}

/**
 Two medium arms of twenty clean fixes each, eleven insertions and nine alterations.
 */
const MEDIUM = armOf({ insertion: 11, alteration: 9, },);

await describe({
  name: 'editor fidelity decision rule',
  children: [
    describe({
      name: 'the constants',
      children: [
        it({
          name: 'FIX THE FLOOR AT TWELVE TRIALS and the kind tolerance at three, the figures the printed rule states',
          fn: async () => {
            expect(POOLED_FLOOR_TRIALS,).toBe(12,);
            expect(KIND_TOLERANCE_TRIALS,).toBe(3,);
          },
        },),
      ],
    },),

    describe({
      name: decisionRuleText.name,
      children: [
        it({
          name: 'STATES THE FLOOR, THE TOLERANCE, WHAT A TIE KEEPS AND WHAT THE INSERTION TRIALS MEASURE, the widened '
            + 'region named for what it is: locating an added sentence among its neighbours and copying the neighbours '
            + 'verbatim, not production\'s span-only view',
          fn: async () => {
            /**
             The rule as printed.
             */
            const rule = decisionRuleText();
            expect(rule,).toContain('by at least 12 trials',);
            expect(rule,).toContain('more than 3 trials below the mean of the two medium arms for that kind',);
            expect(rule,).toContain('A smaller gap, a tie, an effort that fixes fewer, and a gap a kind contradicts all keep the default effort',);
            expect(rule,).toContain('insertion trials measure locating an added sentence among its neighbours and copying the neighbours verbatim',);
            expect(rule,).toContain('not the span-only view production gives an editor',);
            expect(rule,).toContain('0.05',);
            expect(rule.includes('\n',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: decideEffort.name,
      children: [
        it({
          name: 'KEEPS MEDIUM ON A TIE, every arm at the same count, and says it was a tie',
          fn: async () => {
            const verdict = decideEffort({
              low: MEDIUM,
              mediumA: MEDIUM,
              mediumB: MEDIUM,
              high: MEDIUM,
            },);
            expect(verdict.verdict,).toBe('medium',);
            expect(verdict.clause,).toBe('tie',);
            expect(verdict.thresholdTrials,).toBe(12,);
            expect(verdict.mediumMean,).toBe(20,);
            expect(verdict.high.gap,).toBe(0,);
          },
        },),

        it({
          name: 'KEEPS MEDIUM FOR A GAP ONE BELOW THE THRESHOLD and picks the effort for a gap at it, since a difference '
            + 'counts when it reaches the threshold',
          fn: async () => {
            /**
             Eleven clean fixes above the mediums' twenty.
             */
            const below = decideEffort({
              low: MEDIUM,
              mediumA: MEDIUM,
              mediumB: MEDIUM,
              high: armOf({ insertion: 17, alteration: 14, },),
            },);
            expect(below.verdict,).toBe('medium',);
            expect(below.clause,).toBe('inside-threshold',);
            expect(below.high.gap,).toBe(11,);
            /**
             Twelve above.
             */
            const at = decideEffort({
              low: MEDIUM,
              mediumA: MEDIUM,
              mediumB: MEDIUM,
              high: armOf({ insertion: 18, alteration: 14, },),
            },);
            expect(at.verdict,).toBe('high',);
            expect(at.clause,).toBe('beats-medium',);
            expect(at.high.gap,).toBe(12,);
          },
        },),

        it({
          name: 'PICKS THE LOW EFFORT when it beats the mediums by the threshold, since the rule is about the gap and not '
            + 'about which side of medium the effort sits',
          fn: async () => {
            const verdict = decideEffort({
              low: armOf({ insertion: 18, alteration: 14, },),
              mediumA: MEDIUM,
              mediumB: MEDIUM,
              high: MEDIUM,
            },);
            expect(verdict.verdict,).toBe('low',);
            expect(verdict.clause,).toBe('beats-medium',);
          },
        },),

        it({
          name: 'KEEPS MEDIUM FOR AN EFFORT THAT FIXES FEWER by the threshold, however far, and says so',
          fn: async () => {
            const verdict = decideEffort({
              low: MEDIUM,
              mediumA: MEDIUM,
              mediumB: MEDIUM,
              high: armOf({ insertion: 4, alteration: 4, },),
            },);
            expect(verdict.verdict,).toBe('medium',);
            expect(verdict.clause,).toBe('fixes-fewer',);
            expect(verdict.high.gap,).toBe(-12,);
          },
        },),

        it({
          name: 'LETS A KIND DIP OF ONE TRIAL OR OF THE TOLERANCE STAND and vetoes a dip of more, the kind named: the '
            + 'pooled gap is twelve in each, and the alteration trials fall one, three and four below the mediums',
          fn: async () => {
            /**
             Mediums that fix six insertions and fourteen alterations, so the effort can gain twelve in all with an
             alteration dip.
             */
            const medium = armOf({ insertion: 6, alteration: 14, },);
            /**
             The high arm, gaining twelve pooled with the alteration trials the given number below.
             */
            const withDip = (dip: number) => decideEffort({
              low: medium,
              mediumA: medium,
              mediumB: medium,
              high: armOf({ insertion: 6 + 12 + dip, alteration: 14 - dip, },),
            },);
            expect(withDip(1,).verdict,).toBe('high',);
            expect(withDip(3,).verdict,).toBe('high',);
            /**
             A dip of four.
             */
            const vetoed = withDip(4,);
            expect(vetoed.verdict,).toBe('medium',);
            expect(vetoed.clause,).toBe('kind-veto',);
            expect(vetoed.high.kindShortfall,).toBe('alteration',);
            expect(vetoed.high.gap,).toBe(12,);
          },
        },),

        it({
          name: 'READS THE THRESHOLD AS THE FLOOR FOR EQUAL MEDIUMS, and as the mediums\' own difference when that is '
            + 'larger: mediums of fourteen and twenty-eight set the threshold at fourteen',
          fn: async () => {
            const equal = decideEffort({
              low: MEDIUM,
              mediumA: MEDIUM,
              mediumB: MEDIUM,
              high: MEDIUM,
            },);
            expect(equal.thresholdTrials,).toBe(12,);
            /**
             Mediums of 14 and 28 clean fixes, mean 21, difference 14.
             */
            const unequal = {
              low: MEDIUM,
              mediumA: armOf({ insertion: 7, alteration: 7, },),
              mediumB: armOf({ insertion: 14, alteration: 14, },),
            };
            /**
             Thirteen above the mean of 21.
             */
            const inside = decideEffort({
              ...unequal,
              high: armOf({ insertion: 20, alteration: 14, },),
            },);
            expect(inside.thresholdTrials,).toBe(14,);
            expect(inside.mediumMean,).toBe(21,);
            expect(inside.high.gap,).toBe(13,);
            expect(inside.verdict,).toBe('medium',);
            expect(inside.clause,).toBe('inside-threshold',);
            /**
             Fourteen above.
             */
            const at = decideEffort({
              ...unequal,
              high: armOf({ insertion: 21, alteration: 14, },),
            },);
            expect(at.high.gap,).toBe(14,);
            expect(at.verdict,).toBe('high',);
          },
        },),

        it({
          name: 'CHOOSES THE LARGER GAP WHEN BOTH EFFORTS BEAT THE MEDIUMS, and keeps medium when the two gaps are equal',
          fn: async () => {
            const high = armOf({ insertion: 19, alteration: 14, },);
            const low = armOf({ insertion: 18, alteration: 14, },);
            expect(decideEffort({ low, mediumA: MEDIUM, mediumB: MEDIUM, high, },).verdict,).toBe('high',);
            expect(decideEffort({ low: high, mediumA: MEDIUM, mediumB: MEDIUM, high: low, },).verdict,).toBe('low',);
            /**
             Two efforts with the same gap.
             */
            const equal = decideEffort({ low, mediumA: MEDIUM, mediumB: MEDIUM, high: low, },);
            expect(equal.verdict,).toBe('medium',);
            expect(equal.clause,).toBe('tie',);
          },
        },),

        it({
          name: 'SAYS THE VERDICT, THE CLAUSE, THE GAP AND THE THRESHOLD in a sentence for the read step to print',
          fn: async () => {
            const verdict = decideEffort({
              low: MEDIUM,
              mediumA: MEDIUM,
              mediumB: MEDIUM,
              high: armOf({ insertion: 17, alteration: 14, },),
            },);
            expect(verdict.said,).toBe(
              'VERDICT medium (inside-threshold): high has 11 more clean fixes than the mean of the two medium arms '
                + '(20), the threshold is 12, so the default effort stays',
            );
          },
        },),
      ],
    },),
  ],
},);
