/**
 Tests for SSE stream reassembly.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { MalformedCompletionError, } from '@monochromatic-dev/translation-repair-provider-stream';
import { extractStreamedCompletion, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: extractStreamedCompletion.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reassembles content deltas, keeps last usage, ignores reasoning and empty events',
          fn: async () => {
            /** Drained stream mixing content, reasoning, empty, and usage events. */
            const body = [
              'data: {"choices":[{"delta":{"role":"assistant"}}]}',
              'data: {"choices":[{"delta":{"reasoning_content":"猫在想事情"}}]}',
              String.raw`data: {"choices":[{"delta":{"content":"{\"a\":"}}]}`,
              'data: {"choices":[{"delta":{"content":"1}"}}]}',
              'data: {"choices":[],"usage":{"prompt_tokens":5,"completion_tokens":7}}',
              'data: [DONE]',
              '',
            ].join('\n\n',);
            expect(extractStreamedCompletion({ bodyText: body, },),).toEqual({
              text: '{"a":1}',
              usage: {
                prompt_tokens: 5,
                completion_tokens: 7,
              },
            },);
          },
        },),

        it({
          name: 'folds refusal deltas into the first-class refusal field',
          fn: async () => {
            /** Drained stream refusing across two deltas. */
            const body = [
              'data: {"choices":[{"delta":{"refusal":"Request declined "}}]}',
              'data: {"choices":[{"delta":{"refusal":"by policy."}}]}',
              'data: [DONE]',
              '',
            ].join('\n\n',);
            expect(extractStreamedCompletion({ bodyText: body, },),).toEqual({
              text: '',
              refusal: 'Request declined by policy.',
            },);
          },
        },),

        it({
          name: 'leaves the refusal field absent when the refusal deltas show a reader nothing, so a JSON answer '
            + 'beside a refusal channel of spaces, a zero-width space and a Hangul filler reads as the answer',
          fn: async () => {
            /** Drained stream answering in content beside a padded refusal channel. */
            const body = [
              String.raw`data: {"choices":[{"delta":{"content":"{\"a\":1}"}}]}`,
              ...[
                ' ',
                '\u{200B}',
                '\u{3164}',
                '\n',
              ].map(function refusalEvent(refusal,): string {
                return `data: ${JSON.stringify({ choices: [{ delta: { refusal, }, },], },)}`;
              },),
              'data: [DONE]',
              '',
            ].join('\n\n',);
            expect(extractStreamedCompletion({ bodyText: body, },),).toEqual({
              text: '{"a":1}',
            },);
          },
        },),

        it({
          name: 'ignores a usage-only event that names no choices array at all, not only one that '
            + 'names an empty one, while still folding its usage',
          fn: async () => {
            /** Drained stream whose usage event carries no `choices` key whatsoever. */
            const body = [
              'data: {"choices":[{"delta":{"content":"x"}}]}',
              'data: {"usage":{"prompt_tokens":3,"completion_tokens":1}}',
              'data: [DONE]',
              '',
            ].join('\n\n',);
            expect(extractStreamedCompletion({ bodyText: body, },),).toEqual({
              text: 'x',
              usage: {
                prompt_tokens: 3,
                completion_tokens: 1,
              },
            },);
          },
        },),

        it({
          name: 'throws when the stream ends without its terminator',
          fn: async () => {
            /** Value caught from a cut-off stream. */
            let caught: unknown;
            try {
              extractStreamedCompletion({
                bodyText: 'data: {"choices":[{"delta":{"content":"half"}}]}\n',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught instanceof MalformedCompletionError,).toBe(true,);
          },
        },),

        it({
          name: 'throws on events that are not valid JSON objects',
          fn: async () => {
            /** Value caught from a garbage event. */
            let caught: unknown;
            try {
              extractStreamedCompletion({
                bodyText: 'data: not-json\n\ndata: [DONE]\n',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught instanceof MalformedCompletionError,).toBe(true,);
          },
        },),

        it({
          name: 'throws on an event that parses to an array, which was folded as a usage-only event '
            + 'instead of refused (ledger B92)',
          fn: async () => {
            /** Value caught from an array event. */
            let caught: unknown;
            try {
              extractStreamedCompletion({
                bodyText: 'data: [1]\n\ndata: [DONE]\n',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught instanceof MalformedCompletionError,).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'finish reason folding',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the reason off the closing event, whose delta is empty',
          fn: async () => {
            // THE ORDERING THIS PINS. An OpenAI-compatible stream closes with an
            // event carrying an EMPTY delta beside the reason. A fold that checked
            // the delta first and returned early would discard exactly the event
            // worth reading, and every stream would report no reason at all.
            const body = [
              String.raw`data: {"choices":[{"delta":{"content":"{\"a\":"}}]}`,
              'data: {"choices":[{"delta":{},"finish_reason":"length"}]}',
              'data: [DONE]',
              '',
            ].join('\n\n',);
            const extracted = extractStreamedCompletion({ bodyText: body, },);
            expect(extracted.finishReason,).toBe('length',);
            expect(extracted.text,).toBe('{"a":',);
          },
        },),
        it({
          name: 'READS the reason off a closing event that carries no delta FIELD at all, which the '
            + 'case beside this one cannot reach: that one sends an empty delta object, still a JSON '
            + 'record, while this sends none',
          fn: async () => {
            const body = [
              'data: {"choices":[{"delta":{"content":"y"}}]}',
              'data: {"choices":[{"finish_reason":"stop"}]}',
              'data: [DONE]',
              '',
            ].join('\n\n',);
            const extracted = extractStreamedCompletion({ bodyText: body, },);
            expect(extracted.finishReason,).toBe('stop',);
            expect(extracted.text,).toBe('y',);
          },
        },),
        it({
          name: 'KEEPS the LAST reason, since that event closes the stream',
          fn: async () => {
            const body = [
              'data: {"choices":[{"delta":{"content":"x"},"finish_reason":null}]}',
              'data: {"choices":[{"delta":{},"finish_reason":"stop"}]}',
              'data: [DONE]',
              '',
            ].join('\n\n',);
            expect(extractStreamedCompletion({ bodyText: body, },).finishReason,).toBe('stop',);
          },
        },),
        it({
          name: 'REPORTS no reason at all when the provider sends none',
          fn: async () => {
            // ABSENT RATHER THAN DEFAULTED. Reading a missing field as `stop`
            // would assert the very thing this exists to establish.
            const body = [
              'data: {"choices":[{"delta":{"content":"x"}}]}',
              'data: [DONE]',
              '',
            ].join('\n\n',);
            expect(extractStreamedCompletion({ bodyText: body, },).finishReason,).toBe(undefined,);
          },
        },),
        it({
          name: 'REPORTS no reason for one sent EMPTY or NULL, which the case beside '
            + 'this one cannot reach: that one omits the field, so a reader keyed on '
            + 'presence alone still passes it while handing an empty string on as '
            + 'though the model had named why it stopped',
          fn: async () => {
            /**
             Stream whose closing event carries the reason as an empty string.
             */
            const empty = [
              'data: {"choices":[{"delta":{"content":"x"},"finish_reason":""}]}',
              'data: [DONE]',
              '',
            ].join('\n\n',);

            /**
             Stream whose closing event carries it as null, which is what an
             OpenAI-compatible provider sends while it is still generating.
             */
            const nulled = [
              'data: {"choices":[{"delta":{"content":"x"},"finish_reason":null}]}',
              'data: [DONE]',
              '',
            ].join('\n\n',);

            expect([
              extractStreamedCompletion({ bodyText: empty, },).finishReason,
              extractStreamedCompletion({ bodyText: nulled, },).finishReason,
            ],).toEqual([
              undefined,
              undefined,
            ],);
          },
        },),
      ],
    },),
  ],
},);
