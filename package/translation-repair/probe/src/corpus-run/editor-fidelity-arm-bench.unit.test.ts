/**
 Tests for the checks that make three or four logs a bench: distinct arm logs, run at the cards high, medium, a second
 medium if one was run, and low in that order, each arm having declared at least what the guards of the arms before it
 counted, every later arm having walked the first arm's trials in its order, and enough trials shared by every arm.

 The logs hold only the lines these checks read, written by the probe's own writers, so a writer that changes its wording
 fails here before it fails an operator. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ArmLog,
  checkEditorBench,
} from '../../dist/final/node/index.mjs';
import {
  armLogOf,
  inPass,
  poolRows,
  type RowSpec,
  tallyFromRows,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import {
  HIGH_CARD,
  LOW_CARD,
  MEDIUM_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 Trials the first arm of a good bench completes.
 */
const FIRST_ARM_TRIALS = 64;

/**
 The rows of a good arm: the leading trials, every other one a clean fix, all asks ok.
 */
const GOOD_ROWS: readonly RowSpec[] = poolRows({
  count: FIRST_ARM_TRIALS,
  fixed: function everyOther(index,): boolean {
    return (index % 2) === 0;
  },
});

/**
 One arm's log, holding the lines the bench checks read.

 @param arm - arm's name
 @param cardId - card it seated
 @param rows - rows it walked
 @param declaredUsd - what its guard line says the earlier arms spent
 @param countedUsd - what its guard total says it counted
 @param capped - whether the guard capped it after its rows

 @returns Arm

 @example
 ```ts
 const log = armOf({ arm: 'high', cardId: HIGH_CARD, rows: GOOD_ROWS, declaredUsd: 0, countedUsd: 0.875, },);
 ```
 */
function armOf(
  {
    arm,
    cardId,
    rows = GOOD_ROWS,
    declaredUsd,
    countedUsd,
    capped = false,
  }: {
    readonly arm: string;
    readonly cardId: RosterModelId;
    readonly rows?: readonly RowSpec[];
    readonly declaredUsd: number;
    readonly countedUsd: number;
    readonly capped?: boolean;
  },
): ArmLog {
  return armLogOf({
    arm,
    tally: tallyFromRows({ cardId, rows, },),
    stops: [],
    rows,
    planned: capped ? FIRST_ARM_TRIALS : rows.length,
    capped,
    seat: `SEAT ${cardId} asked=${String(rows.length,)} usable=${String(rows.length,)} unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`,
    benchSpentUsd: declaredUsd,
    guardUsd: countedUsd,
  },);
}

/**
 The medium arm with the first rows given as ones in which an ask threw.

 @param count - how many leading rows threw

 @returns Arm

 @example
 ```ts
 const log = mediumThrowing({ count: 3, },);
 ```
 */
function mediumThrowing({ count, }: { readonly count: number; },): ArmLog {
  return armOf({
    arm: 'medium',
    cardId: MEDIUM_CARD,
    rows: GOOD_ROWS.map(function thrown(
      row,
      at,
    ): RowSpec {
      return (at < count) ? { ...row, stops: 'threw+ok', } : row;
    },),
    declaredUsd: 2.5,
    countedUsd: 1.5,
  },);
}

/**
 A bench that ran as it should, one medium arm: each arm declared the sum of the counts before it, to the cent.
 */
const GOOD_BENCH: readonly ArmLog[] = [
  armOf({ arm: 'high', cardId: HIGH_CARD, declaredUsd: 0, countedUsd: 2.5, },),
  armOf({ arm: 'medium', cardId: MEDIUM_CARD, declaredUsd: 2.5, countedUsd: 1.5, },),
  armOf({ arm: 'low', cardId: LOW_CARD, declaredUsd: 4, countedUsd: 0.875, },),
];

/**
 A bench with the second medium arm that the code still reads when one is given.
 */
const FOUR_ARM_BENCH: readonly ArmLog[] = [
  GOOD_BENCH[0] as ArmLog,
  GOOD_BENCH[1] as ArmLog,
  armOf({ arm: 'medium-b', cardId: MEDIUM_CARD, declaredUsd: 4, countedUsd: 1.25, },),
  armOf({ arm: 'low', cardId: LOW_CARD, declaredUsd: 5.25, countedUsd: 0.875, },),
];

/**
 The line a bench check printed.

 @param lines - report's lines
 @param scope - arm or `bench`
 @param check - check's name

 @returns Line, empty when there is none

 @example
 ```ts
 const line = lineFor({ lines, scope: 'bench', check: 'shared units', },);
 ```
 */
function lineFor(
  {
    lines,
    scope,
    check,
  }: {
    readonly lines: readonly string[];
    readonly scope: string;
    readonly check: string;
  },
): string {
  return lines.find(function isCheck(line,): boolean {
    return line.slice(5,)
      .startsWith(`${scope}: ${check}`,);
  },) ?? '';
}

await describe({
  name: checkEditorBench.name,
  children: [
    it({
      name: 'PASSES THE BENCH CHECKS for three distinct logs at the cards high, medium and low in the order run, each '
        + 'declaring at least what the arms before it counted, the later arms walking the first arm\'s trials and enough '
        + 'trials shared',
      fn: async () => {
        const { lines, } = checkEditorBench({ arms: GOOD_BENCH, },);
        expect(lineFor({ lines, scope: 'bench', check: 'three or four distinct arm logs', },),).toBe(
          'PASS bench: three or four distinct arm logs: high, medium, low',
        );
        expect(lineFor({ lines, scope: 'bench', check: 'cards high, medium, a second medium if run, low', },),).toBe(
          'PASS bench: cards high, medium, a second medium if run, low in the order run',
        );
        expect(lineFor({ lines, scope: 'high', check: 'earlier spend declared', },),).toBe(
          'PASS high: earlier spend declared: --bench-spent-usd 0 covers the 0.000000 USD the arms before it counted',
        );
        expect(lineFor({ lines, scope: 'low', check: 'earlier spend declared', },),).toBe(
          'PASS low: earlier spend declared: --bench-spent-usd 4 covers the 4.000000 USD the arms before it counted',
        );
        expect(lineFor({ lines, scope: 'low', check: 'units in the first arm\'s order', },),).toBe(
          'PASS low: units in the first arm\'s order: the 64 units it walked are the first 64 of high\'s',
        );
        expect(lineFor({ lines, scope: 'bench', check: 'shared units', },),).toBe(
          'PASS bench: shared units: 64 shared units over 32 entries, 0 left out as thrown and 0 not completed by '
            + 'every arm, at least the 60 the rule needs',
        );
      },
    },),

    it({
      name: 'READS A SECOND MEDIUM ARM WHEN FOUR LOGS ARE GIVEN, at the cards high, medium, medium and low',
      fn: async () => {
        const { lines, } = checkEditorBench({ arms: FOUR_ARM_BENCH, },);
        expect(lineFor({ lines, scope: 'bench', check: 'three or four distinct arm logs', },),).toBe(
          'PASS bench: three or four distinct arm logs: high, medium, medium-b, low',
        );
        expect(lineFor({ lines, scope: 'bench', check: 'cards high, medium, a second medium if run, low', },),).toBe(
          'PASS bench: cards high, medium, a second medium if run, low in the order run',
        );
      },
    },),

    it({
      name: 'FAILS THE DISTINCT-LOGS CHECK for fewer than three or more than four logs, naming how many were given',
      fn: async () => {
        expect(lineFor({
          lines: checkEditorBench({ arms: GOOD_BENCH.slice(0, 2,), },).lines,
          scope: 'bench',
          check: 'three or four distinct arm logs',
        },),).toBe('FAIL bench: three or four distinct arm logs: 2 logs were given, and a bench is three or four',);
        expect(lineFor({
          lines: checkEditorBench({ arms: [...FOUR_ARM_BENCH, ...GOOD_BENCH,], },).lines,
          scope: 'bench',
          check: 'three or four distinct arm logs',
        },),).toContain('FAIL bench: three or four distinct arm logs: 7 logs were given, and a bench is three or four',);
      },
    },),

    it({
      name: 'FAILS THE DISTINCT-LOGS CHECK for one log given twice, under one name or two, since a repeated log would count '
        + 'one arm as two',
      fn: async () => {
        /**
         The low arm's log replaced by the medium arm's, under its own name.
         */
        const copied: readonly ArmLog[] = [
          ...GOOD_BENCH.slice(0, 2,),
          {
            ...GOOD_BENCH[1] as ArmLog,
            arm: 'low',
          },
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: copied, },).lines,
          scope: 'bench',
          check: 'three or four distinct arm logs',
        },),).toBe('FAIL bench: three or four distinct arm logs: medium and low hold the same lines',);
        /**
         The first log named twice.
         */
        const named: readonly ArmLog[] = [GOOD_BENCH[0] as ArmLog, ...GOOD_BENCH.slice(0, 2,),];
        expect(lineFor({
          lines: checkEditorBench({ arms: named, },).lines,
          scope: 'bench',
          check: 'three or four distinct arm logs',
        },),).toBe('FAIL bench: three or four distinct arm logs: high and high hold the same lines',);
      },
    },),

    it({
      name: 'FAILS THE CARDS CHECK for logs run at other cards or in another order, naming the cards they ran',
      fn: async () => {
        /**
         Low before the medium arm.
         */
        const reordered: readonly ArmLog[] = [
          GOOD_BENCH[0] as ArmLog,
          GOOD_BENCH[2] as ArmLog,
          GOOD_BENCH[1] as ArmLog,
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: reordered, },).lines,
          scope: 'bench',
          check: 'cards high, medium, a second medium if run, low',
        },),).toBe(
          'FAIL bench: cards high, medium, a second medium if run, low in the order run: the logs ran high, low, medium',
        );
        /**
         The old order, with low first, the mediums twice, and high last.
         */
        const old: readonly ArmLog[] = [
          armOf({ arm: 'low', cardId: LOW_CARD, declaredUsd: 0, countedUsd: 0.875, },),
          GOOD_BENCH[1] as ArmLog,
          armOf({ arm: 'medium-b', cardId: MEDIUM_CARD, declaredUsd: 2.38, countedUsd: 1.25, },),
          armOf({ arm: 'high', cardId: HIGH_CARD, declaredUsd: 3.63, countedUsd: 2.5, },),
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: old, },).lines,
          scope: 'bench',
          check: 'cards high, medium, a second medium if run, low',
        },),).toContain('the logs ran low, medium, medium, high',);
      },
    },),

    it({
      name: 'FAILS THE EARLIER-SPEND CHECK of an arm that declared less than the arms before it counted, or declared '
        + 'nothing, and passes the arms before it',
      fn: async () => {
        /**
         The low arm declared a spend short of the two arms before it.
         */
        const short: readonly ArmLog[] = [
          ...GOOD_BENCH.slice(0, 2,),
          armOf({ arm: 'low', cardId: LOW_CARD, declaredUsd: 3.99, countedUsd: 0.875, },),
        ];
        const { lines, } = checkEditorBench({ arms: short, },);
        expect(lineFor({ lines, scope: 'low', check: 'earlier spend declared', },),).toBe(
          'FAIL low: earlier spend declared: --bench-spent-usd 3.99 is less than the 4.000000 USD the arms '
            + 'before it counted',
        );
        expect(lineFor({ lines, scope: 'medium', check: 'earlier spend declared', },).startsWith('PASS ',),).toBe(true,);
        /**
         An arm whose guard line states no spend.
         */
        const silent: readonly ArmLog[] = [
          GOOD_BENCH[0] as ArmLog,
          {
            ...GOOD_BENCH[1] as ArmLog,
            lines: (GOOD_BENCH[1] as ArmLog).lines.filter(function notGuard(line,): boolean {
              return !line.includes('spend guard: ',);
            },),
          },
          GOOD_BENCH[2] as ArmLog,
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: silent, },).lines,
          scope: 'medium',
          check: 'earlier spend declared',
        },),).toBe('FAIL medium: earlier spend declared: the guard line states no --bench-spent-usd',);
      },
    },),

    it({
      name: 'FAILS THE TRIAL-ORDER CHECK of a later arm that walked another trial than the first arm at some row, or more '
        + 'trials than the first arm completed, naming the row and both trials',
      fn: async () => {
        /**
         The low arm walked the alteration of the second entry where the first arm walked its insertion.
         */
        const swapped = GOOD_ROWS.map(function swap(
          row,
          at,
        ): RowSpec {
          return (at === 2) ? { ...row, trial: 'cat-1-s0/alteration', } : row;
        },);
        expect(lineFor({
          lines: checkEditorBench({
            arms: [
              GOOD_BENCH[0] as ArmLog,
              GOOD_BENCH[1] as ArmLog,
              armOf({ arm: 'low', cardId: LOW_CARD, rows: swapped, declaredUsd: 4, countedUsd: 0.875, },),
            ],
          },).lines,
          scope: 'low',
          check: 'units in the first arm\'s order',
        },),).toBe(
          'FAIL low: units in the first arm\'s order: row 3 walked cat-1-s0/alteration where high walked cat-1-s0/insertion',
        );
        /**
         The low arm walked past what the first arm completed.
         */
        const longer = poolRows({
          count: FIRST_ARM_TRIALS + 2,
          fixed: function every(): boolean {
            return true;
          },
        });
        expect(lineFor({
          lines: checkEditorBench({
            arms: [
              GOOD_BENCH[0] as ArmLog,
              GOOD_BENCH[1] as ArmLog,
              armOf({ arm: 'low', cardId: LOW_CARD, rows: longer, declaredUsd: 4, countedUsd: 0.875, },),
            ],
          },).lines,
          scope: 'low',
          check: 'units in the first arm\'s order',
        },),).toBe(
          'FAIL low: units in the first arm\'s order: it walked 66 units and high completed 64',
        );
      },
    },),

    it({
      name: 'CHECKS THE PASS AS WELL AS THE TRIAL IN THE ORDER CHECK: a later arm that walked the first pass again where '
        + 'the first arm walked its second fails at that row, naming the pass, and one that walked both passes in order '
        + 'passes',
      fn: async () => {
        /**
         The rows of two passes over the same trials.
         */
        const twoPasses: readonly RowSpec[] = [...GOOD_ROWS, ...inPass({ rows: GOOD_ROWS, pass: 2, },),];
        /**
         The first arm and a medium arm that walked both passes.
         */
        const first = [
          armOf({ arm: 'high', cardId: HIGH_CARD, rows: twoPasses, declaredUsd: 0, countedUsd: 2.5, },),
          armOf({ arm: 'medium', cardId: MEDIUM_CARD, rows: twoPasses, declaredUsd: 2.5, countedUsd: 1.5, },),
        ];
        expect(lineFor({
          lines: checkEditorBench({
            arms: [
              ...first,
              armOf({ arm: 'low', cardId: LOW_CARD, rows: twoPasses, declaredUsd: 4, countedUsd: 0.875, },),
            ],
          },).lines,
          scope: 'low',
          check: 'units in the first arm\'s order',
        },),).toBe('PASS low: units in the first arm\'s order: the 128 units it walked are the first 128 of high\'s',);
        expect(lineFor({
          lines: checkEditorBench({
            arms: [
              ...first,
              armOf({
                arm: 'low',
                cardId: LOW_CARD,
                rows: [...GOOD_ROWS, ...GOOD_ROWS,],
                declaredUsd: 4,
                countedUsd: 0.875,
              },),
            ],
          },).lines,
          scope: 'low',
          check: 'units in the first arm\'s order',
        },),).toBe(
          'FAIL low: units in the first arm\'s order: row 65 walked cat-0-s0/insertion where high walked '
            + 'cat-0-s0/insertion (pass 2)',
        );
      },
    },),

    it({
      name: 'FAILS THE SHARED-UNITS CHECK when fewer than sixty units remain shared by every arm, and says how many were '
        + 'left out as thrown and how many an arm did not complete; passes at sixty-one with three thrown',
      fn: async () => {
        /**
         The low arm capped after fifty-nine trials.
         */
        const capped = armOf({
          arm: 'low',
          cardId: LOW_CARD,
          rows: GOOD_ROWS.slice(0, 59,),
          declaredUsd: 4,
          countedUsd: 0.875,
          capped: true,
        },);
        expect(lineFor({
          lines: checkEditorBench({ arms: [GOOD_BENCH[0] as ArmLog, GOOD_BENCH[1] as ArmLog, capped,], },).lines,
          scope: 'bench',
          check: 'shared units',
        },),).toBe(
          'FAIL bench: shared units: 59 shared units remain of the 64 the first arm completed (0 left out as thrown, 5 not completed by every arm), under the 60 the rule needs',
        );
        expect(lineFor({
          lines: checkEditorBench({
            arms: [GOOD_BENCH[0] as ArmLog, mediumThrowing({ count: 3, },), GOOD_BENCH[2] as ArmLog,],
          },).lines,
          scope: 'bench',
          check: 'shared units',
        },),).toBe(
          'PASS bench: shared units: 61 shared units over 31 entries, 3 left out as thrown and 0 not completed by every arm, at least the 60 the rule needs',
        );
        expect(lineFor({
          lines: checkEditorBench({
            arms: [GOOD_BENCH[0] as ArmLog, mediumThrowing({ count: 5, },), GOOD_BENCH[2] as ArmLog,],
          },).lines,
          scope: 'bench',
          check: 'shared units',
        },).startsWith(
          'FAIL bench: shared units: 59 shared units remain of the 64 the first arm completed (5 left out as thrown, 0 not completed by every arm)',
        ),).toBe(true,);
      },
    },),

    it({
      name: 'COUNTS UNITS, NOT TRIALS, AGAINST THE FLOOR: two passes over thirty trials are sixty units and pass it, '
        + 'where one pass over the same thirty trials is thirty units and fails it',
      fn: async () => {
        /**
         Thirty trials, fifteen entries.
         */
        const thirty = poolRows({
          count: 30,
          fixed: function everyOther(index,): boolean {
            return (index % 2) === 0;
          },
        });
        /**
         Both passes over them.
         */
        const both: readonly RowSpec[] = [...thirty, ...inPass({ rows: thirty, pass: 2, },),];
        /**
         A bench of three arms over the given rows.
         */
        const benchOver = (rows: readonly RowSpec[],): readonly ArmLog[] => [
          armOf({ arm: 'high', cardId: HIGH_CARD, rows, declaredUsd: 0, countedUsd: 2.5, },),
          armOf({ arm: 'medium', cardId: MEDIUM_CARD, rows, declaredUsd: 2.5, countedUsd: 1.5, },),
          armOf({ arm: 'low', cardId: LOW_CARD, rows, declaredUsd: 4, countedUsd: 0.875, },),
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: benchOver(both,), },).lines,
          scope: 'bench',
          check: 'shared units',
        },),).toBe(
          'PASS bench: shared units: 60 shared units over 15 entries, 0 left out as thrown and 0 not completed by every arm, at least the 60 the rule needs',
        );
        expect(lineFor({
          lines: checkEditorBench({ arms: benchOver(thirty,), },).lines,
          scope: 'bench',
          check: 'shared units',
        },),).toBe(
          'FAIL bench: shared units: 30 shared units remain of the 30 the first arm completed (0 left out as thrown, 0 not completed by every arm), under the 60 the rule needs',
        );
      },
    },),

    it({
      name: 'KEEPS THE CHECKS OF EACH ARM and counts every failure, the bench checks with them',
      fn: async () => {
        /**
         Two logs, which no bench is.
         */
        const report = checkEditorBench({ arms: GOOD_BENCH.slice(0, 2,), },);
        expect(report.lines,).toContain(
          'FAIL bench: three or four distinct arm logs: 2 logs were given, and a bench is three or four',
        );
        expect(report.lines.some(function isArmCheck(line,): boolean {
          return line.startsWith('PASS high: manifest line',);
        },),).toBe(true,);
        expect(report.lines.length,).toBeGreaterThan(20,);
        expect(report.failed,).toBe(report.lines.filter(function isFail(line,): boolean {
          return line.startsWith('FAIL ',);
        },).length,);
      },
    },),
  ],
},);
