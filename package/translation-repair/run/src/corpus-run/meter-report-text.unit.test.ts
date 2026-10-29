/**
 Tests for how the meter report writes a span of time, an instant and the
 longest outage a provider had.

 EVERY UNIT A DURATION IS WRITTEN IN IS CHECKED PRESENT AND ABSENT: a span
 drops the units that carry nothing, so `2h0m5s` reads as `2h5s`, and says `0s`
 for none rather than an empty string.

 AN OUTAGE NAMES ITS OPEN ENDS. A stretch with no wet reading before it may
 have begun before the record, one with none after it may still be running, and
 in either case its upper bound is not a number. Each of the four ways the two
 ends can stand is a case.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type DrySpan,
  outageLines,
  spanText,
  stampText,
} from '../../dist/final/node/index.mjs';

/**
 An outage confirmed over thirty minutes, bounded by wet readings on both sides.
 */
const CLOSED_SPAN: DrySpan = {
  confirmedMs: 1_800_000,
  boundedByMs: 5_400_000,
  openBefore: false,
  openAfter: false,
  firstAt: 1_787_653_800_000,
  lastAt: 1_787_655_600_000,
};

/**
 An outage confirmed over thirty minutes whose upper bound the record does not
 give, which is what an open end leaves.
 */
const UNBOUNDED_SPAN: DrySpan = {
  confirmedMs: 1_800_000,
  openBefore: false,
  openAfter: false,
  firstAt: 1_787_653_800_000,
  lastAt: 1_787_655_600_000,
};

await describe({
  name: 'meter report text',
  concurrency: 1,
  children: [
    describe({
      name: spanText.name,
      concurrency: 1,
      children: [
        it({
          name: 'WRITES hours, minutes and seconds, each only where it carries something',
          fn: async () => {
            expect(spanText({ ms: 3_725_000, },),).toBe('1h2m5s',);
            expect(spanText({ ms: 7_205_000, },),).toBe('2h5s',);
            expect(spanText({ ms: 3_720_000, },),).toBe('1h2m',);
            expect(spanText({ ms: 7_200_000, },),).toBe('2h',);
          },
        },),

        it({
          name: 'WRITES minutes and seconds under an hour, and seconds alone under a minute',
          fn: async () => {
            expect(spanText({ ms: 61_000, },),).toBe('1m1s',);
            expect(spanText({ ms: 1_800_000, },),).toBe('30m',);
            expect(spanText({ ms: 59_000, },),).toBe('59s',);
          },
        },),

        it({
          name: 'WRITES 0s for no time and for less than a second, never an empty string',
          fn: async () => {
            expect(spanText({ ms: 0, },),).toBe('0s',);
            expect(spanText({ ms: 999, },),).toBe('0s',);
          },
        },),
      ],
    },),

    describe({
      name: stampText.name,
      concurrency: 1,
      children: [
        it({
          name: 'WRITES an epoch instant the way the log wrote it, as an ISO stamp in UTC',
          fn: async () => {
            expect(stampText({ at: 0, },),).toBe('1970-01-01T00:00:00.000Z',);
            expect(stampText({ at: 1_787_652_000_000, },),).toBe('2026-08-25T10:00:00.000Z',);
          },
        },),
      ],
    },),

    describe({
      name: outageLines.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS no reading found the provider out when there is no span',
          fn: async () => {
            expect(outageLines({ span: 'no-outage', },),).toEqual([
              '  longest outage: none, no reading found this provider out',
            ],);
          },
        },),

        it({
          name: 'WRITES the confirmed length, the upper bound and where it was confirmed when both ends closed',
          fn: async () => {
            expect(outageLines({ span: CLOSED_SPAN, },),).toEqual([
              '  longest outage: at least 30m, at most 1h30m (2026-08-25T10:30:00.000Z .. 2026-08-25T11:00:00.000Z)',
            ],);
          },
        },),

        it({
          name: 'NAMES the start as possibly before the record where no wet reading precedes the outage',
          fn: async () => {
            expect(outageLines({
              span: {
                ...UNBOUNDED_SPAN,
                openBefore: true,
              },
            },),).toEqual([
              '  longest outage: at least 30m, and NOT BOUNDED ABOVE '
                + '(2026-08-25T10:30:00.000Z .. 2026-08-25T11:00:00.000Z)',
              '    no reading before it found this provider up, so it may have begun earlier than the record',
            ],);
          },
        },),

        it({
          name: 'NAMES the end as possibly still running where no wet reading follows the outage',
          fn: async () => {
            expect(outageLines({
              span: {
                ...UNBOUNDED_SPAN,
                openAfter: true,
              },
            },),).toEqual([
              '  longest outage: at least 30m, and NOT BOUNDED ABOVE '
                + '(2026-08-25T10:30:00.000Z .. 2026-08-25T11:00:00.000Z)',
              '    no reading after it found this provider up, so it may still be running',
            ],);
          },
        },),

        it({
          name: 'NAMES both open ends, the start first, where no wet reading stands on either side',
          fn: async () => {
            expect(outageLines({
              span: {
                ...UNBOUNDED_SPAN,
                confirmedMs: 0,
                openBefore: true,
                openAfter: true,
              },
            },),).toEqual([
              '  longest outage: at least 0s, and NOT BOUNDED ABOVE '
                + '(2026-08-25T10:30:00.000Z .. 2026-08-25T11:00:00.000Z)',
              '    no reading before it found this provider up, so it may have begun earlier than the record',
              '    no reading after it found this provider up, so it may still be running',
            ],);
          },
        },),
      ],
    },),
  ],
},);
