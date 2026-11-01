/**
 Tests for tolerant model-JSON parsing and usage-note formatting.
 Fence and thinking-block handling is covered beside the client in
 `synthetic-client.unit.test.ts`; this file covers the two helpers
 that had only indirect coverage.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  formatUsageNote,
  parseModelJson,
  stripCodeFence,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: parseModelJson.name,
      children: [
        it({
          name: 'parses valid JSON into data',
          fn: async () => {
            expect(parseModelJson({ text: '{"cat":"喵"}', },),).toEqual({
              parsed: true,
              value: { cat: '喵', },
            },);
          },
        },),
        it({
          name: 'returns failure detail as data instead of throwing',
          fn: async () => {
            const attempt = parseModelJson({ text: '{"cat":', },);
            expect(attempt.parsed,).toBe(false,);
            if (attempt.parsed)
              throw new Error('unreachable: asserted failure',);
            expect(attempt.detail,).toContain('SyntaxError',);
          },
        },),
        it({
          name: 'NAMES THE CLASS OF A PARSE REFUSAL and never the opening of the reply V8 quotes in its message',
          fn: async () => {
            /**
             Reply that stops being JSON at its first character, so V8 quotes it.
             */
            const reply = 'Pouncewick was never JSON';

            /**
             What a bare parse of the reply raises, held to show the probe can see the quote.
             */
            const bare = caught(function bareParse(): unknown {
              return JSON.parse(reply,);
            },);
            // Positive control: the bare parse's own message carries the reply's opening.
            expect(String(bare,).includes('Pouncewick',),).toBe(true,);
            expect(parseModelJson({ text: reply, },),).toEqual({
              parsed: false,
              detail: 'refused by SyntaxError',
            },);
          },
        },),
      ],
    },),
    describe({
      name: formatUsageNote.name,
      children: [
        it({
          name: 'formats reported component counts',
          fn: async () => {
            expect(
              formatUsageNote({
                extracted: {
                  text: '喵',
                  usage: {
                    prompt_tokens: 3,
                    completion_tokens: 7,
                  },
                },
              },),
            ).toBe(', 3+7 tokens',);
          },
        },),
        it({
          name: 'stays empty when the server reported no usage',
          fn: async () => {
            expect(formatUsageNote({ extracted: { text: '喵', }, },),).toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: stripCodeFence.name,
      children: [
        it({
          name: 'STRIPS NOTHING from a fence line with no line end after it, since the whole text is the opening '
            + 'line',
          fn: async () => {
            expect(stripCodeFence({ text: '```ts one line', },),).toBe('```ts one line',);
          },
        },),
      ],
    },),
  ],
},);
