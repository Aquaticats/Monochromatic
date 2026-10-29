/**
 Tests for the checks run over the logs of the editor fidelity arms before any figure of them is read: one PASS or FAIL
 line for each check of each arm, and the checks across the arms. Each FAIL has a case that breaks exactly that check
 in an arm log that otherwise passes.

 The log lines are the probe's own writers' (the ceiling, the guard, the tally and the total), so a writer that changes
 its wording fails here before it fails an operator. Fixtures are cat-themed invention.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArmLog,
  ceilingLine,
  checkEditorArms,
  type EditorCardTally,
  editorCeilings,
  editorKeptLine,
  editorTallyLine,
  guardLine,
  guardTotalLine,
  manifestLine,
  plannedLine,
  poolLine,
  trialsLine,
  wordAfter,
} from '../../dist/final/node/index.mjs';
import {
  poolStubs,
  rowLineFor,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import { LOW_CARD, } from '../editor-fidelity.test-fixture.ts';
import { reviewedFixture, } from '../fidelity-reference.test-fixture.ts';

/**
 Trials the pool holds in these logs.
 */
const TRIALS = 40;

/**
 The specification the manifest line of a good arm counts.
 */
const SPEC = reviewedFixture().spec;

/**
 The specifications a good bench's manifest line counts, one.
 */
const SPECS = [SPEC,] as const;

/**
 The specifications of an arm whose manifest differs from the others', two.
 */
const DRIFTED_SPECS = [SPEC, SPEC,] as const;

/**
 The digest the manifest line of the one specification prints, which every arm of a good bench printed.
 */
const DIGEST = wordAfter({
  text: manifestLine({ specs: SPECS, },),
  marker: 'digest ',
},);

/**
 The digest the manifest line of the two specifications prints.
 */
const DRIFTED_DIGEST = wordAfter({
  text: manifestLine({ specs: DRIFTED_SPECS, },),
  marker: 'digest ',
},);

/**
 Prefix the logger puts on every line.
 */
const PREFIX = '[info] [2026-10-23T12:00:00.000Z] [editor-fidelity-probe] ';

/**
 The counts of a card that fixed thirty of the forty.
 */
const TALLY: EditorCardTally = {
  cardId: LOW_CARD,
  trials: TRIALS,
  exact: 28,
  equivalent: 2,
  defectOnly: 4,
  collateral: 5,
  missed: 3,
  refused: 2,
  unusable: 1,
  fixed: 30,
  byKind: {
    insertion: {
      trials: 22,
      exact: 17,
      equivalent: 1,
      defectOnly: 2,
      collateral: 3,
      missed: 1,
      refused: 1,
      unusable: 0,
    },
    alteration: {
      trials: 18,
      exact: 11,
      equivalent: 1,
      defectOnly: 2,
      collateral: 2,
      missed: 2,
      refused: 1,
      unusable: 1,
    },
  },
};

/**
 What a good arm log is made of, each part replaceable by a case.
 */
type GoodArm = {
  /**
   Lines logged before the rows, in order.
   */
  readonly opening?: readonly string[];

  /**
   Rows logged.
   */
  readonly rows?: number;

  /**
   Trials the TRIALS line says were completed, the pool's when omitted.
   */
  readonly completed?: number;

  /**
   Cost of each SPEND line, in dollars.
   */
  readonly costs?: readonly number[];

  /**
   The total the guard says.
   */
  readonly guardUsd?: number;

  /**
   Lines after the tally.
   */
  readonly closing?: readonly string[];

  /**
   The SEAT line.
   */
  readonly seat?: string;

  /**
   Exit status recorded.
   */
  readonly exit?: ArmLog['exit'];
};

/**
 The opening lines of a good arm, with the manifest digest and the cap given.

 @param specs - specifications the manifest line counts

 @param maxUsd - cap the guard line prints

 @param benchSpentUsd - spend of the earlier arms the guard line prints

 @returns Lines, with the logger's prefix on each

 @example
 ```ts
 const lines = openingOf({ specs: SPECS, maxUsd: 3, },);
 ```
 */
function openingOf(
  {
    specs,
    maxUsd,
    benchSpentUsd = 0,
  }: {
    readonly specs: readonly (typeof SPEC)[];
    readonly maxUsd: number;
    readonly benchSpentUsd?: number;
  },
): readonly string[] {
  return [
    `editors: ${LOW_CARD}`,
    manifestLine({ specs, },),
    poolLine({ trials: poolStubs({ count: TRIALS, },), },),
    ceilingLine({
      ceiling: nonNullishOrThrow(editorCeilings({
        cardIds: [LOW_CARD,],
        sheetChars: Array.from(
          { length: TRIALS, },
          function sheet(): number {
            return 27_000;
          },
        ),
      },)[0],),
    },),
    guardLine({ maxUsd, benchSpentUsd, },),
    plannedLine({ planned: TRIALS, total: TRIALS, },),
  ].map(function prefixed(line,): string {
    return `${PREFIX}${line}`;
  },);
}

/**
 A SPEND line for one call.

 @param cost - dollars the call cost

 @returns Line as the logger writes it

 @example
 ```ts
 const line = spendLine({ cost: 0.05, },);
 ```
 */
function spendLine({ cost, }: { readonly cost: number; },): string {
  return `[info] [2026-10-23T12:00:01.000Z] [translation-repair] [reportSpend] SPEND provider=anthropic model=${LOW_CARD} `
    + `prompt=9000 completion=1500 cost=${String(cost,)}`;
}

/**
 The lines of a good arm log.

 @param arm - parts of the arm that differ from a good one

 @returns Its log lines and recorded exit

 @example
 ```ts
 const log = goodArm({ arm: {}, },);
 ```
 */
function goodArm({ arm, }: { readonly arm: GoodArm; }): ArmLog {
  const opening = arm.opening ?? openingOf({
    specs: SPECS,
    maxUsd: 3,
  },);
  const rows = arm.rows ?? TRIALS;
  const costs = arm.costs ?? [0.25, 0.5, 0.125,];
  const guardUsd = arm.guardUsd ?? 0.875;
  const closing = arm.closing ?? [];
  const seat = arm.seat ?? `SEAT ${LOW_CARD} asked=40 usable=39 unusable=1 threw=0 silent=0 abandoned=0 unreachable=0`;
  const exit = arm.exit ?? {
    recorded: true,
    code: 0,
  };
  return {
    arm: 'low',
    exit,
    lines: [
      ...opening,
      ...costs.map(function line(cost,): string {
        return spendLine({ cost, },);
      },),
      ...Array.from(
        { length: rows, },
        function row(
          _unused,
          index,
        ): string {
          return `${PREFIX}${rowLineFor({
            cardId: LOW_CARD,
            row: {
              trial: `tabby-s${String(index,)}/insertion`,
              entry: `cat-${String(index,)}`,
              fixed: true,
              stops: '',
            },
            position: index,
            total: TRIALS,
          },)}`;
        },
      ),
      `${PREFIX}${editorTallyLine({ tally: TALLY, },)}`,
      `${PREFIX}${trialsLine({ completed: arm.completed ?? TRIALS, planned: TRIALS, },)}`,
      `${PREFIX}${editorKeptLine({ count: rows, keptAt: '/runs/x.json', },)}`,
      `${PREFIX}${guardTotalLine({ total: { usd: guardUsd, calls: 40, atCeiling: 0, }, },)}`,
      ...closing,
      seat,
    ],
  };
}

/**
 Runs the checks over one arm and returns the lines naming its check.

 @param log - arm

 @param check - name of the check

 @returns Line the report gives that check of that arm

 @example
 ```ts
 const line = lineFor({ log, check: 'exit 0', },);
 ```
 */
function lineFor({ log, check, }: { readonly log: ArmLog; readonly check: string; }): string {
  const found = checkEditorArms({ arms: [log,], },).lines.find(function isCheck(line,): boolean {
    return line.includes(`${log.arm}: ${check}`,);
  },);
  if (found === undefined)
    throw new Error(`no line for ${check}`,);
  return found;
}

await describe({
  name: checkEditorArms.name,
  children: [
    it({
      name: 'PASSES EVERY CHECK OF A GOOD ARM, one PASS line for each, and counts no failure',
      fn: async () => {
        const report = checkEditorArms({ arms: [goodArm({ arm: {}, },),], },);
        expect(report.failed,).toBe(0,);
        expect(report.lines.filter(function isPass(line,): boolean {
          return line.startsWith('PASS ',);
        },).length,).toBe(13,);
        expect(report.lines.some(function isFail(line,): boolean {
          return line.startsWith('FAIL ',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'FAILS THE EXIT CHECK for a status other than zero, and for an arm whose status nobody recorded',
      fn: async () => {
        expect(lineFor({
          log: goodArm({ arm: { exit: { recorded: true, code: 1, }, }, },),
          check: 'exit 0',
        },),).toBe('FAIL low: exit 0: the process exited with status 1',);
        expect(lineFor({
          log: goodArm({ arm: { exit: { recorded: false, }, }, },),
          check: 'exit 0',
        },),).toBe('FAIL low: exit 0: no exit status was recorded for this arm',);
      },
    },),

    it({
      name: 'FAILS THE MANIFEST CHECK for a log that printed no manifest digest, since the arm cannot then be shown '
        + 'to have asked the pool the others asked',
      fn: async () => {
        const without = openingOf({ specs: SPECS, maxUsd: 3, },).filter(function notManifest(line,): boolean {
          return !line.includes('manifest: ',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: without, }, },), check: 'manifest line', },),).toBe(
          'FAIL low: manifest line: no manifest line was logged',
        );
      },
    },),

    it({
      name: 'FAILS THE POOL CHECK for a log with no pool line, a pool under forty trials, or one with only one defect '
        + 'family',
      fn: async () => {
        const without = openingOf({ specs: SPECS, maxUsd: 3, },).filter(function notPool(line,): boolean {
          return !line.includes('pool: ',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: without, }, },), check: 'pool line', },),).toContain(
          'FAIL low: pool line: ',
        );
        const small = openingOf({ specs: SPECS, maxUsd: 3, },).map(function shrink(line,): string {
          return line.replace('pool: 40 trials', 'pool: 39 trials',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: small, }, },), check: 'pool line', },),).toContain(
          'FAIL low: pool line: 39 trials',
        );
        const oneKind = openingOf({ specs: SPECS, maxUsd: 3, },).map(function lopsided(line,): string {
          return line.replace('(insertion 20, alteration 20)', '(insertion 40, alteration 0)',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: oneKind, }, },), check: 'pool line', },),).toContain(
          'FAIL low: pool line: ',
        );
      },
    },),

    it({
      name: 'FAILS THE PLANNED CHECK for a log with no planned line, and one that planned more trials than the pool holds',
      fn: async () => {
        const without = openingOf({ specs: SPECS, maxUsd: 3, },).filter(function notPlanned(line,): boolean {
          return !line.includes('planned: ',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: without, }, },), check: 'planned line', },),).toBe(
          'FAIL low: planned line: no planned line was logged',
        );
        const over = openingOf({ specs: SPECS, maxUsd: 3, },).map(function grow(line,): string {
          return line.replace('planned: 40 of 40 trials', 'planned: 41 of 40 trials',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: over, }, },), check: 'planned line', },),).toBe(
          'FAIL low: planned line: 41 trials planned of a pool of 40',
        );
        expect(lineFor({ log: goodArm({ arm: {}, },), check: 'planned line', },),).toBe(
          'PASS low: planned line: 40 trials planned of a pool of 40',
        );
      },
    },),

    it({
      name: 'FAILS THE PLANNED CHECK FOR A LOG WITH NO POOL LINE, saying there is no pool line to hold the planned trials '
        + 'against, in place of printing a pool of -1 trials',
      fn: async () => {
        const without = openingOf({ specs: SPECS, maxUsd: 3, },).filter(function notPool(line,): boolean {
          return !line.includes('pool: ',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: without, }, },), check: 'planned line', },),).toBe(
          'FAIL low: planned line: 40 trials planned, and no pool line was logged to hold them against',
        );
      },
    },),

    it({
      name: 'FAILS THE CEILING CHECK for a log with no ceiling line, or one for a different number of trials than the pool',
      fn: async () => {
        const without = openingOf({ specs: SPECS, maxUsd: 3, },).filter(function notCeiling(line,): boolean {
          return !line.includes('ceiling ',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: without, }, },), check: 'ceiling line', },),).toContain(
          'FAIL low: ceiling line: ',
        );
        const other = openingOf({ specs: SPECS, maxUsd: 3, },).map(function cut(line,): string {
          return line.replace(': 40 units, one ask', ': 12 units, one ask',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: other, }, },), check: 'ceiling line', },),).toBe(
          'FAIL low: ceiling line: 12 units, for 40 planned',
        );
      },
    },),

    it({
      name: 'FAILS THE GUARD CHECK for a log with no guard line, and for a guard line that does not state what the '
        + 'earlier arms spent',
      fn: async () => {
        const without = openingOf({ specs: SPECS, maxUsd: 3, },).filter(function notGuard(line,): boolean {
          return !line.includes('spend guard: ',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: without, }, },), check: 'guard line', },),).toContain(
          'FAIL low: guard line: ',
        );
        const unstated = openingOf({ specs: SPECS, maxUsd: 3, },).map(function dropBench(line,): string {
          return line.replace('the arms run before this one spent --bench-spent-usd 0.0000; ', '',);
        },);
        expect(lineFor({ log: goodArm({ arm: { opening: unstated, }, },), check: 'guard line', },),).toBe(
          'FAIL low: guard line: the guard line states no --bench-spent-usd',
        );
        expect(lineFor({ log: goodArm({ arm: {}, },), check: 'guard line', },),).toBe(
          'PASS low: guard line: held to --max-usd 3 USD after 0 USD spent by the arms before it',
        );
      },
    },),

    it({
      name: 'FAILS THE EDITOR CHECK for a log with no EDITOR line for the arm\'s card, or one counting other trials than '
        + 'the pool',
      fn: async () => {
        const log = goodArm({ arm: {}, },);
        const without: ArmLog = {
          ...log,
          lines: log.lines.filter(function notTally(line,): boolean {
            return !line.includes(`EDITOR ${LOW_CARD}: `,);
          },),
        };
        expect(lineFor({ log: without, check: 'EDITOR line', },),).toContain('FAIL low: EDITOR line: ',);
        const other: ArmLog = {
          ...log,
          lines: log.lines.map(function recount(line,): string {
            return line.replace('clean fix 30 of 40', 'clean fix 30 of 38',);
          },),
        };
        expect(lineFor({ log: other, check: 'EDITOR line', },),).toBe(
          'FAIL low: EDITOR line: counts 38 units, for 40 completed',
        );
      },
    },),

    it({
      name: 'FAILS THE ROW CHECK when fewer rows were logged than the trials the arm says it completed, or a row is '
        + 'logged twice',
      fn: async () => {
        expect(lineFor({ log: goodArm({ arm: { rows: 39, }, },), check: 'row count', },),).toBe(
          'FAIL low: row count: 39 rows logged for 40 completed units',
        );
        const log = goodArm({ arm: {}, },);
        const twice: ArmLog = {
          ...log,
          lines: log.lines.map(function repeat(line,): string {
            return line.replace('row 40 of 40', 'row 39 of 40',);
          },),
        };
        expect(lineFor({ log: twice, check: 'row count', },),).toContain('FAIL low: row count: ',);
      },
    },),

    it({
      name: 'FAILS THE ERROR CHECK for any line at the error level, naming the first',
      fn: async () => {
        const log = goodArm({ arm: {}, },);
        const errored: ArmLog = {
          ...log,
          lines: [...log.lines, '[error] [2026-10-23T12:05:00.000Z] [editor-fidelity-probe] the stream broke',],
        };
        expect(lineFor({ log: errored, check: 'no [error] line', },),).toBe(
          'FAIL low: no [error] line: [error] [2026-10-23T12:05:00.000Z] [editor-fidelity-probe] the stream broke',
        );
      },
    },),

    it({
      name: 'FAILS THE SEAT CHECK for a missing SEAT line, one that does not state threw, a withheld seat, a seat '
        + 'the router refused, and a SEATS DARK line',
      fn: async () => {
        const check = 'seat line';
        expect(lineFor({ log: goodArm({ arm: { seat: 'SEAT nobody asked=1', }, },), check, },),).toContain(
          'FAIL low: seat line: ',
        );
        expect(lineFor({
          log: goodArm({ arm: { seat: `SEAT ${LOW_CARD} asked=40 usable=40 unusable=0 silent=0 abandoned=0 unreachable=0`, }, },),
          check,
        },),).toContain('threw is not stated',);
        expect(lineFor({
          log: goodArm({ arm: { seat: `SEAT ${LOW_CARD} asked=0 usable=0 unusable=0 threw=0 silent=0 abandoned=0 unreachable=40`, }, },),
          check,
        },),).toContain('unreachable=40',);
        expect(lineFor({
          log: goodArm({ arm: { closing: [`SEAT CREDENTIAL REFUSED ${LOW_CARD} on anthropic: refused`,], }, },),
          check,
        },),).toContain('credential refused',);
        expect(lineFor({
          log: goodArm({ arm: { closing: [`SEATS DARK: 1 of 1 seat asked produced nothing usable this run: ${LOW_CARD}`,], }, },),
          check,
        },),).toContain('SEATS DARK',);
      },
    },),

    it({
      name: 'FAILS THE SPEND CHECK when the SPEND lines do not add to the guard\'s total, and says both figures',
      fn: async () => {
        expect(lineFor({
          log: goodArm({ arm: { guardUsd: 0.9, }, },),
          check: 'spend identity',
        },),).toBe('FAIL low: spend identity: the SPEND lines add to 0.875000 USD and the guard counted 0.900000 USD',);
        expect(lineFor({ log: goodArm({ arm: {}, },), check: 'spend identity', },),).toBe(
          'PASS low: spend identity: the SPEND lines and the guard both say 0.875000 USD',
        );
      },
    },),

    it({
      name: 'FAILS THE SPEND CHECK for a log with no GUARD total line, and for a SPEND line that states no cost',
      fn: async () => {
        const log = goodArm({ arm: {}, },);
        const noTotal: ArmLog = {
          ...log,
          lines: log.lines.filter(function notTotal(line,): boolean {
            return !line.includes('GUARD total=',);
          },),
        };
        expect(lineFor({ log: noTotal, check: 'spend identity', },),).toContain('FAIL low: spend identity: ',);
        const unpriced: ArmLog = {
          ...log,
          lines: [
            ...log.lines,
            `[info] [2026-10-23T12:00:02.000Z] [translation-repair] [reportSpend] SPEND provider=anthropic model=${LOW_CARD} prompt=1 completion=1`,
          ],
        };
        expect(lineFor({ log: unpriced, check: 'spend identity', },),).toContain('states no cost',);
      },
    },),

    it({
      name: 'CHECKS ACROSS THE ARMS that every arm printed the same manifest digest, and that the money measured and '
        + 'the caps written both fit the sixty dollars of the whole bench',
      fn: async () => {
        const first = goodArm({ arm: {}, },);
        const second: ArmLog = {
          ...goodArm({ arm: {}, },),
          arm: 'medium-a',
        };
        const same = checkEditorArms({ arms: [first, second,], },);
        expect(same.failed,).toBe(0,);
        expect(same.lines,).toContain('PASS bench: same manifest digest in every arm',);
        expect(same.lines.some(function isTotal(line,): boolean {
          return line.startsWith('PASS bench: total ',);
        },),).toBe(true,);
        const drifted: ArmLog = goodArm({
          arm: {
            opening: openingOf({ specs: DRIFTED_SPECS, maxUsd: 3, },),
          },
        },);
        const other = checkEditorArms({ arms: [first, { ...drifted, arm: 'medium-a', },], },);
        expect(other.failed,).toBe(1,);
        expect(other.lines,).toContain(
          `FAIL bench: same manifest digest in every arm: low printed ${DIGEST} and medium-a printed ${DRIFTED_DIGEST}`,
        );
      },
    },),

    it({
      name: 'FAILS THE BENCH TOTAL when the guards counted sixty dollars or more, or the caps written add to more than '
        + 'sixty',
      fn: async () => {
        const between = goodArm({ arm: { costs: [15,], guardUsd: 15, }, },);
        const inside = checkEditorArms({ arms: [between, { ...between, arm: 'b', },], },);
        expect(inside.lines.some(function isPass(line,): boolean {
          return line.startsWith('PASS bench: total ',) && line.includes('30.000000 USD measured under 60 USD',);
        },),).toBe(true,);
        const spent = goodArm({ arm: { costs: [30,], guardUsd: 30, }, },);
        const total = checkEditorArms({ arms: [spent, { ...spent, arm: 'b', },], },);
        expect(total.lines.some(function isFail(line,): boolean {
          return line.startsWith('FAIL bench: total ',) && line.includes('60.000000 USD',);
        },),).toBe(true,);
        const capped = goodArm({
          arm: {
            opening: openingOf({ specs: SPECS, maxUsd: 31, },),
          },
        },);
        const caps = checkEditorArms({ arms: [capped, { ...capped, arm: 'b', },], },);
        expect(caps.lines.some(function isFail(line,): boolean {
          return line.startsWith('FAIL bench: ',) && line.includes('--max-usd',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'COUNTS EVERY FAILURE in the report, so the command can refuse as many as there are',
      fn: async () => {
        const report = checkEditorArms({
          arms: [goodArm({ arm: { exit: { recorded: true, code: 2, }, guardUsd: 1, rows: 38, }, },),],
        },);
        expect(report.failed,).toBe(3,);
      },
    },),
  ],
},);
