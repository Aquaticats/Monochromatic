/**
 Tests for the figures read off an editor fidelity arm's log for the read step: the clean fix counts pooled and for each
 kind of defect, the defect-only count beside them, what went unusable or was refused, what the seat line says threw and
 was abandoned, and how the asks of each row ended.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { armFiguresOf, } from '../../dist/final/node/index.mjs';
import {
  armLogOf,
  tallyOf,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import { LOW_CARD, } from '../editor-fidelity.test-fixture.ts';

/**
 A card that fixed eighteen of twenty-two insertions and twelve of eighteen alterations.
 */
const TALLY = tallyOf({
  cardId: LOW_CARD,
  insertion: { trials: 22, fixed: 18, defectOnly: 2, },
  alteration: { trials: 18, fixed: 12, defectOnly: 1, },
  unusable: 1,
  refused: 2,
});

/**
 A seat line that states every figure.
 */
const SEAT = `SEAT ${LOW_CARD} asked=41 usable=39 unusable=1 threw=2 silent=0 abandoned=3 unreachable=0`;

await describe({
  name: armFiguresOf.name,
  children: [
    it({
      name: 'READS THE CLEAN FIX COUNTS POOLED AND FOR EACH KIND, the defect-only count beside each, and what went '
        + 'unusable or was refused, from the EDITOR lines the probe wrote',
      fn: async () => {
        const figures = armFiguresOf({
          log: armLogOf({ arm: 'low', tally: TALLY, stops: ['ok',], seat: SEAT, benchSpentUsd: 0, guardUsd: 0.5, },),
        },);
        expect(figures.card,).toBe(LOW_CARD,);
        expect(figures.trials,).toBe(40,);
        expect(figures.fixed,).toBe(30,);
        expect(figures.defectOnly,).toBe(3,);
        expect(figures.unusable,).toBe(1,);
        expect(figures.refused,).toBe(2,);
        expect(figures.insertion,).toStrictEqual({ trials: 22, fixed: 18, defectOnly: 2, },);
        expect(figures.alteration,).toStrictEqual({ trials: 18, fixed: 12, defectOnly: 1, },);
      },
    },),

    it({
      name: 'READS WHAT THREW AND WHAT WAS ABANDONED from the seat line, and minus one for a seat line that does not say',
      fn: async () => {
        expect(armFiguresOf({
          log: armLogOf({ arm: 'low', tally: TALLY, stops: [], seat: SEAT, benchSpentUsd: 0, guardUsd: 0.5, },),
        },).threw,).toBe(2,);
        expect(armFiguresOf({
          log: armLogOf({ arm: 'low', tally: TALLY, stops: [], seat: SEAT, benchSpentUsd: 0, guardUsd: 0.5, },),
        },).abandoned,).toBe(3,);
        /**
         A seat line that names neither.
         */
        const silent = armFiguresOf({
          log: armLogOf({ arm: 'low', tally: TALLY, stops: [], seat: `SEAT ${LOW_CARD} asked=41`, benchSpentUsd: 0, guardUsd: 0.5, },),
        },);
        expect(silent.threw,).toBe(-1,);
        expect(silent.abandoned,).toBe(-1,);
      },
    },),

    it({
      name: 'READS HOW THE ASKS OF EACH ROW ENDED, the row number and the trial beside them, in the order the rows were '
        + 'written, and marks a row whose line states nothing as not stating',
      fn: async () => {
        const { rows, } = armFiguresOf({
          log: armLogOf({
            arm: 'low',
            tally: TALLY,
            stops: ['ok', 'refused+ok', undefined, 'output-cap',],
            seat: SEAT,
            benchSpentUsd: 0,
            guardUsd: 0.5,
          },),
        },);
        expect(rows,).toStrictEqual([
          { row: 1, trial: 'tabby-s0/insertion', stops: ['ok',], stated: true, },
          { row: 2, trial: 'tabby-s1/insertion', stops: ['refused', 'ok',], stated: true, },
          { row: 3, trial: 'tabby-s2/insertion', stops: [], stated: false, },
          { row: 4, trial: 'tabby-s3/insertion', stops: ['output-cap',], stated: true, },
        ],);
      },
    },),

    it({
      name: 'SAYS MINUS ONE AND NOTHING for a log with no lines, so a figure that is not there is not read as zero',
      fn: async () => {
        const figures = armFiguresOf({ log: { arm: 'none', lines: [], exit: { recorded: false, }, }, },);
        expect(figures.card,).toBe('',);
        expect(figures.trials,).toBe(-1,);
        expect(figures.fixed,).toBe(-1,);
        expect(figures.insertion.fixed,).toBe(-1,);
        expect(figures.alteration.defectOnly,).toBe(-1,);
        expect(figures.unusable,).toBe(-1,);
        expect(figures.refused,).toBe(-1,);
        expect(figures.rows,).toStrictEqual([],);
      },
    },),
  ],
},);
