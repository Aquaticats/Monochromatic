/**
 Tests for whether one folded quote stands inside another at the edges of words: the match at the start, the middle and the end
 of the longer quote, the match inside a longer word or beside a combining mark that is refused, a later match that is at the
 edges when the first is not, and the blank quote that stands only in a blank one.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { containsWords, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: containsWords.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS THE WORDS AT THE START, THE MIDDLE AND THE END OF THE LONGER QUOTE, and in a quote that is the words',
          fn: async () => {
            expect([
              containsWords({ outer: 'the cat naps', inner: 'the cat', },),
              containsWords({ outer: 'the cat naps', inner: 'cat', },),
              containsWords({ outer: 'the cat naps', inner: 'cat naps', },),
              containsWords({ outer: 'the cat naps', inner: 'the cat naps', },),
            ],).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'REFUSES WORDS THAT STAND INSIDE A LONGER WORD, or beside a combining mark of a spaced script, and words that are '
            + 'not in the quote',
          fn: async () => {
            expect([
              containsWords({ outer: 'the cats nap', inner: 'cat', },),
              containsWords({ outer: 'the concat nap', inner: 'cat', },),
              containsWords({ outer: 'the caté naps', inner: 'cat', },),
              containsWords({ outer: 'the dog naps', inner: 'cat', },),
            ],).toEqual([false, false, false, false,],);
          },
        },),
        it({
          name: 'READS A JOINT OF A LONGER WORD AS CUTTING THE WORD, as the locator does, unless the caller names the earlier reading '
            + 'of an artifact written before joints counted, which asks for no letter, digit or combining mark beside the match and '
            + 'nothing more',
          fn: async () => {
            /**
             The pairs the readings differ on: a match beside an apostrophe, a hyphen and a joint of digits.
             */
            const pairs = [
              { outer: 'the cat\'s bowl', inner: 'cat', },
              { outer: 'the cat-nap', inner: 'cat', },
              { outer: 'the 0.5 cup', inner: '5', },
              { outer: 'the cats nap', inner: 'cat', },
            ] as const;
            expect({
              byDefault: pairs.map(function byDefault(pair,): boolean {
                return containsWords(pair,);
              },),
              locator: pairs.map(function byLocator(pair,): boolean {
                return containsWords({ ...pair, edges: 'locator', },);
              },),
              letters: pairs.map(function byLetters(pair,): boolean {
                return containsWords({ ...pair, edges: 'letters', },);
              },),
            },).toEqual({
              byDefault: [false, false, false, false,],
              locator: [false, false, false, false,],
              letters: [true, true, true, false,],
            },);
          },
        },),
        it({
          name: 'REFUSES WORDS THAT A COMBINING MARK STANDS DIRECTLY BEFORE, as it refuses one directly after them, since the mark runs '
            + 'on from the letter it follows',
          fn: async () => {
            expect([
              containsWords({ outer: 'the e\u{0301}cat naps', inner: 'cat', },),
              containsWords({ outer: 'e\u{0301}cat', inner: 'cat', },),
              containsWords({ outer: 'the cat\u{0301} naps', inner: 'cat', },),
              containsWords({ outer: 'the e\u{0301} cat naps', inner: 'cat', },),
            ],).toEqual([false, false, false, true,],);
          },
        },),
        it({
          name: 'KEEPS LOOKING AFTER A MATCH INSIDE A WORD, so a later match at the edges of words counts',
          fn: async () => {
            expect(containsWords({ outer: 'cats and a cat', inner: 'cat', },),).toBe(true,);
          },
        },),
        it({
          name: 'FINDS A BLANK QUOTE ONLY IN A BLANK ONE, since an empty text is found at every offset',
          fn: async () => {
            expect([
              containsWords({ outer: '', inner: '', },),
              containsWords({ outer: 'the cat', inner: '', },),
              containsWords({ outer: '', inner: 'cat', },),
            ],).toEqual([true, false, false,],);
          },
        },),
      ],
    },),
  ],
},);
