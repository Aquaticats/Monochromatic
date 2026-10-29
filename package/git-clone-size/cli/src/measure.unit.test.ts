/**
 Tests the measurement sentinel and its narrowing guard.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isMeasured,
  UNMEASURED,
} from './measure.ts';

await describe({
  name: isMeasured.name,
  children: [
    it({
      name: 'accepts a numeric measurement, including a legitimate zero',
      fn: async ({ expect, }) => {
        expect(isMeasured(0,)).toBe(true,);
        expect(isMeasured(4_096,)).toBe(true,);
      },
    }),

    it({
      name: 'rejects the UNMEASURED sentinel',
      fn: async ({ expect, }) => {
        expect(isMeasured(UNMEASURED,)).toBe(false,);
      },
    }),
  ],
});
