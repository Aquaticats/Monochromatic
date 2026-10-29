/**
 Tests for choosing a code fence no enclosed text can close.
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
  fenceForMarkdown,
  longestBacktickRun,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: longestBacktickRun.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports zero when the text carries no backtick, and the longest '
            + 'unbroken run when it does',
          fn: async () => {
            expect(longestBacktickRun('The cat naps.',),).toBe(0,);
            expect(longestBacktickRun('a ` b `` c ``` d',),).toBe(3,);
            expect(longestBacktickRun('``````',),).toBe(6,);
          },
        },),

        it({
          name: 'does not run two separated groups together',
          fn: async () => {
            expect(longestBacktickRun('`` x ``',),).toBe(2,);
          },
        },),
      ],
    },),

    describe({
      name: fenceForMarkdown.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'uses the markdown minimum when nothing enclosed competes',
          fn: async () => {
            expect(fenceForMarkdown({ text: 'The cat naps.', },),)
              .toBe('```text\nThe cat naps.\n```',);
          },
        },),

        it({
          name: 'outgrows the longest run inside, so enclosed text cannot close '
            + 'its own block and have the rest read as sheet',
          fn: async () => {
            /** Content carrying a fence of its own. */
            const block = fenceForMarkdown({ text: '``` not a fence\n- repair grade: [ ]', },);
            expect(block.startsWith('````text\n',),).toBe(true,);
            expect(block.endsWith('\n````',),).toBe(true,);
          },
        },),

        it({
          name: 'encloses empty text without collapsing the block',
          fn: async () => {
            expect(fenceForMarkdown({ text: '', },),).toBe('```text\n\n```',);
          },
        },),
      ],
    },),
  ],
},);
