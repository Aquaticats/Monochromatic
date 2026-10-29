/**
 Tests that the page's definitions reach the quote-style restoration of a refinement reply (n160 finding Y8).

 `resolveReply` hands `definitions` to `restoreTypography` (`refine-stage-replies.ts`), so a shortcut reference whose definition
 stands elsewhere on the page resolves there and its label is no prose: a straight apostrophe in the label stays straight, as the
 definition's label writes it. Without the pass-through the same reply read the label as plain text and curled the apostrophe, so
 the paragraph's reference no longer named its definition. The other tests of the file pass the definitions and assert nothing that
 depends on them; this one reads the same reply with and without them.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  deriveRefinableEnvelopes,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { resolveReply, } from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Logger for the function under test.
 */
const l = tagged({ tag: 'refine-stage-replies-definitions-test', },);

/**
 The repaired slice: curly marks in the prose, and a shortcut reference whose label holds a straight apostrophe and whose
 definition stands in the document after the paragraph.
 */
const REPAIRED_TEXT = [
  'The cat’s bowl sits by the window in the sun, and she likes the [dogs\' bowl] on the porch every afternoon, '
    + 'and when the light moves across the floor she follows it without any hurry at all.',
  '',
  '[dogs\' bowl]: https://example.org/bowl',
].join('\n',);

/**
 The rewrite a seat sends: the same paragraph with a straight apostrophe in the prose and the label.
 */
const REWRITE =
  'The cat\'s bowl sits by the window in the sun, and she likes the [dogs\' bowl] on the porch every afternoon, and when the light moves across the floor she follows it without hurry.';

/**
 The slice's envelopes and the definitions the document holds.
 */
const SLICE = deriveRefinableEnvelopes({ document: parseDocument({ text: REPAIRED_TEXT, },), },);

await describe({
  name: resolveReply.name,
  concurrency: 1,
  children: [
    it({
      name: 'KEEPS the straight apostrophe of a reference label whose definition the page holds, and curls the prose',
      fn: async () => {
        expect(SLICE.definitions,).toContain('[dogs\' bowl]:',);
        expect(SLICE.envelopes.length,).toBe(1,);
        const reply = resolveReply({
          voice: {
            modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            value: { rewrites: [{ paragraph: 1, newText: REWRITE, },], },
          },
          envelopes: SLICE.envelopes,
          repairedText: REPAIRED_TEXT,
          definitions: SLICE.definitions,
          l,
        },);
        expect(reply.refusals,).toEqual([],);
        expect(reply.passed.length,).toBe(1,);
        expect(reply.passed[0]?.newText,).toContain('[dogs\' bowl]',);
        expect(reply.passed[0]?.newText,).toContain('cat’s bowl',);
      },
    },),

    it({
      name: 'READS the same reply differently when the definitions are withheld, so the first test depends on them',
      fn: async () => {
        const reply = resolveReply({
          voice: {
            modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            value: { rewrites: [{ paragraph: 1, newText: REWRITE, },], },
          },
          envelopes: SLICE.envelopes,
          repairedText: REPAIRED_TEXT,
          definitions: '',
          l,
        },);
        // Withheld, the label is plain text: the passed paragraph does not hold the label as the page writes it.
        expect(reply.passed[0]?.newText?.includes('[dogs\' bowl]',) ?? false,).toBe(false,);
      },
    },),
  ],
},);
