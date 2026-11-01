/**
 Tests for case-insensitive word-frequency computation.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { computeFrequency, } from './frequency.ts';

await describe({
  name: computeFrequency.name,
  children: [
    it({
      name: 'returns an empty list for an empty word list',
      fn: async function returnsEmptyForEmptyWords(): Promise<void> {
        expect(computeFrequency([],),).toEqual([],);
      },
    },),
    it({
      name: 'excludes words that occur only once',
      fn: async function excludesSingletons(): Promise<void> {
        expect(computeFrequency(['a', 'b', 'c',],),).toEqual([],);
      },
    },),
    it({
      name: 'folds case before counting, displaying the lowercase form',
      fn: async function foldsCase(): Promise<void> {
        expect(
          computeFrequency(['The', 'the', 'cat', 'cat',],),
        )
          .toEqual([
            { word: 'cat', count: 2, percentage: 50, },
            { word: 'the', count: 2, percentage: 50, },
          ],);
      },
    },),
    it({
      name: 'sorts by count descending, breaking ties alphabetically',
      fn: async function sortsByCountThenAlphabetically(): Promise<void> {
        expect(
          computeFrequency(['dog', 'cat', 'dog', 'cat', 'cat',],),
        )
          .toEqual([
            { word: 'cat', count: 3, percentage: 60, },
            { word: 'dog', count: 2, percentage: 40, },
          ],);
      },
    },),
    it({
      name: 'computes percentage against every word, not only the words shown',
      fn: async function computesPercentageAgainstAllWords(): Promise<void> {
        /**
         Nine total words: "the" appears 3 times, "cat" 2 times, five
         other words appear once each and are excluded from the result.
         */
        const words = [
          'the',
          'cat',
          'sat',
          'on',
          'the',
          'mat',
          'the',
          'cat',
          'ran',
        ];

        expect(
          computeFrequency(words,),
        )
          .toEqual([
            { word: 'the', count: 3, percentage: (3 / 9) * 100, },
            { word: 'cat', count: 2, percentage: (2 / 9) * 100, },
          ],);
      },
    },),
  ],
},);
