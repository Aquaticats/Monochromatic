import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { WindowTrialRow, } from './window-trial-ledger.ts';
import { streakAfter, } from './window-trial-protocol.ts';

//region Window trial probe tally
// What the walk has bought and refused so far, and the point at which a run of
// refusals ends it. Moved out of `window-trial-probe.ts`: each step takes the
// tally before a slice and returns the tally after it, so the walk holds no
// counter it mutates.
//
// A REFUSAL IS COUNTED AND WALKED PAST, never fatal to the run. A slice can
// refuse for reasons that are properties of the slice rather than of the
// trial: no neighbouring section to widen to, or a slice with no incumbent
// whose judges all declined. Aborting the walk on one of those would stop the
// run at the same slice on every resumption, and since the refusal is never
// recorded, no amount of restarting would ever get past it.

/**
 Refusals in a row that end the run.

 Small, because slices that genuinely cannot be tried do not cluster: the draw
 interleaves entries and classes, so several in a row is a provider or a
 roster, not a run of awkward slices.
 */
const REFUSALS_BEFORE_STOPPING = 5;

/**
 Slices bought and refused so far.

 @example
 ```ts
 const tally: TrialTally = { count: 0, refused: 0, refusedInARow: 0, };
 ```
 */
export type TrialTally = {
  /**
   Slices that bought at least one arm.
   */
  readonly count: number;

  /**
   Slices that refused.
   */
  readonly refused: number;

  /**
   Refusals since the last slice that bought.
   */
  readonly refusedInARow: number;
};

/**
 The tally before any slice.
 */
export const NOTHING_BOUGHT: TrialTally = {
  count: 0,
  refused: 0,
  refusedInARow: 0,
};

/**
 Counts one refused slice.

 A REFUSAL IS NOT FREE: the slate is produced before any arm is judged, so a
 fault that fails every judging still spends a roster of translator calls per
 slice and leaves an empty ledger. Slices that genuinely cannot be tried are
 scattered through the draw, so a run of them says the fault is the run's
 rather than the slices'.

 @param tally - tally before the slice

 @returns Tally after it

 @throws {@link StatedRefusalError} when the refusals in a row reach the stop

 @example
 ```ts
 const after = tallyRefusal({ tally, },);
 ```
 */
export function tallyRefusal({ tally, }: { readonly tally: TrialTally; },): TrialTally {
  /**
   Tally with this refusal in it.
   */
  const after: TrialTally = {
    ...tally,
    refused: tally.refused + 1,
    refusedInARow: streakAfter({
      refusedInARow: tally.refusedInARow,
      yielded: 'refused',
    },),
  };
  if (after.refusedInARow >= REFUSALS_BEFORE_STOPPING)
    throw new StatedRefusalError({
      says: `${String(after.refusedInARow,)} ${
        wordForCount({
          count: after.refusedInARow,
          one: 'slice',
          many: 'slices',
        },)
      } refused in a row, which is `
        + `a fault in the run rather than in the slices; stopping before `
        + `the rest of the draw is spent producing slates nobody judges`,
    },);
  return after;
}

/**
 Counts one slice that did not refuse.

 A slice the ledger already held bought nothing and leaves the streak alone
 (see `streakAfter`); one that bought is counted and ends the streak.

 @param tally - tally before the slice

 @param rows - arms the slice bought, empty when the ledger held them all

 @returns Tally after it

 @example
 ```ts
 const after = tallyRows({ tally, rows, },);
 ```
 */
export function tallyRows(
  {
    tally,
    rows,
  }: {
    readonly tally: TrialTally;
    readonly rows: readonly WindowTrialRow[];
  },
): TrialTally {
  /**
   Streak after a slice that did not refuse.
   */
  const refusedInARow = streakAfter({
    refusedInARow: tally.refusedInARow,
    yielded: (rows.length === 0) ? 'already-held' : 'bought',
  },);
  return {
    ...tally,
    refusedInARow,
    count: (rows.length === 0) ? tally.count : (tally.count + 1),
  };
}

//endregion Window trial probe tally
