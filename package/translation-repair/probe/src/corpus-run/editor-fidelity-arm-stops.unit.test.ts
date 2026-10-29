/**
 Tests for the checks of how the asks of an editor fidelity arm ended: every row states its stop reasons, no trial ended
 at the output-token cap, nothing threw, and a warning when too many trials went unusable or were refused. The checks
 are read over the logs by `checkEditorBench`.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArmLog,
  checkEditorBench,
} from '../../dist/final/node/index.mjs';
import {
  armLogOf,
  tallyOf,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import { LOW_CARD, } from '../editor-fidelity.test-fixture.ts';

/**
 A seat line that states threw and abandoned as zero.
 */
const CLEAN_SEAT = `SEAT ${LOW_CARD} asked=40 usable=40 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`;

/**
 An arm of the given stops and counts of unusable and refused trials.

 @param stops - for each row, its stop words, empty for a row that states none
 @param unusable - trials with no usable reply
 @param refused - trials where the card was heard and proposed no edit
 @param seat - SEAT line
 @param units - units the tally counts, split evenly between the kinds; 22 insertion and 18 alteration trials when
 omitted

 @returns Arm, named low

 @example
 ```ts
 const log = armWith({ stops: ['ok',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },);
 ```
 */
function armWith(
  {
    stops,
    unusable,
    refused,
    seat,
    units,
  }: {
    readonly stops: readonly string[];
    readonly unusable: number;
    readonly refused: number;
    readonly seat: string;
    readonly units?: number;
  },
): ArmLog {
  return armLogOf({
    arm: 'low',
    tally: tallyOf({
      cardId: LOW_CARD,
      insertion: { trials: (units === undefined) ? 22 : units / 2, fixed: 18, defectOnly: 2, },
      alteration: { trials: (units === undefined) ? 18 : units / 2, fixed: 12, defectOnly: 1, },
      unusable,
      refused,
    },),
    stops,
    seat,
    benchSpentUsd: 0,
    guardUsd: 0.5,
  },);
}

/**
 The report line of one named check of the arm named low.

 @param log - arm
 @param check - name of the check, or the start of a warning's words

 @returns Line, empty when there is none

 @example
 ```ts
 const line = lineOf({ log, check: 'stop reasons stated', },);
 ```
 */
function lineOf({ log, check, }: { readonly log: ArmLog; readonly check: string; },): string {
  return checkEditorBench({ arms: [log,], },).lines.find(function isCheck(line,): boolean {
    return line.slice(5,)
      .startsWith(`low: ${check}`,);
  },) ?? '';
}

await describe({
  name: 'editor fidelity stop checks',
  children: [
    it({
      name: 'PASSES THE STOP-REASONS CHECK for an arm whose every row states how its asks ended, and fails it naming '
        + 'how many rows state nothing',
      fn: async () => {
        expect(lineOf({
          log: armWith({ stops: ['ok', 'refused+ok', 'ok',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'stop reasons stated',
        },),).toBe('PASS low: stop reasons stated: 3 rows each state how their asks ended',);
        expect(lineOf({
          log: armWith({ stops: ['ok', '', '',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'stop reasons stated',
        },),).toBe('FAIL low: stop reasons stated: 2 of 3 rows state no stop reason',);
        expect(lineOf({
          log: armWith({ stops: [], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'stop reasons stated',
        },),).toBe('FAIL low: stop reasons stated: no row line was logged',);
      },
    },),

    it({
      name: 'PASSES THE TRIAL-ENDS CHECK FOR AN ARM WHOSE ASKS ENDED ANY WAY and says what became of the trials that '
        + 'ended at the output-token cap (they score as unusable for the arm) or threw (they are left out of every '
        + 'arm): the rule absorbs both by trial, so neither fails an arm',
      fn: async () => {
        expect(lineOf({
          log: armWith({ stops: ['ok', 'refused+ok', 'unparseable+ok',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'units at the output cap or thrown',
        },),).toBe('PASS low: units at the output cap or thrown: 3 units, none at the cap, none threw',);
        expect(lineOf({
          log: armWith({
            stops: ['ok', 'output-cap', 'ok', 'output-cap+ok',],
            unusable: 0,
            refused: 0,
            seat: CLEAN_SEAT,
          },),
          check: 'units at the output cap or thrown',
        },),).toBe(
          'PASS low: units at the output cap or thrown: 2 units ended at the output-token cap (rows 2, 4) and score '
            + 'as unusable for this arm',
        );
        expect(lineOf({
          log: armWith({ stops: ['ok', 'threw+ok', 'output-cap',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'units at the output cap or thrown',
        },),).toBe(
          'PASS low: units at the output cap or thrown: 1 unit ended at the output-token cap (row 3) and scores as '
            + 'unusable for this arm; 1 unit had an ask that threw (row 2) and is left out of every arm',
        );
      },
    },),

    it({
      name: 'FAILS NO ARM FOR THE SEAT LINE SAYING ASKS THREW, since the threw count belongs to the trials the row '
        + 'lines name and the rule leaves those out',
      fn: async () => {
        const report = checkEditorBench({
          arms: [
            armWith({
              stops: ['ok', 'ok',],
              unusable: 0,
              refused: 0,
              seat: `SEAT ${LOW_CARD} asked=40 usable=39 unusable=0 threw=2 silent=0 abandoned=0 unreachable=0`,
            },),
          ],
        },);
        expect(report.lines.filter(function isTrialEnds(line,): boolean {
          return line.includes('units at the output cap or thrown',) && line.startsWith('FAIL ',);
        },),).toStrictEqual([],);
      },
    },),

    it({
      name: 'WARNS WHEN UNUSABLE PLUS REFUSED EXCEED A TENTH of an arm\'s units, counts the warning in no failure, and '
        + 'says nothing at a tenth',
      fn: async () => {
        /**
         Five unusable and refused together, over a pool of forty.
         */
        const over = armWith({ stops: ['ok',], unusable: 2, refused: 3, seat: CLEAN_SEAT, },);
        expect(lineOf({ log: over, check: 'unusable plus refused', },),).toBe(
          'WARN low: unusable plus refused is 5 of 40 units, over 10 percent',
        );
        expect(lineOf({
          log: armWith({ stops: ['ok',], unusable: 1, refused: 3, seat: CLEAN_SEAT, },),
          check: 'unusable plus refused',
        },),).toBe('',);
        /**
         The report without the warning, over the same arm bar the counts.
         */
        const quiet = checkEditorBench({
          arms: [armWith({ stops: ['ok',], unusable: 1, refused: 3, seat: CLEAN_SEAT, },),],
        },);
        const loud = checkEditorBench({ arms: [over,], },);
        expect(loud.failed,).toBe(quiet.failed,);
      },
    },),

    it({
      name: 'SAYS NOTHING FOR A LOG WITH NO EDITOR AND TRIALS LINES, as a dry run\'s output is: the arm completed '
        + 'nothing and nothing is unusable or refused, so there is no warning to write',
      fn: async () => {
        /**
         An arm with nothing unusable or refused, and the lines that count its units taken out of its log.
         */
        const whole = armWith({ stops: ['ok',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },);
        const dry: ArmLog = {
          ...whole,
          lines: whole.lines.filter(function isNotTrials(line,): boolean {
            return (!line.includes('TRIALS completed ',)) && (!line.includes('] EDITOR ',));
          },),
        };
        // POSITIVE CONTROL: the log has both lines before the cut, so a cut that removed nothing would fail here.
        expect(whole.lines.some(function isTrials(line,): boolean {
          return line.includes('TRIALS completed ',);
        },),).toBe(true,);
        expect(whole.lines.some(function isEditor(line,): boolean {
          return line.includes('] EDITOR ',);
        },),).toBe(true,);
        expect(dry.lines.length,).toBeLessThan(whole.lines.length,);
        expect(lineOf({ log: dry, check: 'unusable plus refused', },),).toBe('',);
      },
    },),

    it({
      name: 'NAMES THE UNKNOWN COUNT OF UNITS WHEN THE LOG STATES NONE AND SOME TRIALS ARE UNUSABLE, in place of a '
        + 'negative count of units',
      fn: async () => {
        /**
         An arm with five trials unusable or refused over a pool of forty.
         */
        const whole = armWith({ stops: ['ok',], unusable: 2, refused: 3, seat: CLEAN_SEAT, },);
        /**
         The same arm with the count of units cut out of its pooled EDITOR line, which leaves the unusable and refused
         counts in it.
         */
        const unknown: ArmLog = {
          ...whole,
          lines: whole.lines.map(function withoutCount(line,): string {
            return line.includes(`] EDITOR ${LOW_CARD}: clean fix `,)
              ? line.replace(' of ', ' from ',)
              : line;
          },),
        };
        // POSITIVE CONTROL: the whole log warns with its count of units, and the cut changed the log.
        expect(lineOf({ log: whole, check: 'unusable plus refused', },),).toBe(
          'WARN low: unusable plus refused is 5 of 40 units, over 10 percent',
        );
        expect(unknown.lines,).not.toStrictEqual(whole.lines,);
        expect(lineOf({ log: unknown, check: 'unusable plus refused', },),).toBe(
          'WARN low: unusable plus refused is 5 of an unknown number of units (the log states none)',
        );
      },
    },),

    it({
      name: 'SCALES THE WARNING TO THE UNITS OF AN ARM THAT WALKS THREE PASSES: 26 of 264 units unusable or refused is '
        + 'under a tenth and says nothing, 27 is over it and warns, where 5 of 40 warns',
      fn: async () => {
        expect(lineOf({
          log: armWith({ stops: ['ok',], unusable: 13, refused: 13, seat: CLEAN_SEAT, units: 264, },),
          check: 'unusable plus refused',
        },),).toBe('',);
        expect(lineOf({
          log: armWith({ stops: ['ok',], unusable: 13, refused: 14, seat: CLEAN_SEAT, units: 264, },),
          check: 'unusable plus refused',
        },),).toBe('WARN low: unusable plus refused is 27 of 264 units, over 10 percent',);
        expect(lineOf({
          log: armWith({ stops: ['ok',], unusable: 2, refused: 3, seat: CLEAN_SEAT, units: 88, },),
          check: 'unusable plus refused',
        },),).toBe('',);
        expect(lineOf({
          log: armWith({ stops: ['ok',], unusable: 4, refused: 5, seat: CLEAN_SEAT, units: 88, },),
          check: 'unusable plus refused',
        },),).toBe('WARN low: unusable plus refused is 9 of 88 units, over 10 percent',);
      },
    },),
  ],
},);
