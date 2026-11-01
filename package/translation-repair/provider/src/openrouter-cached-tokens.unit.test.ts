/**
 Tests for reading cached prompt tokens off an OpenRouter stream.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { openRouterCachedTokensOf, } from '../dist/final/node/index.mjs';

/**
 One stream chunk carrying the given usage block.

 @param usage - usage block as the gateway sends it

 @returns Event line, newline-terminated

 @example
 ```ts
 const line = withUsage({ usage: { prompt_tokens: 10, }, },);
 ```
 */
function withUsage({ usage, }: { readonly usage: Readonly<Record<string, unknown>>; },): string {
  return `data: ${JSON.stringify({ choices: [{ delta: { content: '', }, },], usage, },)}\n\n`;
}

await describe({
  name: openRouterCachedTokensOf.name,
  children: [
    it({
      name: 'READS cached_tokens off the final chunk\'s prompt_tokens_details, ignoring chunks without '
        + 'usage and the [DONE] terminator',
      fn: async () => {
        expect(openRouterCachedTokensOf({
          bodyText: `data: {"choices":[{"delta":{"content":"x"}}]}\n\n${withUsage({ usage: { prompt_tokens: 4_000, completion_tokens: 12, prompt_tokens_details: { cached_tokens: 3_072, }, }, },)}data: [DONE]\n\n`,
        },),).toBe(3_072,);
      },
    },),

    it({
      name: 'REPORTS the named absence when the usage carries no detail block, a non-number, or no '
        + 'usage at all',
      fn: async () => {
        expect(openRouterCachedTokensOf({ bodyText: withUsage({ usage: { prompt_tokens: 4_000, }, },), },),).toBe('unreported',);
        expect(openRouterCachedTokensOf({
          bodyText: withUsage({ usage: { prompt_tokens_details: { cached_tokens: 'many', }, }, },),
        },),).toBe('unreported',);
        expect(openRouterCachedTokensOf({ bodyText: 'data: [DONE]\n\n', },),).toBe('unreported',);
      },
    },),
    it({
      name: 'SKIPS a chunk that is not a record and a cache count that is not a safe integer, since '
        + 'neither states a count, and KEEPS the count a chunk before them stated (ledger T8)',
      fn: async () => {
        expect(openRouterCachedTokensOf({
          bodyText: 'data: 5\n\ndata: [DONE]\n\n',
        },),).toBe('unreported',);
        expect(openRouterCachedTokensOf({
          bodyText: 'data: {"usage":{"prompt_tokens_details":{"cached_tokens":1.5}}}\n\ndata: [DONE]\n\n',
        },),).toBe('unreported',);
        // BOTH SKIPS IN ONE STREAM, behind a count that must survive them: a
        // reader taking the last chunk's field whatever it held would answer
        // with the fraction, and one stopping at the bare number with nothing.
        expect(openRouterCachedTokensOf({
          bodyText: 'data: {"usage":{"prompt_tokens_details":{"cached_tokens":3072}}}\n\n'
            + 'data: 5\n\n'
            + 'data: {"usage":{"prompt_tokens_details":{"cached_tokens":1.5}}}\n\n'
            + 'data: [DONE]\n\n',
        },),).toBe(3_072,);
      },
    },),
  ],
},);
