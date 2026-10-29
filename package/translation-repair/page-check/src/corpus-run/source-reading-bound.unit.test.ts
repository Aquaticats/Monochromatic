/**
 Tests for the cost some way of placing the blocks no range reaches is known to have.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  boundOf,
  indexOfWords,
} from '../../dist/final/node/index.mjs';

await describe({
  name: boundOf.name,
  children: [
    it({
      name: 'IS NOTHING WHERE THE PAGE SHOWS THE SOURCE FROM ITS START TO ITS END',
      fn: async () => {
        const page = ['猫', '睡', '猫', '睡',];
        expect(boundOf({
          page,
          index: indexOfWords({ page, },),
          segments: [
            { reached: false, words: ['猫', '睡',], blocks: 0, },
            { reached: true, words: ['猫', '睡',], blocks: 1, },
          ],
        },),).toBe(0,);
      },
    },),
    it({
      name: 'IS THE BLOCKS A RANGE REACHES WHERE THE PAGE LOSES THEIR WORDS AND THE BLOCK AFTER THEM STANDS ON',
      fn: async () => {
        const page = ['猫', '睡', '鱼', '梦',];
        expect(boundOf({
          page,
          index: indexOfWords({ page, },),
          segments: [
            { reached: false, words: ['猫', '睡',], blocks: 0, },
            { reached: true, words: ['碗', '盆',], blocks: 2, },
            { reached: false, words: ['鱼', '梦',], blocks: 0, },
          ],
        },),).toBe(2,);
      },
    },),
    it({
      name: 'IS INFINITE WHERE A BLOCK NO RANGE REACHES STANDS NOWHERE IN THE PAGE',
      fn: async () => {
        const page = ['猫', '睡',];
        expect(boundOf({
          page,
          index: indexOfWords({ page, },),
          segments: [{ reached: false, words: ['鱼',], blocks: 0, },],
        },),).toBe(Number.POSITIVE_INFINITY,);
      },
    },),
  ],
},);
