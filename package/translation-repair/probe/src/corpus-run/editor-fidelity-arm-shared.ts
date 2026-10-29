import type { ArmLog, } from './editor-fidelity-arm-facts.ts';
import {
  type ArmFigures,
  armFiguresOf,
  type RowStops,
} from './editor-fidelity-arm-figures.ts';
import type { SharedTrial, } from './editor-fidelity-decide-types.ts';

//region Editor fidelity shared trials
// THE UNITS EVERY ARM OF A BENCH COMPLETED, a unit being a trial in one pass over the pool, which are what the rule
// compares. The rows of an arm are keyed by trial and pass, so a trial asked again in a later pass is a unit of its own
// in its entry. The first arm's rows are the reference: the
// later arms walk the same leading trials in the same order, and a trial an arm did not complete is not shared. A trial in
// which any arm's ask threw is left out of every arm and counted. A trial in which an ask ended at the output-token cap
// scores as unusable for its arm, so it is not a clean fix there. Each shared trial carries the entry it comes from, the
// unit the rule's test permutes within.

/**
 Marker that stands before the effort in a card's id.
 */
const EFFORT_MARKER = '-effort-';

/**
 Names the effort a card runs at.

 @param card - card an arm seated, empty when its log names none

 @returns Text after the effort marker, or `(none)` when the card has no effort in its id

 @example
 ```ts
 // yields 'low'
 const effort = effortOf({ card: 'claude-sonnet-5-5-effort-low', },);
 ```
 */
export function effortOf({ card, }: { readonly card: string; }): string {
  /**
   Where the effort marker sits in the id.
   */
  const at = card.lastIndexOf(EFFORT_MARKER,);
  return (at === (-1)) ? '(none)' : card.slice(at + EFFORT_MARKER.length,);
}

/**
 Reads the figures of each arm's log, keeping the arms in the order given.

 @param arms - arms' logs in the order they ran

 @returns Each arm's name with the figures its log states, as `sharedBenchOf` takes them

 @example
 ```ts
 const shared = sharedBenchOf({ arms: withFiguresOf({ arms, },), },);
 ```
 */
export function withFiguresOf({ arms, }: { readonly arms: readonly ArmLog[]; },): readonly {
  readonly arm: string;
  readonly figures: ArmFigures;
}[] {
  return arms.map(function figuresOf(log,): {
    readonly arm: string;
    readonly figures: ArmFigures;
  } {
    return {
      arm: log.arm,
      figures: armFiguresOf({ log, },),
    };
  },);
}

/**
 The trials every arm shared and what was left out.

 @example
 ```ts
 const shared = sharedBenchOf({ arms, },);
 ```
 */
export type SharedBench = {
  /**
   The shared trials, in the first arm's order.
   */
  readonly trials: readonly SharedTrial[];

  /**
   The unit of each shared trial, in the same order, as `unitOf` names it: the trial's handle and its pass.
   */
  readonly units: readonly string[];

  /**
   Entries the shared trials come from.
   */
  readonly entries: number;

  /**
   Trials of the first arm left out because an ask of some arm threw.
   */
  readonly thrown: number;

  /**
   Trials of the first arm left out because some arm did not complete them, or a row could not be read.
   */
  readonly unfinished: number;

  /**
   How many rows each arm logged, in the order given.
   */
  readonly completed: readonly {
    /**
     The arm's name.
     */
    readonly arm: string;

    /**
     Rows it logged.
     */
    readonly rows: number;
  }[];

  /**
   Rows of the first arm, which are the trials it completed.
   */
  readonly firstRows: number;
};

/**
 What became of one trial of the first arm.
 */
type Classified =
  | {
    /**
     Every arm completed it and none threw.
     */
    readonly outcome: 'shared';

    /**
     The trial with each arm's outcome.
     */
    readonly trial: SharedTrial;

    /**
     The unit the trial is, its handle and its pass.
     */
    readonly unit: string;
  }
    | {
    /**
     Some arm did not complete it, or its row could not be read.
     */
      readonly outcome: 'unfinished';
    }
    | {
    /**
     An ask of some arm threw.
     */
      readonly outcome: 'thrown';
    };

/**
 Whether a row counts as a clean fix for its arm: it reads as one and no ask of it ended at the output-token cap.

 @param row - row

 @returns Whether the row is a clean fix

 @example
 ```ts
 const fixed = fixedBy({ row, });
 ```
 */
export function fixedBy({ row, }: { readonly row: RowStops; },): boolean {
  return row.fixed && (!row.stops
    .includes('output-cap',));
}

/**
 Names a row's unit, a trial in one pass over the pool.

 @param row - row

 @returns Trial handle and pass, which no two rows of an arm share

 @example
 ```ts
 // yields 'tabby-s0/insertion|2'
 const unit = unitOf({ row, },);
 ```
 */
export function unitOf({ row, }: { readonly row: RowStops; },): string {
  return `${row.trial}|${String(row.pass,)}`;
}

/**
 Keys an arm's rows by their unit, the trial handle and the pass.

 @param rows - arm's rows, none for an effort the bench has no arm at

 @returns Rows by unit

 @example
 ```ts
 const byUnit = rowsByUnit({ rows, },);
 ```
 */
function rowsByUnit({ rows, }: { readonly rows: readonly RowStops[]; },): ReadonlyMap<string, RowStops> {
  return new Map(rows.map(function keyed(row,): readonly [
    string,
    RowStops
  ] {
    return [
      unitOf({ row, },),
      row,
    ];
  },),);
}

/**
 The figures of the arms run at an effort.

 @param arms - arms with their figures

 @param effort - effort, `high`, `medium` or `low`

 @returns Their figures, in the order the arms are given

 @example
 ```ts
 const [highFigures,] = figuresAt({ arms, effort: 'high', },);
 ```
 */
function figuresAt(
  {
    arms,
    effort,
  }: {
    readonly arms: readonly {
      readonly arm: string;
      readonly figures: ArmFigures;
    }[];
    readonly effort: string;
  },
): readonly ArmFigures[] {
  return arms.filter(function isEffort({ figures, },): boolean {
    return effortOf({ card: figures.card, },) === effort;
  },)
    .map(function figuresOf({ figures, },): ArmFigures {
      return figures;
    },);
}

/**
 Finds the trials every arm completed.

 @param arms - arms in the order they ran, first being the reference, with their figures

 @returns Shared trials, how many were left out as thrown and as not completed by every arm, and how many rows each
 arm logged; no shared trial when the arms lack a high, a low or a medium effort

 @example
 ```ts
 const shared = sharedBenchOf({ arms: [{ arm: 'high', figures, },], },);
 ```
 */
export function sharedBenchOf(
  { arms, }: {
    readonly arms: readonly {
      readonly arm: string;
      readonly figures: ArmFigures;
    }[];
  },
): SharedBench {
  /**
   The first arm's rows, the reference order.
   */
  const reference = arms[0]
    ?.figures
    .rows
    ?? [];
  /**
   The rows by trial of the high, the low and each medium arm.
   */
  const rowsAt = {
    high: rowsByUnit({
      rows: figuresAt({
        arms,
        effort: 'high',
      },)[0]
        ?.rows
        ?? [],
    },),
    low: rowsByUnit({
      rows: figuresAt({
        arms,
        effort: 'low',
      },)[0]
        ?.rows
        ?? [],
    },),
    mediums: figuresAt({
      arms,
      effort: 'medium',
    },)
      .map(function byUnit({ rows, },): ReadonlyMap<string, RowStops> {
        return rowsByUnit({ rows, },);
      },),
  };
  /**
   What became of each trial of the first arm.
   */
  const classified = reference.map(function classify(row,): Classified {
    /**
     This unit, the trial in its pass.
     */
    const unit = unitOf({ row, },);
    /**
     This unit's row in the high arm.
     */
    const highRow = rowsAt.high
      .get(unit,);
    /**
     This unit's row in the low arm.
     */
    const lowRow = rowsAt.low
      .get(unit,);
    /**
     This unit's row in each medium arm that has one.
     */
    const mediumRows = rowsAt.mediums
      .flatMap(function rowIn(map,): readonly RowStops[] {
        /**
         The medium arm's row, absent when it did not walk the unit.
         */
        const found = map.get(unit,);
        return (found === undefined) ? [] : [found,];
      },);
    if ((highRow === undefined) || (lowRow === undefined)
      || (rowsAt.mediums
        .length
        === 0)
      || (mediumRows.length
        !== rowsAt.mediums
          .length)
      || (row.entry === '')
      || (row.kind === ''))
      return { outcome: 'unfinished', };
    if ([
      highRow,
      lowRow,
      ...mediumRows,
    ].some(function threw({ stops, },): boolean {
      return stops.includes('threw',);
    },))
      return { outcome: 'thrown', };
    return {
      outcome: 'shared',
      unit,
      trial: {
        entryId: row.entry,
        kind: row.kind,
        high: fixedBy({ row: highRow, },),
        low: fixedBy({ row: lowRow, },),
        mediums: mediumRows.map(function fixedIn(each,): boolean {
          return fixedBy({ row: each, },);
        },),
      },
    };
  },);
  /**
   The shared trials, in the first arm's order.
   */
  const trials = classified.flatMap(function sharedOf(each,): readonly SharedTrial[] {
    return (each.outcome === 'shared') ? [each.trial,] : [];
  },);
  return {
    trials,
    units: classified.flatMap(function unitOfShared(each,): readonly string[] {
      return (each.outcome === 'shared') ? [each.unit,] : [];
    },),
    entries: new Set(trials.map(function entryOf({ entryId, },): string {
      return entryId;
    },),).size,
    thrown: classified.filter(function isThrown({ outcome, },): boolean {
      return outcome === 'thrown';
    },)
      .length,
    unfinished: classified.filter(function isUnfinished({ outcome, },): boolean {
      return outcome === 'unfinished';
    },)
      .length,
    completed: arms.map(function countOf({
      arm,
      figures,
    },): SharedBench['completed'][number] {
      return {
        arm,
        rows: figures.rows
          .length,
      };
    },),
    firstRows: reference.length,
  };
}

//endregion Editor fidelity shared trials
