/**
 Tests for how the bench's renderings of one page title are counted: the wrapping marks that come off, the
 renderings that count as one, the voices a rendering that shows a reader nothing does not get, and the count the
 leader carries (ledger H16, B24 and B40).

 WHY THIS FILE EXISTS. The counting used to live inside the stage file and was reached only through
 `settlePageTitles`. The stage file moved it out to stay under the line limit, and the count is the part re-check 2
 (finding F1, 2026-10-25) turned on: a leader that one voice alone gave comes back with a count of one, and the stage
 settles only on a count of two or more.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mostGiven,
  unwrappedRendering,
} from '../dist/final/node/index.mjs';

await describe({
  name: `${mostGiven.name} and ${unwrappedRendering.name}`,
  children: [
    it({
      name: 'TAKES OFF THE WRAPPING MARKS and the surrounding space, from either end, of every style a voice answers in',
      fn: async () => {
        expect([
          ' “Song of the Cat” ',
          '「Song of the Cat」',
          '_Song of the Cat_',
          '"*Song of the Cat*"',
        ].map(function unwrap(rendering,): string {
          return unwrappedRendering({ rendering, },);
        },),).toEqual([
          'Song of the Cat',
          'Song of the Cat',
          'Song of the Cat',
          'Song of the Cat',
        ],);
      },
    },),

    it({
      name: 'COUNTS RENDERINGS THAT DIFFER ONLY IN CASE, SPACING OR QUOTE STYLE AS ONE, keeping the first wording given',
      fn: async () => {
        expect(mostGiven({ answers: ['The Cat\'s Song', 'the cat’s  song', 'Cat Song',], },),).toEqual([
          { rendering: 'The Cat\'s Song', voices: 2, },
        ],);
      },
    },),

    it({
      name: 'RETURNS A LEADER ONE VOICE ALONE GAVE WITH A COUNT OF ONE, which the stage does not settle on, and breaks '
        + 'a tie of one voice apiece to the earliest seat',
      fn: async () => {
        expect({
          alone: mostGiven({ answers: ['', 'Song of the Cat', '',], },),
          tied: mostGiven({ answers: ['Dream of Fish', 'The Fish Dream',], },),
        },).toEqual({
          alone: [{ rendering: 'Song of the Cat', voices: 1, },],
          tied: [{ rendering: 'Dream of Fish', voices: 1, },],
        },);
      },
    },),

    it({
      name: 'COUNTS NO VOICE FOR A RENDERING THAT SHOWS A READER NOTHING, and returns no leader where no voice gave words',
      fn: async () => {
        expect({
          padded: mostGiven({ answers: ['\u{200B}', ' \u{3164} ', 'Song of the Cat',], },),
          silent: mostGiven({ answers: ['', '\u{200B}',], },),
        },).toEqual({
          padded: [{ rendering: 'Song of the Cat', voices: 1, },],
          silent: [],
        },);
      },
    },),
  ],
},);
