/**
 Tests the word a count takes in a report line, and the count of occurrences
 a finding says in words.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  howOften,
  wordForCount,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wordForCount.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES THE ONE-FORM AT EXACTLY ONE and the other form at every other count, zero included, as English '
            + 'writes "0 whiskers"',
          fn: async () => {
            expect([0, 1, 2, 7,].map((count,) =>
              wordForCount({
                count,
                one: 'whisker',
                many: 'whiskers',
              },)
            ),).toEqual(['whiskers', 'whisker', 'whiskers', 'whiskers',],);
          },
        },),
      ],
    },),

    describe({
      name: howOften.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS "once" AT ONE and the count with "times" above it',
          fn: async () => {
            expect([1, 2, 12,].map((count,) => howOften({ count, },)),).toEqual(['once', '2 times', '12 times',],);
          },
        },),
      ],
    },),
  ],
},);
