import type { ArmLog, } from './editor-fidelity-arm-facts.ts';
import {
  armFiguresOf,
  type RowStops,
} from './editor-fidelity-arm-figures.ts';
import { fixedBy, } from './editor-fidelity-arm-shared.ts';
import {
  FALSE_PICK_BOUND_ALIKE,
  FALSE_PICK_UNEQUAL_REACHED,
  REPEAT_WARNED_ABOVE_POINTS,
} from './editor-fidelity-decide.ts';
import { ALPHA, } from './editor-fidelity-paired.ts';

//region Editor fidelity arm repeats
// THE REPEAT AGREEMENT OF EACH ARM, which the paired test's false-pick bound depends on. A trial asked in more than one pass
// agrees with itself when it read the same way in every pass, a clean fix in each or in none (an ask that ended at the
// output-token cap is not a clean fix, as in the paired test). The share of such trials that agree is an arm's repeat
// agreement. The exact test holds its bound when the arms repeat themselves alike; when the default effort repeats itself
// far less than the others the simulation reached a higher chance, so the lines below also warn when the arms' agreements
// differ by more than the stated amount. The figures are read off each arm's own rows, a row in which an ask threw being
// left out as the shared trials leave it out, so the lines are printed for one arm read alone, which the runbook does
// before the other arms exist.

/**
 One arm's repeat agreement.
 */
type RepeatFigure = {
  /**
   The arm's name.
   */
  readonly arm: string;

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

 @param rows - the arm's rows

 @returns The trials asked again and how many of them agreed with themselves

 @example
 ```ts
 const figure = repeatFigureOf({ arm: 'high', rows, },);
 ```
 */
function repeatFigureOf(
  {
    arm,
    rows,
  }: {
    readonly arm: string;
    readonly rows: readonly RowStops[];
  },
): RepeatFigure {
  /**
   Whether each pass of a trial read as a clean fix, for each trial, the rows in which an ask threw left out.
   */
  const readings = new Map<string, readonly boolean[]>();
  for (const row of rows) {
    /**
     How the asks of the row ended.
     */
    const { stops, } = row;
    if (stops.includes('threw',))
      continue;
    readings.set(
      row.trial,
      [
        ...(readings.get(row.trial,) ?? []),
        fixedBy({ row, },),
      ],
    );
  }
  /**
   The trials asked in more than one pass, with how each read.
   */
  const again = [...readings.values(),].filter(function askedAgain(passes: readonly boolean[],): boolean {
    return passes.length > 1;
  },);
  /**
   Those that read alike in every pass.
   */
  const alike = again.filter(function readAlike(passes: readonly boolean[],): boolean {
    return passesAgree({ passes, },);
  },);
  return {
    arm,
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
 const share = shareOf({ figure: { arm: 'medium', repeated: 44, agreed: 22, }, },);
 ```
 */
function shareOf({ figure, }: { readonly figure: RepeatFigure; },): number {
  return figure.agreed / figure.repeated;
}

/**
 An arm's repeat agreement as a whole percent.

 @param figure - the arm's figure, with at least one repeated trial

 @returns The percent, rounded

 @example
 ```ts
 // yields 91
 const percent = percentOf({ figure: { arm: 'medium', repeated: 44, agreed: 40, }, },);
 ```
 */
function percentOf({ figure, }: { readonly figure: RepeatFigure; },): number {
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
function repeatLine({ figure, }: { readonly figure: RepeatFigure; },): string {
  if (figure.repeated === 0)
    return `REPEAT ${figure.arm}: no trial was asked in more than one pass`;
  return `REPEAT ${figure.arm}: ${String(figure.agreed,)} of ${String(figure.repeated,)} trials gave the same reading in every pass (${
    String(percentOf({ figure, },),)
  } percent)`;
}

/**
 Joins words as a list that ends in `and`.

 @param words - the words, at least two

 @returns The list

 @example
 ```ts
 // yields 'a, b and c'
 const list = andList({ words: ['a', 'b', 'c',], },);
 ```
 */
function andList({ words, }: { readonly words: readonly string[]; },): string {
  /**
   All the words but the last.
   */
  const leading = words.slice(
    0,
    -1,
  );
  return `${leading.join(', ',)} and ${words.at(-1,) ?? ''}`;
}

/**
 Whether two arms' shares differ by more than the warned-above amount, compared exactly by cross-multiplying the counts so
 that a difference of exactly that amount is not warned of.

 @param low - the arm with the lower share

 @param high - the arm with the higher share

 @returns Whether the difference is over the amount

 @example
 ```ts
 const over = differsOver({ low: lowestFigure, high: highestFigure, },);
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
   The amount warned above, times the same product.
   */
  const scaledLimit = REPEAT_WARNED_ABOVE_POINTS * (low.repeated * high.repeated);
  return scaledGap > scaledLimit;
}

/**
 Says that the arms' repeat agreements differ by more than the warned-above amount.

 @param measured - the arms that have trials asked again, in the order they ran, at least two

 @param lowest - the one with the lowest share

 @param highest - the one with the highest share

 @returns The line, without a newline

 @example
 ```ts
 const line = warningLine({ measured, lowest, highest, },);
 ```
 */
function warningLine(
  {
    measured,
    lowest,
    highest,
  }: {
    readonly measured: readonly RepeatFigure[];
    readonly lowest: RepeatFigure;
    readonly highest: RepeatFigure;
  },
): string {
  /**
   The widest difference between two arms, in points.
   */
  const points = Math.round(100 * (shareOf({ figure: highest, },) - shareOf({ figure: lowest, },)),);
  /**
   Each arm with its percent, in the order they ran.
   */
  const words = measured.map(function wordOf(figure,): string {
    return `${figure.arm} ${String(percentOf({ figure, },),)} percent`;
  },);
  return `WARN repeat agreement: ${andList({ words, },)} differ by ${String(points,)} points, over ${
    String(REPEAT_WARNED_ABOVE_POINTS,)
  }; the bound of ${String(FALSE_PICK_BOUND_ALIKE,)} on choosing `
    + 'another effort when none is better holds when the arms repeat themselves alike, and the simulation reached '
    + `${String(FALSE_PICK_UNEQUAL_REACHED,)} when the default effort repeats itself far less than the others, so a `
    + `verdict of high or low is less sure than the ${String(ALPHA,)} level says`;
}

/**
 The lines the read step prints for the repeat agreement of the arms: one for each arm, and a warning when the arms that
 have one differ by more than the stated number of points.

 @param arms - the arms in the order they ran, one or more

 @returns The REPEAT line of each arm, then the WARN line when the arms differ

 @example
 ```ts
 for (const said of repeatLinesOf({ arms, },))
   process.stdout.write(`${said}\n`,);
 ```
 */
export function repeatLinesOf({ arms, }: { readonly arms: readonly ArmLog[]; },): readonly string[] {
  /**
   Each arm's figure.
   */
  const figures = arms.map(function figureOf(log,): RepeatFigure {
    /**
     The arm's rows.
     */
    const { rows, } = armFiguresOf({ log, },);
    return repeatFigureOf({
      arm: log.arm,
      rows,
    },);
  },);
  /**
   The arms that have trials asked again, in the order they ran.
   */
  const measured = figures.filter(function hasRepeats({ repeated, },): boolean {
    return repeated > 0;
  },);
  /**
   The same arms, lowest share first.
   */
  const lowestFirst = measured.toSorted(function byShare(
    a,
    b,
  ): number {
    return shareOf({ figure: a, },) - shareOf({ figure: b, },);
  },);
  /**
   The arm with the lowest share, when any arm has trials asked again.
   */
  const [lowest,] = lowestFirst;
  /**
   The arm with the highest share, when any arm has trials asked again.
   */
  const highest = lowestFirst.at(-1,);
  /**
   The arms' lines.
   */
  const lines = figures.map(function lineOf(figure,): string {
    return repeatLine({ figure, },);
  },);
  if (measured.length < 2)
    return lines;
  if (lowest === undefined)
    return lines;
  if (highest === undefined)
    return lines;
  if (!differsOver({
    low: lowest,
    high: highest,
  },))
    return lines;
  return [
    ...lines,
    warningLine({
      measured,
      lowest,
      highest,
    },),
  ];
}

//endregion Editor fidelity arm repeats
