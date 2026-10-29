/**
 Guards what the archive-block review tells its judges about the page
 (XingZ60, 2026-10-19). The reviewers of an archive block no original block
 claims were never told that the page published from the pass carries an
 English rendering of every block of the original, so five of them kept a
 notice that the page is "NOT the formal version" and "unfinished" as page
 apparatus that "describes the document's status".

 A notice that the English page is unfinished, not the formal version or
 waiting on another version is false on a page that is complete, so the sheet
 of the review and the criteria of its correction slate say so.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ARCHIVE_BLOCK_SELECTION_CRITERIA,
  buildArchiveBlockReviewMessages,
} from '../dist/final/node/index.mjs';

/**
 What the sheet says about the page being complete.
 */
const COMPLETE_PAGE = 'an English rendering of every block of the original';

/**
 System message of a review of a notice block.
 */
const system = buildArchiveBlockReviewMessages({
  sourceText: '猫在窗台上打盹。',
  targetText: '**Come back later!**\n\nThe cat naps on the sill.',
  blockText: '**Come back later!**',
  priorFindings: [],
},)
  .filter(function isSystem(message,): boolean {
    return message.role === 'system';
  },)
  .map(function contentOf(message,): string {
    return message.content;
  },)
  .join('\n',);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildArchiveBlockReviewMessages.name,
      children: [
        it({
          name: 'TELLS the reviewers the page carries every block of the original, so a notice that it is unfinished, '
            + 'not the formal version or waiting on another version is false on it and no apparatus',
          fn: async () => {
            expect(system,).toContain(COMPLETE_PAGE,);
            expect(system,).toContain('unfinished',);
            expect(system,).toContain('not the formal version',);
          },
        },),
      ],
    },),
    describe({
      name: 'ARCHIVE_BLOCK_SELECTION_CRITERIA',
      children: [
        it({
          name: 'SAYS THE SAME to the correction slate, in the criterion that outranks the rest',
          fn: async () => {
            expect(ARCHIVE_BLOCK_SELECTION_CRITERIA[0],).toContain(COMPLETE_PAGE,);
          },
        },),
      ],
    },),
  ],
},);
