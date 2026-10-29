/**
 Tests for the page text every page-assembly pass reads per slice: a lane's
 replacement where one exists, else the archive's own text, and a refusal
 for a slice the map was never built from (ledger B83). Cat-themed invention
 throughout; no corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  pageTextBySlice,
  pageTextOf,
  SliceNotOnPageError,
} from '../../dist/final/node/index.mjs';

/**
 One slice whose archive text is the given text.

 @param sliceIndex - where the slice stands

 @param target - archive text of the slice

 @returns Prepared pair

 @example
 ```ts
 const slice = archived({ sliceIndex: 0, target: 'It sleeps.', },);
 ```
 */
function archived(
  {
    sliceIndex,
    target,
  }: {
    readonly sliceIndex: number;
    readonly target: string;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 3,
      text: '猫睡了',
    },
    target: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: target.length,
      text: target,
    },
  };
}

/**
 Page text of two slices, the second replaced by a lane.
 */
const PAGE_TEXT = pageTextBySlice({
  slices: [
    archived({
      sliceIndex: 0,
      target: 'It sleeps.',
    },),
    archived({
      sliceIndex: 1,
      target: 'It wakes.',
    },),
  ],
  replacements: [{
    sliceIndex: 1,
    replacementText: 'It wakes up.',
  },],
},);

await describe({
  name: 'page text per slice',
  children: [
    describe({
      name: pageTextOf.name,
      children: [
        it({
          name: 'READS THE LANE\'S REPLACEMENT where one exists, else the archive\'s own text',
          fn: async () => {
            expect([
              0,
              1,
            ].map(function textOf(sliceIndex,): string {
              return pageTextOf({
                pageText: PAGE_TEXT,
                sliceIndex,
              },);
            },),).toEqual([
              'It sleeps.',
              'It wakes up.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A SLICE THE MAP WAS NEVER BUILT FROM rather than reading it as empty text, which once '
            + 'shipped a section the page never replaced as nothing (ledger B83)',
          fn: async () => {
            /**
             What reading a slice the map lacks threw.
             */
            const refusal = caught(function readsMissing(): void {
              pageTextOf({
                pageText: PAGE_TEXT,
                sliceIndex: 2,
              },);
            },);
            expect(refusal,).toBeInstanceOf(SliceNotOnPageError,);
            expect((refusal as SliceNotOnPageError).sliceIndex,).toBe(2,);
            expect((refusal as Error).message,).toBe(
              'slice 2 is not among the slices this page text was built from, so a page-assembly pass asked for a '
                + 'slice of some other list; reading it as empty text would ship that section as nothing',
            );
          },
        },),
      ],
    },),
  ],
},);
