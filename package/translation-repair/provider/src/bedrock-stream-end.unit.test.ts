/**
 Tests for how a Bedrock stream is known to be whole: the sentinel on the
 Gemma route, the usage chunk on the gpt-oss route, and the sentinel supplied
 to the shared reader where the route sends none.

 Streams are shaped as the probes of 2026-09-07 captured them; the words are
 cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { MalformedCompletionError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  extractStreamedCompletion,
  InStreamProviderError,
  InStreamRefusalError,
  requireBedrockStreamEnd,
  withDoneSentinel,
} from '../dist/final/node/index.mjs';

/**
 One content chunk.
 */
const CONTENT_CHUNK = 'data: {"choices":[{"index":0,"delta":{"content":"{\\"spot\\":\\"sunbeam\\"}"},"finish_reason":null}]}\n\n';

/**
 Chunk carrying the usage block and no choices, as the gpt-oss route ends.
 */
const USAGE_CHUNK = 'data: {"choices":[],"usage":{"prompt_tokens":98,"completion_tokens":75,"total_tokens":173}}\n\n';

/**
 Gemma route stream: content, usage, sentinel.
 */
const GEMMA_STREAM = `${CONTENT_CHUNK}${USAGE_CHUNK}data: [DONE]\n\n`;

/**
 Error chunk an OpenAI-shaped stream ends on when the call failed.

 @param code - status the chunk reports

 @returns Framed error event

 @example
 ```ts
 const chunk = errorChunk({ code: 400, },);
 ```
 */
function errorChunk({ code, }: { readonly code: number; },): string {
  return `data: ${JSON.stringify({ error: { code, message: 'the sill refused', metadata: { error_type: 'invalid_request', }, }, },)}\n\n`;
}

/**
 gpt-oss route stream: content, usage, nothing more.
 */
const GPT_OSS_STREAM = `${CONTENT_CHUNK}${USAGE_CHUNK}`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: requireBedrockStreamEnd.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a sentinel-ended stream on the sentinel route and REFUSES one cut off before it',
          fn: async () => {
            expect(function whole(): void {
              requireBedrockStreamEnd({
                bodyText: GEMMA_STREAM,
                streamEnd: 'done-sentinel',
              },);
            },).not.toThrow();
            expect(function cut(): void {
              requireBedrockStreamEnd({
                bodyText: GPT_OSS_STREAM,
                streamEnd: 'done-sentinel',
              },);
            },).toThrow(MalformedCompletionError,);
          },
        },),

        it({
          name: 'ACCEPTS a stream ending on its usage chunk on the usage route and REFUSES one that never '
            + 'reached it, since the gpt-oss route sends no sentinel (measured 2026-09-07)',
          fn: async () => {
            expect(function whole(): void {
              requireBedrockStreamEnd({
                bodyText: GPT_OSS_STREAM,
                streamEnd: 'usage-chunk',
              },);
            },).not.toThrow();
            expect(function cut(): void {
              requireBedrockStreamEnd({
                bodyText: CONTENT_CHUNK,
                streamEnd: 'usage-chunk',
              },);
            },).toThrow(MalformedCompletionError,);
          },
        },),

        it({
          name: 'NAMES BEDROCK AS THE PROVIDER of the refusal an error chunk raises, whichever way the stream ends',
          fn: async () => {
            /**
             What a 4xx error chunk raised on each route.
             */
            const raised = (['done-sentinel', 'usage-chunk',] as const).map(function refuseOn(streamEnd,): unknown {
              try {
                requireBedrockStreamEnd({
                  bodyText: `${CONTENT_CHUNK}${errorChunk({ code: 400, },)}`,
                  streamEnd,
                },);
              } catch (error) {
                return error;
              }
              return undefined;
            },);
            expect(raised.map(function providerOf(error,): unknown {
              return (error instanceof InStreamRefusalError) ? error.provider : error;
            },),).toStrictEqual(['bedrock', 'bedrock',],);
          },
        },),

        it({
          name: 'NAMES AN ERROR CHUNK on either route by its cause, a refusal for a 4xx code and a provider '
            + 'failure for any other, where both read as a stream cut off before its end',
          fn: async () => {
            /**
             What each route and code ended with.
             */
            const ended: Record<string, string> = {};
            for (const streamEnd of ['done-sentinel', 'usage-chunk',] as const) {
              for (const code of [400, 503,]) {
                try {
                  requireBedrockStreamEnd({
                    bodyText: `${CONTENT_CHUNK}${errorChunk({ code, },)}`,
                    streamEnd,
                  },);
                  ended[`${streamEnd} ${String(code,)}`] = 'accepted';
                } catch (error) {
                  ended[`${streamEnd} ${String(code,)}`] = (error instanceof InStreamRefusalError)
                    ? 'refusal'
                    : (error instanceof InStreamProviderError)
                      ? 'provider failure'
                      : (error instanceof MalformedCompletionError)
                        ? 'cut off'
                        : String(error,);
                }
              }
            }
            expect(ended,).toStrictEqual({
              'done-sentinel 400': 'refusal',
              'done-sentinel 503': 'provider failure',
              'usage-chunk 400': 'refusal',
              'usage-chunk 503': 'provider failure',
            },);
          },
        },),

        it({
          name: 'IGNORES a line that is not JSON while looking for the usage chunk, as a comment line or a '
            + 'torn frame is not one',
          fn: async () => {
            expect(function torn(): void {
              requireBedrockStreamEnd({
                bodyText: `: keepalive\n\ndata: {"choices":[{"index":0,"de\n\n${USAGE_CHUNK}`,
                streamEnd: 'usage-chunk',
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'ACCEPTS A USAGE CHUNK SENT IN THE TIGHT FORM, `data:{...}` being the same message as '
            + '`data: {...}` (audit area six: the prefix was spelled with its space)',
          fn: async () => {
            expect(function tight(): void {
              requireBedrockStreamEnd({
                bodyText: `${CONTENT_CHUNK}${USAGE_CHUNK.replace('data: ', 'data:',)}`,
                streamEnd: 'usage-chunk',
              },);
            },).not.toThrow();
          },
        },),
      ],
    },),

    describe({
      name: withDoneSentinel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SUPPLIES THE SENTINEL on the usage route so the shared reader folds the stream, and leaves '
            + 'a sentinel route\'s stream as it came',
          fn: async () => {
            expect(withDoneSentinel({
              bodyText: GEMMA_STREAM,
              streamEnd: 'done-sentinel',
            },),).toBe(GEMMA_STREAM,);

            /**
             gpt-oss stream as the shared reader receives it.
             */
            const supplied = withDoneSentinel({
              bodyText: GPT_OSS_STREAM,
              streamEnd: 'usage-chunk',
            },);
            expect(supplied.endsWith('data: [DONE]\n',),).toBe(true,);

            /**
             What the shared reader makes of it.
             */
            const extracted = extractStreamedCompletion({ bodyText: supplied, },);
            expect(extracted.text,).toBe('{"spot":"sunbeam"}',);
            expect(extracted.usage?.completion_tokens,).toBe(75,);
          },
        },),
      ],
    },),
  ],
},);
