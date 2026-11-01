/**
 Tests for reading which ids a stage roster seats more than once.

 The refusal `gatherStageVoices` makes of such a roster is pinned in
 `stage-quorum.unit.test.ts`, where a gather runs; this file holds the
 reading of the repeats alone.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { repeatedRosterIds, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: repeatedRosterIds.name,
  children: [
    it({
      name: 'READS NO ID FROM A ROSTER THAT SEATS EACH MODEL ONCE',
      fn: async () => {
        expect(repeatedRosterIds({
          modelIds: [SEAT_HYPER_ONLY, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
        },),).toEqual([],);
      },
    },),

    it({
      name: 'READS EACH REPEATED ID ONCE, in the order each first repeats, however often it is seated',
      fn: async () => {
        /** Roster seating the first id three times and the second twice. */
        const modelIds: readonly RosterModelId[] = [
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_HYPER_ONLY,
          SEAT_HYPER_ONLY,
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_HYPER_ONLY,
        ];
        expect(repeatedRosterIds({ modelIds, },),).toEqual([
          SEAT_HYPER_ONLY,
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        ],);
      },
    },),
  ],
},);
