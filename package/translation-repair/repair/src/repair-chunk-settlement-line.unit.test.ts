/**
 Tests for the line an operator watches a repair run by.

 WHAT THESE PIN is that the summary cannot drift from the settlement it
 summarises. Every number in the line is a count of something the verdict
 decided, and they arrive as four separate arguments in one sentence, so a
 transposition renders perfectly and reads as an ordinary run. The subject's
 own note says what that costs: a run reads as healthy while shipping
 something else.

 The line is emitted at `info` and never returned, so nothing downstream can
 catch a wrong one; this file is where it is read.

 WHY "UNCHANGED" NOW SAYS WHICH (ledger L12). The word covered three
 settlements: the archive beat the patch on the measurements, the patch won
 and wrote no byte, and the patch won and was refused for dropping a declared
 name. The log could not tell a lost repair from one that wrote nothing.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { describeChunkSettlement, } from '../dist/final/node/index.mjs';

/**
 Slice these lines report.

 Not zero, so a line that lost the index reads differently from one that kept
 it.
 */
const SLICE_INDEX = 7;

/**
 Four counts, each distinct, so any pair swapped in the template changes the
 line. Equal counts would render the same either way round.
 */
const COUNTS = {
  resolvedCount: 1,
  creditableCount: 2,
  acceptedCount: 3,
  unenvelopedCount: 4,
} as const;

await describe({
  name: describeChunkSettlement.name,
  children: [
    it({
      name:
        'PLACES ALL FOUR COUNTS where the sentence says they are, which is what stops a summary '
          + 'drifting from the settlement it summarises: served-and-resolved reads as a fraction, and '
          + 'accepted and unenveloped ride in the parenthesis, so a transposed pair still renders',
      fn: async () => {
        expect(describeChunkSettlement({
          sliceIndex: SLICE_INDEX,
          changed: true,
          patchSelected: true,
          refused: false,
          ...COUNTS,
        },),).toBe(
          'chunk 7: repaired, 1/2 served accepted issues resolved (3 accepted, 4 unenveloped)',
        );
      },
    },),
    it({
      name: 'SAYS THE ARCHIVE WON where the patch lost on the measurements',
      fn: async () => {
        expect(describeChunkSettlement({
          sliceIndex: SLICE_INDEX,
          changed: false,
          patchSelected: false,
          refused: false,
          ...COUNTS,
        },),).toBe(
          'chunk 7: unchanged, the archive won, 1/2 served accepted issues resolved (3 accepted, 4 unenveloped)',
        );
      },
    },),
    it({
      name: 'SAYS WHAT THE PATCH LOST ON where the archive won it, so a chunk that served accepted issues all '
        + 'resolved and still closed unchanged explains itself in the log',
      fn: async () => {
        /**
         Slice and counts of a patch the archive beat.
         */
        const lost = {
          sliceIndex: SLICE_INDEX,
          changed: false,
          patchSelected: false,
          refused: false,
          ...COUNTS,
        } as const;
        expect({
          integrity: describeChunkSettlement({
            ...lost,
            patchLostOn: 'integrity',
          },),
          regression: describeChunkSettlement({
            ...lost,
            patchLostOn: 'regression',
          },),
        },).toEqual({
          integrity: 'chunk 7: unchanged, the archive won (the patch lost on integrity: the chunk\'s grammar or '
            + 'footnote findings rose above the archive\'s), 1/2 served accepted issues resolved (3 accepted, 4 '
            + 'unenveloped)',
          regression: 'chunk 7: unchanged, the archive won (the patch lost on regressions: it made an accepted issue '
            + 'worse), 1/2 served accepted issues resolved (3 accepted, 4 unenveloped)',
        },);
      },
    },),
    it({
      name: 'SAYS THE PATCH WROTE NOTHING where it won and its operations left the archive\'s wording standing',
      fn: async () => {
        expect(describeChunkSettlement({
          sliceIndex: SLICE_INDEX,
          changed: false,
          patchSelected: true,
          refused: false,
          ...COUNTS,
        },),).toBe(
          'chunk 7: unchanged, the patch won and wrote nothing, 1/2 served accepted issues resolved '
            + '(3 accepted, 4 unenveloped)',
        );
      },
    },),
    it({
      name: 'SAYS THE PATCH WAS REFUSED where it won and dropped a declared name',
      fn: async () => {
        expect(describeChunkSettlement({
          sliceIndex: SLICE_INDEX,
          changed: false,
          patchSelected: true,
          refused: true,
          ...COUNTS,
        },),).toBe(
          'chunk 7: unchanged, the patch won and was refused for dropping a declared name, 1/2 served '
            + 'accepted issues resolved (3 accepted, 4 unenveloped)',
        );
      },
    },),
  ],
},);
