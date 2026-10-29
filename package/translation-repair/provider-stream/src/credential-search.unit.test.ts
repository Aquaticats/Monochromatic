/**
 Tests for the one-pass search of several needles in a text, which the
 credential mask runs over every reply.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { findNeedles, } from '../dist/final/node/index.mjs';

await describe({
  name: findNeedles.name,
  children: [
    it({
      name: 'FINDS NOTHING when no needle is listed, and nothing in a text that holds none',
      fn: async () => {
        expect(findNeedles({ text: 'the cat napped', needles: [], },),).toEqual([],);
        expect(findNeedles({ text: 'the cat napped', needles: ['whisker', 'purr',], },),).toEqual([],);
      },
    },),
    it({
      name: 'FINDS EVERY NEEDLE with its position in the list, in the order the ends fall',
      fn: async () => {
        expect(findNeedles({ text: 'xxabxx', needles: ['ab', 'xa',], },),).toEqual([
          { needle: 1, start: 1, end: 3, },
          { needle: 0, start: 2, end: 4, },
        ],);
      },
    },),
    it({
      name: 'FINDS A NEEDLE THAT ENDS INSIDE A LONGER ONE and one that is the longer one\'s suffix',
      fn: async () => {
        expect(findNeedles({ text: 'ushers', needles: ['he', 'she', 'hers', 'his',], },),).toEqual([
          { needle: 1, start: 1, end: 4, },
          { needle: 0, start: 2, end: 4, },
          { needle: 2, start: 2, end: 6, },
        ],);
      },
    },),
    it({
      name: 'FINDS OVERLAPPING COPIES of one needle and a needle after a near miss',
      fn: async () => {
        expect(findNeedles({ text: 'aaaa', needles: ['aa',], },),).toEqual([
          { needle: 0, start: 0, end: 2, },
          { needle: 0, start: 1, end: 3, },
          { needle: 0, start: 2, end: 4, },
        ],);
        expect(findNeedles({ text: 'ababac', needles: ['abac',], },),).toEqual([{ needle: 0, start: 2, end: 6, },],);
      },
    },),
  ],
},);
