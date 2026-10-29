/**
 Tests for the rule the editor fidelity bench is read by: which effort it picks from the trials every arm shared, by a
 one-sided exact paired permutation test of each effort against the default at 0.025, and which clause decided. The rule
 is code, and the line printed before any call is made from the same source.

 Trials are written two to an entry, the insertion then the alteration, as the frozen pool draws them. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ALPHA,
  BOOTSTRAP_REPLICATES,
  BOOTSTRAP_SEED,
  clusterBootstrapInterval,
  decideEffort,
  decisionRuleText,
  MIN_SHARED_TRIALS,
  type SharedTrial,
} from '../../dist/final/node/index.mjs';

/**
 Shared trials of an invented bench, two to an entry.

 @param count - how many shared trials
 @param high - whether the high effort fixed the trial at an index
 @param low - whether the low effort fixed it
 @param mediums - whether each medium arm fixed it, one function for each medium arm

 @returns Trials

 @example
 ```ts
 const trials = trialsOf({ count: 40, high: every, low: none, mediums: [none,], },);
 ```
 */
function trialsOf(
  {
    count,
    high,
    low,
    mediums,
  }: {
    readonly count: number;
    readonly high: (index: number,) => boolean;
    readonly low: (index: number,) => boolean;
    readonly mediums: readonly ((index: number,) => boolean)[];
  },
): readonly SharedTrial[] {
  return Array.from(
    { length: count, },
    function trialAt(
      _unused,
      index,
    ): SharedTrial {
      return {
        entryId: `cat-${String(Math.floor(index / 2,),)}`,
        kind: ((index % 2) === 0) ? 'insertion' : 'alteration',
        high: high(index,),
        low: low(index,),
        mediums: mediums.map(function fixedBy(medium,): boolean {
          return medium(index,);
        },),
      };
    },
  );
}

/**
 An arm that fixes every trial.
 */
function every(): boolean {
  return true;
}

/**
 An arm that fixes no trial.
 */
function none(): boolean {
  return false;
}

/**
 An arm that fixes the leading trials, as many as given.

 @param count - how many leading trials it fixes

 @returns Arm

 @example
 ```ts
 const arm = leading({ count: 25, },);
 ```
 */
function leading({ count, }: { readonly count: number; },): (index: number,) => boolean {
  return function fixes(index,): boolean {
    return index < count;
  };
}

await describe({
  name: 'editor fidelity decision rule',
  children: [
    describe({
      name: 'the constants',
      children: [
        it({
          name: 'FIX THE LEVEL AT 0.025 FOR EACH EFFORT, the interval at 95 percent over ten thousand resamples, and the '
            + 'least shared units at sixty, the figures the printed rule states',
          fn: async () => {
            expect(ALPHA,).toBe(0.025,);
            expect(BOOTSTRAP_REPLICATES,).toBe(10_000,);
            expect(MIN_SHARED_TRIALS,).toBe(60,);
          },
        },),
      ],
    },),

    describe({
      name: decisionRuleText.name,
      children: [
        it({
          name: 'STATES THE TEST, THE LEVEL, THE TIE-BREAK, WHAT A MEDIUM VERDICT MEANS, THE INTERVAL AND THE LEAST SHARED '
            + 'TRIALS, and what the insertion trials measure',
          fn: async () => {
            /**
             The rule as printed.
             */
            const rule = decisionRuleText();
            expect(rule,).toContain('one-sided exact paired permutation test',);
            expect(rule,).toContain('permuting the arm labels within each entry, with entries as the units',);
            expect(rule,).toContain('at 0.025 for each effort',);
            expect(rule,).toContain('When both efforts are shown better, the one with the larger estimate is chosen; when the estimates are equal, the cheaper effort is chosen',);
            expect(rule,).toContain('A medium verdict means no effort was shown better than medium, not that the others are as good',);
            expect(rule,).toContain('95 percent interval from a seeded bootstrap over entries',);
            expect(rule,).toContain(BOOTSTRAP_SEED,);
            expect(rule,).toContain('fewer than 60 shared units',);
            expect(rule,).toContain('a unit is a trial in one pass over the pool',);
            expect(rule,).toContain('a trial asked again joins the cluster of its entry',);
            expect(rule,).toContain('seeds edr7-sim and edr8-sim',);
            expect(rule,).toContain('at true fix rates from 0.30 to 0.90, within-entry correlations from 0 to 1',);
            expect(rule.includes('0.30 to 0.95',),).toBe(false,);
            expect(rule,).toContain('an agreement of a repeat with the arm\'s first answer from 0 to 1',);
            expect(rule,).toContain('scores as unusable for its arm',);
            expect(rule,).toContain('left out of every arm\'s comparison and counted',);
            expect(rule,).toContain('insertion trials measure locating an added sentence among its neighbours and copying the neighbours verbatim',);
            expect(rule,).toContain('not the span-only view production gives an editor',);
            expect(rule,).toContain('is at most 0.05 when every arm repeats itself alike',);
            expect(rule,).toContain('the worst of 192 cells was 0.055 at 800 replicates, within the noise of 0.008',);
            expect(rule,).toContain('and 0.107 when the default effort repeats itself far less than the others (a true fix rate of 0.9, three passes, 4000 replicates of the committed fixture)',);
            expect(rule.includes('up to 0.107',),).toBe(false,);
            expect(rule,).toContain('Both figures hold for fix rates up to 0.90 and no higher.',);
            expect(rule,).toContain('Above 0.90 the chance when the default effort repeats itself far less than the others was higher: the committed fixture read 0.108 at 0.91, 0.123 at 0.92, about 0.13 at 0.93 and 0.175 at 0.95 (0.108 at 0.91 and 0.123 at 0.92 at reviewer seed edrc6 and 8000 replicates; 0.130 at 0.93 at seed edr9-fx and 4000 replicates, and 0.132 at 0.93 at seed edrc6 and 8000 replicates; 0.175 at 0.95 at seed edr9-fx and 4000 replicates; each in three passes of 264 units with correlation 1), and an independent re-implementation of the model read 0.142 at 0.93, 0.173 at 0.95 and 0.241 at 0.97 (reviewer seed edrc5, 4000 replicates, 264 units).',);
            expect(rule,).toContain('prints WARN high fix rate when the default effort fixes 90 percent or more of the shared units, since the repeat warning can miss unequal repeats at fix rates this high and a verdict of high or low then stands under the caveat',);
            expect(rule.includes('93 percent',),).toBe(false,);
            expect(rule,).toContain('prints each arm\'s repeat agreement over the units every arm holds and warns when an exact one-sided test of the default effort\'s count of trials that read alike in every pass against that of the other arm with the highest share is at or under 0.05 divided by the number of other arms',);
            expect(rule,).toContain('which holds the chance of that warning at or under 0.05 when every arm repeats itself alike, counting the trials of an arm as independent draws',);
            expect(rule.includes('15 points',),).toBe(false,);
            expect(rule.includes('at most 0.055',),).toBe(false,);
            expect(rule,).toContain('about 26 points when medium fixes half',);
            expect(rule,).toContain('over 176 units (two passes) 25, 21 and 14 points where a repeat always gives the arm\'s first answer again',);
            expect(rule.includes('\n',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: decideEffort.name,
      children: [
        it({
          name: 'KEEPS MEDIUM WHEN NO EFFORT IS SHOWN BETTER, arms that fix alike, and says that is not a finding that the '
            + 'others are as good',
          fn: async () => {
            const verdict = decideEffort({
              trials: trialsOf({ count: 60, high: leading({ count: 30, },), low: leading({ count: 30, },), mediums: [leading({ count: 30, },),], },),
            },);
            expect(verdict.verdict,).toBe('medium',);
            expect(verdict.clause,).toBe('none-shown',);
            expect(verdict.high.shown,).toBe(false,);
            expect(verdict.low.shown,).toBe(false,);
            expect(verdict.high.estimate,).toBe(0,);
            expect(verdict.said.startsWith('VERDICT medium (none-shown): ',),).toBe(true,);
            expect(verdict.said,).toContain('does not show the others are as good',);
          },
        },),

        it({
          name: 'PICKS HIGH WHEN ONLY HIGH IS SHOWN BETTER, with its estimate the paired difference per shared trial: '
            + 'high fixes all sixty trials, medium thirty, low the same thirty',
          fn: async () => {
            const verdict = decideEffort({
              trials: trialsOf({ count: 60, high: every, low: leading({ count: 30, },), mediums: [leading({ count: 30, },),], },),
            },);
            expect(verdict.verdict,).toBe('high',);
            expect(verdict.clause,).toBe('shown-better',);
            expect(verdict.high.shown,).toBe(true,);
            expect(verdict.high.estimate,).toBeCloseTo(0.5, 10,);
            expect(verdict.low.shown,).toBe(false,);
            expect(verdict.trials,).toBe(60,);
            expect(verdict.entries,).toBe(30,);
          },
        },),

        it({
          name: 'PICKS LOW WHEN ONLY LOW IS SHOWN BETTER, since the rule tests each effort against medium and not high '
            + 'against low',
          fn: async () => {
            const verdict = decideEffort({
              trials: trialsOf({ count: 60, high: none, low: every, mediums: [leading({ count: 30, },),], },),
            },);
            expect(verdict.verdict,).toBe('low',);
            expect(verdict.clause,).toBe('shown-better',);
            expect(verdict.low.estimate,).toBeCloseTo(0.5, 10,);
            expect(verdict.high.estimate,).toBeCloseTo(-0.5, 10,);
          },
        },),

        it({
          name: 'PICKS THE LARGER ESTIMATE WHEN BOTH EFFORTS ARE SHOWN BETTER: medium fixes none, low twenty-five of forty '
            + 'and high thirty-five, so high is chosen by the larger estimate',
          fn: async () => {
            const verdict = decideEffort({
              trials: trialsOf({ count: 40, high: leading({ count: 35, },), low: leading({ count: 25, },), mediums: [none,], },),
            },);
            expect(verdict.low.shown,).toBe(true,);
            expect(verdict.high.shown,).toBe(true,);
            expect(verdict.verdict,).toBe('high',);
            expect(verdict.clause,).toBe('larger-estimate',);
            expect(verdict.said,).toContain('larger estimate',);
          },
        },),

        it({
          name: 'PICKS THE CHEAPER EFFORT WHEN BOTH ARE SHOWN BETTER AND THE ESTIMATES ARE EQUAL: low and high fix the same '
            + 'twenty-five of forty trials, so low is chosen',
          fn: async () => {
            const verdict = decideEffort({
              trials: trialsOf({ count: 40, high: leading({ count: 25, },), low: leading({ count: 25, },), mediums: [none,], },),
            },);
            expect(verdict.verdict,).toBe('low',);
            expect(verdict.clause,).toBe('cheaper-on-equal',);
            expect(verdict.said,).toContain('cheaper',);
          },
        },),

        it({
          name: 'READS TWO MEDIUM ARMS AS ONE CONTROL: the estimate is the effort\'s fixes minus the mean of the two mediums\', '
            + 'high fixing all forty, one medium twenty and the other ten',
          fn: async () => {
            const verdict = decideEffort({
              trials: trialsOf({
                count: 40,
                high: every,
                low: none,
                mediums: [leading({ count: 20, },), leading({ count: 10, },),],
              },),
            },);
            expect(verdict.mediumArms,).toBe(2,);
            expect(verdict.high.estimate,).toBeCloseTo((40 - 15) / 40, 10,);
            expect(verdict.high.shown,).toBe(true,);
            expect(verdict.verdict,).toBe('high',);
          },
        },),

        it({
          name: 'GIVES THE INTERVAL OF THE SEEDED CLUSTER BOOTSTRAP over the entries, the one clusterBootstrapInterval '
            + 'gives for the same entries and seed, so a reader can reproduce it',
          fn: async () => {
            const trials = trialsOf({ count: 60, high: every, low: leading({ count: 30, },), mediums: [leading({ count: 30, },),], },);
            const verdict = decideEffort({ trials, },);
            /**
             The interval of the same entries by the module itself.
             */
            const expected = clusterBootstrapInterval({
              clusters: Array.from(
                { length: 30, },
                function entryAt(
                  _unused,
                  at,
                ) {
                  return {
                    entryId: `cat-${String(at,)}`,
                    trials: 2,
                    fixed: [2, at < 15 ? 2 : 0,],
                  };
                },
              ),
              seed: BOOTSTRAP_SEED,
              replicates: BOOTSTRAP_REPLICATES,
            },);
            expect(verdict.high.interval,).toStrictEqual(expected,);
          },
        },),

        it({
          name: 'PRINTS EACH KIND\'S PAIRED ESTIMATE AS A SECONDARY LINE and lets no kind veto: high gains overall and falls '
            + 'on the alteration trials, and is still chosen',
          fn: async () => {
            /**
             High fixes every insertion and no alteration; medium fixes every other alteration and no insertion.
             */
            const verdict = decideEffort({
              trials: trialsOf({
                count: 120,
                high: function highFixes(index,): boolean {
                  return (index % 2) === 0;
                },
                low: none,
                mediums: [
                  function mediumFixes(index,): boolean {
                    return (index % 4) === 3;
                  },
                ],
              },),
            },);
            expect(verdict.verdict,).toBe('high',);
            expect(verdict.high.kinds.insertion.estimate,).toBeCloseTo(1, 10,);
            expect(verdict.high.kinds.alteration.estimate,).toBeCloseTo(-0.5, 10,);
            expect(verdict.high.kinds.insertion.trials,).toBe(60,);
            expect(verdict.lines.some(function isKinds(line,): boolean {
              return line.startsWith('KINDS high vs medium (secondary, no veto): insertion +100.0 points over 60 units, alteration -50.0 points over 60 units',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'PRINTS A PAIRED LINE FOR EACH EFFORT with the estimate in points, the interval, the p-value, the verdict at '
            + '0.025 and the shared trials, and the verdict last',
          fn: async () => {
            const verdict = decideEffort({
              trials: trialsOf({ count: 60, high: every, low: leading({ count: 30, },), mediums: [leading({ count: 30, },),], },),
            },);
            expect(verdict.lines.at(-1,),).toBe(verdict.said,);
            /**
             The line for the high effort.
             */
            const high = verdict.lines.find(function isHigh(line,): boolean {
              return line.startsWith('PAIRED high vs medium: ',);
            },);
            expect(high,).toContain('+50.0 points (95% interval +',);
            expect(high,).toContain('seeded bootstrap over 30 entries)',);
            expect(high,).toContain('p under 0.0001',);
            expect(high,).toContain('shown better at 0.025: yes',);
            expect(high,).toContain('60 shared units',);
            /**
             The line for the low effort, whose estimate is zero.
             */
            const low = verdict.lines.find(function isLow(line,): boolean {
              return line.startsWith('PAIRED low vs medium: ',);
            },);
            expect(low,).toContain('+0.0 points',);
            expect(low,).toContain('shown better at 0.025: no',);
          },
        },),
      ],
    },),
  ],
},);
