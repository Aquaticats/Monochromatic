/**
 Tests for `formatTrackedDuration`.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { formatTrackedDuration, } from '@monochromatic-dev/numeric-format-core';

const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 3_600;
const SECONDS_PER_DAY = 86_400;
const SECONDS_PER_WEEK = 7 * SECONDS_PER_DAY;
const SECONDS_PER_MONTH = 30 * SECONDS_PER_DAY;
const SECONDS_PER_YEAR = 365 * SECONDS_PER_DAY;

await describe({
  name: formatTrackedDuration.name,
  children: [
    //region Seconds-only branch (top-1)

    it({
      name: 'renders zero as "0s"',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(0,),).toBe('0s',);
      },
    },),

    it({
      name: 'renders sub-minute durations as seconds only',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(1,),).toBe('1s',);
        expect(formatTrackedDuration(45,),).toBe('45s',);
        expect(formatTrackedDuration(SECONDS_PER_MINUTE - 1,),).toBe('59s',);
      },
    },),

    //endregion Seconds-only branch (top-1)

    //region Minute branch: m + s

    it({
      name: 'renders minute boundary as "1m0s"',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(SECONDS_PER_MINUTE,),).toBe('1m0s',);
      },
    },),

    it({
      name: 'renders sub-hour durations as minutes + seconds',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(90,),).toBe('1m30s',);
        expect(formatTrackedDuration(SECONDS_PER_HOUR - 1,),).toBe('59m59s',);
      },
    },),

    //endregion Minute branch: m + s

    //region Hour branch: h + m (m means minutes here)

    it({
      name: 'renders hour boundary as "1h0m"',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(SECONDS_PER_HOUR,),).toBe('1h0m',);
      },
    },),

    it({
      name: 'renders sub-day durations as hours + minutes',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(5_400,),).toBe('1h30m',);
        expect(formatTrackedDuration(SECONDS_PER_DAY - 1,),).toBe('23h59m',);
      },
    },),

    //endregion Hour branch: h + m (m means minutes here)

    //region Day branch: d + h

    it({
      name: 'renders day boundary as "1d0h"',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(SECONDS_PER_DAY,),).toBe('1d0h',);
      },
    },),

    it({
      name: 'renders multi-day durations as days + hours',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(SECONDS_PER_DAY + SECONDS_PER_HOUR,),).toBe('1d1h',);
        expect(formatTrackedDuration(263_400,),).toBe('3d1h',);
      },
    },),

    //endregion Day branch: d + h

    //region Week branch: w + d

    it({
      name: 'renders week boundary as "1w0d"',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(SECONDS_PER_WEEK,),).toBe('1w0d',);
      },
    },),

    it({
      name: 'renders multi-week durations as weeks + days',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration((2 * SECONDS_PER_WEEK) + (3 * SECONDS_PER_DAY),),)
          .toBe('2w3d',);
      },
    },),

    //endregion Week branch: w + d

    //region Month branch: m + w (m means months here)

    it({
      name: 'renders month boundary (30d) as "1m0w"',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(SECONDS_PER_MONTH,),).toBe('1m0w',);
      },
    },),

    it({
      name:
        'drops trailing days when months pairs with weeks=0 (strict-adjacency precision loss)',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(35 * SECONDS_PER_DAY,),).toBe('1m0w',);
      },
    },),

    it({
      name: 'renders months + weeks when both non-zero',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration((2 * SECONDS_PER_MONTH) + SECONDS_PER_WEEK,),).toBe(
          '2m1w',
        );
      },
    },),

    //endregion Month branch: m + w (m means months here)

    //region Year branch: y + m (m means months here)

    it({
      name: 'renders year boundary (365d) as "1y0m"',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(SECONDS_PER_YEAR,),).toBe('1y0m',);
      },
    },),

    it({
      name: 'renders user-stated example 1y2m at (365 + 60)d',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration((365 + 60) * SECONDS_PER_DAY,),).toBe('1y2m',);
      },
    },),

    it({
      name: 'drops trailing days under month threshold (390d → "1y0m")',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(390 * SECONDS_PER_DAY,),).toBe('1y0m',);
      },
    },),

    //endregion Year branch: y + m (m means months here)

    //region Input sanitization

    it({
      name: 'clamps negative input to zero',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(-5,),).toBe('0s',);
        expect(formatTrackedDuration(-SECONDS_PER_HOUR,),).toBe('0s',);
      },
    },),

    it({
      name: 'floors fractional input',
      fn: async ({ expect, },) => {
        expect(formatTrackedDuration(0.7,),).toBe('0s',);
        expect(formatTrackedDuration(60.9,),).toBe('1m0s',);
        expect(formatTrackedDuration(5_400.5,),).toBe('1h30m',);
      },
    },),
    //endregion Input sanitization
  ],
},);
