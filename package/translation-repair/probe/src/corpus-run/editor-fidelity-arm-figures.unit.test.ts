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

import {
  type ArmLog,
  armFiguresOf,
} from '../../dist/final/node/index.mjs';
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
            stops: ['ok', 'refused+ok', '', 'output-cap',],
            seat: SEAT,
            benchSpentUsd: 0,
            guardUsd: 0.5,
          },),
        },);
        expect(rows,).toStrictEqual([
          { row: 1, trial: 'tabby-s0/insertion', entry: 'cat-0', kind: 'insertion', fixed: true, stops: ['ok',], stated: true, },
          { row: 2, trial: 'tabby-s1/insertion', entry: 'cat-1', kind: 'insertion', fixed: true, stops: ['refused', 'ok',], stated: true, },
          { row: 3, trial: 'tabby-s2/insertion', entry: 'cat-2', kind: 'insertion', fixed: true, stops: [], stated: false, },
          { row: 4, trial: 'tabby-s3/insertion', entry: 'cat-3', kind: 'insertion', fixed: true, stops: ['output-cap',], stated: true, },
        ],);
      },
    },),

    it({
      name: 'READS THE ENTRY, THE KIND AND WHETHER THE ROW IS A CLEAN FIX of each row line: a reading that starts with '
        + 'clean fix is one, and one that says defect only, not a clean fix is not',
      fn: async () => {
        /**
         The first row fixed, the second not, the third a clean fix on an alteration trial of another entry.
         */
        const log = armLogOf({
          arm: 'low',
          tally: TALLY,
          stops: [],
          rows: [
            { trial: 'tabby-s0/insertion', entry: 'tabby', fixed: true, stops: 'ok', },
            { trial: 'tabby-s0/alteration', entry: 'tabby', fixed: false, stops: 'ok', },
            { trial: 'ginger-reference/alteration', entry: 'ginger', fixed: true, stops: 'ok', },
          ],
          seat: SEAT,
          benchSpentUsd: 0,
          guardUsd: 0.5,
        },);
        const defectOnly: ArmLog = {
          ...log,
          lines: log.lines.map(function reword(line,): string {
            return line.replace('on ginger-reference/alteration: clean fix exact, nothing else touched', 'on ginger-reference/alteration: defect only, not a clean fix, nothing else touched',);
          },),
        };
        expect(armFiguresOf({ log, },).rows.map(function digest({ entry, kind, fixed, },): string {
          return `${entry} ${kind} ${String(fixed,)}`;
        },),).toStrictEqual(['tabby insertion true', 'tabby alteration false', 'ginger alteration true',],);
        expect(armFiguresOf({ log: defectOnly, },).rows.map(function fixedOf({ fixed, },): boolean {
          return fixed;
        },),).toStrictEqual([true, false, false,],);
      },
    },),

    it({
      name: 'READS NO ENTRY AS EMPTY for a row line from before the entry was written, so the bench check names it and '
        + 'the figures do not guess',
      fn: async () => {
        const log = armLogOf({ arm: 'low', tally: TALLY, stops: ['ok',], seat: SEAT, benchSpentUsd: 0, guardUsd: 0.5, },);
        const old: ArmLog = {
          ...log,
          lines: log.lines.map(function drop(line,): string {
            return line.replace('; entry cat-0', '',);
          },),
        };
        expect(armFiguresOf({ log: old, },).rows[0]?.entry,).toBe('',);
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
