/**
 Tests for the publication guard that keeps a sealed span as the archive has
 it.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ArchiveOriginalCompletenessError,
  assertArchiveOriginalComplete,
} from '../../dist/final/node/index.mjs';

/**
 Archive with a sealed letter.
 */
const ARCHIVE = 'Intro.\n\n<!-- note -->\n\n> The cans are in the cupboard,\n\nTime for a nap, friends.\n';

/**
 The sealed span: from the note's end to the archive's end.
 */
const SPAN = {
  startOffset: ARCHIVE.indexOf('-->',) + '-->'.length,
  endOffset: ARCHIVE.length,
  note: 'note',
};

await describe({
  name: assertArchiveOriginalComplete.name,
  children: [
    it({
      name: 'passes a page carrying the sealed bytes verbatim, wherever the slices before them moved',
      fn: async () => {
        expect(function guard(): void {
          assertArchiveOriginalComplete({
            entryId: 'mooncat',
            archiveText: ARCHIVE,
            pageText: ARCHIVE.replace(
              'Intro.',
              'A longer introduction than the archive had.',
            ),
            spans: [ SPAN, ],
          },);
        },).not
          .toThrow();
      },
    },),

    it({
      name: 'passes with nothing sealed',
      fn: async () => {
        expect(function guard(): void {
          assertArchiveOriginalComplete({
            entryId: 'mooncat',
            archiveText: ARCHIVE,
            pageText: 'Anything.\n',
            spans: [],
          },);
        },).not
          .toThrow();
      },
    },),

    it({
      name: 'refuses a page whose sealed span was reworded, naming the entry and the span',
      fn: async () => {
        expect(function guard(): void {
          assertArchiveOriginalComplete({
            entryId: 'mooncat',
            archiveText: ARCHIVE,
            pageText: ARCHIVE.replace(
              'The cans are in the cupboard,',
              'The cans are in the top cupboard,',
            ),
            spans: [ SPAN, ],
          },);
        },).toThrow(ArchiveOriginalCompletenessError,);
        expect(function guard(): void {
          assertArchiveOriginalComplete({
            entryId: 'mooncat',
            archiveText: ARCHIVE,
            pageText: 'Nothing of it.\n',
            spans: [ SPAN, ],
          },);
        },).toThrow('entry mooncat page does not carry archive-original span 0',);
      },
    },),

    it({
      name: 'CARRIES THE ENTRY, SPAN AND OFFSETS as fields, so a caller finds the span without parsing the '
        + 'message (ledger F-12)',
      fn: async () => {
        /**
         Refusal thrown for the page that lost the span.
         */
        const refusal = (function thrown(): unknown {
          try {
            assertArchiveOriginalComplete({
              entryId: 'mooncat',
              archiveText: ARCHIVE,
              pageText: 'Nothing of it.\n',
              spans: [ SPAN, ],
            },);
          }
          catch (error: unknown) {
            return error;
          }
          throw new Error('the guard did not refuse the page',);
        })();
        expect(refusal,).toBeInstanceOf(ArchiveOriginalCompletenessError,);
        expect({
          entryId: (refusal as ArchiveOriginalCompletenessError).entryId,
          spanIndex: (refusal as ArchiveOriginalCompletenessError).spanIndex,
          startOffset: (refusal as ArchiveOriginalCompletenessError).startOffset,
          endOffset: (refusal as ArchiveOriginalCompletenessError).endOffset,
        },).toEqual({
          entryId: 'mooncat',
          spanIndex: 0,
          startOffset: SPAN.startOffset,
          endOffset: SPAN.endOffset,
        },);
      },
    },),
  ],
},);
