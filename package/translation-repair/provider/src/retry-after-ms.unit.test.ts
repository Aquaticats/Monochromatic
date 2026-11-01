/**
 Tests the wait a refused reply's own body names, read by one forward scan
 (`retry-stated-wait.ts`). The ladder that sleeps it is exercised in
 `transient-retry.unit.test.ts`; these cases pin what the scan reads out of a
 body, where a number runs to the end of it and where the parts of one wait
 are joined by the word "and". Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { retryAfterMsOf, } from '../dist/final/node/index.mjs';

await describe({
  name: retryAfterMsOf.name,
  children: [
    it({
      name: 'READS NO WAIT FROM A NUMBER THE BODY ENDS ON, since a number with no unit after it names no time',
      fn: async () => {
        expect([
          retryAfterMsOf({ bodyText: 'The cat is asleep, try again in 30', },),
          retryAfterMsOf({ bodyText: 'The cat is asleep, try again in 30 ', },),
        ],).toEqual([0, 0,],);
      },
    },),
    it({
      name: 'SUMS THE PARTS OF A WAIT JOINED BY "and", with spaces or commas around it',
      fn: async () => {
        expect([
          retryAfterMsOf({ bodyText: 'The cat is asleep, try again in 1 minute and 30 seconds.', },),
          retryAfterMsOf({ bodyText: 'The cat is asleep, try again in 2 hours, and 15 minutes', },),
        ],).toEqual([90_000, 8_100_000,],);
      },
    },),
  ],
},);
