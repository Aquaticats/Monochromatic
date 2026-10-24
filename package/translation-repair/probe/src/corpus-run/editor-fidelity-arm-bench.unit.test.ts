/**
 Tests for the checks that make four logs a bench: four distinct arm logs, run at the cards low, medium, medium and high
 in that order, each arm having declared at least what the guards of the arms before it counted.

 The logs hold only the lines these checks read, written by the probe's own writers, so a writer that changes its wording
 fails here before it fails an operator. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  type ArmLog,
  checkEditorBench,
  guardLine,
  guardTotalLine,
} from '../../dist/final/node/index.mjs';
import {
  HIGH_CARD,
  LOW_CARD,
  MEDIUM_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 Prefix the logger puts on every line.
 */
const PREFIX = '[info] [2026-10-23T12:00:00.000Z] [editor-fidelity-probe] ';

/**
 One arm's log, holding the lines the bench checks read.

 @param arm - the arm's name
 @param card - card it seated
 @param declaredUsd - what its guard line says the earlier arms spent
 @param countedUsd - what its guard total says it counted

 @returns The arm

 @example
 ```ts
 const log = armOf({ arm: 'low', card: LOW_CARD, declaredUsd: 0, countedUsd: 0.875, },);
 ```
 */
function armOf(
  {
    arm,
    card,
    declaredUsd,
    countedUsd,
  }: {
    readonly arm: string;
    readonly card: string;
    readonly declaredUsd: number;
    readonly countedUsd: number;
  },
): ArmLog {
  return {
    arm,
    exit: {
      recorded: true,
      code: 0,
    },
    lines: [
      `${PREFIX}editors: ${card}`,
      `${PREFIX}${guardLine({ maxUsd: 3, benchSpentUsd: declaredUsd, },)}`,
      `${PREFIX}${guardTotalLine({ total: { usd: countedUsd, calls: 40, atCeiling: 0, }, },)}`,
    ],
  };
}

/**
 A bench that ran as it should: each arm declared the sum of the counts before it, to the cent.
 */
const GOOD_BENCH: readonly ArmLog[] = [
  armOf({ arm: 'low', card: LOW_CARD, declaredUsd: 0, countedUsd: 0.875, },),
  armOf({ arm: 'medium-a', card: MEDIUM_CARD, declaredUsd: 0.88, countedUsd: 1.5, },),
  armOf({ arm: 'medium-b', card: MEDIUM_CARD, declaredUsd: 2.38, countedUsd: 1.25, },),
  armOf({ arm: 'high', card: HIGH_CARD, declaredUsd: 3.63, countedUsd: 2.5, },),
];

/**
 The line a bench check printed.

 @param lines - the report's lines
 @param scope - the arm or `bench`
 @param check - the check's name

 @returns The line, empty when there is none

 @example
 ```ts
 const line = lineFor({ lines, scope: 'bench', check: 'four distinct arm logs', },);
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
      name: 'PASSES THE BENCH CHECKS for four distinct logs at the cards low, medium, medium and high in the order run, '
        + 'each declaring at least what the arms before it counted',
      fn: async () => {
        const { lines, } = checkEditorBench({ arms: GOOD_BENCH, },);
        expect(lineFor({ lines, scope: 'bench', check: 'four distinct arm logs', },),).toBe(
          'PASS bench: four distinct arm logs: low, medium-a, medium-b, high',
        );
        expect(lineFor({ lines, scope: 'bench', check: 'cards low, medium, medium, high', },),).toBe(
          'PASS bench: cards low, medium, medium, high in the order run',
        );
        expect(lineFor({ lines, scope: 'low', check: 'earlier spend declared', },),).toBe(
          'PASS low: earlier spend declared: --bench-spent-usd 0 covers the 0.000000 USD the arms before it counted',
        );
        expect(lineFor({ lines, scope: 'high', check: 'earlier spend declared', },),).toBe(
          'PASS high: earlier spend declared: --bench-spent-usd 3.63 covers the 3.625000 USD the arms before it counted',
        );
      },
    },),

    it({
      name: 'FAILS THE FOUR-LOGS CHECK for fewer or more than four logs, naming how many were given',
      fn: async () => {
        expect(lineFor({
          lines: checkEditorBench({ arms: GOOD_BENCH.slice(0, 3,), },).lines,
          scope: 'bench',
          check: 'four distinct arm logs',
        },),).toBe('FAIL bench: four distinct arm logs: 3 logs were given, and a bench is four',);
        expect(lineFor({
          lines: checkEditorBench({ arms: [...GOOD_BENCH, ...GOOD_BENCH,], },).lines,
          scope: 'bench',
          check: 'four distinct arm logs',
        },),).toContain('FAIL bench: four distinct arm logs: 8 logs were given, and a bench is four',);
      },
    },),

    it({
      name: 'FAILS THE FOUR-LOGS CHECK for one log given twice, under one name or two, since a repeated log would count '
        + 'one arm as two',
      fn: async () => {
        /**
         The second medium arm's log replaced by the first's, under its own name.
         */
        const copied: readonly ArmLog[] = [
          ...GOOD_BENCH.slice(0, 2,),
          {
            ...GOOD_BENCH[1] as ArmLog,
            arm: 'medium-b',
          },
          ...GOOD_BENCH.slice(3,),
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: copied, },).lines,
          scope: 'bench',
          check: 'four distinct arm logs',
        },),).toBe('FAIL bench: four distinct arm logs: medium-a and medium-b hold the same lines',);
        /**
         The first log named twice.
         */
        const named: readonly ArmLog[] = [GOOD_BENCH[0] as ArmLog, ...GOOD_BENCH.slice(0, 3,),];
        expect(lineFor({
          lines: checkEditorBench({ arms: named, },).lines,
          scope: 'bench',
          check: 'four distinct arm logs',
        },),).toBe('FAIL bench: four distinct arm logs: low and low hold the same lines',);
      },
    },),

    it({
      name: 'FAILS THE CARDS CHECK for logs run at other cards or in another order, naming the cards they ran',
      fn: async () => {
        /**
         High before the mediums.
         */
        const reordered: readonly ArmLog[] = [
          GOOD_BENCH[0] as ArmLog,
          GOOD_BENCH[3] as ArmLog,
          GOOD_BENCH[1] as ArmLog,
          GOOD_BENCH[2] as ArmLog,
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: reordered, },).lines,
          scope: 'bench',
          check: 'cards low, medium, medium, high',
        },),).toBe(
          'FAIL bench: cards low, medium, medium, high in the order run: the logs ran low, high, medium, medium',
        );
        /**
         Low twice instead of the second medium.
         */
        const doubled: readonly ArmLog[] = [
          ...GOOD_BENCH.slice(0, 2,),
          armOf({ arm: 'medium-b', card: LOW_CARD, declaredUsd: 2.38, countedUsd: 1.25, },),
          GOOD_BENCH[3] as ArmLog,
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: doubled, },).lines,
          scope: 'bench',
          check: 'cards low, medium, medium, high',
        },),).toContain('the logs ran low, medium, low, high',);
      },
    },),

    it({
      name: 'FAILS THE EARLIER-SPEND CHECK of an arm that declared less than the arms before it counted, or declared '
        + 'nothing, and passes the arms before it',
      fn: async () => {
        /**
         The second medium arm declared a spend short of the two arms before it.
         */
        const short: readonly ArmLog[] = [
          ...GOOD_BENCH.slice(0, 2,),
          armOf({ arm: 'medium-b', card: MEDIUM_CARD, declaredUsd: 2.37, countedUsd: 1.25, },),
          GOOD_BENCH[3] as ArmLog,
        ];
        const { lines, } = checkEditorBench({ arms: short, },);
        expect(lineFor({ lines, scope: 'medium-b', check: 'earlier spend declared', },),).toBe(
          'FAIL medium-b: earlier spend declared: --bench-spent-usd 2.37 is less than the 2.375000 USD the arms '
            + 'before it counted',
        );
        expect(lineFor({ lines, scope: 'medium-a', check: 'earlier spend declared', },).startsWith('PASS ',),).toBe(
          true,
        );
        // The high arm declared 3.63, which still covers the 3.625 the arms before it counted.
        expect(lineFor({ lines, scope: 'high', check: 'earlier spend declared', },).startsWith('PASS ',),).toBe(true,);
        /**
         An arm whose guard line states no spend.
         */
        const silent: readonly ArmLog[] = [
          armOf({ arm: 'low', card: LOW_CARD, declaredUsd: 0, countedUsd: 0.875, },),
          {
            ...GOOD_BENCH[1] as ArmLog,
            lines: (GOOD_BENCH[1] as ArmLog).lines.filter(function notGuard(line,): boolean {
              return !line.includes('spend guard: ',);
            },),
          },
          ...GOOD_BENCH.slice(2,),
        ];
        expect(lineFor({
          lines: checkEditorBench({ arms: silent, },).lines,
          scope: 'medium-a',
          check: 'earlier spend declared',
        },),).toBe('FAIL medium-a: earlier spend declared: the guard line states no --bench-spent-usd',);
      },
    },),

    it({
      name: 'KEEPS THE CHECKS OF EACH ARM and counts every failure, the bench checks with them',
      fn: async () => {
        /**
         Two logs, which no bench is.
         */
        const report = checkEditorBench({ arms: GOOD_BENCH.slice(0, 2,), },);
        expect(report.lines.some(function isExit(line,): boolean {
          return line.startsWith('FAIL low: manifest line',);
        },),).toBe(true,);
        expect(report.lines.length,).toBeGreaterThan(20,);
        expect(report.failed,).toBe(report.lines.filter(function isFail(line,): boolean {
          return line.startsWith('FAIL ',);
        },).length,);
      },
    },),
  ],
},);
