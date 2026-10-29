/**
 Tests for the read step over arms that walk the pool in several passes, the form the runbook's `--passes 3` arms write:
 the planned line counts asks (units) through the passes where the pool line counts trials, so every count the checks
 compare with the pool, the ceiling, the tally or the rows has a case for one pass, three whole passes, a capped first
 arm and a later arm given `--trials`.

 The logs are the probe's own writers' (the opening lines, the planned, row, capped and trials lines, the tallies and
 the guard's total), through the arm-log fixture, so a writer that changes its wording fails here before it fails an
 operator. Each case reaches the checks of the arm, the checks across the arms, the figures and the verdict, and the
 built command reads the same logs from files. Fixtures are cat-themed invention.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';

import {
  type ArmLog,
  armFiguresOf,
  benchReportLines,
  checkEditorArms,
  checkEditorBench,
} from '../../dist/final/node/index.mjs';
import {
  armLogOf,
  POOL_TRIALS,
  type RowSpec,
  tallyFromRows,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import {
  HIGH_CARD,
  LOW_CARD,
  MEDIUM_CARD,
} from '../editor-fidelity.test-fixture.ts';
import { readAsBuilt as readAsBuiltIn, } from './editor-fidelity-arm-read-built.test-fixture.ts';

/**
 The units of an arm that walks the leading units of the pool through its passes: the pool's trials in one order, then
 again for each pass, a pass named on the row from the second on.

 @param count - how many leading units
 @param fixed - whether the unit at an index reads as a clean fix

 @returns Rows

 @example
 ```ts
 const rows = unitRows({ count: 264, fixed: function every(): boolean { return true; }, },);
 ```
 */
function unitRows(
  {
    count,
    fixed,
  }: {
    readonly count: number;
    readonly fixed: (index: number,) => boolean;
  },
): readonly RowSpec[] {
  return Array.from(
    { length: count, },
    function rowAt(
      _unused,
      index,
    ): RowSpec {
      /**
       The trial of this unit in the pool's order.
       */
      const trialAt = index % POOL_TRIALS;
      /**
       The entry of the trial, two trials to an entry.
       */
      const entry = `cat-${String(Math.floor(trialAt / 2,),)}`;
      /**
       The pass over the pool, counted from one.
       */
      const pass = Math.floor(index / POOL_TRIALS,) + 1;
      return {
        trial: `${entry}-s0/${((trialAt % 2) === 0) ? 'insertion' : 'alteration'}`,
        entry,
        fixed: fixed(index,),
        stops: 'ok',
        ...((pass === 1) ? {} : { pass, }),
      };
    },
  );
}

/**
 Whether the unit at an index is a clean fix for the arm that fixes everything.
 */
function everyUnit(): boolean {
  return true;
}

/**
 Whether the unit at an index is a clean fix for the arm that fixes the insertions only.
 */
function insertionsOnly(index: number,): boolean {
  return ((index % POOL_TRIALS) % 2) === 0;
}

/**
 One arm of a bench.

 @param arm - arm's name
 @param cardId - card it seated
 @param rows - units it walked
 @param planned - units its planned line says, rows when omitted
 @param passes - times it was asked to walk the pool, latest pass a row names when omitted
 @param capped - whether the guard capped it after its rows
 @param declaredUsd - what its guard line says the earlier arms spent
 @param countedUsd - what its guard total says it counted

 @returns Arm's log

 @example
 ```ts
 const log = armOf({ arm: 'high', cardId: HIGH_CARD, rows, declaredUsd: 0, countedUsd: 2.5, },);
 ```
 */
function armOf(
  {
    arm,
    cardId,
    rows,
    planned,
    passes,
    capped = false,
    declaredUsd,
    countedUsd,
  }: {
    readonly arm: string;
    readonly cardId: RosterModelId;
    readonly rows: readonly RowSpec[];
    readonly planned?: number;
    readonly passes?: number;
    readonly capped?: boolean;
    readonly declaredUsd: number;
    readonly countedUsd: number;
  },
): ArmLog {
  return armLogOf({
    arm,
    tally: tallyFromRows({ cardId, rows, },),
    stops: [],
    rows,
    ...((planned === undefined) ? {} : { planned, }),
    ...((passes === undefined) ? {} : { passes, }),
    capped,
    seat: `SEAT ${cardId} asked=${String(rows.length,)} usable=${String(rows.length,)} unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`,
    benchSpentUsd: declaredUsd,
    guardUsd: countedUsd,
  },);
}

/**
 A bench of three arms that walked the same units, high fixing every one and the others the insertions.

 @param count - units each arm walked
 @param planned - units each planned line says, the units walked when omitted
 @param firstPlanned - units the first arm's planned line says, `planned` when omitted
 @param passes - times each arm was asked to walk the pool, as many as the larger planned count needs

 @returns Three logs, high first

 @example
 ```ts
 const arms = benchOf({ count: 264, },);
 ```
 */
function benchOf(
  {
    count,
    planned = count,
    firstPlanned = planned,
    passes = Math.ceil(Math.max(planned, firstPlanned,) / POOL_TRIALS,),
  }: {
    readonly count: number;
    readonly planned?: number;
    readonly firstPlanned?: number;
    readonly passes?: number;
  },
): readonly ArmLog[] {
  return [
    armOf({
      arm: 'high',
      cardId: HIGH_CARD,
      rows: unitRows({ count, fixed: everyUnit, },),
      planned: firstPlanned,
      passes,
      capped: firstPlanned > count,
      declaredUsd: 0,
      countedUsd: 2.5,
    },),
    armOf({
      arm: 'medium',
      cardId: MEDIUM_CARD,
      rows: unitRows({ count, fixed: insertionsOnly, },),
      planned,
      passes,
      declaredUsd: 2.5,
      countedUsd: 1.5,
    },),
    armOf({
      arm: 'low',
      cardId: LOW_CARD,
      rows: unitRows({ count, fixed: insertionsOnly, },),
      planned,
      passes,
      declaredUsd: 4,
      countedUsd: 0.875,
    },),
  ];
}

/**
 The first arm of the billed run of `dd4b15b75c` as its log read: the pool once with a call for each unit, then 62 units of
 the second pass answered by the prompt reuse layer, each with a `PROMPT-REUSE` line in front of its row.

 @returns Arm's log, 150 rows of which 62 made no call

 @example
 ```ts
 const log = reusedHighArm();
 ```
 */
function reusedHighArm(): ArmLog {
  return armOf({
    arm: 'high',
    cardId: HIGH_CARD,
    rows: unitRows({ count: 150, fixed: everyUnit, },)
      .map(function replayed(
        row,
        at,
      ): RowSpec {
        return (at < POOL_TRIALS) ? row : { ...row, reused: true, };
      },),
    planned: 264,
    passes: 3,
    capped: true,
    declaredUsd: 0,
    countedUsd: 2.5,
  },);
}

/**
 The line a check of an arm printed.

 @param log - arm
 @param check - check's name

 @returns Line

 @throws {@link Error} when the checks print none for it

 @example
 ```ts
 const line = lineFor({ log, check: 'planned line', },);
 ```
 */
function lineFor({ log, check, }: { readonly log: ArmLog; readonly check: string; },): string {
  return nonNullishOrThrow(checkEditorArms({ arms: [log,], },).lines.find(function isCheck(line,): boolean {
    return line.includes(`${log.arm}: ${check}`,);
  },),);
}

/**
 Runs the built read step over the arms' logs.

 @param arms - logs, in the order the arms ran

 @returns Exit status and standard output

 @example
 ```ts
 const run = await readAsBuilt({ arms, },);
 ```
 */
function readAsBuilt({ arms, }: { readonly arms: readonly ArmLog[]; },): ReturnType<typeof readAsBuiltIn> {
  return readAsBuiltIn({
    arms,
    prefix: 'editor-fidelity-passes-',
  },);
}

await describe({
  name: 'the read step over arms that walk the pool in passes',
  children: [
    it({
      name: 'PASSES THE PLANNED CHECK OF AN ARM THAT WALKS THREE WHOLE PASSES: the planned line of 264 units over a pool of '
        + '88 trials is within the pool\'s three passes, and the ceiling, EDITOR and row checks agree with it',
      fn: async () => {
        const [high,] = benchOf({ count: 264, },);
        const log = nonNullishOrThrow(high,);
        expect(lineFor({ log, check: 'planned line', },),).toBe(
          'PASS high: planned line: 264 units planned in 3 passes over a pool of 88 trials',
        );
        expect(lineFor({ log, check: 'ceiling line', },),).toBe('PASS high: ceiling line: 264 units priced',);
        expect(lineFor({ log, check: 'EDITOR line', },).startsWith(
          `PASS high: EDITOR line: EDITOR ${HIGH_CARD}: clean fix 264 of 264`,
        ),).toBe(true,);
        expect(lineFor({ log, check: 'row count', },),).toBe('PASS high: row count: 264 rows logged, each once',);
        expect(checkEditorArms({ arms: [log,], },).failed,).toBe(0,);
      },
    },),

    it({
      name: 'PASSES THE CHECKS OF A FIRST ARM THE GUARD CAPPED AT 150 OF 264 UNITS: the planned line still says 264, the '
        + 'CAPPED line says why 150 rows were logged, and the row check counts the units across the passes',
      fn: async () => {
        const [high,] = benchOf({ count: 150, planned: 264, },);
        const log = nonNullishOrThrow(high,);
        expect(log.lines.some(function isCapped(line,): boolean {
          return line.includes('CAPPED at unit 151 of 264: 150 units completed; the guard refused',);
        },),).toBe(true,);
        expect(lineFor({ log, check: 'planned line', },),).toBe(
          'PASS high: planned line: 264 units planned in 3 passes over a pool of 88 trials',
        );
        expect(lineFor({ log, check: 'row count', },),).toBe(
          'PASS high: row count: 150 rows logged, each once; the guard capped the arm before unit 151',
        );
        expect(checkEditorArms({ arms: [log,], },).failed,).toBe(0,);
      },
    },),

    it({
      name: 'PASSES THE CHECKS OF A LATER ARM GIVEN --trials 150 OF THREE PASSES: its planned line says 150 of 264 asks '
        + 'and its ceiling is priced over the 150',
      fn: async () => {
        const log = nonNullishOrThrow(benchOf({ count: 150, planned: 150, firstPlanned: 264, },)[1],);
        expect(log.lines.some(function isPlanned(line,): boolean {
          return line.includes('planned: 150 of 264 asks (3 passes over the 88 trials of the pool)',);
        },),).toBe(true,);
        expect(lineFor({ log, check: 'planned line', },),).toBe(
          'PASS medium: planned line: 150 units planned in 3 passes over a pool of 88 trials',
        );
        expect(lineFor({ log, check: 'ceiling line', },),).toBe('PASS medium: ceiling line: 150 units priced',);
        expect(checkEditorArms({ arms: [log,], },).failed,).toBe(0,);
      },
    },),

    it({
      name: 'KEEPS THE SINGLE PASS WORDING: a planned line of 88 of 88 trials, and a capped one of 41 of 88, each passes '
        + 'against the pool of 88 trials and says how many trials were planned of it',
      fn: async () => {
        expect(lineFor({
          log: nonNullishOrThrow(benchOf({ count: 88, },)[0],),
          check: 'planned line',
        },),).toBe('PASS high: planned line: 88 trials planned of a pool of 88',);
        expect(lineFor({
          log: nonNullishOrThrow(benchOf({ count: 41, planned: 41, },)[1],),
          check: 'planned line',
        },),).toBe('PASS medium: planned line: 41 trials planned of a pool of 88',);
      },
    },),

    it({
      name: 'FAILS THE PLANNED CHECK OF AN ARM THAT WALKS THREE PASSES WHEN NO POOL LINE WAS LOGGED, saying so in place of a '
        + 'pool of -1 trials',
      fn: async () => {
        const log = nonNullishOrThrow(benchOf({ count: 264, },)[0],);
        expect(lineFor({
          log: {
            ...log,
            lines: log.lines.filter(function notPool(line,): boolean {
              return !line.includes('pool: ',);
            },),
          },
          check: 'planned line',
        },),).toBe('FAIL high: planned line: 264 units planned, and no pool line was logged to hold them against',);
      },
    },),

    it({
      name: 'FAILS THE PLANNED CHECK FOR MORE UNITS THAN THE PASSES HOLD: 265 planned of three passes over 88 trials, and '
        + '200 planned of two passes, each naming how many the passes hold',
      fn: async () => {
        expect(lineFor({
          log: armOf({
            arm: 'high',
            cardId: HIGH_CARD,
            rows: unitRows({ count: 264, fixed: everyUnit, },),
            planned: 265,
            declaredUsd: 0,
            countedUsd: 2.5,
          },),
          check: 'planned line',
        },),).toBe(
          'FAIL high: planned line: 265 units planned in 3 passes over a pool of 88 trials, more than the 264 units the passes hold',
        );
        expect(lineFor({
          log: armOf({
            arm: 'high',
            cardId: HIGH_CARD,
            rows: unitRows({ count: 176, fixed: everyUnit, },),
            planned: 200,
            declaredUsd: 0,
            countedUsd: 2.5,
          },),
          check: 'planned line',
        },),).toBe(
          'FAIL high: planned line: 200 units planned in 2 passes over a pool of 88 trials, more than the 176 units the passes hold',
        );
      },
    },),

    it({
      name: 'READS A BENCH OF THREE WHOLE PASSES TO A VERDICT: no check fails, 264 units are shared over 44 entries, and '
        + 'the last line is the verdict, high shown better than medium',
      fn: async () => {
        const arms = benchOf({ count: 264, },);
        const report = checkEditorBench({ arms, },);
        expect(report.lines.filter(function isFail(line,): boolean {
          return line.startsWith('FAIL ',);
        },),).toStrictEqual([],);
        expect(report.failed,).toBe(0,);
        const lines = benchReportLines({ arms, },);
        expect(lines.some(function isShared(line,): boolean {
          return line.startsWith('SHARED 264 units over 44 entries (',);
        },),).toBe(true,);
        expect(lines.some(function isPaired(line,): boolean {
          return line.startsWith('PAIRED high vs medium: +50.0 points',);
        },),).toBe(true,);
        expect(lines.at(-1,)?.startsWith('VERDICT high (shown-better): ',),).toBe(true,);
      },
    },),

    it({
      name: 'READS A BENCH WHOSE FIRST ARM WAS CAPPED AT 150 OF 264 UNITS, the later arms given --trials 150, to a '
        + 'verdict over the 150 units every arm walked',
      fn: async () => {
        const arms = benchOf({ count: 150, planned: 150, firstPlanned: 264, },);
        const report = checkEditorBench({ arms, },);
        expect(report.lines.filter(function isFail(line,): boolean {
          return line.startsWith('FAIL ',);
        },),).toStrictEqual([],);
        const lines = benchReportLines({ arms, },);
        expect(lines.some(function isShared(line,): boolean {
          return line.startsWith('SHARED 150 units over 44 entries (',);
        },),).toBe(true,);
        expect(lines.some(function isPaired(line,): boolean {
          return line.startsWith('PAIRED high vs medium: +',);
        },),).toBe(true,);
        expect(lines.at(-1,)?.startsWith('VERDICT ',),).toBe(true,);
      },
    },),

    it({
      name: 'READS A SINGLE PASS BENCH OF 88 UNITS TO A VERDICT: no check fails, 88 units are shared over 44 entries, and '
        + 'the last line is the verdict, high shown better than medium',
      fn: async () => {
        const arms = benchOf({ count: 88, },);
        expect(checkEditorBench({ arms, },).failed,).toBe(0,);
        const lines = benchReportLines({ arms, },);
        expect(lines.some(function isShared(line,): boolean {
          return line.startsWith('SHARED 88 units over 44 entries (',);
        },),).toBe(true,);
        expect(lines.at(-1,)?.startsWith('VERDICT high (shown-better): ',),).toBe(true,);
      },
    },),

    it({
      name: 'COUNTS ONLY THE ROWS THAT MADE A CALL: of the 150 rows the billed first arm logged, 62 follow a PROMPT-REUSE '
        + 'line, so its figures hold 88 rows and say 62 were answered by reuse',
      fn: async () => {
        const figures = armFiguresOf({ log: reusedHighArm(), },);
        expect(figures.rows.length,).toBe(POOL_TRIALS,);
        expect(figures.reusedRows,).toBe(62,);
        expect(figures.rows.every(function inFirstPass({ pass, },): boolean {
          return pass === 1;
        },),).toBe(true,);
      },
    },),

    it({
      name: 'FAILS THE ROW CHECK OF AN ARM WITH ROWS THAT MADE NO CALL, saying how many of its rows reuse answered, and '
        + 'passes the arm whose rows all made a call',
      fn: async () => {
        expect(lineFor({ log: reusedHighArm(), check: 'row count', },),).toBe(
          'FAIL high: row count: 62 of 150 rows were answered by prompt reuse and made no call, so none of them is a trial',
        );
        expect(lineFor({ log: benchOf({ count: 150, planned: 150, firstPlanned: 264, },)[0] as ArmLog, check: 'row count', },),)
          .toContain('PASS high: row count: ',);
      },
    },),

    it({
      name: 'SHARES ONLY THE UNITS THAT MADE A CALL IN EVERY ARM: the billed first arm with 62 replayed rows, and a medium '
        + 'and a low arm that walked all 150 units for real, share 88 units, print no repeat reading of the replays, '
        + 'and fail the checks that the first arm and the later arms walked the same units',
      fn: async () => {
        const [, medium, low,] = benchOf({ count: 150, planned: 150, firstPlanned: 264, },);
        const arms = [reusedHighArm(), nonNullishOrThrow(medium,), nonNullishOrThrow(low,),];
        const lines = benchReportLines({ arms, },);
        expect(lines.some(function isShared(line,): boolean {
          return line.startsWith('SHARED 88 units over 44 entries (',);
        },),).toBe(true,);
        expect(lines.filter(function readsRepeats(line,): boolean {
          return line.startsWith('REPEAT high',) && line.includes('gave the same reading in every pass',);
        },),).toStrictEqual([],);
        expect(checkEditorBench({ arms, },).failed,).toBeGreaterThan(0,);
        expect(checkEditorBench({ arms, },).lines,).toContain(
          'FAIL medium: units in the first arm\'s order: it walked 150 units and high completed 88',
        );
      },
    },),

    it({
      name: 'REFUSES THE BILLED LOGS AS THE BUILT COMMAND: the first arm\'s 62 rows that reuse answered make the row check '
        + 'fail, so the command exits with a failure and prints that line, and the replays are no repeat reading',
      fn: async () => {
        const [, medium, low,] = benchOf({ count: 150, planned: 150, firstPlanned: 264, },);
        const run = await readAsBuilt({ arms: [reusedHighArm(), nonNullishOrThrow(medium,), nonNullishOrThrow(low,),], },);
        expect(run.stdout,).toContain(
          'FAIL high: row count: 62 of 150 rows were answered by prompt reuse and made no call, so none of them is a trial',
        );
        expect(run.code,).not.toBe(0,);
        // The figures are printed only for logs whose checks passed; the repeat reading is printed with the checks.
        expect(run.stdout.some(function readsNoRepeat(line,): boolean {
          return line.startsWith('REPEAT high',) && line.includes('no trial was asked in more than one pass',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'READS THE LOGS OF THREE WHOLE PASSES AS THE BUILT COMMAND, which exits zero, prints no FAIL line and ends on '
        + 'the verdict',
      fn: async () => {
        const run = await readAsBuilt({ arms: benchOf({ count: 264, },), },);
        expect(run.stdout.filter(function isFail(line,): boolean {
          return line.startsWith('FAIL ',);
        },),).toStrictEqual([],);
        expect(run.code,).toBe(0,);
        expect(run.stdout.some(function isChecks(line,): boolean {
          return line.startsWith('CHECKS ',) && line.endsWith(' pass, 0 fail',);
        },),).toBe(true,);
        expect(run.stdout.some(function isPaired(line,): boolean {
          return line.startsWith('PAIRED high vs medium: ',);
        },),).toBe(true,);
        expect(run.stdout.at(-1,)?.startsWith('VERDICT ',),).toBe(true,);
      },
    },),
  ],
},);
