import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  collapseExcessNewlines,
  collapseHorizontalRuns,
} from './css-mixin-verify.ts';

await describe({
  name: collapseHorizontalRuns.name,
  children: [
    it({
      name: 'returns the empty string unchanged',
      fn: async () => {
        expect(collapseHorizontalRuns('',),).toBe('',);
      },
    },),
    it({
      name: 'collapses a run of spaces to a single space',
      fn: async () => {
        expect(collapseHorizontalRuns('a    b',),).toBe('a b',);
      },
    },),
    it({
      name: 'collapses a mixed space-and-tab run to a single space',
      fn: async () => {
        expect(collapseHorizontalRuns('a \t \t b',),).toBe('a b',);
      },
    },),
    it({
      name: 'leaves a single space unchanged',
      fn: async () => {
        expect(collapseHorizontalRuns('a b',),).toBe('a b',);
      },
    },),
    it({
      name: 'collapses an all-whitespace run to a single space',
      fn: async () => {
        expect(collapseHorizontalRuns('     ',),).toBe(' ',);
      },
    },),
    it({
      name: 'passes newlines through and collapses the spaces around them',
      fn: async () => {
        expect(collapseHorizontalRuns('a  \n  b',),).toBe('a \n b',);
      },
    },),
    it({
      name: 'collapses a long horizontal run in a single linear pass',
      fn: async () => {
        const runLength = 100_000;
        expect(collapseHorizontalRuns(`a${' '.repeat(runLength,)}b`,),).toBe('a b',);
      },
    },),
  ],
},);

await describe({
  name: collapseExcessNewlines.name,
  children: [
    it({
      name: 'returns the empty string unchanged',
      fn: async () => {
        expect(collapseExcessNewlines('',),).toBe('',);
      },
    },),
    it({
      name: 'leaves a single newline unchanged',
      fn: async () => {
        expect(collapseExcessNewlines('a\nb',),).toBe('a\nb',);
      },
    },),
    it({
      name: 'leaves exactly two newlines unchanged',
      fn: async () => {
        expect(collapseExcessNewlines('a\n\nb',),).toBe('a\n\nb',);
      },
    },),
    it({
      name: 'collapses three newlines down to two',
      fn: async () => {
        expect(collapseExcessNewlines('a\n\n\nb',),).toBe('a\n\nb',);
      },
    },),
    it({
      name: 'collapses a long newline run down to two',
      fn: async () => {
        expect(collapseExcessNewlines('a\n\n\n\n\nb',),).toBe('a\n\nb',);
      },
    },),
    it({
      name: 'leaves horizontal whitespace untouched',
      fn: async () => {
        expect(collapseExcessNewlines('a   b',),).toBe('a   b',);
      },
    },),
    it({
      name: 'collapses a long newline run in a single linear pass',
      fn: async () => {
        const runLength = 100_000;
        expect(
          collapseExcessNewlines('\n'.repeat(runLength,),),
        ).toBe('\n\n',);
      },
    },),
  ],
},);
