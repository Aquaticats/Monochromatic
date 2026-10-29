/**
 Tests for the item expiry-stamp guard, driven directly because the guard
 is package-internal while its truthiness quirks decide lazy expiry.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  hasExpiryStamp,
} from '../dist/final/neutral/index.mjs';

/**
 Falsy expiry stamps upstream's truthiness gate reads as "never expires",
 with the label each test case reports.
 */
const FALSY_STAMPS: readonly (readonly [string, number])[] = [
  [
    'zero',
    0,
  ],
  [
    'negative zero',
    -0,
  ],
  [
    'NaN',
    Number.NaN,
  ],
];

await describe({
  name: 'cache item expiry stamps',
  children: [
    it({
      name: 'reports a missing stamp as no expiry',
      fn: async () => {
        expect(hasExpiryStamp({
          value: 1,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'reports a positive stamp as expiring',
      fn: async () => {
        expect(hasExpiryStamp({
          value: 1,
          expiry: 500,
        },),).toBe(true,);
      },
    },),

    ...FALSY_STAMPS.map(function mapFalsyStamp(entry: readonly [string, number],) {
      /**
       Label and timestamp of the falsy stamp under test.
       */
      const [label, expiry,] = entry;
      return it({
        name: `reports the falsy stamp ${label} as no expiry, matching upstream truthiness`,
        fn: async () => {
          expect(hasExpiryStamp({
            value: 1,
            expiry,
          },),).toBe(false,);
        },
      },);
    },),
  ],
},);
