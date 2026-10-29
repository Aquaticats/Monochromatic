/**
 Tests the page assembly section: its parsing across generations and the
 per-slice override reading.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  NO_PAGE_ASSEMBLY,
  pageAssemblyOverrideAt,
  parsePageAssembly,
} from '../../dist/final/node/index.mjs';

/**
 A recorded section with one trim and one withdrawal.
 */
const RECORDED = {
  trimmed: [{ sliceIndex: 14, replacementText: '[^1]: The note.', },],
  withdrawn: [7,],
  findings: ['assembly-footnote-trimmed orphan-definition gfm 2 (slice 14)',],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parsePageAssembly.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads an absent section on an older generation as a guard that touched nothing',
          fn: async () => {
            expect(parsePageAssembly({ value: undefined, path: 'x.pageAssembly', required: false, },),)
              .toEqual(NO_PAGE_ASSEMBLY,);
          },
        },),
        it({
          name: 'REFUSES an absent section on a generation that writes it',
          fn: async () => {
            expect(() => parsePageAssembly({ value: undefined, path: 'x.pageAssembly', required: true, },),)
              .toThrow(ArtifactParseError,);
          },
        },),
        it({
          name: 'reads a recorded section back as written and refuses a stray key',
          fn: async () => {
            expect(parsePageAssembly({ value: RECORDED, path: 'x.pageAssembly', required: true, },),)
              .toEqual(RECORDED,);
            expect(() => parsePageAssembly({
              value: { ...RECORDED, extra: true, },
              path: 'x.pageAssembly',
              required: true,
            },),).toThrow(ArtifactParseError,);
          },
        },),
        it({
          name: 'REFUSES TWO TRIMMED ROWS NAMING ONE SLICE, which the page guard cannot write and whose first the '
            + 'override reading would take without a word, and READS two rows naming different slices',
          fn: async () => {
            /**
             What reading a section whose two rows both name slice 14 throws.
             */
            const refusal = caught(function readRepeat(): unknown {
              return parsePageAssembly({
                value: {
                  trimmed: [
                    { sliceIndex: 14, replacementText: '[^1]: The note.', },
                    { sliceIndex: 14, replacementText: '[^1]: Another note.', },
                  ],
                  withdrawn: [],
                  findings: [],
                },
                path: 'x.pageAssembly',
                required: true,
              },);
            },);
            expect(refusal,).toBeInstanceOf(ArtifactParseError,);
            expect(String(refusal,),).toBe(
              'ArtifactParseError: artifact parse failed at x.pageAssembly.trimmed: expected one row per slice; '
                + 'slice 14 appears more than once.',
            );
            /**
             The same two texts under two slices, which the guard does write.
             */
            const apart = {
              trimmed: [
                { sliceIndex: 14, replacementText: '[^1]: The note.', },
                { sliceIndex: 15, replacementText: '[^1]: Another note.', },
              ],
              withdrawn: [],
              findings: [],
            };
            expect(parsePageAssembly({ value: apart, path: 'x.pageAssembly', required: true, },),).toEqual(apart,);
          },
        },),
      ],
    },),

    describe({
      name: pageAssemblyOverrideAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names a trimmed slice with its text, a withdrawn slice, and an untouched one',
          fn: async () => {
            expect(pageAssemblyOverrideAt({ pageAssembly: RECORDED, sliceIndex: 14, },),)
              .toEqual({ kind: 'trimmed', text: '[^1]: The note.', },);
            expect(pageAssemblyOverrideAt({ pageAssembly: RECORDED, sliceIndex: 7, },),)
              .toEqual({ kind: 'withdrawn', },);
            expect(pageAssemblyOverrideAt({ pageAssembly: RECORDED, sliceIndex: 0, },),)
              .toEqual({ kind: 'untouched', },);
          },
        },),
        it({
          name: 'READS A WITHDRAWN SLICE THAT ALSO CARRIES A ROW as that row: the lane\'s wording was taken back and '
            + 'the page passes rewrote the archive text standing there (ledger K5)',
          fn: async () => {
            expect(pageAssemblyOverrideAt({
              pageAssembly: {
                trimmed: [{
                  sliceIndex: 7,
                  replacementText: 'The cat napped on April 29, 2024.',
                },],
                withdrawn: [7,],
                findings: [],
              },
              sliceIndex: 7,
            },),).toEqual({
              kind: 'trimmed',
              text: 'The cat napped on April 29, 2024.',
            },);
          },
        },),
      ],
    },),
  ],
},);
