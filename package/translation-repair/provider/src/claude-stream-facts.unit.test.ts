/**
 Tests for the facts a drained Messages stream states beyond its text (`claude-stream-facts.ts`): the refusal stop with
 its category wherever the details sit, and the usage split into the four counts Anthropic bills at four rates.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { anthropicFrameOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/anthropic-frames.test-fixture.ts';
import { claudeStreamFactsOf, } from '../dist/final/node/index.mjs';
import {
  claudeStreamOf,
  PLAIN_USAGE,
} from './claude-client.test-fixture.ts';

await describe({
  name: claudeStreamFactsOf.name,
  children: [
    it({
      name: 'READS A REFUSAL WITH ITS CATEGORY, and one with a null or missing category as a refusal with none',
      fn: async () => {
        expect(claudeStreamFactsOf({
          bodyText: claudeStreamOf({
            text: '',
            stopReason: 'refusal',
            stopDetails: { type: 'refusal', category: 'bio', explanation: 'declined', },
            usage: PLAIN_USAGE,
          }),
        },).refusal,).toEqual({ category: 'bio', },);
        expect(claudeStreamFactsOf({
          bodyText: claudeStreamOf({
            text: '',
            stopReason: 'refusal',
            stopDetails: { type: 'refusal', category: null, explanation: null, },
            usage: PLAIN_USAGE,
          }),
        },).refusal,).toEqual({},);
        expect(claudeStreamFactsOf({
          bodyText: claudeStreamOf({ text: '', stopReason: 'refusal', usage: PLAIN_USAGE, }),
        },).refusal,).toEqual({},);
      },
    },),

    it({
      name: 'FINDS THE DETAILS ON THE FRAME ITSELF OR ON THE OPENING MESSAGE too, since the streamed shape is not '
        + 'documented, and reads a refusal from no other stop reason',
      fn: async () => {
        expect(claudeStreamFactsOf({
          bodyText: anthropicFrameOf({
            body: {
              type: 'message_delta',
              delta: { stop_reason: 'refusal', },
              stop_details: { type: 'refusal', category: 'cyber', },
              usage: { output_tokens: 3, },
            },
          },),
        },).refusal,).toEqual({ category: 'cyber', },);
        expect(claudeStreamFactsOf({
          bodyText: [
            anthropicFrameOf({
              body: {
                type: 'message_start',
                message: { stop_details: { type: 'refusal', category: 'general_harms', }, usage: { input_tokens: 4, }, },
              },
            },),
            anthropicFrameOf({ body: { type: 'message_delta', delta: { stop_reason: 'refusal', }, }, },),
          ].join('',),
        },).refusal,).toEqual({ category: 'general_harms', },);
        for (const stopReason of ['end_turn', 'max_tokens', 'stop_sequence',]) {
          const facts = claudeStreamFactsOf({
            bodyText: claudeStreamOf({
              text: 'purr',
              stopReason,
              stopDetails: { type: 'refusal', category: 'cyber', },
              usage: PLAIN_USAGE,
            }),
          },);
          expect(facts.refusal,).toBeUndefined();
          expect(facts.stopReason,).toBe(stopReason,);
        }
      },
    },),

    it({
      name: 'SPLITS THE USAGE INTO ITS FOUR COUNTS, the last report of each standing, with a cache count that was '
        + 'never reported read as zero',
      fn: async () => {
        expect(claudeStreamFactsOf({
          bodyText: claudeStreamOf({
            text: 'purr',
            stopReason: 'end_turn',
            usage: { input: 12, cacheWrite: 300, cacheRead: 4_000, output: 77, },
          }),
        },).usage,).toEqual({ input: 12, cacheWrite: 300, cacheRead: 4_000, output: 77, },);
        expect(claudeStreamFactsOf({
          bodyText: [
            anthropicFrameOf({ body: { type: 'message_start', message: { usage: { input_tokens: 9, }, }, }, },),
            anthropicFrameOf({ body: { type: 'message_delta', delta: {}, usage: { output_tokens: 5, }, }, },),
          ].join('',),
        },).usage,).toEqual({ input: 9, cacheWrite: 0, cacheRead: 0, output: 5, },);
      },
    },),

    it({
      name: 'REPORTS NO USAGE unless the stream gave both its prompt count and its completion count, and nothing at '
        + 'all from a body with no frames',
      fn: async () => {
        expect(claudeStreamFactsOf({
          bodyText: anthropicFrameOf({ body: { type: 'message_delta', delta: {}, usage: { output_tokens: 5, }, }, },),
        },).usage,).toBeUndefined();
        expect(claudeStreamFactsOf({
          bodyText: anthropicFrameOf({ body: { type: 'message_start', message: { usage: { input_tokens: 5, }, }, }, },),
        },).usage,).toBeUndefined();
        expect(claudeStreamFactsOf({ bodyText: 'not a stream\n: comment\n\n', },),).toEqual({},);
      },
    },),
  ],
},);
