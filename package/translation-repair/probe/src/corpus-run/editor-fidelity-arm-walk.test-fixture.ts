import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';

import type { ArmLog, } from '../../dist/final/node/index.mjs';
import {
  armLogOf,
  inPass,
  POOL_TRIALS,
  poolRows,
  type RowSpec,
  tallyFromRows,
} from '../editor-fidelity-arm-log.test-fixture.ts';

//region Editor fidelity arm walk
// Arm logs over rows that walk the invented pool in passes, for the cases that read the repeat agreement and the fix rate of
// arms from their logs. Cat-themed invention; no corpus content.

/**
 Where a unit stands in a walk of the invented pool.
 */
type WalkPosition = {
  /**
   The pass, counted from one.
   */
  readonly pass: number;

  /**
   The trial's index in the pool.
   */
  readonly index: number;

  /**
   The unit's position in the walk, counted from zero.
   */
  readonly unit: number;
};

/**
 The rows of an arm that walks the invented pool in passes and stops after a number of units: every pass takes the
 pool's trials in order.

 @param units - how many units, a unit being a trial in one pass

 @param fixedAt - whether the unit at a position reads as a clean fix

 @returns Rows, pass by pass

 @example
 ```ts
 const rows = walkOf({ units: 150, fixedAt: ({ pass, },) => pass === 1, },);
 ```
 */
export function walkOf(
  {
    units,
    fixedAt,
  }: {
    readonly units: number;
    readonly fixedAt: (at: WalkPosition,) => boolean;
  },
): readonly RowSpec[] {
  return Array.from(
    { length: Math.ceil(units / POOL_TRIALS,), },
    function passRows(
      _unused,
      passAt,
    ): readonly RowSpec[] {
      /**
       The pass, counted from one.
       */
      const pass = passAt + 1;
      /**
       The rows of this pass.
       */
      const rows = poolRows({
        count: Math.min(
          POOL_TRIALS,
          units - (passAt * POOL_TRIALS),
        ),
        fixed: function fixesInPass(index,): boolean {
          return fixedAt({
            pass,
            index,
            unit: (passAt * POOL_TRIALS) + index,
          },);
        },
      },);
      return (pass === 1)
        ? rows
        : inPass({
          rows,
          pass,
        },);
    },
  )
    .flat();
}

/**
 One arm's log over the given rows.

 @param arm - arm's name

 @param cardId - card it seated

 @param rows - rows it walked

 @param declaredUsd - what its guard line says the earlier arms spent

 @param countedUsd - what its guard total says it counted

 @returns Arm's log

 @example
 ```ts
 const log = armOf({ arm: 'high', cardId: HIGH_CARD, rows, },);
 ```
 */
export function armOf(
  {
    arm,
    cardId,
    rows,
    declaredUsd = 0,
    countedUsd = 0.5,
  }: {
    readonly arm: string;
    readonly cardId: RosterModelId;
    readonly rows: readonly RowSpec[];
    readonly declaredUsd?: number;
    readonly countedUsd?: number;
  },
): ArmLog {
  return armLogOf({
    arm,
    tally: tallyFromRows({
      cardId,
      rows,
    },),
    stops: [],
    rows,
    seat: `SEAT ${cardId} asked=${String(rows.length,)} usable=${String(rows.length,)} unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`,
    benchSpentUsd: declaredUsd,
    guardUsd: countedUsd,
  },);
}

//endregion Editor fidelity arm walk
