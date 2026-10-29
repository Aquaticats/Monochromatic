import {
  type ArmLog,
  countWithNoun,
} from './editor-fidelity-arm-facts.ts';
import type { RowStops, } from './editor-fidelity-arm-figures.ts';
import {
  REPEAT_ALPHA,
  type RepeatShortfall,
  repeatShortfallsOf,
} from './editor-fidelity-arm-repeat-test.ts';
import {
  effortOf,
  fixedBy,
  sharedBenchOf,
  unitOf,
  withFiguresOf,
} from './editor-fidelity-arm-shared.ts';
import { pSaid, } from './editor-fidelity-decide-lines.ts';
import {
  FALSE_PICK_BOUND_ALIKE,
  FALSE_PICK_UNEQUAL_REACHED,
} from './editor-fidelity-decide.ts';
import { ALPHA, } from './editor-fidelity-paired.ts';

//region Editor fidelity arm repeats
// THE REPEAT AGREEMENT OF EACH ARM, which the paired test's false-pick bound depends on. A trial asked in more than one
// pass agrees with itself when it read the same way in every pass, a clean fix in each or in none (an ask that ended at the
// output-token cap is not a clean fix, as in the paired test). The share of such trials that agree is an arm's repeat
// agreement. The exact test holds its bound when the arms repeat themselves alike; when the default effort repeats itself
// far less than the others the simulation reached a higher chance, and in no other direction, so the repeat lines also warn
// when an exact one-sided test (`editor-fidelity-arm-repeat-test.ts`) finds the default effort's count of trials that
// agree with themselves low against another arm's. The warning prints both counts and the p-value, so that a small number
// of trials asked again shows as one: the test has no floor on them, and needs a larger difference when they are few.
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

 @param arm - arm's name

 @param effort - effort its card runs at

 @param rows - arm's rows

 @param held - units to read, each as `unitOf` names it; every row when omitted

 @returns Trials asked again and how many of them agreed with themselves

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

 @param figure - arm's figure, with at least one repeated trial

 @returns Share of the trials asked again that agreed with themselves, from 0 to 1

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

 @param figure - arm's figure, with at least one repeated trial

 @returns Percent, rounded

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

 @param figure - arm's figure

 @returns Line, without a newline

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
 The level the exact test of one warning was run at, as the line says it.

 @param comparisons - other arms the level was divided among

 @returns Level, with its division when there is more than one other arm

 @example
 ```ts
 // yields '0.05 divided by 2, the number of other arms'
 const said = levelSaid({ comparisons: 2, },);
 ```
 */
function levelSaid({ comparisons, }: { readonly comparisons: number; },): string {
  return (comparisons === 1)
    ? String(REPEAT_ALPHA,)
    : `${String(REPEAT_ALPHA,)} divided by ${String(comparisons,)}, the number of other arms`;
}

/**
 Says that the default effort repeats itself less than another arm does, at the level of the exact test.

 @param low - default effort's figure

 @param shortfall - arm it is under, p-value against it and comparisons the level was divided among

 @returns Line, without a newline

 @example
 ```ts
 const line = warningLine({ low: defaultFigure, shortfall, },);
 ```
 */
function warningLine(
  {
    low,
    shortfall,
  }: {
    readonly low: RepeatFigure;
    readonly shortfall: RepeatShortfall<RepeatFigure>;
  },
): string {
  /**
   The arm the default effort is under.
   */
  const { other: high, } = shortfall;
  return `WARN repeat agreement: ${low.arm} gave the same reading in every pass on ${String(low.agreed,)} of ${
    countWithNoun({
      count: low.repeated,
      one: 'trial',
      many: 'trials',
    },)
  } it was asked again (${String(percentOf({ figure: low, },),)} percent), against ${String(high.agreed,)} of ${
    String(high.repeated,)
  } (${String(percentOf({ figure: high, },),)} percent) for ${high.arm}; an exact one-sided test of the two counts gives p ${
    pSaid({ p: shortfall.p, },)
  }, at or under ${levelSaid({ comparisons: shortfall.comparisons, },)}; the bound of ${String(FALSE_PICK_BOUND_ALIKE,)} on choosing `
    + 'another effort when none is better holds when the arms repeat themselves alike, and the simulation reached '
    + `${String(FALSE_PICK_UNEQUAL_REACHED,)} at a true fix rate of 0.9 when the default effort repeats itself far less `
    + `than the others, so a verdict of high or low is less sure than the ${String(ALPHA,)} level says`;
}

/**
 The warning for one arm of the default effort, when it is under another arm at the level of the exact test.

 @param figure - default effort's figure, with at least one repeated trial

 @param others - figures of the other arms that have trials asked again, in the order they ran

 @returns Warning, none when the default effort is under no other arm at the level

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
   The other arm the default effort is under, when there is one at the level.
   */
  const shortfalls = repeatShortfallsOf({
    figure,
    others,
  },);
  return shortfalls.map(function lineOf(shortfall,): string {
    return warningLine({
      low: figure,
      shortfall,
    },);
  },);
}

/**
 The lines the read step prints for the repeat agreement of the arms: one for each arm, and a warning for each arm of the
 default effort that an exact test finds repeating itself less than another arm, at the level the warning prints.

 @param arms - arms in the order they ran, one or more

 @returns REPEAT line of each arm, then the WARN line of each arm of the default effort that is under another

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
  const withFigures = withFiguresOf({ arms, },);
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
