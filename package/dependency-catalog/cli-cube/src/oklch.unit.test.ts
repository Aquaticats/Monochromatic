/**
 Tests for the OKLCH → sRGB conversion utilities.
 
 Covers `oklchToSrgb` (direct conversion, gamut clamping) and
 `oklchLerpToSrgb` (linear interpolation in OKLCH then conversion).
 Tolerance is loose because the reference values come from
 out-of-process tools; the goal is to catch sign/scale regressions,
 not bit-exactness.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  oklchLerpToSrgb,
  oklchToSrgb,
} from './oklch.ts';

/** Numeric tolerance for "close to expected" comparisons (8-bit RGB units). */
const RGB_TOLERANCE = 8;

/**
 Asserts `actual` is within {@link RGB_TOLERANCE} of `expected` per channel.
 
 @param actual - Conversion output.
 @param expected - Reference triple.
 */
function expectCloseRgb(
  {
    actual,
    expected,
  }: {
    actual: readonly [number, number, number,];
    expected: readonly [number, number, number,];
  },
): void {
  for (const channelIndex of [0, 1, 2,] as const) {
    expect(
      Math.abs(actual[channelIndex] - expected[channelIndex],),
    )
      .toBeLessThanOrEqual(RGB_TOLERANCE,);
  }
}

await describe({
  name: 'oklch',
  children: [
    //region oklchToSrgb
    it({
      name: 'oklchToSrgb converts mid-grey OKLCH to mid-grey sRGB',
      fn: async () => {
        const rgb = oklchToSrgb({
          L: 0.6,
          C: 0,
          H: 0,
        },);
        expect(rgb[0],).toBe(rgb[1],);
        expect(rgb[1],).toBe(rgb[2],);
        expect(rgb[0],).toBeGreaterThan(0,);
        expect(rgb[0],).toBeLessThan(255,);
      },
    },),

    it({
      name: 'oklchToSrgb returns a reddish triple for low-hue endpoint',
      fn: async () => {
        const rgb = oklchToSrgb({
          L: 0.65,
          C: 0.22,
          H: 29,
        },);
        expect(rgb[0],).toBeGreaterThan(rgb[1],);
        expect(rgb[0],).toBeGreaterThan(rgb[2],);
      },
    },),

    it({
      name: 'oklchToSrgb returns a greenish triple for mid-hue endpoint',
      fn: async () => {
        const rgb = oklchToSrgb({
          L: 0.74,
          C: 0.2,
          H: 145,
        },);
        expect(rgb[1],).toBeGreaterThan(rgb[0],);
        expect(rgb[1],).toBeGreaterThan(rgb[2],);
      },
    },),

    it({
      name: 'oklchToSrgb clamps out-of-gamut inputs into the 8-bit range',
      fn: async () => {
        const rgb = oklchToSrgb({
          L: 0.9,
          C: 0.5,
          H: 145,
        },);
        for (const channel of rgb) {
          expect(channel,).toBeGreaterThanOrEqual(0,);
          expect(channel,).toBeLessThanOrEqual(255,);
          expect(Number.isFinite(channel,),).toBe(true,);
        }
      },
    },),
    //endregion oklchToSrgb

    //region oklchLerpToSrgb
    it({
      name: 'oklchLerpToSrgb at t=0 returns the start colour',
      fn: async () => {
        const start = {
          L: 0.65,
          C: 0.22,
          H: 29,
        };
        const end = {
          L: 0.74,
          C: 0.2,
          H: 145,
        };
        expectCloseRgb({
          actual: oklchLerpToSrgb({
            start,
            end,
            t: 0,
          },),
          expected: oklchToSrgb(start,),
        },);
      },
    },),

    it({
      name: 'oklchLerpToSrgb at t=1 returns the end colour',
      fn: async () => {
        const start = {
          L: 0.65,
          C: 0.22,
          H: 29,
        };
        const end = {
          L: 0.74,
          C: 0.2,
          H: 145,
        };
        expectCloseRgb({
          actual: oklchLerpToSrgb({
            start,
            end,
            t: 1,
          },),
          expected: oklchToSrgb(end,),
        },);
      },
    },),

    it({
      name:
        'oklchLerpToSrgb midpoint is amber-ish (green channel dominant, red channel still high)',
      fn: async () => {
        const red = {
          L: 0.65,
          C: 0.22,
          H: 29,
        };
        const green = {
          L: 0.74,
          C: 0.2,
          H: 145,
        };
        const mid = oklchLerpToSrgb({
          start: red,
          end: green,
          t: 0.5,
        },);
        const startRgb = oklchToSrgb(red,);
        const endRgb = oklchToSrgb(green,);
        expect(mid[1],).toBeGreaterThan(startRgb[1],);
        expect(mid[0],).toBeGreaterThan(endRgb[0],);
      },
    },),
    //endregion oklchLerpToSrgb
  ],
},);
