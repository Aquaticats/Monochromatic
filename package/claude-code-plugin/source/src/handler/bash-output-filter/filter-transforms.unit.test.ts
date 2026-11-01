import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { MAX_REPEATED_CHARS, } from './filter-patterns.ts';
import {
  COLLAPSE_THRESHOLD,
  collapseRepeatedChars,
} from './filter-transforms.ts';

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
          name: 'leaves a run shorter than the threshold verbatim',
          fn: async () => {
            const shortRun = '='.repeat(COLLAPSE_THRESHOLD - 1,);
            expect(collapseRepeatedChars(shortRun,),).toBe(shortRun,);
          },
        },),
        it({
          name: 'collapses a run at the threshold',
          fn: async () => {
            expect(collapseRepeatedChars('='.repeat(COLLAPSE_THRESHOLD,),),).toBe(
              `${'='.repeat(MAX_REPEATED_CHARS,)} (x${String(COLLAPSE_THRESHOLD,)} repeated characters)`,
            );
          },
        },),
        it({
          name: 'collapses a long run and preserves surrounding text',
          fn: async () => {
            expect(
              collapseRepeatedChars(`ab${'='.repeat(COLLAPSE_THRESHOLD,)}cd`,),
            ).toBe(
              `ab${'='.repeat(MAX_REPEATED_CHARS,)} (x${String(COLLAPSE_THRESHOLD,)} repeated characters)cd`,
            );
          },
        },),
        it({
          name: 'collapses multiple distinct runs independently',
          fn: async () => {
            const count = `(x${String(COLLAPSE_THRESHOLD,)} repeated characters)`;
            expect(
              collapseRepeatedChars(
                `${'+'.repeat(COLLAPSE_THRESHOLD,)} and ${'*'.repeat(COLLAPSE_THRESHOLD,)}`,
              ),
            ).toBe(
              `${'+'.repeat(MAX_REPEATED_CHARS,)} ${count} and ${'*'.repeat(MAX_REPEATED_CHARS,)} ${count}`,
            );
          },
        },),
        it({
          name: 'treats adjacent runs of different characters separately',
          fn: async () => {
            // Both runs stay below the threshold, so both are emitted verbatim.
            const shortRun = COLLAPSE_THRESHOLD - 1;
            expect(
              collapseRepeatedChars(`${'='.repeat(shortRun,)}${'-'.repeat(shortRun,)}`,),
            ).toBe(`${'='.repeat(shortRun,)}${'-'.repeat(shortRun,)}`,);
          },
        },),
        it({
          name: 'collapses a very long run without overflowing (linear pass)',
          fn: async () => {
            expect(
              collapseRepeatedChars('='.repeat(1_000_000,),),
            ).toBe(
              `${'='.repeat(MAX_REPEATED_CHARS,)} (x1000000 repeated characters)`,
            );
          },
        },),
      ],
    },),
  ],
},);
