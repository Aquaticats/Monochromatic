/**
 Tests for the models a cut picture reading may be re-asked of (ledger B1074, B1100, B1201).

 The list is the models the reader role may take that read pictures, in the order of their runaway shares, with the seat
 the asks are claimed on and every model left out with why. A model another reader of the picture took is left out.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createReaskClaims,
  readsImages,
} from '@monochromatic-dev/translation-repair-provider';

import { otherReadersOf, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 The reader whose reply was cut.
 */
const READER = SEAT_SYNTHETIC_VISION_WITHHELD;

await describe({
  name: otherReadersOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'LISTS ONLY MODELS THAT READ PICTURES, never the reader or the bench, and names the others as reading none',
      fn: async () => {
        const { readers, seat, leftOut, } = otherReadersOf({
          modelId: READER,
          benchModelIds: [READER,],
          reask: 'other-model',
        },);
        expect(readers.length,).toBeGreaterThan(0,);
        expect(readers.every(function readsOne(reader,): boolean {
          return readsImages({ modelId: reader, },) && (reader !== READER);
        },),).toBe(true,);
        expect(seat.role,).toBe('reader',);
        expect(seat.bench,).toEqual([READER,],);
        expect(seat.reask,).toBe('other-model',);
        /**
         The models named as reading no picture.
         */
        const namedAsReadingNone = leftOut.filter(function readsNone({ why, },): boolean {
          return why === 'reads no picture';
        },);
        // A LIST THAT COULD BE EMPTY PASSES `every` WHATEVER IT HOLDS (B1234): the roster has models that read none.
        expect(namedAsReadingNone.length,).toBeGreaterThan(0,);
        expect(namedAsReadingNone.every(function readsNoPicture({ modelId, },): boolean {
          return !readsImages({ modelId, },);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES OUT A MODEL ANOTHER READER OF THE PICTURE TOOK, the record they share',
      fn: async () => {
        const claims = createReaskClaims();
        const [first,] = otherReadersOf({
          modelId: READER,
          benchModelIds: [READER,],
          reask: 'other-model',
        },).readers;
        if (first === undefined)
          throw new Error('the reader has a model to take',);
        claims.take({ modelId: first, },);
        const taken = otherReadersOf({
          modelId: READER,
          benchModelIds: [READER,],
          reask: 'other-model',
          claims,
        },);
        expect(taken.readers,).not.toContain(first,);
        expect(taken.leftOut.some(function namesIt({ modelId, },): boolean {
          return modelId === first;
        },),).toBe(true,);
      },
    },),
  ],
},);
