import {
  type ArmLog,
  countWithNoun,
} from './editor-fidelity-arm-facts.ts';
import {
  type ArmFigures,
  armFiguresOf,
  type RowStops,
} from './editor-fidelity-arm-figures.ts';
import {
  effortOf,
  fixedBy,
  sharedBenchOf,
  unitOf,
} from './editor-fidelity-arm-shared.ts';
import {
  FALSE_PICK_BOUND_ALIKE,
  FALSE_PICK_UNEQUAL_REACHED,
  REPEAT_WARNED_ABOVE_POINTS,
} from './editor-fidelity-decide.ts';
import { ALPHA, } from './editor-fidelity-paired.ts';

//region Editor fidelity arm repeats
// THE REPEAT AGREEMENT OF EACH ARM, which the paired test's false-pick bound depends on. A trial asked in more than one
// pass agrees with itself when it read the same way in every pass, a clean fix in each or in none (an ask that ended at the
// output-token cap is not a clean fix, as in the paired test). The share of such trials that agree is an arm's repeat
// agreement. The exact test holds its bound when the arms repeat themselves alike; when the default effort repeats itself
// far less than the others the simulation reached a higher chance, and in no other direction, so the repeat lines also warn
// when the default effort's agreement is under another arm's by more than the stated amount.
//
// THE FIGURES ARE READ OVER THE UNITS EVERY ARM HOLDS when several logs are read (the units the paired test reads, a row in
// which an ask of any arm threw being left out of every arm), so that an arm the spend guard stopped early is compared with
// the others over the same trials and the same passes. One log read alone, which the runbook does before the other arms
// exist, and logs that form no bench (no unit shared by a high, a medium and a low arm; the bench checks refuse them) are
// read over each arm's own rows, a row in which an ask threw being left out as the shared units leave it out.

/**
 One arm's repeat agreement.
 */
type RepeatFigure = {
  /**
   The arm's name.
   */
  readonly arm: string;

  /**
   The effort the arm's card runs at, `(none)` when its log names no card.
   */
  readonly effort: string;

  /**
   Trials the arm was asked in more than one pass.
   */
  readonly repeated: number;

  /**
   Those whose passes all read the same way.
   */
  readonly agreed: number;
};

/**
 Whether every pass of a trial read the way its first pass did.

 @param passes - whether each pass of the trial read as a clean fix

 @returns Whether the passes agree

 @example
 ```ts
 // yields false
 const alike = passesAgree({ passes: [true, false,], },);
 ```
 */
function passesAgree({ passes, }: { readonly passes: readonly boolean[]; },): boolean {
  return passes.every(function sameAsFirst(fixed,): boolean {
    return fixed === passes[0];
  },);
}

/**
 Reads an arm's repeat agreement off its rows.

 @param arm - the arm's name

 @param effort - the effort its card runs at

 @param rows - the arm's rows

 @param held - the units to read, each as `unitOf` names it; every row when omitted

 @returns The trials asked again and how many of them agreed with themselves

 @example
 ```ts
 const figure = repeatFigureOf({ arm: 'high', effort: 'high', rows, },);
 ```
 */
function repeatFigureOf(
  {
    arm,
    effort,
    rows,
    held,
  }: {
    readonly arm: string;
    readonly effort: string;
    readonly rows: readonly RowStops[];
    readonly held?: ReadonlySet<string>;
  },
): RepeatFigure {
  /**
   The rows in which no ask threw and that are in a held unit.
   */
  const usable = rows
    .filter(function inHeldUnit(row,): boolean {
      if (held === undefined)
        return true;
      return held.has(unitOf({ row, },),);
    },)
    .filter(function noThrow({ stops, },): boolean {
      return !stops.includes('threw',);
    },);
  /**
   The rows of each trial, in the order of its passes.
   */
  const byTrial = Map.groupBy(
    usable,
    function trialOf({ trial, },): string {
      return trial;
    },
  );
  /**
   Whether each pass read as a clean fix, for the trials asked in more than one pass.
   */
  const again = [...byTrial.values(),]
    .filter(function askedAgain(passes: readonly RowStops[],): boolean {
      return passes.length > 1;
    },)
    .map(function readings(passes: readonly RowStops[],): readonly boolean[] {
      return passes.map(function fixedIn(row,): boolean {
        return fixedBy({ row, },);
      },);
    },);
  /**
   Those that read alike in every pass.
   */
  const alike = again.filter(function readAlike(passes: readonly boolean[],): boolean {
    return passesAgree({ passes, },);
  },);
  return {
    arm,
    effort,
    repeated: again.length,
    agreed: alike.length,
  };
}

/**
 An arm's repeat agreement as a share.

 @param figure - the arm's figure, with at least one repeated trial

 @returns The share of the trials asked again that agreed with themselves, from 0 to 1

 @example
 ```ts
 // yields 0.5
 const share = shareOf({ figure: { arm: 'medium', effort: 'medium', repeated: 44, agreed: 22, }, },);
 ```
 */
function shareOf({ figure, }: { readonly figure: RepeatFigure; }): number {
  return figure.agreed / figure.repeated;
}

/**
 An arm's repeat agreement as a whole percent.

 @param figure - the arm's figure, with at least one repeated trial

 @returns The percent, rounded

 @example
 ```ts
 // yields 91
 const percent = percentOf({ figure: { arm: 'medium', effort: 'medium', repeated: 44, agreed: 40, }, },);
 ```
 */
function percentOf({ figure, }: { readonly figure: RepeatFigure; }): number {
  return Math.round(100 * shareOf({ figure, },),);
}

/**
 Says an arm's repeat agreement.

 @param figure - the arm's figure

 @returns The line, without a newline

 @example
 ```ts
 const line = repeatLine({ figure, },);
 ```
 */
function repeatLine({ figure, }: { readonly figure: RepeatFigure; }): string {
  if (figure.repeated === 0)
    return `REPEAT ${figure.arm}: no trial was asked in more than one pass`;
  return `REPEAT ${figure.arm}: ${String(figure.agreed,)} of ${
    countWithNoun({
      count: figure.repeated,
      one: 'trial',
      many: 'trials',
    },)
  } gave the same reading in every pass (${
    String(percentOf({ figure, },),)
  } percent)`;
}

/**
 Whether one arm's share is under another's by more than the number of points the read step warns over, compared exactly
 by cross-multiplying the counts so that a difference of exactly that number is not warned of.

 @param low - the arm with the lower share

 @param high - the arm with the higher share

 @returns Whether the difference is over the amount

 @example
 ```ts
 const over = differsOver({ low: defaultFigure, high: otherFigure, },);
 ```
 */
function differsOver(
  {
    low,
    high,
  }: {
    readonly low: RepeatFigure;
    readonly high: RepeatFigure;
  },
): boolean {
  /**
   The difference of the shares in points, times the product of the two counts of trials asked again.
   */
  const scaledGap = 100 * ((high.agreed * low.repeated) - (low.agreed * high.repeated));
  /**
   The number of points the read step warns over, times the same product.
   */
  const scaledLimit = REPEAT_WARNED_ABOVE_POINTS * (low.repeated * high.repeated);
  return scaledGap > scaledLimit;
}

/**
 Says that the default effort repeats itself less than another arm does, by more than the number of points the read step
 warns over.

 @param low - the default effort's figure

 @param high - the figure of the arm it is under

 @returns The line, without a newline

 @example
 ```ts
 const line = warningLine({ low: defaultFigure, high: otherFigure, },);
 ```
 */
function warningLine(
  {
    low,
    high,
  }: {
    readonly low: RepeatFigure;
    readonly high: RepeatFigure;
  },
): string {
  /**
   The difference between the two arms, in points.
   */
  const points = Math.round(100 * (shareOf({ figure: high, },) - shareOf({ figure: low, },)),);
  return `WARN repeat agreement: ${low.arm} gave the same reading in every pass on ${
    String(percentOf({ figure: low, },),)
  } percent of the trials it was asked again, ${String(points,)} points under ${high.arm} at ${
    String(percentOf({ figure: high, },),)
  } percent, over ${String(REPEAT_WARNED_ABOVE_POINTS,)}; the bound of ${
    String(FALSE_PICK_BOUND_ALIKE,)
  } on choosing `
    + 'another effort when none is better holds when the arms repeat themselves alike, and the simulation reached '
    + `${String(FALSE_PICK_UNEQUAL_REACHED,)} at a true fix rate of 0.9 when the default effort repeats itself far less `
    + `than the others, so a verdict of high or low is less sure than the ${String(ALPHA,)} level says`;
}

/**
 The warning for one arm of the default effort, when another arm repeats itself more by more than the stated amount.

 @param figure - the default effort's figure, with at least one repeated trial

 @param others - the figures of the other arms that have trials asked again, in the order they ran

 @returns The warning, none when no other arm is over the default effort by more than the stated amount

 @example
 ```ts
 const warnings = warningsFor({ figure, others, },);
 ```
 */
function warningsFor(
  {
    figure,
    others,
  }: {
    readonly figure: RepeatFigure;
    readonly others: readonly RepeatFigure[];
  },
): readonly string[] {
  /**
   The other arms, the highest share first, the arm that ran first among equals.
   */
  const highestFirst = others.toSorted(function byShare(
    a,
    b,
  ): number {
    return shareOf({ figure: b, },) - shareOf({ figure: a, },);
  },);
  /**
   The other arm with the highest share, when there is one.
   */
  const [highest,] = highestFirst;
  if (highest === undefined)
    return [];
  if (!differsOver({
    low: figure,
    high: highest,
  },))
    return [];
  return [
    warningLine({
      low: figure,
      high: highest,
    },),
  ];
}

/**
 The lines the read step prints for the repeat agreement of the arms: one for each arm, and a warning for each arm of the
 default effort that repeats itself less than another arm by more than the stated number of points.

 @param arms - the arms in the order they ran, one or more

 @returns The REPEAT line of each arm, then the WARN line of each arm of the default effort that is under another

 @example
 ```ts
 for (const said of repeatLinesOf({ arms, },))
   process.stdout.write(`${said}\n`,);
 ```
 */
export function repeatLinesOf({ arms, }: { readonly arms: readonly ArmLog[]; },): readonly string[] {
  /**
   Each arm with the figures its log states.
   */
  const withFigures = arms.map(function figuresOf(log,): {
    readonly arm: string;
    readonly figures: ArmFigures;
  } {
    return {
      arm: log.arm,
      figures: armFiguresOf({ log, },),
    };
  },);
  /**
   The units every arm holds, none when one log is read alone or the logs form no bench.
   */
  const sharedUnits = (arms.length > 1)
    ? sharedBenchOf({ arms: withFigures, },)
      .units
    : [];
  /**
   The units the figures are read over, every row when there are none shared.
   */
  const held = (sharedUnits.length === 0) ? undefined : new Set(sharedUnits,);
  /**
   Each arm's figure.
   */
  const figures = withFigures.map(function figureOf({
    arm,
    figures: read,
  },): RepeatFigure {
    return repeatFigureOf({
      arm,
      effort: effortOf({ card: read.card, },),
      rows: read.rows,
      ...((held === undefined) ? {} : { held, }),
    },);
  },);
  /**
   The arms that have trials asked again, in the order they ran.
   */
  const measured = figures.filter(function hasRepeats({ repeated, },): boolean {
    return repeated > 0;
  },);
  /**
   The arms' lines.
   */
  const lines = figures.map(function lineOf(figure,): string {
    return repeatLine({ figure, },);
  },);
  /**
   A warning for each arm of the default effort that is under another arm.
   */
  const warnings = measured
    .filter(function isDefault({ effort, },): boolean {
      return effort === 'medium';
    },)
    .flatMap(function warningOf(figure,): readonly string[] {
      return warningsFor({
        figure,
        others: measured.filter(function isOther(other,): boolean {
          return other !== figure;
        },),
      },);
    },);
  return [
    ...lines,
    ...warnings,
  ];
}

//endregion Editor fidelity arm repeats
