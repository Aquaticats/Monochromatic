import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { WindowTrialRow, } from './window-trial-ledger.ts';

//region Window trial report
// What the window trial's rows mean, computed in one place so no reader has to
// re-derive it and derive it differently.
//
// THE READING IS THE RISKY PART, more than the running. This measurement has
// already been designed wrong twice on paper: once as two arms, which cannot
// separate the window from a resampled slate, and once with a band taken from a
// single repeat, which understates the noise it is meant to bound. Both were
// caught before any quota was spent. What survives here is the discipline those
// corrections produced.
//
// ONLY COMPLETE TRIPLES ARE ANALYSED. A slice is read only when all three arms
// exist for it: the two narrow runs and the wide one. A slice missing an arm is
// counted and excluded, never partially credited, because every number this report prints is
// a PAIRED comparison on one slice and a half-populated pair is not a smaller
// sample, it is a different one.

/**
 Arm names the trial buys, and the only ones this report reads.

 TWO NARROW ARMS ON PURPOSE. Their difference is the run-to-run band, and
 without it the narrow-to-wide difference has nothing to be judged against.
 */
export const TRIAL_ARMS = {
  narrowFirst: 'narrow-a',
  narrowSecond: 'narrow-b',
  wide: 'wide',
} as const;

/**
 How one arm fared over one class of slice.

 @example
 ```ts
 const rates: ArmRate = { arm: 'wide', trials: 22, replaced: 9, };
 ```
 */
export type ArmRate = {
  /**
   Arm these counts are for.
   */
  readonly arm: string;

  /**
   Slices this arm was read over, which is the complete-triple population.
   */
  readonly trials: number;

  /**
   How many of them this arm replaced the archive on.
   */
  readonly replaced: number;
};

/**
 How the wide arm moved against the first narrow arm, slice by slice.

 PAIRED COUNTS RATHER THAN TWO RATES, because two rates that match can hide
 equal traffic in both directions, and traffic in both directions is not the
 window working. This is the shape that distinguishes them.

 @example
 ```ts
 const moved: Transitions = { replaceToKeep: 4, keepToReplace: 1, heldReplace: 10, heldKeep: 7, };
 ```
 */
export type Transitions = {
  /**
   Slices the narrow arm replaced and the wide arm kept, which is the
   direction the relocation finding predicts if displacement drives replacement.
   */
  readonly replaceToKeep: number;

  /**
   Slices the narrow arm kept and the wide arm replaced.
   */
  readonly keepToReplace: number;

  /**
   Slices both replaced.
   */
  readonly heldReplace: number;

  /**
   Slices neither replaced.
   */
  readonly heldKeep: number;
};

/**
 Everything the trial says about one class of slice.

 @example
 ```ts
 const report: ClassReport = { sliceClass: 'relocation-high', arms: [], transitions, bandTransitions, pairedExcess: 0.2, entries: 9, incomplete: 0, degraded: 0, repeated: 0, };
 ```
 */
export type ClassReport = {
  /**
   Class the displacement screen flagged, or the control label.
   */
  readonly sliceClass: string;

  /**
   Each arm's replacement count over the complete-triple population.
   */
  readonly arms: readonly ArmRate[];

  /**
   How the wide arm moved against the first narrow arm.
   */
  readonly transitions: Transitions;

  /**
   How the SECOND NARROW ARM moved against the first.

   THIS IS THE BAND, and it is the number `transitions` must beat. Both narrow
   arms were shown the same evidence over the same slate, so every transition
   here is noise by construction. A wide arm that moves no more than this has
   moved nothing.
   */
  readonly bandTransitions: Transitions;

  /**
   Slices excluded for missing an arm, so a reader can see how much of the
   class the analysis covers.
   */
  readonly incomplete: number;

  /**
   Mean, over the read triples, of the two narrow arms' replacement rate minus
   the wide arm's.

   THE PRIMARY NUMBER, and positive means the window reduced replacement. The
   transition counts describe; this estimates. It uses BOTH narrow arms rather
   than privileging the first, which is what the transition counts do, and it
   is a paired difference on one slice, so nothing about the slice can explain
   it. Zero when nothing was read.
   */
  readonly pairedExcess: number;

  /**
   Documents the read triples came from.

   CARRIED BECAUSE SLICES ARE NOT INDEPENDENT. Several come from one entry, and
   relocation endpoints overlap by construction, so a spread computed as though
   every slice were its own document would be too narrow. This is the number a
   reader needs to know that.
   */
  readonly entries: number;

  /**
   Complete triples excluded because some arm judged on a short panel.

   SEPARATE FROM {@link ClassReport.incomplete} because the two say different
   things. A missing arm is a run that stopped. A short panel is a run that
   proceeded on fewer judges than it seated, and the wide arm is the one most
   exposed to it, so a class where this number is large is a class whose
   comparison was being pulled by lost voices rather than by evidence.
   */
  readonly degraded: number;

  /**
   Slices left out because the ledger holds two rows for one of their arms.

   SEPARATE FROM {@link ClassReport.incomplete} because the two say different
   things. A missing arm is a run that stopped. A repeated arm is a ledger
   that two runs or a hand wrote to, so no row of that arm can be taken as the
   one the protocol bought, and a class where this is not zero rests on a
   ledger that was not written the way the runner writes it.
   */
  readonly repeated: number;
};

/**
 Counts one arm's replacements over a set of complete triples.

 @param triples - slices with all three arms

 @param arm - arm to count

 @returns That arm's trial and replacement counts

 @example
 ```ts
 const rate = rateOf({ triples, arm: TRIAL_ARMS.wide, },);
 ```
 */
function rateOf(
  {
    triples,
    arm,
  }: {
    readonly triples: readonly ReadonlyMap<string, WindowTrialRow>[];
    readonly arm: string;
  },
): ArmRate {
  return {
    arm,
    trials: triples.length,
    replaced: triples.filter(function replacedHere(triple,): boolean {
      return triple.get(arm,)
        ?.shipped
        === true;
    },)
      .length,
  };
}

/**
 Counts how one arm moved against another, slice by slice.

 @param triples - slices with all three arms

 @param from - arm the comparison starts at

 @param to - arm it moves to

 @returns Paired transition counts

 @example
 ```ts
 const moved = transitionsBetween({ triples, from: TRIAL_ARMS.narrowFirst, to: TRIAL_ARMS.wide, },);
 ```
 */
function transitionsBetween(
  {
    triples,
    from,
    to,
  }: {
    readonly triples: readonly ReadonlyMap<string, WindowTrialRow>[];
    readonly from: string;
    readonly to: string;
  },
): Transitions {
  /**
   Each slice as a pair of booleans, which is all a transition reads.
   */
  const moves = triples.map(function toMove(triple,): readonly [
    boolean,
    boolean,
  ] {
    return [
      triple.get(from,)
        ?.shipped
        === true,
      triple.get(to,)
        ?.shipped
        === true,
    ];
  },);

  return {
    replaceToKeep: moves.filter(function fell([before, after,],): boolean {
      return before && (!after);
    },)
      .length,
    keepToReplace: moves.filter(function rose([before, after,],): boolean {
      return (!before) && after;
    },)
      .length,
    heldReplace: moves.filter(function both([before, after,],): boolean {
      return before && after;
    },)
      .length,
    heldKeep: moves.filter(function neither([before, after,],): boolean {
      return (!before) && (!after);
    },)
      .length,
  };
}

/**
 Mean paired difference between the narrow pair and the wide arm.

 PER SLICE FIRST, THEN AVERAGED, which is what makes it paired: each slice
 contributes the difference between what it did with the window and what the
 same slate did twice without it, so anything about the slice cancels.

 @param triples - slices with all three arms on a full panel

 @returns Positive when the window reduced replacement, zero over nothing

 @example
 ```ts
 const excess = pairedExcessOf({ triples, },);
 ```
 */
function pairedExcessOf(
  { triples, }: {
    readonly triples: readonly ReadonlyMap<string, WindowTrialRow>[];
  },
): number {
  if (triples.length === 0)
    return 0;

  /**
   Halving factor, since the two narrow arms are averaged.
   */
  const HALF = 1 / 2;

  return triples.reduce(
    function addDifference(
      running,
      triple,
    ): number {
      /**
       Whether one arm replaced the archive at this slice, as a number the
       paired difference can add.

       @param arm - arm whose ledger row is read

       @returns One where the arm shipped its rendering, zero where the
       archive stood

       @example
       ```ts
       const wide = shippedOf(TRIAL_ARMS.wide,);
       ```
       */
      function shippedOf(arm: string,): number {
        return (triple.get(arm,)
          ?.shipped
          === true)
          ? 1
          : 0;
      }

      return running
        + (((shippedOf(TRIAL_ARMS.narrowFirst,) + shippedOf(TRIAL_ARMS.narrowSecond,)) * HALF)
          - shippedOf(TRIAL_ARMS.wide,));
    },
    0,
  ) / triples.length;
}

/**
 Documents a set of triples came from.

 @param triples - slices with all three arms on a full panel

 @returns Count of distinct entries

 @example
 ```ts
 const entries = entriesOf({ triples, },);
 ```
 */
function entriesOf(
  { triples, }: {
    readonly triples: readonly ReadonlyMap<string, WindowTrialRow>[];
  },
): number {
  return new Set(triples.flatMap(function toEntry(triple,): readonly string[] {
    return [...triple.values(),].map(function toId(row,): string {
      return row.entryId;
    },);
  },),).size;
}

/**
 One class's rows grouped by slice.
 */
type SliceGroups = {
  /**
   One map per slice, keyed by arm, for the slices no arm of which repeats.
   */
  readonly slices: readonly ReadonlyMap<string, WindowTrialRow>[];

  /**
   Slices left out because the ledger holds more than one row for one of
   their arms.
   */
  readonly repeated: number;
};

/**
 Groups one class's rows into per-slice arm maps.

 A SLICE WITH TWO ROWS FOR ONE ARM IS LEFT OUT WHOLE AND COUNTED. The runner
 skips an arm it already holds, so a ledger carries one row per arm unless
 two runs appended to one file or a line was added by hand; the later row is
 not more right than the earlier, and keeping either would credit a
 measurement the protocol did not buy once. The ledger is append-only, so the
 repeat cannot be removed, and a refusal would leave the whole report unread
 for want of a repair nobody may make; the count says how much was left out.

 @param rows - rows of one class

 @returns One map per slice, keyed by arm, and how many slices were left out

 @example
 ```ts
 const { slices, repeated, } = groupBySlice({ rows, },);
 ```
 */
function groupBySlice(
  { rows, }: { readonly rows: readonly WindowTrialRow[]; },
): SliceGroups {
  /**
   Arms per slice, keyed by entry and slice together since two entries can
   both carry a slice at the same index.
   */
  const bySlice = new Map<string, Map<string, WindowTrialRow>>();

  /**
   Slices some arm of which the ledger holds twice.
   */
  const repeatedKeys = new Set<string>();
  for (const row of rows) {
    /**
     This slice's identity across entries.
     */
    const key = `${row.entryId} ${String(row.sliceIndex,)}`;

    /**
     Arms recorded for it so far.
     */
    const arms = bySlice.get(key,) ?? new Map<string, WindowTrialRow>();
    if (arms.has(row.arm,))
      repeatedKeys.add(key,);
    arms.set(
      row.arm,
      row,
    );
    bySlice.set(
      key,
      arms,
    );
  }
  return {
    slices: [...bySlice,]
      .filter(function notRepeated([key,],): boolean {
        return !repeatedKeys.has(key,);
      },)
      .map(function toArms([, arms,],): ReadonlyMap<string, WindowTrialRow> {
        return arms;
      },),
    repeated: repeatedKeys.size,
  };
}

/**
 Decimal places the paired estimate is printed to.

 Two, because the draw cannot resolve a third: at the size measured here the
 spread on this estimate is larger than a hundredth, and printing more digits
 would suggest a precision the sample does not have.
 */
const EXCESS_DIGITS = 2;

/**
 The log line the window trial prints for one class's report.

 @param report - one class's report

 @returns The line, naming the paired excess, the arm rates, the transitions
 against the band, and how many slices were left out and why

 @example
 ```ts
 l.info(windowTrialReportLine({ report, },),);
 ```
 */
export function windowTrialReportLine(
  { report, }: { readonly report: ClassReport; },
): string {
  return `${report.sliceClass}: window moved replacement by ${
    report.pairedExcess
      .toFixed(EXCESS_DIGITS,)
  } over ${String(report.entries,)} ${
    wordForCount({
      count: report.entries,
      one: 'entry',
      many: 'entries',
    },)
  }; ${
    report.arms
      .map(function toRate(rate,): string {
        return `${rate.arm} ${String(rate.replaced,)}/${String(rate.trials,)}`;
      },)
      .join(' ',)
  }; wide moved ${String(report.transitions
    .replaceToKeep,)} down and ${
    String(report.transitions
      .keepToReplace,)
  } up, against a band of ${String(report.bandTransitions
    .replaceToKeep,)} down and ${
    String(report.bandTransitions
      .keepToReplace,)
  } up; ${String(report.incomplete,)} incomplete, ${
    String(report.degraded,)
  } dropped for a short panel, ${String(report.repeated,)} ${
    wordForCount({
      count: report.repeated,
      one: 'slice',
      many: 'slices',
    },)
  } left out for an arm the ledger holds twice`;
}

/**
 Reports what the trial found, one entry per class.

 READS ONE PROTOCOL AND NO OTHER. The ledger is append-only and outlives any
 single experiment, so it holds rows bought under rosters, corpus pins and code
 that have since moved. The digest was already keeping those out of RESUMPTION;
 without the same filter here it kept them out of the buying and let them into
 the answer, which is the half that matters.

 @param rows - every completed arm, from the ledger

 @param protocol - digest to read, which every other row is excluded by

 @returns One report per class present in that protocol's rows

 @example
 ```ts
 const reports = reportWindowTrial({ rows, protocol, },);
 ```
 */
export function reportWindowTrial(
  {
    rows,
    protocol,
  }: {
    readonly rows: readonly WindowTrialRow[];
    readonly protocol: string;
  },
): readonly ClassReport[] {
  /**
   Rows this protocol bought, which are the only ones any number of this
   report rests on.
   */
  const mine = rows.filter(function underThisProtocol(row,): boolean {
    return row.protocol === protocol;
  },);

  /**
   Classes present, so a report covers what was run rather than what was
   expected to run.
   */
  const classes = [...new Set(mine.map(function toClass(row,): string {
    return row.sliceClass;
  },),),];

  return classes.map(function toReport(sliceClass,): ClassReport {
    /**
     Every slice of this class, as arm maps.
     */
    const {
      slices: bySlice,
      repeated,
    } = groupBySlice({ rows: mine.filter(function inClass(row,): boolean {
      return row.sliceClass === sliceClass;
    },), },);

    /**
     Slices carrying all three arms.
     */
    const complete = bySlice.filter(function hasEveryArm(arms,): boolean {
      return arms.has(TRIAL_ARMS.narrowFirst,)
        && arms.has(TRIAL_ARMS.narrowSecond,)
        && arms.has(TRIAL_ARMS.wide,);
    },);

    /**
     Of those, the ones every arm decided on a full panel, which is the only
     population read.

     A SHORT PANEL IS NOT A SMALLER SAMPLE OF THE SAME THING. The fan-out
     proceeds once half the roster answers, so an arm that lost judges still
     returns a decision, and the wide arm sends the longest sheets under the
     same deadline. Reading those rows would credit lost voices to the window.
     */
    const triples = complete.filter(function fullPanel(arms,): boolean {
      return [...arms.values(),].every(function whole(row,): boolean {
        return (row.judgesSeated > 0) && (row.judgesHeard === row.judgesSeated);
      },);
    },);

    return {
      sliceClass,
      arms: [
        TRIAL_ARMS.narrowFirst,
        TRIAL_ARMS.narrowSecond,
        TRIAL_ARMS.wide,
      ].map(function toRate(arm,): ArmRate {
        return rateOf({
          triples,
          arm,
        },);
      },),
      transitions: transitionsBetween({
        triples,
        from: TRIAL_ARMS.narrowFirst,
        to: TRIAL_ARMS.wide,
      },),
      bandTransitions: transitionsBetween({
        triples,
        from: TRIAL_ARMS.narrowFirst,
        to: TRIAL_ARMS.narrowSecond,
      },),
      pairedExcess: pairedExcessOf({ triples, },),
      entries: entriesOf({ triples, },),
      incomplete: bySlice.length - complete.length,
      degraded: complete.length - triples.length,
      repeated,
    };
  },);
}

//endregion Window trial report
