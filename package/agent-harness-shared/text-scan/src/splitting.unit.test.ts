import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { splitWhitespace, } from './index.ts';

await describe({
  name: splitWhitespace.name,
  children: [
    it({
      name: 'splits on single spaces',
      fn: async () => {
        expect(splitWhitespace('a b c',),).toEqual(['a', 'b', 'c',],);
      },
    },),
    it({
      name: 'collapses runs of whitespace and trims edges',
      fn: async () => {
        expect(splitWhitespace('  a\tb\nc  ',),).toEqual(['a', 'b', 'c',],);
      },
    },),
    it({
      name: 'returns an empty array for the empty string',
      fn: async () => {
        expect(splitWhitespace('',),).toEqual([],);
      },
    },),
    it({
      name: 'returns an empty array for whitespace-only input',
      fn: async () => {
        expect(splitWhitespace('   \t\n  ',),).toEqual([],);
      },
    },),
    it({
      name: 'tokenises a very large input in linear time',
      fn: async () => {
        /**
         Token count large enough that an array-spread accumulator would stall.
         */
        const count = 200_000;
        /**
         Result of tokenising `count` single-character tokens separated by spaces.
         */
        const result = splitWhitespace('a '.repeat(count,),);
        expect(result.length,).toBe(count,);
        expect(result[0],).toBe('a',);
        expect(result.at(-1,),).toBe('a',);
      },
    },),
  ],
},);
