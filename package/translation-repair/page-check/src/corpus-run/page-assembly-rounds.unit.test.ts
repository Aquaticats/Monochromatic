/**
 Tests for when the page-assembly rounds run again (ledger K5, T8).

 A ROUND RUNS AGAIN ONLY FOR A SLICE NO EARLIER ROUND TOOK BACK. Two page
 passes rewrite the archive's text on every slice, so a later round can
 carry a pass-made row at a slice whose lane row an earlier round withdrew;
 were its guard to take that row back, every later round would make and lose
 it again. Until 2026-09-29 that round threw and stopped the entry's page;
 it now settles, the slice staying withdrawn. No fixture is known to reach
 that round through the real passes, so these cases drive the rule the loop
 runs on over plain rounds.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  freshlyTakenBack,
  type RoundTakeBacks,
} from '../../dist/final/node/index.mjs';

/**
 A round that withheld some halves and withdrew some rows.

 @param withheld - halves withheld

 @param reverted - rows the guard withdrew

 @returns Round as the rule reads it
 */
function roundOf(
  {
    withheld,
    reverted,
  }: {
    readonly withheld: readonly number[];
    readonly reverted: readonly number[];
  },
): RoundTakeBacks {
  return {
    halves: { withheld, },
    guarded: { revertedChunkIndices: reverted, },
  };
}

await describe({
  name: freshlyTakenBack.name,
  children: [
    it({
      name: 'NAMES the halves then the rows a round took back that no earlier round had, each once',
      fn: async () => {
        expect(freshlyTakenBack({
          round: roundOf({
            withheld: [2,],
            reverted: [5, 2, 7,],
          },),
          takenBack: [5,],
        },),).toEqual([2, 7,],);
      },
    },),
    it({
      name: 'NAMES NOTHING for a round that took back only what earlier rounds had, so the rounds settle '
        + 'there rather than making and losing the same pass-made row forever (ledger T8)',
      fn: async () => {
        expect(freshlyTakenBack({
          round: roundOf({
            withheld: [],
            reverted: [3, 5,],
          },),
          takenBack: [5, 3,],
        },),).toEqual([],);
      },
    },),
    it({
      name: 'NAMES NOTHING for a round that took nothing back',
      fn: async () => {
        expect(freshlyTakenBack({
          round: roundOf({
            withheld: [],
            reverted: [],
          },),
          takenBack: [],
        },),).toEqual([],);
      },
    },),
  ],
},);
