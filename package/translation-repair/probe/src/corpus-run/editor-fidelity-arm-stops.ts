import type { Answer, } from './editor-fidelity-arm-answers.ts';
import { countWithNoun, } from './editor-fidelity-arm-facts.ts';
import type { ArmFigures, } from './editor-fidelity-arm-figures.ts';

//region Editor fidelity arm stop checks
// THE CHECKS OF HOW THE ASKS OF AN ARM ENDED. A trial that reads unusable says only that it yielded nothing; whether the
// card was cut off at the output-token cap or the exchange threw says why, and an arm in which either happened measured
// the cap or the wire and not the effort. Each row line states the ends of its trial's asks, and these checks fail an arm
// whose rows state none, whose asks ended at the cap or threw, and warn of one in which many trials were refused or
// unusable.

/**
 The most unusable plus refused trials an arm may have before the read step warns of it.
 */
export const UNUSABLE_PLUS_REFUSED_WARNED_ABOVE = 4;

/**
 The answer of the check that every row line states how its asks ended.

 @param figures - the arm's figures

 @returns Whether the arm logged rows and each states a stop reason

 @example
 ```ts
 const answer = stopsStatedAnswer({ figures, },);
 ```
 */
export function stopsStatedAnswer({ figures, }: { readonly figures: ArmFigures; },): Answer {
  if (figures.rows
    .length
    === 0) {
    return {
      ok: false,
      detail: 'no row line was logged',
    };
  }
  /**
   Rows whose line states no stop reason.
   */
  const unstated = figures.rows
    .filter(function states(row,): boolean {
      return !row.stated;
    },);
  return (unstated.length > 0)
    ? {
      ok: false,
      detail: `${String(unstated.length,)} of ${
        countWithNoun({
          count: figures.rows
            .length,
          one: 'row',
          many: 'rows',
        },)
      } state no stop reason`,
    }
    : {
      ok: true,
      detail: `${
        countWithNoun({
          count: figures.rows
            .length,
          one: 'row',
          many: 'rows',
        },)
      } each state how their asks ended`,
    };
}

/**
 Writes the rows of a list for a message: `row 2` or `rows 2, 4`.

 @param rows - the row numbers

 @returns The label and the numbers

 @example
 ```ts
 // yields 'rows 2, 4'
 const said = rowsSaid({ rows: [2, 4,], },);
 ```
 */
function rowsSaid({ rows, }: { readonly rows: readonly number[]; },): string {
  return `${(rows.length === 1) ? 'row' : 'rows'} ${rows.join(', ',)}`;
}

/**
 The answer of the check that no ask ended at the output-token cap and none threw.

 @param figures - the arm's figures

 @returns Whether every ask ended some other way, naming the rows where one did not

 @example
 ```ts
 const answer = capAndThrowAnswer({ figures, },);
 ```
 */
export function capAndThrowAnswer({ figures, }: { readonly figures: ArmFigures; },): Answer {
  /**
   The numbers of the rows with an ask that ended at the cap.
   */
  const capped = figures.rows
    .filter(function endedAtCap({ stops, },): boolean {
      return stops.includes('output-cap',);
    },)
    .map(function numberOf({ row, },): number {
      return row;
    },);
  /**
   The numbers of the rows with an ask that threw.
   */
  const threw = figures.rows
    .filter(function threwAsk({ stops, },): boolean {
      return stops.includes('threw',);
    },)
    .map(function numberOf({ row, },): number {
      return row;
    },);
  /**
   What went wrong, one phrase for each way.
   */
  const found = [
    ...((capped.length === 0)
      ? []
      : [
        `${
          countWithNoun({
            count: capped.length,
            one: 'trial',
            many: 'trials',
          },)
        } ended at the output-token cap (${rowsSaid({ rows: capped, },)})`,
      ]),
    ...((threw.length === 0)
      ? []
      : [
        `${
          countWithNoun({
            count: threw.length,
            one: 'trial',
            many: 'trials',
          },)
        } had an ask that threw (${rowsSaid({ rows: threw, },)})`,
      ]),
    ...((figures.threw > 0)
      ? [
        `the seat line says ${
          countWithNoun({
            count: figures.threw,
            one: 'ask',
            many: 'asks',
          },)
        } threw`,
      ]
      : []),
  ];
  return (found.length === 0)
    ? {
      ok: true,
      detail: `${
        countWithNoun({
          count: figures.rows
            .length,
          one: 'trial',
          many: 'trials',
        },)
      }, none at the cap, none threw`,
    }
    : {
      ok: false,
      detail: found.join('; ',),
    };
}

/**
 The warning for an arm with many trials unusable or refused.

 @param arm - the arm's name

 @param figures - the arm's figures

 @returns The line alone, or nothing when the arm has no more than the warned-above count

 @example
 ```ts
 const warnings = unusableWarning({ arm: 'low', figures, });
 ```
 */
export function unusableWarning({
  arm,
  figures,
}: {
  readonly arm: string;
  readonly figures: ArmFigures
},): readonly string[] {
  /**
   Trials that yielded no edit, unusable and refused together.
   */
  const together = Math.max(
    0,
    figures.unusable,
  ) + Math.max(
    0,
    figures.refused,
  );
  return (together > UNUSABLE_PLUS_REFUSED_WARNED_ABOVE)
    ? [
      `WARN ${arm}: unusable plus refused is ${String(together,)} of ${
        countWithNoun({
          count: figures.trials,
          one: 'trial',
          many: 'trials',
        },)
      }, over ${String(UNUSABLE_PLUS_REFUSED_WARNED_ABOVE,)}`,
    ]
    : [];
}

//endregion Editor fidelity arm stop checks
