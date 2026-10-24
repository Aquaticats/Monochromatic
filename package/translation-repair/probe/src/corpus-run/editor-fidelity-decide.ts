import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { countWithNoun, } from './editor-fidelity-arm-facts.ts';
import {
  decidedOf,
  kindsLine,
  pairedLine,
} from './editor-fidelity-decide-lines.ts';
import type {
  EffortReading,
  EffortVerdict,
  KindEstimate,
  SharedTrial,
} from './editor-fidelity-decide-types.ts';
import {
  ALPHA,
  BOOTSTRAP_REPLICATES,
  BOOTSTRAP_SEED,
  clusterBootstrapInterval,
  type PairedCluster,
  pairedPermutationTest,
} from './editor-fidelity-paired.ts';

//region Editor fidelity decision rule
// THE RULE THE ARMS OF THE EDITOR FIDELITY BENCH ARE READ BY, as code. It is applied by the read step to the trials every
// arm shared, and the line printed before any call is made from the same text, so a log shows the rule it will be read by
// and the verdict cannot be reached by another one.
//
// WHAT IS COMPARED. Each of the low and the high effort against the default (medium), trial by trial: a trial is a clean
// fix for an arm when its reading is exact or equivalent and no ask of it ended at the output-token cap. The efforts are
// tested by a one-sided exact paired permutation test, permuting the arm labels within each entry with entries as the
// units (two trials of one entry are not two independent draws), at 0.025 for each effort so that the chance of choosing
// another effort when there is none to find stays under 0.05 for the pair. The test, its interval and the simulation that
// bounds its false picks and states its power are `editor-fidelity-paired.ts` and
// `editor-fidelity-paired-null.test-fixture.ts`.
//
// WHAT IS PICKED. An effort is picked only when its test is at or under 0.025. When both are, the larger estimate wins;
// when the estimates are equal, the cheaper effort. A medium verdict means no effort was shown better, which is not a
// finding that the others are as good. The per-kind estimates are printed and veto nothing.
//
// WHAT THE INSERTION TRIALS MEASURE. Their region is widened by the neighbouring sentence on each side (so that deleting
// the quoted region does not read as a clean fix), which makes the trial one of locating an added sentence among its
// neighbours and copying the neighbours verbatim. Production's editor sees the span only. The rule says so, because a
// reader of the verdict should not take the insertion count for production's.

/**
 The fewest units every arm must share for the read step to give a verdict, a unit being a trial in one pass over the
 pool: with fewer, a difference the size of the smallest this bench can tell apart would not be told apart. The
 simulation's smallest difference chosen with chance 0.8 is 30 points at 60 units when medium fixes half and 24 points
 when it fixes seven in ten, against 37 and 28 points at 40 units, and at 60 units it is not reached by 14 points when
 medium fixes eighty-five in a hundred, as at 40.
 */
export const MIN_SHARED_TRIALS = 60;

/**
 The worst chance of choosing another effort when none is better that the simulation measured over the committed pool
 walked in one to three passes, at true fix rates from 0.30 to 0.95, within-entry correlations from 0 to 1 and
 agreements of a repeat with the arm's first answer from 0 to 1. Each cell ran 1000 replicates, so a reading is about
 0.007 either way, and the test's own bound is 0.05 because each of the two efforts is tested at 0.025 exactly.
 */
const FALSE_PICK_WORST = 0.055;

/**
 The efforts the rule tests against the default.
 */
const CHALLENGERS = [
  'high',
  'low',
] as const;

/**
 The decision rule as the line before any call prints it, after its `RULE ` marker.

 @returns The rule in one line

 @example
 ```ts
 log.info(`RULE ${decisionRuleText()}`,);
 ```
 */
export function decisionRuleText(): string {
  return 'clean fix is exact plus equivalent, counted trial by trial. A trial in which an ask ended at the output-token cap '
    + 'scores as unusable for its arm, and a unit in which an ask of any arm threw is left out of every arm\'s comparison '
    + 'and counted, where a unit is a trial in one pass over the pool (an arm may walk the pool several times, pass by '
    + 'pass). The arms are compared over the units every arm completed, which are the leading units of one fixed seeded order, '
    + 'grouped by the entry each comes from, so a trial asked again joins the cluster of its entry. Each of the low and '
    + 'the high effort is tested against the '
    + 'default (medium) by a one-sided exact paired permutation test, permuting the arm labels within each entry, with '
    + `entries as the units, at ${String(ALPHA,)} for each effort (a second medium arm, if one is run, joins the labels `
    + 'and is read as one control). The paired difference is printed per shared unit with a 95 percent interval from a '
    + `seeded bootstrap over entries (${String(BOOTSTRAP_REPLICATES,)} resamples, seed ${BOOTSTRAP_SEED}) and the exact `
    + `p-value. An effort other than the default is chosen only when its test is at or under ${String(ALPHA,)}. When both `
    + 'efforts are shown better, the one with the larger estimate is chosen; when the estimates are equal, the cheaper '
    + 'effort is chosen. A medium verdict means no effort was shown better than medium, not that the others are as good. '
    + 'The estimates for each kind of defect (insertion, alteration) are printed beside it and veto nothing. With fewer '
    + `than ${
      countWithNoun({
        count: MIN_SHARED_TRIALS,
        one: 'shared unit',
        many: 'shared units',
      },)
    } the read step refuses to give a verdict. By simulation over the `
    + 'committed pool (editor-fidelity-paired-null.test-fixture.ts, seed edr7-sim), at true fix rates from 0.30 to 0.95, '
    + 'within-entry correlations from 0 to 1 and an agreement of a repeat with the arm\'s first answer from 0 to 1, the '
    + `chance of choosing another effort when none is better is at most ${String(FALSE_PICK_WORST,)}; over the 88 trials `
    + 'walked once the smallest difference chosen with chance 0.8 is about 26 points when medium fixes half, 20 points '
    + 'when it fixes seven in ten and 14 points when it fixes eighty-five in a hundred, and over 176 units (two passes) '
    + '25, 21 and 14 points where a repeat always gives the arm\'s first answer again, 21, 17 and 12 points where it does '
    + 'so half the time and 17, 14 and 10 points where it never does, and a smaller difference cannot be told apart. The '
    + 'insertion trials measure locating an added sentence among its '
    + 'neighbours and copying the neighbours verbatim, not the span-only view production gives an editor.';
}

/**
 Groups trials into the entries the test permutes within.

 @param trials - the shared trials

 @param effort - the effort under test, placed first among the arms

 @param kind - the kind of defect to keep, every kind when omitted

 @returns One cluster for each entry, in the order the entries first appear

 @example
 ```ts
 const clusters = clustersOf({ trials, effort: 'high', },);
 ```
 */
function clustersOf(
  {
    trials,
    effort,
    kind,
  }: {
    readonly trials: readonly SharedTrial[];
    readonly effort: 'low' | 'high';
    readonly kind?: SharedTrial['kind'];
  },
): readonly PairedCluster[] {
  /**
   The trials kept, grouped by entry.
   */
  const byEntry = Map.groupBy(
    trials.filter(function isKept(trial,): boolean {
      return (kind === undefined) || (trial.kind === kind);
    },),
    function entryOf({ entryId, },): string {
      return entryId;
    },
  );
  return Array.from(
    byEntry.keys(),
    function clusterOf(entryId,): PairedCluster {
      /**
       The entry's trials.
       */
      const group = nonNullishOrThrow(byEntry.get(entryId,),);
      /**
       The arms of each trial, the effort under test first.
       */
      const arms = group.map(function armsOf(trial,): readonly boolean[] {
        return [
          trial[effort],
          ...trial.mediums,
        ];
      },);
      return {
        entryId,
        trials: group.length,
        fixed: Array.from(
          {
            length: arms[0]
              ?.length
              ?? 0,
          },
          function fixedBy(
            _unused,
            armAt,
          ): number {
            return arms.filter(function fixes(trialArms,): boolean {
              return trialArms[armAt] === true;
            },)
              .length;
          },
        ),
      };
    },
  );
}

/**
 The paired estimate of an effort over the trials of one kind.

 @param trials - the shared trials

 @param effort - the effort under test

 @param kind - the kind of defect

 @returns The estimate and the number of trials of the kind

 @example
 ```ts
 const kind = kindEstimateOf({ trials, effort: 'high', kind: 'insertion', },);
 ```
 */
function kindEstimateOf(
  {
    trials,
    effort,
    kind,
  }: {
    readonly trials: readonly SharedTrial[];
    readonly effort: 'low' | 'high';
    readonly kind: SharedTrial['kind'];
  },
): KindEstimate {
  /**
   The test over the entries' trials of the kind, read for its estimate alone.
   */
  const test = pairedPermutationTest({
    clusters: clustersOf({
      trials,
      effort,
      kind,
    },),
  },);
  return {
    estimate: test.estimate,
    trials: test.trials,
  };
}

/**
 Reads one effort against the default.

 @param trials - the shared trials

 @param effort - the effort

 @returns The estimate, its interval, the exact p-value, whether it is shown better and the estimate for each kind

 @example
 ```ts
 const reading = readEffort({ trials, effort: 'high', },);
 ```
 */
function readEffort(
  {
    trials,
    effort,
  }: {
    readonly trials: readonly SharedTrial[];
    readonly effort: 'low' | 'high';
  },
): EffortReading {
  /**
   The entries, the effort first.
   */
  const clusters = clustersOf({
    trials,
    effort,
  },);
  /**
   The exact test.
   */
  const test = pairedPermutationTest({ clusters, },);
  return {
    effort,
    estimate: test.estimate,
    interval: clusterBootstrapInterval({
      clusters,
      seed: BOOTSTRAP_SEED,
      replicates: BOOTSTRAP_REPLICATES,
    },),
    p: test.p,
    shown: test.atMostAlpha,
    kinds: {
      insertion: kindEstimateOf({
        trials,
        effort,
        kind: 'insertion',
      },),
      alteration: kindEstimateOf({
        trials,
        effort,
        kind: 'alteration',
      },),
    },
  };
}

/**
 Applies the rule to the shared trials of the arms.

 @param trials - the trials every arm completed and none threw on, each with whether every arm fixed it cleanly

 @returns The effort picked, the clause that decided, how each effort stands and the lines the read step prints

 @example
 ```ts
 const verdict = decideEffort({ trials, },);
 ```
 */
export function decideEffort({ trials, }: { readonly trials: readonly SharedTrial[]; },): EffortVerdict {
  /**
   The two efforts read against the default.
   */
  const readings = {
    high: readEffort({
      trials,
      effort: 'high',
    },),
    low: readEffort({
      trials,
      effort: 'low',
    },),
  };
  /**
   Entries the trials come from.
   */
  const entries = new Set(trials.map(function entryOf({ entryId, },): string {
    return entryId;
  },),).size;
  /**
   The efforts shown better, the high effort first.
   */
  const shown = CHALLENGERS.filter(function isShown(effort,): boolean {
    return readings[effort]
      .shown;
  },);
  /**
   The effort picked, the clause that decided and the sentence that says so.
   */
  const decided = decidedOf({
    readings,
    shown,
  },);
  return {
    verdict: decided.verdict,
    clause: decided.clause,
    trials: trials.length,
    entries,
    mediumArms: trials[0]
      ?.mediums
      .length
      ?? 0,
    low: readings.low,
    high: readings.high,
    lines: [
      pairedLine({
        reading: readings.high,
        trials: trials.length,
        entries,
      },),
      kindsLine({ reading: readings.high, },),
      pairedLine({
        reading: readings.low,
        trials: trials.length,
        entries,
      },),
      kindsLine({ reading: readings.low, },),
      decided.said,
    ],
    said: decided.said,
  };
}

//endregion Editor fidelity decision rule
