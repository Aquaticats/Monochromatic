/**
 Tests for what one heard rewriter's reply comes to before the judges see it:
 bound to the sheet, read by the atom gate, applied to the slice, and ordered
 by the roster.

 The stage's own cases (`refine-stage.unit.test.ts`) reach these through a
 whole run; these hold the refusals the patch can give a hand-built reply,
 which no run produces.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  deriveRefinableEnvelopes,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  applyReply,
  inRosterOrder,
  resolveReply,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Logger for the functions under test.
 */
const l = tagged({ tag: 'refine-stage-replies-test', },);

/**
 Repaired slice, one long single-line paragraph so it clears eligibility.
 */
const REPAIRED_TEXT =
  'The cat is doing the sunbathing on the windowsill in every afternoon, and when the light is moving across the floor she is following it without any hurry at all.';

/**
 A more natural rendering carrying the same content.
 */
const SMOOTH_TEXT =
  'The cat sunbathes on the windowsill every afternoon, and when the light moves across the floor she follows it without hurry.';

/**
 The slice's one eligible paragraph and the definitions around it.
 */
const SLICE = deriveRefinableEnvelopes({ document: parseDocument({ text: REPAIRED_TEXT, },), },);

/**
 The paragraph as the sheet shows it.
 */
const [PARAGRAPH,] = SLICE.envelopes;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: resolveReply.name,
      children: [
        it({
          name: 'PASSES a rewrite that keeps every protected atom and leaves no refusal',
          fn: async () => {
            const reply = resolveReply({
              voice: {
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                value: { rewrites: [{ paragraph: 1, newText: SMOOTH_TEXT, },], },
              },
              envelopes: SLICE.envelopes,
              repairedText: REPAIRED_TEXT,
              definitions: SLICE.definitions,
              l,
            },);
            expect(reply.modelId,).toBe(SEAT_HYPER_OPENROUTER_VISION_EDITOR,);
            expect(reply.passed,).toEqual([
              {
                envelopeId: PARAGRAPH?.envelopeId,
                baseHash: PARAGRAPH?.baseHash,
                newText: SMOOTH_TEXT,
              },
            ],);
            expect(reply.refusals,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: applyReply.name,
      children: [
        it({
          name: 'RETURNS no proposal and no finding for a reply whose operations the gate all refused',
          fn: async () => {
            expect(applyReply({
              reply: {
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                resolution: { operations: [], findings: [], },
                passed: [],
                refusals: [`${SEAT_HYPER_OPENROUTER_VISION_EDITOR}: refine-atom-gate-refused (paragraph 1, x)`,],
              },
              repairedText: REPAIRED_TEXT,
              envelopes: SLICE.envelopes,
            },),).toEqual({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              candidates: [],
              rejections: [],
            },);
          },
        },),

        it({
          name: 'RETURNS the patched slice as the rewriter\'s proposal for an operation that changes the paragraph',
          fn: async () => {
            expect(applyReply({
              reply: {
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                resolution: { operations: [], findings: [], },
                passed: [
                  {
                    envelopeId: PARAGRAPH?.envelopeId ?? '',
                    baseHash: PARAGRAPH?.baseHash ?? '',
                    newText: SMOOTH_TEXT,
                  },
                ],
                refusals: [],
              },
              repairedText: REPAIRED_TEXT,
              envelopes: SLICE.envelopes,
            },),).toEqual({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              candidates: [
                {
                  producer: { kind: 'model', modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },
                  value: SMOOTH_TEXT,
                  rendered: SMOOTH_TEXT,
                },
              ],
              rejections: [],
            },);
          },
        },),

        it({
          name: 'NAMES an operation that writes the paragraph back as an unchanged rewrite of paragraph 1 and '
            + 'proposes nothing',
          fn: async () => {
            expect(applyReply({
              reply: {
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                resolution: { operations: [], findings: [], },
                passed: [
                  {
                    envelopeId: PARAGRAPH?.envelopeId ?? '',
                    baseHash: PARAGRAPH?.baseHash ?? '',
                    newText: REPAIRED_TEXT,
                  },
                ],
                refusals: [],
              },
              repairedText: REPAIRED_TEXT,
              envelopes: SLICE.envelopes,
            },),).toEqual({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              candidates: [],
              rejections: [`${SEAT_HYPER_OPENROUTER_VISION_EDITOR}: refine-unchanged-rewrite (paragraph 1)`,],
            },);
          },
        },),

        it({
          name: 'NAMES an operation whose rewrite leaves a text nested 300 list markers deep as an unreadable '
            + 'rewrite of paragraph 1 and proposes nothing, where the atom gate read the rewrite alone',
          fn: async () => {
            expect(applyReply({
              reply: {
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                resolution: { operations: [], findings: [], },
                passed: [
                  {
                    envelopeId: PARAGRAPH?.envelopeId ?? '',
                    baseHash: PARAGRAPH?.baseHash ?? '',
                    newText: `${'- '.repeat(300,)}${SMOOTH_TEXT}`,
                  },
                ],
                refusals: [],
              },
              repairedText: REPAIRED_TEXT,
              envelopes: SLICE.envelopes,
            },),).toEqual({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              candidates: [],
              rejections: [`${SEAT_HYPER_OPENROUTER_VISION_EDITOR}: refine-unreadable-rewrite (paragraph 1)`,],
            },);
          },
        },),

        it({
          name: 'THROWS naming the reason when the patch refuses an operation for a stale base hash, which '
            + 'the resolver\'s own hash echo cannot produce',
          fn: async () => {
            const refusal = caught(function act(): unknown {
              return applyReply({
                reply: {
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  resolution: { operations: [], findings: [], },
                  passed: [
                    {
                      envelopeId: PARAGRAPH?.envelopeId ?? '',
                      baseHash: 'not-the-hash',
                      newText: SMOOTH_TEXT,
                    },
                  ],
                  refusals: [],
                },
                repairedText: REPAIRED_TEXT,
                envelopes: SLICE.envelopes,
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe(
              `Error: unreachable: the patch refused ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}'s rewrite as `
                + 'stale-base-hash, but this lane binds its operations to its own envelopes, applies them to '
                + 'the text those envelopes came from with preservation skipped, and so can be refused only '
                + 'as unchanged-region or as a text no grammar reads',
            );
          },
        },),

        it({
          name: 'THROWS naming the reason when the patch refuses an operation for an envelope the sheet '
            + 'does not hold',
          fn: async () => {
            const refusal = caught(function act(): unknown {
              return applyReply({
                reply: {
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  resolution: { operations: [], findings: [], },
                  passed: [
                    {
                      envelopeId: 'paragraph/never-shown',
                      baseHash: PARAGRAPH?.baseHash ?? '',
                      newText: SMOOTH_TEXT,
                    },
                  ],
                  refusals: [],
                },
                repairedText: REPAIRED_TEXT,
                envelopes: SLICE.envelopes,
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe(
              `Error: unreachable: the patch refused ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}'s rewrite as `
                + 'unknown-envelope, but this lane binds its operations to its own envelopes, applies them to '
                + 'the text those envelopes came from with preservation skipped, and so can be refused only '
                + 'as unchanged-region or as a text no grammar reads',
            );
          },
        },),
      ],
    },),

    describe({
      name: inRosterOrder.name,
      children: [
        it({
          name: 'PUTS replies in the order their rewriters sit on the roster, whatever order they were heard in',
          fn: async () => {
            expect(inRosterOrder({
              replies: [
                { modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },
                { modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },
                { modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },
              ],
              roster: [
                SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                SEAT_SYNTHETIC_VISION_WITHHELD,
              ],
            },),).toEqual([
              { modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },
              { modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },
              { modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },
            ],);
          },
        },),
      ],
    },),
  ],
},);
