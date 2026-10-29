/**
 Tests for the words the dropped-destination refusal writes.

 WHAT THESE PIN: the refusal says how many destinations the page would drop
 and which slices carry them in the words of that count, one form at exactly
 one and the other at any other count, and says where a destination outside
 every slice sits.

 Fixtures are invented slice numbers about a bookshop cat, so there is no
 corpus text here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { DroppedDestinationError, } from '../../dist/final/node/index.mjs';

await describe({
  name: DroppedDestinationError.name,
  children: [
    it({
      name: 'WORDS the refusal by count, one source destination carried in one slice',
      fn: async function wordsOne(): Promise<void> {
        const refusal = new DroppedDestinationError({
          entryId: 'Cat',
          droppedCount: 1,
          traces: [{
            sourceSlices: [4,],
            archiveSlices: [],
            shippedSlices: [],
          },],
        },);
        expect(refusal.message,).toBe(
          'entry Cat would drop 1 source destination, carried by the original in slice 4',
        );
      },
    },),
    it({
      name: 'WORDS the refusal by count, two source destinations carried in two slices',
      fn: async function wordsMany(): Promise<void> {
        const refusal = new DroppedDestinationError({
          entryId: 'Cat',
          droppedCount: 2,
          traces: [
            {
              sourceSlices: [7,],
              archiveSlices: [],
              shippedSlices: [],
            },
            {
              sourceSlices: [4,],
              archiveSlices: [],
              shippedSlices: [],
            },
          ],
        },);
        expect(refusal.message,).toBe(
          'entry Cat would drop 2 source destinations, carried by the original in slices 4, 7',
        );
      },
    },),
    it({
      name: 'WORDS the refusal by count, one source destination carried in no slice',
      fn: async function wordsOutside(): Promise<void> {
        const refusal = new DroppedDestinationError({
          entryId: 'Cat',
          droppedCount: 1,
          traces: [{
            sourceSlices: [],
            archiveSlices: [],
            shippedSlices: [],
          },],
        },);
        expect(refusal.message,).toBe(
          'entry Cat would drop 1 source destination, carried by the original outside every slice',
        );
      },
    },),
    it({
      name: 'WORDS the refusal by count, two source destinations carried in one shared slice',
      fn: async function wordsShared(): Promise<void> {
        const refusal = new DroppedDestinationError({
          entryId: 'Cat',
          droppedCount: 2,
          traces: [
            {
              sourceSlices: [4,],
              archiveSlices: [],
              shippedSlices: [],
            },
            {
              sourceSlices: [4,],
              archiveSlices: [],
              shippedSlices: [],
            },
          ],
        },);
        expect(refusal.message,).toBe(
          'entry Cat would drop 2 source destinations, carried by the original in slice 4',
        );
      },
    },),
  ],
},);
