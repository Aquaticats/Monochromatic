//region Editor fidelity decision rule
// THE RULE THE ARMS OF THE EDITOR FIDELITY BENCH ARE READ BY, as code. It is applied by the read step to the counts the
// logs state, and the line printed before any call is made from the same text, so a log shows the rule it will be read
// by and the verdict cannot be reached by another one.
//
// WHAT IS COMPARED. An effort's clean fix count (exact plus equivalent, over the trials of both kinds of defect) against
// the mean of the two medium arms, which are one card run twice. The gap counts when it reaches the threshold: the
// larger of a floor and the two medium arms' own difference. The floor is the smallest gap that keeps the chance of
// choosing another effort under 0.05 when there is none to find, at every true fix rate from 0.30 to 0.95, by the exact
// enumeration in `editor-fidelity-decide-null.ts` whose unit test pins it. The mediums' own difference raises it when
// two runs of one card disagree by more.
//
// WHAT A KIND CAN VETO. An effort that gains overall but falls more than a tolerance below the mean of the mediums on
// one kind of defect has not separated: it moved the other way on that kind. The tolerance is three trials, about one
// standard deviation of a count out of twenty. It can only turn a pick into a keep, so it lowers the chance of choosing
// another effort, and the enumeration (which leaves it out) bounds that chance from above.
//
// WHAT IS KEPT. A smaller gap, a tie, an effort that fixes fewer, and a gap a kind contradicts all keep the default
// effort (medium). The rule never picks an effort for being cheaper: the pick is made on the gap alone.
//
// WHAT THE INSERTION TRIALS MEASURE. Their region is widened by the neighbouring sentence on each side (so that deleting
// the quoted region does not read as a clean fix), which makes the trial one of locating an added sentence among its
// neighbours and copying the neighbours verbatim. Production's editor sees the span only. The rule says so, because a
// reader of the verdict should not take the insertion count for production's.

/**
 The least pooled gap that counts, in trials: the smallest floor the enumeration in `editor-fidelity-decide-null.ts`
 holds the chance of choosing another effort under 0.05 for, at every true fix rate from 0.30 to 0.95 and every
 correlation between the two trials of an entry.
 */
export const POOLED_FLOOR_TRIALS = 12;

/**
 How many trials below the mean of the mediums a kind of defect may fall before it vetoes a gain: about one standard
 deviation of a count out of twenty.
 */
export const KIND_TOLERANCE_TRIALS = 3;

/**
 The decision rule as the line before any call prints it, after its `RULE ` marker.

 @returns The rule in one line

 @example
 ```ts
 log.info(`RULE ${decisionRuleText()}`,);
 ```
 */
export function decisionRuleText(): string {
  return 'clean fix is exact plus equivalent, counted over the trials of both kinds of defect. An effort other than the '
    + 'default (medium) is chosen only when its clean fix count is above the mean of the two medium arms by at least '
    + `${String(POOLED_FLOOR_TRIALS,)} trials, or by at least the two medium arms' own difference when that is `
    + `larger, and neither kind of defect (insertion, alteration) is more than ${String(KIND_TOLERANCE_TRIALS,)} `
    + 'trials below the mean of the two medium arms for that kind. A smaller gap, a tie, an effort that fixes fewer, and '
    + 'a gap a kind contradicts all keep the default effort. With no true difference, a gap of '
    + `${String(POOLED_FLOOR_TRIALS,)} trials is the least that keeps the chance of choosing another effort under 0.05 `
    + 'at every true fix rate from 0.30 to 0.95, counting the two trials of an entry as one draw at the worst '
    + '(editor-fidelity-decide-null.ts), so the bench cannot separate efforts that differ by less. The insertion '
    + 'trials measure locating an added sentence among its neighbours and copying the neighbours verbatim, not the '
    + 'span-only view production gives an editor.';
}

/**
 How many trials of one kind of defect an arm ran and fixed cleanly.

 @example
 ```ts
 const counts: KindCount = { trials: 22, fixed: 18, };
 ```
 */
export type KindCount = {
  /**
   Trials of the kind.
   */
  readonly trials: number;

  /**
   Trials fixed cleanly, exact plus equivalent.
   */
  readonly fixed: number;
};

/**
 The clean fix counts of an arm for each kind of defect.

 @example
 ```ts
 const arm: ArmCount = { insertion: { trials: 22, fixed: 18, }, alteration: { trials: 18, fixed: 12, }, };
 ```
 */
export type ArmCount = {
  /**
   The insertion trials.
   */
  readonly insertion: KindCount;

  /**
   The alteration trials.
   */
  readonly alteration: KindCount;
};

/**
 Which clause of the rule decided.
 */
export type DecidingClause =
  | 'tie'
    | 'inside-threshold'
    | 'fixes-fewer'
    | 'kind-veto'
    | 'beats-medium';

/**
 How one effort stands against the two medium arms.

 @example
 ```ts
 const reading: EffortReading = { effort: 'high', gap: 12, clause: 'beats-medium', kindShortfall: '', };
 ```
 */
export type EffortReading = {
  /**
   The effort read.
   */
  readonly effort: 'low' | 'high';

  /**
   Its clean fix count minus the mean of the two medium arms', in trials.
   */
  readonly gap: number;

  /**
   The clause that decided for this effort.
   */
  readonly clause: DecidingClause;

  /**
   The kind of defect that fell more than the tolerance below the mediums where the clause is a veto, else empty.
   */
  readonly kindShortfall: '' | 'insertion' | 'alteration';
};

/**
 What the rule decided over the four arms.

 @example
 ```ts
 const verdict = decideEffort({ low, mediumA, mediumB, high, },);
 ```
 */
export type EffortVerdict = {
  /**
   The effort the rule picks.
   */
  readonly verdict: 'low' | 'medium' | 'high';

  /**
   The clause that decided it: that of the effort picked, else that of the effort furthest from the mediums.
   */
  readonly clause: DecidingClause;

  /**
   The threshold a gap had to reach, in trials.
   */
  readonly thresholdTrials: number;

  /**
   The mean of the two medium arms' clean fix counts.
   */
  readonly mediumMean: number;

  /**
   How the low effort stands.
   */
  readonly low: EffortReading;

  /**
   How the high effort stands.
   */
  readonly high: EffortReading;

  /**
   The verdict as the read step prints it.
   */
  readonly said: string;
};

/**
 The clean fix count of an arm over both kinds.

 @param arm - the arm's counts

 @returns Insertion plus alteration clean fixes

 @example
 ```ts
 // yields 30
 const fixed = pooledFixed({ arm: { insertion: { trials: 22, fixed: 18, }, alteration: { trials: 18, fixed: 12, }, }, },);
 ```
 */
function pooledFixed({ arm, }: { readonly arm: ArmCount; },): number {
  return arm.insertion
    .fixed
    + arm.alteration
      .fixed;
}

/**
 Reads one effort against the two medium arms.

 @param effort - the effort
 
 @param arm - its counts
 
 @param mediumA - the first medium arm's counts
 
 @param mediumB - the second medium arm's counts
 
 @param thresholdTrials - the gap that counts

 @returns The effort's gap, the clause that decided for it and the kind that vetoed it, if one did

 @example
 ```ts
 const reading = readEffort({ effort: 'high', arm, mediumA, mediumB, thresholdTrials: 12, },);
 ```
 */
function readEffort(
  {
    effort,
    arm,
    mediumA,
    mediumB,
    thresholdTrials,
  }: {
    readonly effort: 'low' | 'high';
    readonly arm: ArmCount;
    readonly mediumA: ArmCount;
    readonly mediumB: ArmCount;
    readonly thresholdTrials: number;
  },
): EffortReading {
  /**
   The gap in trials, a half when the mediums' counts add to an odd number.
   */
  const gap = pooledFixed({ arm, },) - ((pooledFixed({ arm: mediumA, },) + pooledFixed({ arm: mediumB, },)) / 2);
  if (gap === 0) {
    return {
      effort,
      gap,
      clause: 'tie',
      kindShortfall: '',
    };
  }
  if (gap <= (-thresholdTrials)) {
    return {
      effort,
      gap,
      clause: 'fixes-fewer',
      kindShortfall: '',
    };
  }
  if (gap < thresholdTrials) {
    return {
      effort,
      gap,
      clause: 'inside-threshold',
      kindShortfall: '',
    };
  }
  /**
   How far the effort stands below the mediums' mean on each kind, positive when it stands below.
   */
  const below = {
    insertion: ((mediumA.insertion
      .fixed
      + mediumB.insertion
        .fixed) / 2)
      - arm.insertion
        .fixed,
    alteration: ((mediumA.alteration
      .fixed
      + mediumB.alteration
        .fixed) / 2)
      - arm.alteration
        .fixed,
  };
  if (below.insertion > KIND_TOLERANCE_TRIALS) {
    return {
      effort,
      gap,
      clause: 'kind-veto',
      kindShortfall: 'insertion',
    };
  }
  if (below.alteration > KIND_TOLERANCE_TRIALS) {
    return {
      effort,
      gap,
      clause: 'kind-veto',
      kindShortfall: 'alteration',
    };
  }
  return {
    effort,
    gap,
    clause: 'beats-medium',
    kindShortfall: '',
  };
}

/**
 Says what the rule decided, for the line the read step prints.

 @param verdict - the effort picked
 
 @param reading - the reading the verdict is told by
 
 @param mediumMean - the mean of the two medium arms' counts
 
 @param thresholdTrials - the gap that counts

 @returns The sentence

 @example
 ```ts
 const said = verdictSaid({ verdict: 'medium', reading, mediumMean: 20, thresholdTrials: 12, },);
 ```
 */
function verdictSaid(
  {
    verdict,
    reading,
    mediumMean,
    thresholdTrials,
  }: {
    readonly verdict: EffortVerdict['verdict'];
    readonly reading: EffortReading;
    readonly mediumMean: number;
    readonly thresholdTrials: number;
  },
): string {
  /**
   What follows from the clause.
   */
  const tail = (reading.clause === 'beats-medium')
    ? `${verdict} is chosen`
    : (reading.clause === 'kind-veto')
      ? `the ${reading.kindShortfall} trials fall more than ${String(KIND_TOLERANCE_TRIALS,)} below the mediums, so the default effort stays`
      : 'the default effort stays';
  return `VERDICT ${verdict} (${reading.clause}): ${reading.effort} has ${String(Math.abs(reading.gap,),)} ${
    (reading.gap < 0) ? 'fewer' : 'more'
  } clean fixes than the mean of the two medium arms (${String(mediumMean,)}), the threshold is ${
    String(thresholdTrials,)
  }, ${(reading.clause === 'beats-medium') ? 'and ' : 'so '}${tail}`;
}

/**
 Applies the rule to the four arms.

 @param low - the low effort's counts
 
 @param mediumA - the first medium arm's counts
 
 @param mediumB - the second medium arm's counts
 
 @param high - the high effort's counts

 @returns The effort picked, the clause that decided and how each effort stands

 @example
 ```ts
 const verdict = decideEffort({ low, mediumA, mediumB, high, },);
 ```
 */
export function decideEffort(
  {
    low,
    mediumA,
    mediumB,
    high,
  }: {
    readonly low: ArmCount;
    readonly mediumA: ArmCount;
    readonly mediumB: ArmCount;
    readonly high: ArmCount;
  },
): EffortVerdict {
  /**
   The mediums' own difference, which raises the threshold above the floor when two runs of one card disagree.
   */
  const thresholdTrials = Math.max(
    POOLED_FLOOR_TRIALS,
    Math.abs(pooledFixed({ arm: mediumA, },) - pooledFixed({ arm: mediumB, },),),
  );
  /**
   The mean of the mediums' counts.
   */
  const mediumMean = (pooledFixed({ arm: mediumA, },) + pooledFixed({ arm: mediumB, },)) / 2;
  /**
   The two efforts read against the mediums.
   */
  const readings = {
    low: readEffort({
      effort: 'low',
      arm: low,
      mediumA,
      mediumB,
      thresholdTrials,
    },),
    high: readEffort({
      effort: 'high',
      arm: high,
      mediumA,
      mediumB,
      thresholdTrials,
    },),
  };
  /**
   The efforts that beat the mediums, with no kind against them.
   */
  const winners = [
    readings.low,
    readings.high,
  ].filter(function beats(reading,): boolean {
    return reading.clause === 'beats-medium';
  },);
  /**
   The larger gap among the winners, and whether two winners share it.
   */
  const bestGap = Math.max(...winners.map(function gapOf({ gap, },): number {
    return gap;
  },),);
  /**
   The winners at the larger gap.
   */
  const best = winners.filter(function isBest({ gap, },): boolean {
    return gap === bestGap;
  },);
  /**
   The reading furthest from the mediums, the one that tells a keep: the low effort where two stand equally far.
   */
  const furthest = (Math.abs(readings.high
    .gap,) > Math.abs(readings.low
      .gap,)) ? readings.high : readings.low;
  /**
   The one winner at the larger gap, absent when there is none or two share it.
   */
  const [chosen,] = best;
  if ((best.length === 1) && (chosen !== undefined)) {
    return {
      verdict: chosen.effort,
      clause: chosen.clause,
      thresholdTrials,
      mediumMean,
      ...readings,
      said: verdictSaid({
        verdict: chosen.effort,
        reading: chosen,
        mediumMean,
        thresholdTrials,
      },),
    };
  }
  /**
   Two winners at one gap are a tie between efforts; otherwise the clause of the effort furthest from the mediums.
   */
  const keptReading: EffortReading = (best.length > 1) ? {
    ...furthest,
    clause: 'tie',
  } : furthest;
  return {
    verdict: 'medium',
    clause: keptReading.clause,
    thresholdTrials,
    mediumMean,
    ...readings,
    said: verdictSaid({
      verdict: 'medium',
      reading: keptReading,
      mediumMean,
      thresholdTrials,
    },),
  };
}

//endregion Editor fidelity decision rule
