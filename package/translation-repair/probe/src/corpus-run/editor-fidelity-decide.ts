import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { countWithNoun, } from './editor-fidelity-arm-facts.ts';
import { REPEAT_ALPHA, } from './editor-fidelity-arm-repeat-test.ts';
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
 The bound on the chance of choosing another effort when none is better, which holds when every arm repeats itself alike
 and the true fix rate is no higher than 0.9: each of the two efforts is tested at 0.025 exactly, and the simulation over
 the committed pool walked in one to three passes, at true fix rates from 0.30 to 0.90 and within-entry correlations from
 0 to 1, read 0.055 at the worst of 192 cells at 800 replicates (noise 0.008) and 0.035 at most for the four cells over
 0.05 re-run at 4000. Above a fix rate of 0.9 the grid did not go.
 */
export const FALSE_PICK_BOUND_ALIKE = 0.05;

/**
 The chance of choosing another effort when none is better that the committed fixture read when the arms repeat
 themselves unlike one another: the default effort asked afresh in each pass and the others repeating their first
 answer, at a true fix rate of 0.9 over three passes of 264 units, at 4000 replicates (0.098 at the same cell of a full
 grid re-run at 4000). It is one cell's reading at a fix rate of 0.9, not a ceiling: the same model read higher above
 0.9 (see `FALSE_PICK_READINGS_ABOVE_NINE_TENTHS`).
 */
export const FALSE_PICK_UNEQUAL_REACHED = 0.107;

/**
 The chance of choosing another effort when none is better that the committed fixture read at true fix rates above 0.9,
 when the default effort repeats itself far less than the others (high and low repeating their first answer, the default
 effort asked afresh in each pass, three passes of 264 units, within-entry correlation 1): 0.108 at 0.91 and 0.123 at
 0.92 (the second reviewer's run of the fixture's own functions, seed prefix edrc6, 8000 replicates each), 0.130 at 0.93
 (seed edr9-fx, 4000 replicates; 0.132 at seed edrc6, 8000 replicates, so about 0.13) and 0.175 at 0.95 (seed edr9-fx,
 4000 replicates), against 0.1072 at 0.9 under the round 8 seed. At a correlation of 0.5 the fixture read 0.0975 at 0.93
 and 0.1365 at 0.95. The independent re-implementation of the model read other figures, listed in
 `FALSE_PICK_REIMPLEMENTATION_READINGS`.
 */
export const FALSE_PICK_READINGS_ABOVE_NINE_TENTHS = '0.108 at 0.91, 0.123 at 0.92, about 0.13 at 0.93 and 0.175 at 0.95';

/**
 The chance of choosing another effort when none is better that an independent re-implementation of the model read at
 fix rates above 0.9, in the same bench (re-check 5, seed prefix edrc5, 4000 replicates, three passes of 264 units,
 within-entry correlation 1). It is a second source and its readings are kept apart from the fixture's, which read
 0.130 at 0.93 and 0.175 at 0.95 in the same cells.
 */
export const FALSE_PICK_REIMPLEMENTATION_READINGS = '0.142 at 0.93, 0.173 at 0.95 and 0.241 at 0.97';

/**
 The share of the units every arm holds, in percent, that the default effort cleanly fixes at or above which the read
 step prints `WARN high fix rate`. It is read on the share observed, and it is 90 so that the region the rule states its
 bounds for (true fix rates 0.30 to 0.90) and the warning meet: a threshold of 93 left a true rate of 0.91 or 0.92 with
 neither a stated bound nor a warning, where the fixture read the false pick chance 0.108 and 0.123. The repeat warning
 compares measured repeat agreements, and an arm that gives a fresh reading in each pass reads three passes alike with
 chance r^3 + (1-r)^3 for a fix rate r, so its expected agreement is 73 percent at 0.9, 80.5 at 0.93, 85.7 at 0.95 and
 91.3 at 0.97 against 100 for an arm that always repeats: a gap of 27, 19.5, 14.3 and 8.7 points that the exact test of
 the repeat warning must see in a limited number of trials. The measured rates at which each warning fires are in the
 README and the ledger.
 */
export const DEFAULT_FIX_RATE_WARNED_AT_PERCENT = 90;

/**
 The efforts the rule tests against the default.
 */
const CHALLENGERS = [
  'high',
  'low',
] as const;

/**
 The decision rule as the line before any call prints it, after its `RULE ` marker.

 @returns Rule in one line

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
    + 'committed pool (editor-fidelity-paired-null.test-fixture.ts, seeds edr7-sim and edr8-sim), at true fix rates from '
    + '0.30 to 0.90, within-entry correlations from 0 to 1 and an agreement of a repeat with the arm\'s first answer from '
    + '0 to 1, the chance of choosing another effort when none is better is at most '
    + `${String(FALSE_PICK_BOUND_ALIKE,)} when every arm repeats itself alike (the worst of 192 cells was 0.055 at 800 `
    + 'replicates, within the noise of 0.008, and the four cells over 0.05 were at most 0.035 at 4000), and '
    + `${String(FALSE_PICK_UNEQUAL_REACHED,)} when the default effort repeats itself far less than the others (a true fix `
    + 'rate of 0.9, three passes, 4000 replicates of the committed fixture). Both figures hold for fix rates up to 0.90 and '
    + 'no higher. Above 0.90 the chance when the default effort repeats itself far less than the others was higher: the '
    + `committed fixture read ${FALSE_PICK_READINGS_ABOVE_NINE_TENTHS} (0.108 at 0.91 and 0.123 at 0.92 at reviewer seed `
    + 'edrc6 and 8000 replicates; 0.130 at 0.93 at seed edr9-fx and 4000 replicates, and 0.132 at 0.93 at seed edrc6 and '
    + '8000 replicates; 0.175 at 0.95 at seed edr9-fx and 4000 replicates; each in three passes of 264 units with '
    + 'correlation 1), and an independent re-implementation of the model read '
    + `${FALSE_PICK_REIMPLEMENTATION_READINGS} (reviewer seed edrc5, 4000 replicates, 264 units). The read step prints each `
    + 'arm\'s repeat agreement over the '
    + 'units every arm holds and warns when an exact one-sided test of the default effort\'s count of trials that read '
    + 'alike in every pass against that of the other arm with the highest share is at or under '
    + `${String(REPEAT_ALPHA,)} divided by the number of other arms, which holds the chance of that warning at or under `
    + `${String(REPEAT_ALPHA,)} when every arm repeats itself alike, counting the trials of an arm as independent draws, a `
    + 'warning relied on for fix rates up to 0.90, and '
    + `prints WARN high fix rate when the default effort fixes ${String(DEFAULT_FIX_RATE_WARNED_AT_PERCENT,)} percent or `
    + 'more of the shared units, since the repeat warning can miss unequal repeats at fix rates this high and a verdict of '
    + 'high or low then stands under the caveat. Over the 88 trials '
    + 'walked once the smallest difference chosen with chance 0.8 is about 26 points when medium fixes half, 20 points '
    + 'when it fixes seven in ten and 14 points when it fixes eighty-five in a hundred, and over 176 units (two passes) '
    + '25, 21 and 14 points where a repeat always gives the arm\'s first answer again, 21, 17 and 12 points where it does '
    + 'so half the time and 17, 14 and 10 points where it never does, and a smaller difference cannot be told apart. The '
    + 'insertion trials measure locating an added sentence among its '
    + 'neighbours and copying the neighbours verbatim, not the span-only view production gives an editor.';
}

/**
 Groups trials into the entries the test permutes within.

 @param trials - shared trials

 @param effort - effort under test, placed first among the arms

 @param kind - kind of defect to keep, every kind when omitted

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

 @param trials - shared trials

 @param effort - effort under test

 @param kind - kind of defect

 @returns Estimate and number of trials of the kind

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

 @param trials - shared trials

 @param effort - effort

 @returns Estimate, its interval, the exact p-value, whether it is shown better and the estimate for each kind

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

 @param trials - trials every arm completed and none threw on, each with whether every arm fixed it cleanly

 @returns Effort picked, clause that decided, how each effort stands and lines the read step prints

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
