import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { collapseRepeatedChars, } from './filter-transforms.ts';

await describe({
  name: 'filter-transforms',
  children: [
    describe({
      name: collapseRepeatedChars.name,
      children: [
        it({
          name: 'returns the empty string unchanged',
          fn: async () => {
            expect(collapseRepeatedChars('',),).toBe('',);
          },
        },),
        it({
          name: 'leaves word characters untouched even when repeated',
          fn: async () => {
            expect(collapseRepeatedChars('aaaaaaaaaaaa',),).toBe('aaaaaaaaaaaa',);
          },
        },),
        it({
          name: 'leaves whitespace runs untouched (not collapse candidates)',
          fn: async () => {
            expect(collapseRepeatedChars('          ',),).toBe('          ',);
          },
        },),
        it({
          name: 'leaves a run shorter than the threshold (9 chars) verbatim',
          fn: async () => {
            expect(collapseRepeatedChars('=========',),).toBe('=========',);
          },
        },),
        it({
          name: 'collapses a run at the threshold (10 chars)',
          fn: async () => {
            expect(collapseRepeatedChars('==========',),).toBe(
              '=== (x10 repeated characters)',
            );
          },
        },),
        it({
          name: 'collapses a long run and preserves surrounding text',
          fn: async () => {
            expect(collapseRepeatedChars('ab==========cd',),).toBe(
              'ab=== (x10 repeated characters)cd',
            );
          },
        },),
        it({
          name: 'collapses multiple distinct runs independently',
          fn: async () => {
            expect(collapseRepeatedChars('++++++++++ and **********',),).toBe(
              '+++ (x10 repeated characters) and *** (x10 repeated characters)',
            );
          },
        },),
        it({
          name: 'treats adjacent runs of different characters separately',
          fn: async () => {
            // 4 of each, both below threshold, both emitted verbatim
            expect(collapseRepeatedChars('====----',),).toBe('====----',);
          },
        },),
        it({
          name: 'collapses a very long run without overflowing (linear pass)',
          fn: async () => {
            expect(
              collapseRepeatedChars('='.repeat(1_000_000,),),
            ).toBe(
              '=== (x1000000 repeated characters)',
            );
          },
        },),
      ],
    },),
  ],
},);
