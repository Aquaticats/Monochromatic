/**
 One add's row as the 2026-11-21 host measurement writes it, the order a round makes its adds in, and the medians a phase
 prints from the rows it collected.

 The rows go to `-host-loose-adds.jsonl`, `-host-packed-adds.jsonl` and `-host-refused-adds.jsonl`, which the planning record
 `doc/planning/cli-git-fence-size-read-beside-source-check.md` ("Where the numbers are") names as the files its figures come
 from. This module is the part of that work a unit test can hold: a median, the per-arm summary of a counted round, and the
 summary of the refused-source rounds.
 */
import type { AddTimingSpent, } from './fence-size-read-beside-the-check-2026-11-21-add-timing-record.ts';


/**
 The factor every printed median is rounded to a tenth of a millisecond by.
 */
const tenthOfAMillisecond = 10;

/**
 The number written where a record carries no figure at all, which JSON writes as `null` and which no median can use.
 */
const noFigure = Number.NaN;

/**
 One add's row, as a phase writes it and as the summary of that phase reads it back.
 */
export type Row = {
  /**
   The layout the phase measured, which names the row file the run wrote.
   */
  readonly layout: string;
  /**
   The arm's name, which is the binary and whether the read ran beside the check.
   */
  readonly arm: string;
  /**
   Whether the size read started beside the source check on this add.
   */
  readonly overlap: boolean;
  /**
   The add's own tag, unique within the phase.
   */
  readonly tag: string;
  /**
   Whether the add counts towards the phase's medians, which the warm-up adds do not.
   */
  readonly counted: boolean;
  /**
   The add's whole wall time, as this host measured it around the command.
   */
  readonly wallMs: number;
  /**
   The add's own total from its timing record.
   */
  readonly totalMs: number;
  /**
   The route the add's record names, or nothing when it names none.
   */
  readonly route?: string;
  /**
   The status the add ended with.
   */
  readonly exitCode: number;
  /**
   Whether the add failed.
   */
  readonly failed: boolean;
  /**
   The last of the add's standard error when it failed, which says why.
   */
  readonly stderr: string;
  /**
   The add's steps, each with its own time in milliseconds.
   */
  readonly steps: Readonly<Record<string, number>>;
  /**
   The work the add's other threads did, each with its time in milliseconds.
   */
  readonly work: Readonly<Record<string, number>>;
  /**
   The counted round this row belongs to, absent for a warm-up or a probe.
   */
  readonly round?: number;
  /**
   The row's place in that round, absent for a warm-up or a probe.
   */
  readonly position?: number;
};

/**
 The time each name of a record's steps or worker-thread work spent.

 @param spent - the record's steps or work, each entry carrying its own time

 @returns the time of each name in milliseconds, in the order the record lists them

 @example
 ```ts
 durationsOf({ spent: { 'git ls-tree': { ms: 12, calls: 1 } } }); // { 'git ls-tree': 12 }
 ```
 */
export function durationsOf({
  spent,
}: {
  readonly spent: Readonly<Record<string, AddTimingSpent>>;
}): Readonly<Record<string, number>> {
  /**
   The time of each name, which is what a summary reads.
   */
  const durations: Record<string, number> = {};
  for (const [name, value,] of Object.entries(spent,)) durations[name] = value.ms;
  return durations;
}

/**
 The middle value of a list of times, which is the figure the phases compare.

 @param values - the times, in the order they were measured

 @returns the middle value, or the no-figure value for a list with nothing in it, as the summary's own reading of an
   absent total is

 @example
 ```ts
 median({ values: [3, 1, 2] }); // 2
 ```
 */
export function median({ values, }: { readonly values: readonly number[]; }): number {
  /**
   The times in order, which the middle is read from.
   */
  const ordered = values.toSorted(function byValue(
    left,
    right
  ) {
    return left - right;
  },);
  return ordered[Math.floor(ordered.length / 2,)] ?? noFigure;
}

/**
 The median of each name's times, rounded the way every printed figure of this measurement is.

 @param lists - the times of each name, in the order they were measured

 @returns the rounded median of each name, in the order the names were first seen
 */
function mediansOf({ lists, }: { readonly lists: Readonly<Record<string, readonly number[]>>; }): Record<string, number> {
  /**
   The rounded median of each name.
   */
  const medians: Record<string, number> = {};
  for (const [name, values,] of Object.entries(lists,)) medians[name] = Math.round(median({ values, },) * tenthOfAMillisecond,) / tenthOfAMillisecond;
  return medians;
}

/**
 The taken routes of some rows, each named once.

 @param rows - the rows to read

 @returns the distinct route names in the order they were first taken
 */
function routesOf({ rows, }: { readonly rows: readonly Row[]; }): readonly string[] {
  return [
    ...new Set(rows
      .map(function routeOf(row) {
        return row.route;
      },)
      .filter(function wasTaken(route) {
        return route !== undefined;
      },)),
  ];
}

/**
 What one counted round says about each arm: how many adds counted, what they took, and where their time went.

 @param rows - every row the phase wrote, warm-ups included

 @param arms - the arm names the phase measured

 @returns for each arm, its counts, routes, totals and medians, as the phase prints them

 @example
 ```ts
 summaryOfAdds({ rows, arms: ['main', 'branch-serial'] });
 ```
 */
export function summaryOfAdds({
  rows,
  arms,
}: {
  readonly rows: readonly Row[];
  readonly arms: readonly string[];
}): Record<string, unknown> {
  /**
   The rows that count towards a median, which the warm-ups do not.
   */
  const counted = rows.filter(function counted(row) {
    return row.counted;
  },);
  /**
   What this round says about each arm.
   */
  const summary: Record<string, unknown> = {};
  for (const arm of arms) {
    /**
     This arm's rows that count and did not fail.
     */
    const mine = counted.filter(function isThisArmAndLanded(row) {
      return (row.arm === arm) && (!row.failed);
    },);
    /**
     The times each step of this arm was measured with.
     */
    const steps: Record<string, number[]> = {};
    /**
     The times each piece of this arm's worker-thread work was measured with.
     */
    const work: Record<string, number[]> = {};
    for (const row of mine) {
      for (const [step, ms,] of Object.entries(row.steps,)) (steps[step] ??= []).push(ms,);
      for (const [step, ms,] of Object.entries(row.work,)) (work[step] ??= []).push(ms,);
    }
    summary[arm] = {
      n: mine.length,
      failed: counted.filter(function isThisArmAndFailed(row) {
        return (row.arm === arm) && row.failed;
      },)
        .length,
      routes: routesOf({ rows: mine, },),
      medianTotalMs: mine.length > 0 ? median({
        values: mine.map(function totalOf(row) {
          return row.totalMs;
        },),
      },) : null,
      totals: mine.map(function totalOf(row) {
        return row.totalMs;
      },),
      walls: mine.map(function wallOf(row) {
        return row.wallMs;
      },),
      orders: mine.map(function orderOf(row) {
        return [
          row.round,
          row.position,
        ];
      },),
      stepMedians: mediansOf({ lists: steps, },),
      workMedians: mediansOf({ lists: work, },),
    };
  }
  return summary;
}

/**
 What the refused-source rounds say about each arm: how much of a read a source that never reaches the fence paid, and what
 its own check cost.

 @param rows - every row the refused phase wrote

 @param arms - the arm names the phase measured

 @returns for each arm, its counts, routes, totals, the read its adds started and its check's own time

 @example
 ```ts
 summaryOfRefusedAdds({ rows, arms: ['main', 'branch-overlap'] });
 ```
 */
export function summaryOfRefusedAdds({
  rows,
  arms,
}: {
  readonly rows: readonly Row[];
  readonly arms: readonly string[];
}): Record<string, unknown> {
  /**
   What these rounds say about each arm.
   */
  const summary: Record<string, unknown> = {};
  for (const arm of arms) {
    /**
     This arm's rows that did not fail.
     */
    const mine = rows.filter(function isThisArmAndLanded(row) {
      return (row.arm === arm) && (!row.failed);
    },);
    /**
     The time each add spent reading the source's sizes, which a refused source never uses.
     */
    const read = mine.map(function readOf(row) {
      return (row.steps['git ls-tree'] ?? 0) + (row.work
        .fenceSizesBesideTheCheck
        ?? 0);
    },);
    summary[arm] = {
      n: mine.length,
      routes: routesOf({ rows: mine, },),
      totals: mine.map(function totalOf(row) {
        return row.totalMs;
      },),
      medianTotalMs: mine.length > 0 ? median({
        values: mine.map(function totalOf(row) {
          return row.totalMs;
        },),
      },) : null,
      readMs: read,
      checkSourceMs: mine.map(function checkOf(row) {
        return row.steps
          .checkSource;
      },),
    };
  }
  return summary;
}

/**
 The order one counted round makes its adds in.

 The first round takes the arms in the order they are named and the second takes them the other way, because the host's page
 cache and disk warm up through a run and an arm that always went first would pay for it. Within a round the order is
 rotated by the round's number, so no arm keeps a place either.

 @param round - the round's number, counted from zero

 @param armNames - the arm names in the order the phase names them

 @returns the arms of this round, in the order their adds are made

 @example
 ```ts
 orderOfRound({ round: 1, armNames: ['main', 'branch-overlap'] }); // ['branch-overlap', 'main']
 ```
 */
export function orderOfRound({
  round,
  armNames,
}: {
  readonly round: number;
  readonly armNames: readonly string[];
}): readonly string[] {
  /**
   How many places this round's order is rotated by, which is its own number's place in the arms' cycle.
   */
  const shift = round % armNames.length;
  /**
   The arms in the direction this round takes them: the names as given, or the other way round for every other cycle.
   */
  const base = (Math.floor(round / armNames.length) % 2) === 0
    ? [...armNames,]
    : armNames.toReversed();
  return [
    ...base.slice(shift,),
    ...base.slice(
      0,
      shift,
    ),
  ];
}

/**
 The order one refused-source round makes its adds in.

 The refused phase rotates the arms by the round's number and never reverses them, which is the order the refused rounds
 of 2026-11-21 were made in.

 @param round - the round's number, counted from zero

 @param armNames - the arm names in the order the phase names them

 @returns the arms of this round, in the order their adds are made

 @example
 ```ts
 refusedOrderOfRound({ round: 1, armNames: ['main', 'branch-overlap'] }); // ['branch-overlap', 'main']
 ```
 */
export function refusedOrderOfRound({
  round,
  armNames,
}: {
  readonly round: number;
  readonly armNames: readonly string[];
}): readonly string[] {
  /**
   How many places this round's order is rotated by.
   */
  const shift = round % armNames.length;
  return [
    ...armNames.slice(shift,),
    ...armNames.slice(
      0,
      shift,
    ),
  ];
}

/**
 One add's row as the phase writes it to its file.

 The record's own fields a row may not hold (the route an add did not take, and the round a warm-up is not part of) are
 written as `null` and as an absent key here, which is the shape the row files of 2026-11-21 hold and the shape their
 readers expect.

 @param row - the row the phase measured

 @returns the fields the row file carries, in the order it has always carried them

 @example
 ```ts
 rowAsWritten({ row });
 ```
 */
export function rowAsWritten({ row, }: { readonly row: Row; }): Record<string, unknown> {
  return {
    layout: row.layout,
    arm: row.arm,
    overlap: row.overlap,
    tag: row.tag,
    counted: row.counted,
    wallMs: row.wallMs,
    totalMs: row.totalMs,
    route: row.route ?? null,
    exitCode: row.exitCode,
    failed: row.failed,
    stderr: row.stderr,
    steps: row.steps,
    work: row.work,
    ...(row.round === undefined ? {} : { round: row.round, }),
    ...(row.position === undefined ? {} : { position: row.position, }),
  };
}
