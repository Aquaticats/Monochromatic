import type { Answer, } from './editor-fidelity-arm-answers.ts';
import { countWithNoun, } from './editor-fidelity-arm-facts.ts';
import type { ArmFigures, } from './editor-fidelity-arm-figures.ts';

//region Editor fidelity arm stop checks
// THE CHECKS OF HOW THE ASKS OF AN ARM ENDED. A trial that reads unusable says only that it yielded nothing; whether the
// card was cut off at the output-token cap or the exchange threw says why. Each row line states the ends of its trial's
// asks, and these checks fail an arm whose rows state none, say which trials ended at the cap (they score as unusable for
// the arm) or threw (they are left out of every arm, so the cap or the wire does not stand in for the effort), and warn of
// an arm in which many trials were refused or unusable.

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
 The answer that says which trials ended at the output-token cap and which threw, and what the rule does with each: a
 trial that ended at the cap scores as unusable for this arm, and a trial in which an ask threw is left out of every arm.
 Neither fails the arm; the bench check of shared units fails only when too few remain.

 @param figures - the arm's figures

 @returns An answer that holds, naming the rows where an ask ended at the cap or threw

 @example
 ```ts
 const answer = trialEndsAnswer({ figures, },);
 ```
 */
export function trialEndsAnswer({ figures, }: { readonly figures: ArmFigures; },): Answer {
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
        } ended at the output-token cap (${rowsSaid({ rows: capped, },)}) and ${
          (capped.length === 1) ? 'scores' : 'score'
        } as unusable for this arm`,
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
        } had an ask that threw (${rowsSaid({ rows: threw, },)}) and ${
          (threw.length === 1) ? 'is' : 'are'
        } left out of every arm`,
      ]),
  ];
  return {
    ok: true,
    detail: (found.length === 0)
      ? `${
        countWithNoun({
          count: figures.rows
            .length,
          one: 'trial',
          many: 'trials',
        },)
      }, none at the cap, none threw`
      : found.join('; ',),
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
