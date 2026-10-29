import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  splitOnBlankLines,
  stripAllWhitespace,
} from './sudoku-grid.ts';

await describe({
  name: stripAllWhitespace.name,
  children: [
    it({
      name: 'returns the empty string unchanged',
      fn: async () => {
        expect(stripAllWhitespace('',),).toBe('',);
      },
    },),
    it({
      name: 'collapses an all-whitespace string to empty',
      fn: async () => {
        expect(stripAllWhitespace(' \t\n\r\f\v',),).toBe('',);
      },
    },),
    it({
      name: 'drops every interior whitespace kind',
      fn: async () => {
        expect(stripAllWhitespace('a b\tc\nd\re\ff\vg',),).toBe('abcdefg',);
      },
    },),
    it({
      name: 'leaves a string without whitespace unchanged',
      fn: async () => {
        expect(stripAllWhitespace('534678912',),).toBe('534678912',);
      },
    },),
    it({
      name: 'strips leading and trailing whitespace',
      fn: async () => {
        expect(stripAllWhitespace('   abc   ',),).toBe('abc',);
      },
    },),
    it({
      name: 'strips a long whitespace run in a single linear pass',
      fn: async () => {
        const runLength = 100_000;
        expect(stripAllWhitespace(`a${' '.repeat(runLength,)}b`,),).toBe('ab',);
      },
    },),
  ],
},);

await describe({
  name: splitOnBlankLines.name,
  children: [
    it({
      name: 'returns no blocks for the empty string',
      fn: async () => {
        expect(splitOnBlankLines('',),).toEqual([],);
      },
    },),
    it({
      name: 'returns a single block when there is no blank line',
      fn: async () => {
        expect(splitOnBlankLines('a\nb',),).toEqual(['a\nb',],);
      },
    },),
    it({
      name: 'splits two blocks on a blank line',
      fn: async () => {
        expect(splitOnBlankLines('a\n\nb',),).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'collapses consecutive blank lines into one separator',
      fn: async () => {
        expect(splitOnBlankLines('a\n\n\nb',),).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'treats a whitespace-only line as a separator',
      fn: async () => {
        expect(splitOnBlankLines('a\n  \nb',),).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'drops a leading blank line',
      fn: async () => {
        expect(splitOnBlankLines('\na',),).toEqual(['a',],);
      },
    },),
    it({
      name: 'drops a trailing blank line',
      fn: async () => {
        expect(splitOnBlankLines('a\n',),).toEqual(['a',],);
      },
    },),
    it({
      name: 'preserves interior newlines within a block',
      fn: async () => {
        expect(splitOnBlankLines('a\nb\n\nc\nd',),).toEqual(['a\nb', 'c\nd',],);
      },
    },),
    it({
      name: 'splits thousands of blocks without an O(n^2) accumulator',
      fn: async () => {
        const blockCount = 5_000;
        const input = Array.from({ length: blockCount, }, function block(): string {
          return 'x';
        },).join('\n\n',);
        const result = splitOnBlankLines(input,);
        expect(result.length,).toBe(blockCount,);
        expect(result.every(function isX(block,): boolean {
          return block === 'x';
        },),).toBe(true,);
      },
    },),
  ],
},);
