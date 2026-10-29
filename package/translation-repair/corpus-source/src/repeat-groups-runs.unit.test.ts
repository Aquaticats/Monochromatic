/**
 Tests for the finder of maximal runs behind the repeat groups: runs of twelve
 or more elements that stand at two or more places of a wording stream.
 Fixtures are cat-themed invention only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BREAK,
  maximalRunsOf,
} from '../dist/final/node/index.mjs';

/**
 Twelve elements, the shortest run the finder reports.
 */
const RUN_TWELVE = Array.from('猫咪在窗台上晒太阳打盹儿');

/**
 Another twelve elements, none of them in the first run's order.
 */
const RUN_OTHER = Array.from('尾巴一卷一卷地扫来扫去啊');

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: maximalRunsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Returns no run whose occurrences overlap into fewer than two places',
          fn: async () => {
            /** Thirteen of one character: two starts one apart overlap, so one place is left once the overlap is dropped. */
            const stream = Array.from('猫'.repeat(13,),);

            expect(maximalRunsOf({ stream, },),).toEqual([],);
          },
        },),

        it({
          name: 'Finds a run of exactly twelve elements at two places',
          fn: async () => {
            expect(maximalRunsOf({ stream: [...RUN_TWELVE, BREAK, ...RUN_TWELVE,], },),).toEqual([
              {
                length: 12,
                starts: [0, 13,],
              },
            ],);
          },
        },),

        it({
          name: 'Finds no run of eleven elements',
          fn: async () => {
            /** Eleven elements of the twelve. */
            const eleven = RUN_TWELVE.slice(0, 11,);

            expect(maximalRunsOf({ stream: [...eleven, BREAK, ...eleven,], },),).toEqual([],);
          },
        },),

        it({
          name: 'Returns no run for a stream shorter than twelve elements or for an empty one',
          fn: async () => {
            expect(maximalRunsOf({ stream: RUN_TWELVE.slice(0, 11,), },),).toEqual([],);
            expect(maximalRunsOf({ stream: [], },),).toEqual([],);
          },
        },),

        it({
          name: 'Returns no run from a stream of breaks alone',
          fn: async () => {
            expect(maximalRunsOf({ stream: Array.from({ length: 30, }, () => BREAK,), },),).toEqual([],);
          },
        },),

        it({
          name: 'Extends a run to the left and to the right while every occurrence agrees',
          fn: async () => {
            expect(maximalRunsOf({
              stream: ['甲', ...RUN_TWELVE, '乙', BREAK, '甲', ...RUN_TWELVE, '乙',],
            },),).toEqual([
              {
                length: 14,
                starts: [0, 15,],
              },
            ],);
          },
        },),

        it({
          name: 'Never lets a run cross a break, though the stretches either side of it repeat',
          fn: async () => {
            /** First half of the twelve. */
            const head = RUN_TWELVE.slice(0, 6,);
            /** Second half of the twelve. */
            const tail = RUN_TWELVE.slice(6,);

            expect(maximalRunsOf({ stream: [...head, BREAK, ...tail, BREAK, ...head, BREAK, ...tail,], },),).toEqual([],);
          },
        },),

        it({
          name: 'Lists the longer run the last two occurrences share beside the shorter run all three share',
          fn: async () => {
            expect(maximalRunsOf({
              stream: [...RUN_TWELVE, '甲', BREAK, ...RUN_TWELVE, '甲', BREAK, ...RUN_TWELVE, '乙',],
            },),).toEqual([
              {
                length: 12,
                starts: [0, 14, 28,],
              },
              {
                length: 13,
                starts: [0, 14,],
              },
            ],);
          },
        },),

        it({
          name: 'Keeps each occurrence from overlapping the one before it',
          fn: async () => {
            /** Thirteen of one character in each of two stretches, so every shift of twelve matches inside one. */
            const thirteen = Array.from('猫'.repeat(13,),);

            expect(maximalRunsOf({ stream: [...thirteen, BREAK, ...thirteen,], },),).toEqual([
              {
                length: 12,
                starts: [0, 14,],
              },
            ],);
          },
        },),

        it({
          name: 'Returns a run once though each of its windows finds it',
          fn: async () => {
            /** Fifteen elements, three more than the shortest run. */
            const fifteen = [...RUN_TWELVE, ...Array.from('呀真好'),];

            expect(maximalRunsOf({ stream: [...fifteen, BREAK, ...fifteen,], },),).toEqual([
              {
                length: 15,
                starts: [0, 16,],
              },
            ],);
          },
        },),

        it({
          name: 'Returns a long copied stretch once, at its whole length',
          fn: async () => {
            /** Eighty distinct ideographs. */
            const stretch = Array.from({ length: 80, }, (_, index,) => String.fromCodePoint(0x4E_00 + (index * 37),),);

            expect(maximalRunsOf({ stream: [...stretch, BREAK, ...stretch,], },),).toEqual([
              {
                length: 80,
                starts: [0, 81,],
              },
            ],);
          },
        },),

        it({
          name: 'Counts an element beyond the first plane as one element of a run',
          fn: async () => {
            /** Eleven plain elements and one beyond the first plane. */
            const run = [...RUN_TWELVE.slice(0, 11,), '😀',];

            expect(maximalRunsOf({ stream: [...run, BREAK, ...run,], },),).toEqual([
              {
                length: 12,
                starts: [0, 13,],
              },
            ],);
          },
        },),

        it({
          name: 'Lists runs in the order their first windows appear',
          fn: async () => {
            expect(maximalRunsOf({
              stream: [...RUN_TWELVE, BREAK, ...RUN_OTHER, BREAK, ...RUN_TWELVE, BREAK, ...RUN_OTHER,],
            },),).toEqual([
              {
                length: 12,
                starts: [0, 26,],
              },
              {
                length: 12,
                starts: [13, 39,],
              },
            ],);
          },
        },),
      ],
    },),

  ],
},);
