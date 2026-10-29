/**
 Tests for finding the web and mail addresses of a place's English and for
 dropping the stands of a quote that lie inside one.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  addressRangesOf,
  standsOutsideAddresses,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: addressRangesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS AN ADDRESS FROM ITS SCHEME TO THE END OF ITS RUN, the letters before the colon belonging to it and '
            + 'the text of a link before its address not',
          fn: async () => {
            expect([
              addressRangesOf({ text: 'Visit https://cats.example.org/page now.', },),
              addressRangesOf({ text: 'Visit [the cats](https://cats.example.org/page) now.', },),
              addressRangesOf({ text: 'Visit <http://cats.example.org> now.', },),
              addressRangesOf({ text: 'ftp://cats.example.org/page', },),
              addressRangesOf({ text: '://cats', },),
            ],).toEqual([
              [{ start: 6, end: 35, },],
              [{ start: 17, end: 47, },],
              [{ start: 7, end: 31, },],
              [{ start: 0, end: 27, },],
              [{ start: 0, end: 7, },],
            ],);
          },
        },),
        it({
          name: 'FINDS A RUN THAT STARTS WITH WWW. WHOLE, in any case, and not a bare domain',
          fn: async () => {
            expect([
              addressRangesOf({ text: 'See www.cats.example.org now.', },),
              addressRangesOf({ text: 'See WWW.cats.example.org now.', },),
              addressRangesOf({ text: 'See (www.cats.example.org) now.', },),
              addressRangesOf({ text: 'See cats.example.org now.', },),
              addressRangesOf({ text: 'See wwwcats.example.org now.', },),
            ],).toEqual([
              [{ start: 4, end: 24, },],
              [{ start: 4, end: 24, },],
              [],
              [],
              [],
            ],);
          },
        },),
        it({
          name: 'FINDS A MAIL ADDRESS FROM THE START OF ITS LOCAL PART, an @ with a letter or a digit on each side and a full stop '
            + 'after it, and not an @ without one',
          fn: async () => {
            expect([
              addressRangesOf({ text: 'Mail cat@example.com now.', },),
              addressRangesOf({ text: '[mail](mailto:cats.5@home.org)', },),
              addressRangesOf({ text: 'Write cats@home today.', },),
              addressRangesOf({ text: 'Write cats @example.com today.', },),
              addressRangesOf({ text: 'Write cats@ example.com today.', },),
              addressRangesOf({ text: 'Write 猫@example.com today.', },),
            ],).toEqual([
              [{ start: 5, end: 20, },],
              [{ start: 14, end: 30, },],
              [],
              [],
              [],
              [{ start: 6, end: 19, },],
            ],);
          },
        },),
        it({
          name: 'FINDS EVERY ADDRESS OF A TEXT IN ORDER, one for a run, and none in a text without one or an empty one',
          fn: async () => {
            expect([
              addressRangesOf({ text: 'See www.cats.org and https://dogs.org or cat@home.org.', },),
              addressRangesOf({ text: 'The cats nap.', },),
              addressRangesOf({ text: '', },),
              addressRangesOf({ text: 'https://cats.org\n\nwww.dogs.org', },),
            ],).toEqual([
              [{ start: 4, end: 16, }, { start: 21, end: 37, }, { start: 41, end: 54, },],
              [],
              [],
              [{ start: 0, end: 16, }, { start: 18, end: 30, },],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: standsOutsideAddresses.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DROPS A STAND THAT LIES WHOLLY INSIDE AN ADDRESS and keeps one that ends before it, starts after it or only '
            + 'reaches into it',
          fn: async () => {
            expect(standsOutsideAddresses({
              stands: [2, 10, 16, 18, 24, 30, 40,],
              length: 4,
              addresses: [{ start: 10, end: 20, }, { start: 24, end: 28, },],
            },),).toEqual([2, 18, 30, 40,],);
          },
        },),
        it({
          name: 'KEEPS EVERY STAND WHEN THERE IS NO ADDRESS, and when no stand is given',
          fn: async () => {
            expect([
              standsOutsideAddresses({ stands: [1, 5,], length: 2, addresses: [], },),
              standsOutsideAddresses({ stands: [], length: 2, addresses: [{ start: 0, end: 9, },], },),
            ],).toEqual([[1, 5,], [],],);
          },
        },),
      ],
    },),
  ],
},);
