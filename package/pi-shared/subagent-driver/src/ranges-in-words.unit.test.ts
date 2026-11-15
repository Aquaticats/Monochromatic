/**
 What a review's ranges are in words: each range named with what it holds, so a reviewer who does not know git's three dots reads
 the change as the set of commits it is, and the list a review copies into its own file.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  rangeInWords,
  rangesInALine,
  rangesInWords,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: rangeInWords.name,
      children: [
        it({
          name: 'says a three-dot range is everything the commit under review holds that its left side does not',
          fn: async () => {
            expect(rangeInWords({ range: 'main...fix/a', },),).toBe('`main...fix/a`, which is everything the commit you start on holds that `main` does not',);
          },
        },),
        it({
          name: 'says a range from the first parent of a merge the branch was taken in by is held against that parent',
          fn: async () => {
            expect(rangeInWords({ range: '9821cb2c10^1...fix/a', },),).toBe('`9821cb2c10^1...fix/a`, which is everything the commit you start on holds that `9821cb2c10^1` does not',);
          },
        },),
        it({
          name: 'says a two-dot range is the difference between its two commits, which is not a change read from a merge base',
          fn: async () => {
            expect(rangeInWords({ range: 'main..fix/a', },),).toBe('`main..fix/a`, the difference between `main` and `fix/a`',);
          },
        },),
        it({
          name: 'says a range with neither form is taken as git diff takes it, rather than claiming what it holds',
          fn: async () => {
            expect(rangeInWords({ range: 'abc1234', },),).toBe('`abc1234`, as `git diff` takes it',);
            expect(rangeInWords({ range: '...fix/a', },),).toBe('`...fix/a`, as `git diff` takes it',);
          },
        },),
      ],
    },),
    describe({
      name: rangesInWords.name,
      children: [
        it({
          name: 'names the one range a change read over one range was given',
          fn: async () => {
            expect(rangesInWords({ ranges: ['main...fix/a',], },),).toBe('the range `main...fix/a`, which is everything the commit you start on holds that `main` does not',);
          },
        },),
        it({
          name: 'names every range of a change read over two merges, the last joined by and',
          fn: async () => {
            expect(rangesInWords({ ranges: ['main...fix/a', 'merge^1...fix/a',], },),)
              .toBe('the ranges `main...fix/a`, which is everything the commit you start on holds that `main` does not, and `merge^1...fix/a`, which is everything the commit you start on holds that `merge^1` does not',);
          },
        },),
        it({
          name: 'says a change that names no range names none, rather than naming the empty list',
          fn: async () => {
            expect(rangesInWords({ ranges: [], },),).toBe('no range',);
          },
        },),
      ],
    },),
    describe({
      name: rangesInALine.name,
      children: [
        it({
          name: 'writes every range in backticks, separated by commas, which is the line a review copies',
          fn: async () => {
            expect(rangesInALine({ ranges: ['main...fix/a',], },),).toBe('`main...fix/a`',);
            expect(rangesInALine({ ranges: ['main...fix/a', 'merge^1...fix/a',], },),).toBe('`main...fix/a`, `merge^1...fix/a`',);
          },
        },),
      ],
    },),
  ],
},);
