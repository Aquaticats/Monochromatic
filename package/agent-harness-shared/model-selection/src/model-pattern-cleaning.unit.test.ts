/**
 Unit tests for the shared model-pattern cleanup.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { cleanModelPatterns, } from '../dist/final/node/index.mjs';

await describe({
  name: cleanModelPatterns.name,
  children: [
    it({
      name: 'trims every pattern and drops empty and whitespace-only entries',
      fn: async function testTrimAndDrop() {
        expect(cleanModelPatterns([
          '  cheap/*  ',
          '',
          '   ',
          '\texpensive/reviewer\n',
        ],),).toEqual([
          'cheap/*',
          'expensive/reviewer',
        ],);
      },
    },),
    it({
      name: 'keeps order and repeated patterns unchanged',
      fn: async function testOrderAndRepeats() {
        expect(cleanModelPatterns([
          'b/*',
          'a/*',
          'b/*',
        ],),).toEqual([
          'b/*',
          'a/*',
          'b/*',
        ],);
      },
    },),
    it({
      name: 'returns an empty list for an empty input',
      fn: async function testEmptyInput() {
        expect(cleanModelPatterns([],),).toEqual([],);
      },
    },),
  ],
},);
