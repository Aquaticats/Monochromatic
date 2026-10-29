import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  forceDashSeparatorsToOwnLine,
  splitOnDashOnlyLines,
} from './stak-simulation.ts';

await describe({
  name: forceDashSeparatorsToOwnLine.name,
  children: [
    it({
      name: 'returns the empty string unchanged',
      fn: async () => {
        expect(forceDashSeparatorsToOwnLine('',),).toBe('',);
      },
    },),
    it({
      name: 'inserts a newline before a separator glued to preceding content',
      fn: async () => {
        expect(forceDashSeparatorsToOwnLine('a---b',),).toBe('a\n---b',);
      },
    },),
    it({
      name: 'leaves a separator already at the start of a line unchanged',
      fn: async () => {
        expect(forceDashSeparatorsToOwnLine('a\n---b',),).toBe('a\n---b',);
      },
    },),
    it({
      name: 'leaves a separator at start-of-string unchanged',
      fn: async () => {
        expect(forceDashSeparatorsToOwnLine('---a',),).toBe('---a',);
      },
    },),
    it({
      name: 'splits a glued run of six dashes into two separators',
      fn: async () => {
        expect(forceDashSeparatorsToOwnLine('------',),).toBe('---\n---',);
      },
    },),
    it({
      name: 'normalizes multiple glued separators',
      fn: async () => {
        expect(forceDashSeparatorsToOwnLine('a---b---c',),).toBe('a\n---b\n---c',);
      },
    },),
    it({
      name: 'inserts newlines for a long run of separators in a single linear pass',
      fn: async () => {
        const separatorCount = 50_000;
        const input = '---'.repeat(separatorCount,);
        const expected = Array.from({ length: separatorCount, }, function dashes(): string {
          return '---';
        },).join('\n',);
        expect(forceDashSeparatorsToOwnLine(input,),).toBe(expected,);
      },
    },),
  ],
},);

await describe({
  name: splitOnDashOnlyLines.name,
  children: [
    it({
      name: 'returns one empty section for the empty string',
      fn: async () => {
        expect(splitOnDashOnlyLines('',),).toEqual(['',],);
      },
    },),
    it({
      name: 'splits on a line that is exactly three dashes',
      fn: async () => {
        expect(splitOnDashOnlyLines('a\n---\nb',),).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'does not split on a line with more than three dashes',
      fn: async () => {
        expect(splitOnDashOnlyLines('a\n----\nb',),).toEqual(['a\n----\nb',],);
      },
    },),
    it({
      name: 'emits an empty section between consecutive separators',
      fn: async () => {
        expect(splitOnDashOnlyLines('a\n---\n---\nb',),).toEqual(['a', '', 'b',],);
      },
    },),
    it({
      name: 'emits a leading empty section for a leading separator',
      fn: async () => {
        expect(splitOnDashOnlyLines('---\na',),).toEqual(['', 'a',],);
      },
    },),
    it({
      name: 'emits a trailing empty section for a trailing separator',
      fn: async () => {
        expect(splitOnDashOnlyLines('a\n---',),).toEqual(['a', '',],);
      },
    },),
    it({
      name: 'preserves interior newlines within a section',
      fn: async () => {
        expect(splitOnDashOnlyLines('a\nb\n---\nc\nd',),).toEqual(['a\nb', 'c\nd',],);
      },
    },),
    it({
      name: 'splits thousands of sections without an O(n^2) accumulator',
      fn: async () => {
        const sectionCount = 5_000;
        const input = Array.from({ length: sectionCount, }, function section(): string {
          return 'x';
        },).join('\n---\n',);
        const result = splitOnDashOnlyLines(input,);
        expect(result.length,).toBe(sectionCount,);
        expect(result.every(function isX(section,): boolean {
          return section === 'x';
        },),).toBe(true,);
      },
    },),
  ],
},);
