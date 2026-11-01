/**
 Tests for reading a provider failure off a success-status OpenRouter stream.

 WHAT THESE PIN is the difference between "the reply was cut off" and "the
 upstream failed with code 504": on 2026-09-04, 114 of 115 truncation retries
 in a day's runs were one endpoint's timeouts, and the log could not say so.
 The recorded frame is the shape a direct probe of that endpoint captured the
 same day, with the upstream's free text replaced.

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
  InStreamProviderError,
  InStreamRefusalError,
  openRouterStreamErrorOf,
  requireNoStreamError,
  STREAM_ERROR_ABSENT,
} from '../dist/final/node/index.mjs';

/**
 One chunk as the gateway frames it.

 @param chunk - object to frame

 @returns Framed event line

 @example
 ```ts
 const line = framed({ chunk: { provider: 'Sill', }, },);
 ```
 */
function framed({ chunk, }: { readonly chunk: Readonly<Record<string, unknown>>; },): string {
  return `data: ${JSON.stringify(chunk,)}\n\n`;
}

/**
 The failing stream's shape: one chunk carrying the error and the upstream's
 name, no terminator.
 */
const FAILED_STREAM = `: OPENROUTER PROCESSING\n\n${
  framed({
    chunk: {
      id: 'gen-cat',
      provider: 'Sill',
      model: 'cat/napper',
      object: 'chat.completion.chunk',
      created: 1,
      choices: [],
      error: {
        code: 504,
        message: 'the sill was busy',
        metadata: { error_type: 'timeout', },
      },
    },
  },)
}`;

/**
 A stream refused for what the request held: one chunk carrying a 4xx error.
 */
const REFUSED_STREAM = framed({
  chunk: {
    id: 'gen-cat',
    provider: 'Sill',
    choices: [],
    error: {
      code: 400,
      message: 'the sill refused',
      metadata: { error_type: 'invalid_request', },
    },
  },
},);

/**
 An ordinary stream: content and the terminator, no error.
 */
const WHOLE_STREAM = `${
  framed({
    chunk: {
      id: 'gen-cat',
      provider: 'Sill',
      choices: [{ index: 0, delta: { content: 'ready', }, finish_reason: 'stop', },],
    },
  },)
}data: [DONE]\n\n`;

/**
 The eighteenth class's shape (2026-09-08): reasoning, no content, a choice
 closed with `finish_reason: "error"` and no error object, then the
 terminator.
 */
const ERROR_FINISH_STREAM = `${
  framed({
    chunk: {
      id: 'gen-cat',
      provider: 'Sill',
      choices: [{
        index: 0,
        delta: {
          content: '',
          reasoning: 'is the sill warm',
        },
        finish_reason: null,
      },],
    },
  },)
}${
  framed({
    chunk: {
      id: 'gen-cat',
      provider: 'Sill',
      choices: [{
        index: 0,
        delta: { content: '', },
        finish_reason: 'error',
        native_finish_reason: 'upstream_error',
      },],
    },
  },)
}data: [DONE]\n\n`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: openRouterStreamErrorOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE CODE, THE KIND AND THE ENDPOINT off the error chunk, which is everything an '
            + 'operator needs to act on and nothing the upstream wrote',
          fn: async () => {
            expect(openRouterStreamErrorOf({ bodyText: FAILED_STREAM, },),).toEqual({
              found: true,
              code: 504,
              errorType: 'timeout',
              endpoint: 'Sill',
            },);
          },
        },),

        it({
          name: 'FINDS NOTHING IN A WHOLE STREAM, so an ordinary answer never reads as a failure',
          fn: async () => {
            expect(openRouterStreamErrorOf({ bodyText: WHOLE_STREAM, },),).toEqual(STREAM_ERROR_ABSENT,);
            expect(openRouterStreamErrorOf({ bodyText: '', },),).toEqual(STREAM_ERROR_ABSENT,);
          },
        },),

        it({
          name: 'NAMES WHAT THE WIRE LEFT OUT rather than inventing it, when the error object carries no '
            + 'code, no kind and the chunks no upstream',
          fn: async () => {
            /**
             An error chunk stripped to the bare object.
             */
            const bare = framed({ chunk: { error: { message: 'the sill was busy', }, }, },);

            expect(openRouterStreamErrorOf({ bodyText: bare, },),).toEqual({
              found: true,
              code: 'unnamed',
              errorType: 'unnamed',
              endpoint: 'unnamed',
            },);
          },
        },),

        it({
          name: 'READS A CHOICE THAT STOPPED ON AN ERROR FINISH as the failure it is, the eighteenth class, '
            + 'naming the native reason as the kind and no code, when no chunk carried an error object',
          fn: async () => {
            expect(openRouterStreamErrorOf({ bodyText: ERROR_FINISH_STREAM, },),).toEqual({
              found: true,
              code: 'unnamed',
              errorType: 'upstream_error',
              endpoint: 'Sill',
            },);
            /**
             The same closing choice with no native reason and no upstream named.
             */
            const unnamed = `${
          framed({
            chunk: {
              choices: [{
                index: 0,
                delta: { content: '', },
                finish_reason: 'error',
              },],
            },
          },)
        }data: [DONE]\n\n`;
            expect(openRouterStreamErrorOf({ bodyText: unnamed, },),).toEqual({
              found: true,
              code: 'unnamed',
              errorType: 'error-finish',
              endpoint: 'unnamed',
            },);
          },
        },),

        it({
          name: 'TREATS AN ARRAY error FIELD AS NO ERROR AT ALL, falling through to the finish-reason '
            + 'check rather than reporting a found failure with every field unnamed, which reading an '
            + 'array chunk field as a record produces today (ledger B92)',
          fn: async () => {
            /**
             A chunk whose error field is a bare array rather than a record.
             */
            const arrayErrorStream = framed({
              chunk: {
                provider: 'Sill',
                choices: [],
                error: [504,],
              },
            },);

            expect(openRouterStreamErrorOf({ bodyText: arrayErrorStream, },),).toEqual(STREAM_ERROR_ABSENT,);
          },
        },),
      ],
    },),

    describe({
      name: requireNoStreamError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'THROWS THE PROVIDER FAILURE CLASS on a failed stream, carrying the code and the endpoint '
            + 'as fields a reader can act on, and a message that names them and nothing else',
          fn: async () => {
            /**
             What the failed stream produces.
             */
            let thrown: unknown;
            try {
              requireNoStreamError({ provider: 'openrouter', bodyText: FAILED_STREAM, },);
            } catch (error) {
              thrown = error;
            }
            expect(thrown instanceof InStreamProviderError,).toBe(true,);
            expect((thrown as InStreamProviderError).code,).toBe(504,);
            expect((thrown as InStreamProviderError).endpoint,).toBe('Sill',);
            expect((thrown as InStreamProviderError).message,).toContain('code 504',);
            expect((thrown as InStreamProviderError).message,).toContain('type timeout',);
            expect((thrown as InStreamProviderError).message,).toContain('served by Sill',);
            expect((thrown as InStreamProviderError).message.includes('busy',),).toBe(false,);
          },
        },),

        it({
          name: 'CARRIES THE PROVIDER THE CALLER NAMED on the refusal a 4xx error chunk raises, since the chunk '
            + 'itself cannot say whether OpenRouter or another route spoke it',
          fn: async () => {
            /**
             What the refused stream produces on each of the two routes that speak this chunk shape.
             */
            const thrown = (['openrouter', 'bedrock',] as const).map(function refuseAs(provider,): unknown {
              try {
                requireNoStreamError({ provider, bodyText: REFUSED_STREAM, },);
              } catch (error) {
                return error;
              }
              return undefined;
            },);
            expect(thrown.map(function providerOf(error,): unknown {
              return (error instanceof InStreamRefusalError) ? error.provider : error;
            },),).toStrictEqual(['openrouter', 'bedrock',],);
          },
        },),

        it({
          name: 'LETS A WHOLE STREAM THROUGH, since the terminator check that follows is the one that '
            + 'judges framing',
          fn: async () => {
            // A throw here fails the test on its own; the assertion records the
            // reading the check was made from.
            requireNoStreamError({ provider: 'openrouter', bodyText: WHOLE_STREAM, },);
            expect(openRouterStreamErrorOf({ bodyText: WHOLE_STREAM, },).found,).toBe(false,);
          },
        },),

        it({
          name: 'THROWS THE SAME FAILURE CLASS on an error finish, so the call rides the retry ladder '
            + 'under its own name instead of reaching the reply ladder as an empty answer',
          fn: async () => {
            /**
             What the error-finish stream produces.
             */
            let thrown: unknown;
            try {
              requireNoStreamError({ provider: 'openrouter', bodyText: ERROR_FINISH_STREAM, },);
            } catch (error) {
              thrown = error;
            }
            expect(thrown instanceof InStreamProviderError,).toBe(true,);
            expect((thrown as InStreamProviderError).code,).toBe('unnamed',);
            expect((thrown as InStreamProviderError).endpoint,).toBe('Sill',);
            expect((thrown as InStreamProviderError).message,).toContain('type upstream_error',);
            expect((thrown as InStreamProviderError).message,).toContain('served by Sill',);
          },
        },),
      ],
    },),
  ],
},);
