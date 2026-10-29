/**
 Tests for time and duration ratio constants.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DAYS_PER_WEEK,
  DAYS_PER_YEAR,
  HOURS_PER_DAY,
  MINUTES_PER_HOUR,
  MONTHS_PER_YEAR,
  MS_PER_DAY,
  MS_PER_HOUR,
  MS_PER_MINUTE,
  MS_PER_SECOND,
  MS_PER_WEEK,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'time',
  children: [
    it({
      name: 'primitive ratios match physical definitions',
      fn: async () => {
        expect(MS_PER_SECOND,).toBe(1_000,);
        expect(SECONDS_PER_MINUTE,).toBe(60,);
        expect(MINUTES_PER_HOUR,).toBe(60,);
        expect(HOURS_PER_DAY,).toBe(24,);
        expect(DAYS_PER_WEEK,).toBe(7,);
        expect(DAYS_PER_YEAR,).toBe(365,);
        expect(MONTHS_PER_YEAR,).toBe(12,);
      },
    },),
    it({
      name: 'composed millisecond constants compute correctly',
      fn: async () => {
        expect(MS_PER_MINUTE,).toBe(60_000,);
        expect(MS_PER_HOUR,).toBe(3_600_000,);
        expect(MS_PER_DAY,).toBe(86_400_000,);
        expect(MS_PER_WEEK,).toBe(604_800_000,);
      },
    },),
    it({
      name: 'composed second constants compute correctly',
      fn: async () => {
        expect(SECONDS_PER_HOUR,).toBe(3_600,);
        expect(SECONDS_PER_DAY,).toBe(86_400,);
      },
    },),
  ],
},);
