import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { splitOnDashLines, } from './sudoku-output.ts';

await describe({
  name: splitOnDashLines.name,
  children: [
    it({
      name: 'returns one empty section for the empty string',
      fn: async () => {
        expect(splitOnDashLines('',),).toEqual(['',],);
      },
    },),
    it({
      name: 'returns a single section when there is no dash line',
      fn: async () => {
        expect(splitOnDashLines('a\nb',),).toEqual(['a\nb',],);
      },
    },),
    it({
      name: 'splits on a dash-only line',
      fn: async () => {
        expect(splitOnDashLines('a\n---\nb',),).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'emits an empty section between consecutive separators (no collapse)',
      fn: async () => {
        expect(splitOnDashLines('a\n---\n---\nb',),).toEqual(['a', '', 'b',],);
      },
    },),
    it({
      name: 'emits a leading empty section for a leading separator',
      fn: async () => {
        expect(splitOnDashLines('---\na',),).toEqual(['', 'a',],);
      },
    },),
    it({
      name: 'emits a trailing empty section for a trailing separator',
      fn: async () => {
        expect(splitOnDashLines('a\n---',),).toEqual(['a', '',],);
      },
    },),
    it({
      name: 'treats a run of dashes as a single separator',
      fn: async () => {
        expect(splitOnDashLines('a\n-----\nb',),).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'does not treat a dash line with other characters as a separator',
      fn: async () => {
        expect(splitOnDashLines('a\n-x-\nb',),).toEqual(['a\n-x-\nb',],);
      },
    },),
    it({
      name: 'preserves interior newlines within a section',
      fn: async () => {
        expect(splitOnDashLines('a\nb\n---\nc\nd',),).toEqual(['a\nb', 'c\nd',],);
      },
    },),
    it({
      name: 'splits thousands of sections without an O(n^2) accumulator',
      fn: async () => {
        const sectionCount = 5_000;
        const input = Array.from({ length: sectionCount, }, function section(): string {
          return 'x';
        },).join('\n---\n',);
        const result = splitOnDashLines(input,);
        expect(result.length,).toBe(sectionCount,);
        expect(result.every(function isX(section,): boolean {
          return section === 'x';
        },),).toBe(true,);
      },
    },),
  ],
},);
