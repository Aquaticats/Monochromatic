/**
 Tests for the selection fan-out: when a judge bench is too thin for the window to hold two answering judges, the
 ballots a winner needs (the owner's rulings of 2026-10-25 removed the weight the window used to be sized by).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { MIN_SELECTION_BALLOTS, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  FANOUT_SPARE,
  firstRoundWindow,
} from '@monochromatic-dev/translation-repair-stage';
import { selectionFanOut, } from '../dist/final/node/index.mjs';

await describe({
  name: selectionFanOut.name,
  children: [
    it({
      name: 'ASKS the window from a four-seat bench, since the window holds at least two judges, which is all a '
        + 'winner needs; weight no longer sizes it',
      fn: async () => {
        expect(firstRoundWindow({ benchSize: 4, },) >= MIN_SELECTION_BALLOTS,).toBe(true,);
        expect(selectionFanOut({ judgeCount: 4, },),).toBe('window',);
        expect(selectionFanOut({ judgeCount: 4 + FANOUT_SPARE + 1, },),).toBe('window',);
      },
    },),
    it({
      name: 'ASKS the whole bench where the window holds fewer judges than a winner needs ballots',
      fn: async () => {
        expect(firstRoundWindow({ benchSize: 1, },) < MIN_SELECTION_BALLOTS,).toBe(true,);
        expect(selectionFanOut({ judgeCount: 1, },),).toBe('whole-bench',);
      },
    },),
    it({
      name: 'HONOURS a whole-bench request on any bench, since a fixture scripting every seat reads its '
        + 'tally over the bench it wrote',
      fn: async () => {
        expect(selectionFanOut({ judgeCount: 9, requested: 'whole-bench', },),)
          .toBe('whole-bench',);
        expect(selectionFanOut({ judgeCount: 9, requested: 'window', },),)
          .toBe('window',);
      },
    },),
  ],
},);
