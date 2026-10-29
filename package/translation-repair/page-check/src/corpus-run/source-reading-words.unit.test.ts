/**
 Tests for the index of the words of a page and the questions asked of it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  firstAtLeast,
  indexOfWords,
  matchesAt,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the words of a page',
  children: [
    describe({
      name: indexOfWords.name,
      children: [
        it({
          name: 'LISTS WHERE EACH WORD OF THE PAGE STANDS, in order',
          fn: async () => {
            expect([...indexOfWords({ page: ['a', 'b', 'a', 'c', 'a',], },),],).toEqual([
              ['a', [0, 2, 4,],],
              ['b', [1,],],
              ['c', [3,],],
            ],);
            expect([...indexOfWords({ page: [], },),],).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: matchesAt.name,
      children: [
        it({
          name: 'SAYS WHETHER A RUN OF WORDS STANDS AT AN INDEX, a run that does not fit never',
          fn: async () => {
            expect(matchesAt({ page: ['a', 'b', 'c',], at: 1, words: ['b', 'c',], },),).toBe(true,);
            expect(matchesAt({ page: ['a', 'b', 'c',], at: 1, words: ['b', 'd',], },),).toBe(false,);
            expect(matchesAt({ page: ['a', 'b', 'c',], at: 2, words: ['c', 'd',], },),).toBe(false,);
            expect(matchesAt({ page: ['a', 'b', 'c',], at: -1, words: ['a',], },),).toBe(false,);
          },
        },),
        it({
          name: 'SAYS AN EMPTY RUN STANDS ANYWHERE FROM THE START OF THE PAGE TO ITS END',
          fn: async () => {
            expect(matchesAt({ page: ['a',], at: 0, words: [], },),).toBe(true,);
            expect(matchesAt({ page: ['a',], at: 1, words: [], },),).toBe(true,);
            expect(matchesAt({ page: ['a',], at: 2, words: [], },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: firstAtLeast.name,
      children: [
        it({
          name: 'FINDS THE FIRST OF ASCENDING NUMBERS THAT IS NOT BELOW A NUMBER, the length of the list when there is none',
          fn: async () => {
            expect(firstAtLeast({ positions: [1, 4, 9,], from: 4, },),).toBe(1,);
            expect(firstAtLeast({ positions: [1, 4, 9,], from: 5, },),).toBe(2,);
            expect(firstAtLeast({ positions: [1, 4, 9,], from: 10, },),).toBe(3,);
            expect(firstAtLeast({ positions: [], from: 0, },),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
