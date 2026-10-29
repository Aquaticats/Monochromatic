/**
 Tests for how the width probe reads which half of the sample it spends and
 takes that half out of the sample.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  halfOfSample,
  readWidthDraw,
} from '../../dist/final/node/index.mjs';
import { benchSliceOf, } from './editor-calibrate-rounds.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';

/**
 Five slices of one invented entry, in sample order.
 */
const SAMPLE = [0, 1, 2, 3, 4,].map(function sliceAt(index,) {
  return benchSliceOf({ entryId: 'mittens', index, },);
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readWidthDraw.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS draw A when the line names a count and no draw',
          fn: async () => {
            expect(readWidthDraw({ line: lineOf({ command: 'editor-width-probe', typed: ['12',], },), },),).toBe('a',);
          },
        },),

        it({
          name: 'READS draw A when the line names nothing at all',
          fn: async () => {
            expect(readWidthDraw({ line: lineOf({ command: 'editor-width-probe', typed: [], },), },),).toBe('a',);
          },
        },),

        it({
          name: 'READS draw B when the line names it after the count',
          fn: async () => {
            expect(readWidthDraw({ line: lineOf({ command: 'editor-width-probe', typed: ['12', 'b',], },), },),)
              .toBe('b',);
          },
        },),

        it({
          name: 'REFUSES a draw that is neither half, naming it and the held-back half it would have burned',
          fn: async () => {
            /**
             What reading a draw named `c` refused with.
             */
            const refusal = caught(function drawC(): unknown {
              return readWidthDraw({ line: lineOf({ command: 'editor-width-probe', typed: ['12', 'c',], },), },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: editor width probe refused: draw must be \'a\' or \'b\', not \'c\'; spending '
                + 'draw A under another name would burn the held-back half of the sample without saying so',
            );
          },
        },),

        it({
          name: 'REFUSES an empty draw name rather than reading it as draw A',
          fn: async () => {
            /**
             What reading a draw named by an empty argument refused with.
             */
            const refusal = caught(function drawEmpty(): unknown {
              return readWidthDraw({ line: lineOf({ command: 'editor-width-probe', typed: ['12', '',], },), },);
            },);

            expect(String(refusal,),).toBe(
              'StatedRefusalError: editor width probe refused: draw must be \'a\' or \'b\', not \'\'; spending '
                + 'draw A under another name would burn the held-back half of the sample without saying so',
            );
          },
        },),
      ],
    },),

    describe({
      name: halfOfSample.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES the first, third and fifth slices for draw A',
          fn: async () => {
            expect(halfOfSample({ sample: SAMPLE, draw: 'a', },),).toEqual([SAMPLE[0], SAMPLE[2], SAMPLE[4],],);
          },
        },),

        it({
          name: 'TAKES the second and fourth slices for draw B, the half draw A leaves',
          fn: async () => {
            expect(halfOfSample({ sample: SAMPLE, draw: 'b', },),).toEqual([SAMPLE[1], SAMPLE[3],],);
          },
        },),

        it({
          name: 'TAKES nothing for draw B out of a sample of one, and the one slice for draw A',
          fn: async () => {
            /**
             The only slice of the smallest sample.
             */
            const only = SAMPLE.slice(0, 1,);

            expect(halfOfSample({ sample: only, draw: 'b', },),).toEqual([],);
            expect(halfOfSample({ sample: only, draw: 'a', },),).toEqual(only,);
          },
        },),
      ],
    },),
  ],
},);
