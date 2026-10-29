/**
 Tests for size/ratio/savings formatting and clamping.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  clamp,
  computeRatio,
  computeSavings,
  roundTo,
  toSize,
} from './format.ts';

await describe({
  name: toSize.name,
  children: [
    it({
      name: 'attaches a human string and floors negatives to 0',
      fn: async ({ expect, }) => {
        expect(toSize(422)).toEqual({ bytes: 422, human: '422 B', });
        expect(toSize(-5)).toEqual({ bytes: 0, human: '0 B', });
        expect(toSize(1_048_576).human).toBe('1.0 MiB');
      },
    }),
  ],
});

await describe({
  name: roundTo.name,
  children: [
    it({
      name: 'rounds to the requested decimals',
      fn: async ({ expect, }) => {
        expect(roundTo({ value: 0.04421, decimals: 4, })).toBe(0.0442);
        expect(roundTo({ value: 95.64, decimals: 1, })).toBe(95.6);
      },
    }),
  ],
});

await describe({
  name: clamp.name,
  children: [
    it({
      name: 'confines values to the range',
      fn: async ({ expect, }) => {
        expect(clamp({ value: 1.3, min: 0, max: 1, })).toBe(1);
        expect(clamp({ value: -2, min: 0, max: 1, })).toBe(0);
        expect(clamp({ value: 0.5, min: 0, max: 1, })).toBe(0.5);
      },
    }),
  ],
});

await describe({
  name: computeRatio.name,
  children: [
    it({
      name: 'maps the denominator high end to the ratio low end',
      fn: async ({ expect, }) => {
        const ratio = computeRatio({
          shallowBytes: 10,
          full: {
            point: { bytes: 100, human: '100 B', },
            lo: { bytes: 50, human: '50 B', },
            hi: { bytes: 200, human: '200 B', },
          },
        });
        expect(ratio.point).toBe(0.1);
        expect(ratio.lo).toBe(0.05);
        expect(ratio.hi).toBe(0.2);
      },
    }),

    it({
      name: 'clamps to 1 and guards a non-positive denominator',
      fn: async ({ expect, }) => {
        const ratio = computeRatio({
          shallowBytes: 100,
          full: {
            point: { bytes: 0, human: '0 B', },
            lo: { bytes: 0, human: '0 B', },
            hi: { bytes: 50, human: '50 B', },
          },
        });
        expect(ratio.point).toBe(1);
        expect(ratio.hi).toBe(1);
        expect(ratio.lo).toBe(1);
      },
    }),
  ],
});

await describe({
  name: computeSavings.name,
  children: [
    it({
      name: 'inverts the ratio into a clamped percentage',
      fn: async ({ expect, }) => {
        const savings = computeSavings({ ratio: { point: 0.044, lo: 0.028, hi: 0.07, }, });
        expect(savings.point).toBe(95.6);
        expect(savings.lo).toBe(93);
        expect(savings.hi).toBe(97.2);
      },
    }),

    it({
      name: 'clamps a ratio above 1 to 0 percent savings',
      fn: async ({ expect, }) => {
        const savings = computeSavings({ ratio: { point: 1, lo: 1, hi: 1, }, });
        expect(savings.point).toBe(0);
      },
    }),
  ],
});
