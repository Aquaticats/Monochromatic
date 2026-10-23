/**
 Tests the one comparison of a NeuralWatt return against a bound, and the words that say a return lies past the pass.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  pastPassWordsOf,
  returnFitsWithin,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'NeuralWatt return fit',
  concurrency: 1,
  children: [
    describe({
      name: returnFitsWithin.name,
      concurrency: 1,
      children: [
        it({
          name: 'FITS A RETURN AT THE BOUND AND NOT ONE MILLISECOND PAST IT, counted as the refusal states it',
          fn: async () => {
            expect(returnFitsWithin({ returnsInMs: 600_000, boundMs: 600_000, },),).toBe(true,);
            expect(returnFitsWithin({ returnsInMs: 600_001, boundMs: 600_000, },),).toBe(false,);
          },
        },),

        it({
          name: 'FITS NO RETURN INSIDE A BOUND OF ZERO OR BELOW',
          fn: async () => {
            expect(returnFitsWithin({ returnsInMs: 1, boundMs: 0, },),).toBe(false,);
            expect(returnFitsWithin({ returnsInMs: 1, boundMs: -5, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: pastPassWordsOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS A RETURN IS PAST THE PASS only when it is, and says nothing where nothing bounds the wait',
          fn: async () => {
            expect(pastPassWordsOf({ returnsInMs: 700_000, passLeftMs: 600_000, },),).toEqual([
              'past the 10 min this pass has left',
            ],);
            expect(pastPassWordsOf({ returnsInMs: 600_000, passLeftMs: 600_000, },),).toEqual([],);
            expect(pastPassWordsOf({ returnsInMs: 700_000, },),).toEqual([],);
          },
        },),

        it({
          name: 'NAMES THE BOUND THAT WON: "the entry has left" when the entry\'s own cap is the smaller bound, and '
            + '"this pass has left" for the pass\'s or when no owner is named',
          fn: async () => {
            expect(pastPassWordsOf({ returnsInMs: 3_600_000, passLeftMs: 1_200_000, boundOwner: 'entry', },),).toEqual([
              'past the 20 min the entry has left',
            ],);
            expect(pastPassWordsOf({ returnsInMs: 3_600_000, passLeftMs: 1_200_000, boundOwner: 'pass', },),).toEqual([
              'past the 20 min this pass has left',
            ],);
          },
        },),

        it({
          name: 'READS A SPENT PASS AS 0 s, never a negative number',
          fn: async () => {
            expect(pastPassWordsOf({ returnsInMs: 5, passLeftMs: -1, },),).toEqual([
              'past the 0 s this pass has left',
            ],);
          },
        },),
      ],
    },),
  ],
},);
