/**
 Tests for the lines the read step prints once the checks of a bench pass: the figures of each arm with the defect-only
 count beside the headline, the trials every arm shared, how the asks of each row ended in a line short enough to read,
 and the paired lines and verdict of the decision rule over the three or four arms.

 Fixtures are cat-themed invention.

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
  benchReportLines,
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
 Trials every arm of a good bench walked.
 */
const TRIALS = 44;

/**
 Rows of an arm that fixes every other trial, the insertions, and asks ok.
 */
const HALF_ROWS: readonly RowSpec[] = poolRows({
  count: TRIALS,
  fixed: function insertionsOnly(index,): boolean {
    return (index % 2) === 0;
  },
});

/**
 Rows of an arm that fixes every trial.
 */
const ALL_ROWS: readonly RowSpec[] = poolRows({
  count: TRIALS,
  fixed: function every(): boolean {
    return true;
  },
});

/**
 One arm of a bench.

 @param arm - arm's name
 @param cardId - card it seated
 @param rows - rows it walked

 @returns Arm's log

 @example
 ```ts
 const log = benchArm({ arm: 'low', cardId: LOW_CARD, rows: HALF_ROWS, },);
 ```
 */
function benchArm(
  {
    arm,
    cardId,
    rows,
  }: {
    readonly arm: string;
    readonly cardId: RosterModelId;
    readonly rows: readonly RowSpec[];
  },
): ArmLog {
  return armLogOf({
    arm,
    tally: tallyFromRows({ cardId, rows, },),
    stops: [],
    rows,
    seat: `SEAT ${cardId} asked=${String(rows.length,)} usable=${String(rows.length,)} unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`,
    benchSpentUsd: 0,
    guardUsd: 0.5,
  },);
}

/**
 Rows with the stop words of some rows replaced.

 @param rows - rows
 @param stops - stop words of the rows at an index, for the indexes named

 @returns Rows with those stop words

 @example
 ```ts
 const rows = withStops({ rows: HALF_ROWS, stops: { 2: 'output-cap', }, },);
 ```
 */
function withStops(
  { rows, stops, }: {
    readonly rows: readonly RowSpec[];
    readonly stops: Readonly<Record<number, string>>;
  },
): readonly RowSpec[] {
  return rows.map(function stopped(
    row,
    at,
  ): RowSpec {
    return { ...row, stops: stops[at] ?? row.stops, };
  },);
}

/**
 A bench in which high fixes every trial and medium and low fix the insertions.
 */
const BENCH: readonly ArmLog[] = [
  benchArm({ arm: 'high', cardId: HIGH_CARD, rows: ALL_ROWS, },),
  benchArm({ arm: 'medium', cardId: MEDIUM_CARD, rows: HALF_ROWS, },),
  benchArm({ arm: 'low', cardId: LOW_CARD, rows: HALF_ROWS, },),
];

await describe({
  name: benchReportLines.name,
  children: [
    it({
      name: 'PRINTS THE FIGURES OF EACH ARM with the clean fix count pooled and for each kind, the defect-only count '
        + 'beside it as a secondary count, and what went unusable, was refused, threw and was abandoned, in the order given',
      fn: async () => {
        const lines = benchReportLines({ arms: BENCH, },);
        expect(lines[0],).toBe(
          `ARM high (${HIGH_CARD}): clean fix 44 of 44 (insertion 22 of 22, alteration 22 of 22); `
            + 'defect only 0 (insertion 0, alteration 0); unusable 0, refused 0, threw 0, abandoned 0',
        );
        expect(lines.filter(function isArm(line,): boolean {
          return line.startsWith('ARM ',);
        },).length,).toBe(3,);
        expect(lines.find(function isLow(line,): boolean {
          return line.startsWith('ARM low ',);
        },),).toContain('clean fix 22 of 44 (insertion 22 of 22, alteration 0 of 22)',);
      },
    },),

    it({
      name: 'SAYS THE TRIALS EVERY ARM SHARED, how many came from how many entries, what each arm completed, and how '
        + 'many were left out as thrown or not completed by every arm',
      fn: async () => {
        expect(benchReportLines({ arms: BENCH, },),).toContain(
          'SHARED 44 units over 22 entries (high completed 44, medium completed 44, low completed 44; 0 left out as '
            + 'thrown, 0 not completed by every arm)',
        );
      },
    },),

    it({
      name: 'PRINTS HOW THE ASKS OF EVERY ROW ENDED AS A COUNT FOR EACH STOP REASON and the rows that are not ok, one '
        + 'line for each arm',
      fn: async () => {
        /**
         The high arm with a refusal and then ok on two rows, and one row that ended at the output cap.
         */
        const stopped = withStops({
          rows: ALL_ROWS,
          stops: { 40: 'refused+ok', 41: 'refused+ok', 42: 'output-cap', },
        },);
        const stops = benchReportLines({
          arms: [benchArm({ arm: 'high', cardId: HIGH_CARD, rows: stopped, },), BENCH[1] as ArmLog, BENCH[2] as ArmLog,],
        },).filter(function isStops(line,): boolean {
          return line.startsWith('STOPS ',);
        },);
        expect(stops,).toStrictEqual([
          'STOPS high: ok 41, refused+ok 2, output-cap 1; not ok: row 41 refused+ok, row 42 refused+ok, row 43 output-cap',
          'STOPS medium: ok 44',
          'STOPS low: ok 44',
        ],);
      },
    },),

    it({
      name: 'SCORES A TRIAL THAT ENDED AT THE OUTPUT CAP AS UNUSABLE FOR THAT ARM: high\'s first trial reads as a clean fix '
        + 'but an ask of it ended at the cap, so it is not counted fixed and the paired difference falls from 50.0 to '
        + '47.7 points',
      fn: async () => {
        /**
         The high arm whose second trial ended at the cap.
         */
        const capped = benchArm({
          arm: 'high',
          cardId: HIGH_CARD,
          rows: withStops({ rows: ALL_ROWS, stops: { 1: 'output-cap', }, },),
        },);
        const lines = benchReportLines({ arms: [capped, BENCH[1] as ArmLog, BENCH[2] as ArmLog,], },);
        expect(benchReportLines({ arms: BENCH, },).find(function isHigh(line,): boolean {
          return line.startsWith('PAIRED high vs medium: ',);
        },),).toContain('+50.0 points',);
        expect(lines.find(function isHigh(line,): boolean {
          return line.startsWith('PAIRED high vs medium: ',);
        },),).toContain('+47.7 points',);
        expect(lines,).toContain(
          'SHARED 44 units over 22 entries (high completed 44, medium completed 44, low completed 44; 0 left out as '
            + 'thrown, 0 not completed by every arm)',
        );
      },
    },),

    it({
      name: 'LEAVES A THROWN TRIAL OUT OF EVERY ARM and counts it: the third trial of the medium arm threw, so the '
        + 'comparison is over 43 trials of all three arms',
      fn: async () => {
        /**
         The medium arm whose third trial had an ask that threw.
         */
        const threw = benchArm({
          arm: 'medium',
          cardId: MEDIUM_CARD,
          rows: withStops({ rows: HALF_ROWS, stops: { 2: 'threw+ok', }, },),
        },);
        const lines = benchReportLines({ arms: [BENCH[0] as ArmLog, threw, BENCH[2] as ArmLog,], },);
        expect(lines,).toContain(
          'SHARED 43 units over 22 entries (high completed 44, medium completed 44, low completed 44; 1 left out as '
            + 'thrown, 0 not completed by every arm)',
        );
        expect(lines.find(function isHigh(line,): boolean {
          return line.startsWith('PAIRED high vs medium: ',);
        },),).toContain('43 shared units',);
      },
    },),

    it({
      name: 'KEYS THE ROWS BY PASS AND TRIAL, so a trial asked again in a second pass is a second unit in its entry, '
        + 'compared with the other arms\' answer in the same pass: high fixes every trial in the first pass and none in '
        + 'the second, so over 88 units it stands level with medium',
      fn: async () => {
        /**
         The rows of a second pass over the same trials.
         */
        const second = inPass({ rows: HALF_ROWS, pass: 2, },);
        /**
         High fixed every trial the first time and none the second.
         */
        const highRows = [
          ...ALL_ROWS,
          ...inPass({
            rows: poolRows({
              count: TRIALS,
              fixed: function none(): boolean {
                return false;
              },
            }),
            pass: 2,
          },),
        ];
        const lines = benchReportLines({
          arms: [
            benchArm({ arm: 'high', cardId: HIGH_CARD, rows: highRows, },),
            benchArm({ arm: 'medium', cardId: MEDIUM_CARD, rows: [...HALF_ROWS, ...second,], },),
            benchArm({ arm: 'low', cardId: LOW_CARD, rows: [...HALF_ROWS, ...second,], },),
          ],
        },);
        expect(lines.some(function isShared(line,): boolean {
          return line.startsWith('SHARED 88 ',) && line.includes(' over 22 entries (high completed 88,',);
        },),).toBe(true,);
        expect(lines.find(function isHigh(line,): boolean {
          return line.startsWith('PAIRED high vs medium: ',);
        },),).toContain('+0.0 points',);
      },
    },),

    it({
      name: 'LEAVES OUT ONLY THE UNIT IN WHICH AN ASK THREW: a second-pass row of the medium arm threw, so 87 of 88 units '
        + 'are shared and the same trial in the first pass stays',
      fn: async () => {
        /**
         The rows of a second pass, the third of them with an ask that threw.
         */
        const second = withStops({ rows: inPass({ rows: HALF_ROWS, pass: 2, },), stops: { 2: 'threw+ok', }, },);
        const lines = benchReportLines({
          arms: [
            benchArm({ arm: 'high', cardId: HIGH_CARD, rows: [...ALL_ROWS, ...inPass({ rows: ALL_ROWS, pass: 2, },),], },),
            benchArm({ arm: 'medium', cardId: MEDIUM_CARD, rows: [...HALF_ROWS, ...second,], },),
            benchArm({ arm: 'low', cardId: LOW_CARD, rows: [...HALF_ROWS, ...inPass({ rows: HALF_ROWS, pass: 2, },),], },),
          ],
        },);
        expect(lines.some(function isShared(line,): boolean {
          return line.startsWith('SHARED 87 ',) && line.includes('; 1 left out as thrown, 0 not completed by every arm)',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES OUT TRIALS A LATER ARM DID NOT COMPLETE: the low arm capped after 42 trials, so 42 are shared',
      fn: async () => {
        /**
         The low arm capped after forty-two trials.
         */
        const capped = armLogOf({
          arm: 'low',
          tally: tallyFromRows({ cardId: LOW_CARD, rows: HALF_ROWS.slice(0, 42,), },),
          stops: [],
          rows: HALF_ROWS.slice(0, 42,),
          planned: TRIALS,
          capped: true,
          seat: `SEAT ${LOW_CARD} asked=42 usable=42 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`,
          benchSpentUsd: 0,
          guardUsd: 0.5,
        },);
        expect(benchReportLines({ arms: [BENCH[0] as ArmLog, BENCH[1] as ArmLog, capped,], },),).toContain(
          'SHARED 42 units over 21 entries (high completed 44, medium completed 44, low completed 42; 0 left out as '
            + 'thrown, 2 not completed by every arm)',
        );
      },
    },),

    it({
      name: 'PRINTS THE PAIRED LINES AND THE VERDICT LAST, the verdict the decision rule gives for the shared trials: '
        + 'high is shown better than medium, low is not, and a line per kind follows the pair',
      fn: async () => {
        const lines = benchReportLines({ arms: BENCH, },);
        expect(lines.at(-1,)?.startsWith('VERDICT high (shown-better): ',),).toBe(true,);
        expect(lines.some(function isLow(line,): boolean {
          return line.startsWith('PAIRED low vs medium: +0.0 points',);
        },),).toBe(true,);
        expect(lines.some(function isKinds(line,): boolean {
          return line.startsWith('KINDS high vs medium (secondary, no veto): insertion +0.0 points over 22 units, alteration +100.0 points over 22 units',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'READS A SECOND MEDIUM ARM WHEN GIVEN FOUR LOGS and prints an ARM line for each of the four',
      fn: async () => {
        const lines = benchReportLines({
          arms: [
            BENCH[0] as ArmLog,
            BENCH[1] as ArmLog,
            benchArm({ arm: 'medium-b', cardId: MEDIUM_CARD, rows: HALF_ROWS, },),
            BENCH[2] as ArmLog,
          ],
        },);
        expect(lines.filter(function isArm(line,): boolean {
          return line.startsWith('ARM ',);
        },).length,).toBe(4,);
        expect(lines.at(-1,)?.startsWith('VERDICT high (shown-better): ',),).toBe(true,);
      },
    },),

    it({
      name: 'THROWS FOR OTHER THAN THREE OR FOUR ARMS, since a verdict over fewer is not the rule\'s and the read step '
        + 'prints figures only after the bench checks have passed',
      fn: async () => {
        expect(function readsTwo(): void {
          benchReportLines({ arms: BENCH.slice(0, 2,), },);
        },).toThrow('three or four arms',);
        expect(function readsFive(): void {
          benchReportLines({ arms: [...BENCH, ...BENCH,], },);
        },).toThrow('three or four arms',);
      },
    },),
  ],
},);
