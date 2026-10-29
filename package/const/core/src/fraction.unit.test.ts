/**
 Tests for fractional constants.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  HALF,
  QUARTER,
  THIRD,
  THREE_QUARTERS,
  TWO_THIRDS,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'fraction',
  children: [
    it({
      name: 'HALF, QUARTER, THREE_QUARTERS have exact values',
      fn: async () => {
        expect(HALF,).toBe(0.5,);
        expect(QUARTER,).toBe(0.25,);
        expect(THREE_QUARTERS,).toBe(0.75,);
      },
    },),
    it({
      name: 'THIRD and TWO_THIRDS approximate 1/3 and 2/3',
      fn: async () => {
        expect(THIRD,).toBeCloseTo(1 / 3, 12,);
        expect(TWO_THIRDS,).toBeCloseTo(2 / 3, 12,);
      },
    },),
    it({
      name: 'composition relationships hold',
      fn: async () => {
        expect(QUARTER * 2,).toBe(HALF,);
        expect(HALF + QUARTER,).toBe(THREE_QUARTERS,);
        expect(THIRD + THIRD,).toBe(TWO_THIRDS,);
      },
    },),
  ],
},);
