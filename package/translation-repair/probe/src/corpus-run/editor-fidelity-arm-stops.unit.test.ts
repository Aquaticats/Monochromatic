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
} from '@monochromatic-dev/module-test/ts';

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

 @param stops - for each row, its stop words, absent for a row that states none
 @param unusable - trials with no usable reply
 @param refused - trials where the card was heard and proposed no edit
 @param seat - the SEAT line

 @returns The arm, named low

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
  }: {
    readonly stops: readonly (string | undefined)[];
    readonly unusable: number;
    readonly refused: number;
    readonly seat: string;
  },
): ArmLog {
  return armLogOf({
    arm: 'low',
    tally: tallyOf({
      cardId: LOW_CARD,
      insertion: { trials: 22, fixed: 18, defectOnly: 2, },
      alteration: { trials: 18, fixed: 12, defectOnly: 1, },
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

 @param log - the arm
 @param check - name of the check, or the start of a warning's words

 @returns The line, empty when there is none

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
      name: 'PASSES THE STOP-REASONS CHECK for an arm whose every row states how its asks ended, and fails it naming how '
        + 'many rows state nothing',
      fn: async () => {
        expect(lineOf({
          log: armWith({ stops: ['ok', 'refused+ok', 'ok',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'stop reasons stated',
        },),).toBe('PASS low: stop reasons stated: 3 rows each state how their asks ended',);
        expect(lineOf({
          log: armWith({ stops: ['ok', undefined, undefined,], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'stop reasons stated',
        },),).toBe('FAIL low: stop reasons stated: 2 of 3 rows state no stop reason',);
        expect(lineOf({
          log: armWith({ stops: [], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'stop reasons stated',
        },),).toBe('FAIL low: stop reasons stated: no row line was logged',);
      },
    },),

    it({
      name: 'FAILS THE CAP-AND-THROW CHECK when any trial ended at the output-token cap, naming the rows, and passes an '
        + 'arm whose asks ended any other way',
      fn: async () => {
        expect(lineOf({
          log: armWith({ stops: ['ok', 'refused+ok', 'unparseable+ok',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'no trial ended at the output cap or threw',
        },),).toBe('PASS low: no trial ended at the output cap or threw: 3 trials, none at the cap, none threw',);
        expect(lineOf({
          log: armWith({ stops: ['ok', 'output-cap', 'ok', 'output-cap+ok',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'no trial ended at the output cap or threw',
        },),).toBe('FAIL low: no trial ended at the output cap or threw: 2 trials ended at the output-token cap (rows 2, 4)',);
      },
    },),

    it({
      name: 'FAILS THE CAP-AND-THROW CHECK when any ask threw, from the row lines or from the seat line, and says which',
      fn: async () => {
        expect(lineOf({
          log: armWith({ stops: ['ok', 'threw+ok',], unusable: 0, refused: 0, seat: CLEAN_SEAT, },),
          check: 'no trial ended at the output cap or threw',
        },),).toBe('FAIL low: no trial ended at the output cap or threw: 1 trial had an ask that threw (row 2)',);
        expect(lineOf({
          log: armWith({
            stops: ['ok', 'ok',],
            unusable: 0,
            refused: 0,
            seat: `SEAT ${LOW_CARD} asked=40 usable=39 unusable=0 threw=2 silent=0 abandoned=0 unreachable=0`,
          },),
          check: 'no trial ended at the output cap or threw',
        },),).toBe('FAIL low: no trial ended at the output cap or threw: the seat line says 2 asks threw',);
      },
    },),

    it({
      name: 'WARNS WHEN UNUSABLE PLUS REFUSED EXCEED FOUR of an arm\'s trials, counts the warning in no failure, and says '
        + 'nothing at four',
      fn: async () => {
        /**
         Five unusable and refused together, over a pool of forty.
         */
        const over = armWith({ stops: ['ok',], unusable: 2, refused: 3, seat: CLEAN_SEAT, },);
        expect(lineOf({ log: over, check: 'unusable plus refused', },),).toBe(
          'WARN low: unusable plus refused is 5 of 40 trials, over 4',
        );
        expect(lineOf({
          log: armWith({ stops: ['ok',], unusable: 1, refused: 3, seat: CLEAN_SEAT, },),
          check: 'unusable plus refused',
        },),).toBe('',);
        /**
         The report without the warning, over the same arm bar the counts.
         */
        const quiet = checkEditorBench({ arms: [armWith({ stops: ['ok',], unusable: 1, refused: 3, seat: CLEAN_SEAT, },),], },);
        const loud = checkEditorBench({ arms: [over,], },);
        expect(loud.failed,).toBe(quiet.failed,);
      },
    },),
  ],
},);
