/**
 Tests for what the patch gate says of an edit by the text the edit leaves:
 read against the nesting bound with the edits already accepted beside it,
 and named by a reason a scorecard counts.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isUnreadableReason,
  nestingWithEdit,
  unreadableReason,
} from '../dist/final/node/index.mjs';

//region Patch nesting tests

/**
 Text of two paragraphs that every case writes edits into.
 */
const TARGET_TEXT = 'The kitten dozes.\n\nThe bowl is empty.\n';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: nestingWithEdit.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a text an ordinary edit leaves as within the bound',
          fn: async () => {
            expect(nestingWithEdit({
              targetText: TARGET_TEXT,
              accepted: [],
              candidate: { start: 19, end: 37, text: 'The bowl is full.', },
            },),).toEqual({ kind: 'within', },);
          },
        },),
        it({
          name: 'NAMES the place in the text the edit leaves where its 257th list marker is, not in the edit alone',
          fn: async () => {
            expect(nestingWithEdit({
              targetText: TARGET_TEXT,
              accepted: [],
              candidate: { start: 19, end: 37, text: `${'- '.repeat(300,)}The bowl is full.`, },
            },),).toEqual({
              kind: 'beyond',
              measure: 'container markers',
              bound: 256,
              line: 3,
              column: 513,
            },);
          },
        },),
        it({
          name: 'READS the edits already accepted as part of the text, so an accepted edit that opens a fence the parser '
            + 'reads leaves the candidate inside code',
          fn: async () => {
            /**
             An accepted edit leaving the first paragraph as the line that opens a fence.
             */
            const accepted = { start: 0, end: 17, text: '```', };
            /**
             A candidate leaving the second paragraph 300 brackets deep.
             */
            const candidate = { start: 19, end: 37, text: `${'['.repeat(300,)}The bowl is full.`, };

            expect(nestingWithEdit({ targetText: TARGET_TEXT, accepted: [], candidate, },),).toEqual({
              kind: 'beyond',
              measure: 'brackets',
              bound: 256,
              line: 3,
              column: 257,
            },);
            expect(nestingWithEdit({ targetText: TARGET_TEXT, accepted: [accepted,], candidate, },),).toEqual({
              kind: 'within',
            },);
          },
        },),
      ],
    },),

    describe({
      name: unreadableReason.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS the account in the parenthetical detail of a reason that begins with the kind, quoting no text',
          fn: async () => {
            expect(unreadableReason({
              excess: { measure: 'brackets', bound: 256, line: 3, column: 9, },
            },),).toBe(
              'unreadable-replacement (nested too deeply to read: its brackets pass the bound of 256 at line 3, column 9)',
            );
          },
        },),
      ],
    },),

    describe({
      name: isUnreadableReason.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KNOWS the reason it made and no other reason a patch gives',
          fn: async () => {
            expect(isUnreadableReason({
              reason: unreadableReason({ excess: { measure: 'brackets', bound: 256, line: 3, column: 9, }, },),
            },),).toBe(true,);
            expect(isUnreadableReason({ reason: 'unchanged-region', },),).toBe(false,);
            expect(isUnreadableReason({ reason: 'unreadable-replacement', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Patch nesting tests
