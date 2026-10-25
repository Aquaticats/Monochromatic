/**
 Tests for the pass's memory of slices that ended unfilled on a bench below its
 floor: one memory per entry, held for the whole pass, so a later attempt of
 the same entry finds what an earlier one left and another entry does not.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { floorLimitedByEntry, } from '../../dist/final/node/index.mjs';

await describe({
  name: floorLimitedByEntry.name,
  concurrency: 1,
  children: [
    it({
      name: 'HANDS THE SAME MEMORY TO EVERY ATTEMPT OF AN ENTRY and another to another entry, each starting empty',
      fn: async () => {
        const memories = floorLimitedByEntry();
        const first = memories.of({ entryId: 'tabby', },);
        first.set('slice 3', {
          reachable: 1,
          unfilled: {
            sliceIndex: 3,
            reason: 'no-candidate-backed',
            findings: [],
          },
          findings: [],
          told: false,
        },);
        expect({
          sameEntrySameMemory: memories.of({ entryId: 'tabby', },) === first,
          sameEntryKeepsWhatWasLeft: memories.of({ entryId: 'tabby', },).size,
          otherEntryDistinct: memories.of({ entryId: 'mittens', },) !== first,
          otherEntryEmpty: memories.of({ entryId: 'mittens', },).size,
          aNewPassStartsEmpty: floorLimitedByEntry().of({ entryId: 'tabby', },).size,
        },).toEqual({
          sameEntrySameMemory: true,
          sameEntryKeepsWhatWasLeft: 1,
          otherEntryDistinct: true,
          otherEntryEmpty: 0,
          aNewPassStartsEmpty: 0,
        },);
      },
    },),
  ],
},);
