/**
 Tests for choosing a prompt fence no enclosed text can reproduce.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  longestFenceRun,
  selectFence,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: longestFenceRun.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'measures the longest unbroken run rather than the total count, '
            + 'since only a contiguous run can act as a delimiter',
          fn: async () => {
            expect(longestFenceRun('= a == b ==== c ==',),).toBe(4,);
            expect(longestFenceRun('no fence characters here',),).toBe(0,);
            expect(longestFenceRun('',),).toBe(0,);
          },
        },),

        it({
          name: 'counts a run that ends the text, which a scan looking only for a '
            + 'following non-fence character would miss',
          fn: async () => {
            expect(longestFenceRun('The cat naps.\n======',),).toBe(6,);
          },
        },),
      ],
    },),

    describe({
      name: selectFence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns the five-character minimum when nothing enclosed competes',
          fn: async () => {
            expect(
              selectFence({
                texts: [
                  'The cat naps.',
                  '猫在睡觉。',
                ],
              },),
            ).toBe('=====',);
          },
        },),

        it({
          name: 'outgrows the longest run across ALL enclosed texts, not just the '
            + 'first, because one prompt fences several independent strings',
          fn: async () => {
            expect(
              selectFence({
                texts: [
                  'The cat naps.',
                  'A heading\n========\nunderlined',
                  'She wakes.',
                ],
              },),
            ).toBe('=========',);
          },
        },),

        it({
          name: 'stays strictly longer than the content run, so the content can '
            + 'never produce a line the sheet would read as its own delimiter',
          fn: async () => {
            /** Text carrying a run exactly at the minimum fence width. */
            const atMinimum = '=====';

            /** Fence chosen against it. */
            const fence = selectFence({ texts: [atMinimum,], },);
            expect(fence.length,).toBeGreaterThan(atMinimum.length,);
            expect(fence,).toBe('======',);
          },
        },),

        it({
          name: 'handles an empty text list, since a prompt may fence nothing at '
            + 'all before any region exists',
          fn: async () => {
            expect(selectFence({ texts: [], },),).toBe('=====',);
          },
        },),
      ],
    },),
  ],
},);
