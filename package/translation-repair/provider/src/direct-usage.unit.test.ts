/**
 Tests for the readers of what DeepSeek's stream says besides its text: the cache split of the usage block and the
 system fingerprint. Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  deepSeekCacheSplitOf,
  NOT_REPORTED,
  systemFingerprintOf,
} from '../dist/final/node/index.mjs';

/**
 One event line for a chunk.

 @param chunk - chunk's members

 @returns Event text, blank-line terminated

 @example
 ```ts
 const line = eventOf({ chunk: { choices: [], }, },);
 ```
 */
function eventOf({ chunk, }: { readonly chunk: Readonly<Record<string, unknown>>; },): string {
  return `data: ${JSON.stringify(chunk,)}\n\n`;
}

await describe({
  name: 'direct usage',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: deepSeekCacheSplitOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the last usage block\'s cache split and ignores counts that are not whole numbers',
          fn: async () => {
            const stream = [
              eventOf({
                chunk: { choices: [], usage: { prompt_cache_hit_tokens: 1, prompt_cache_miss_tokens: 2, }, },
              },),
              eventOf({
                chunk: { choices: [], usage: { prompt_cache_hit_tokens: 40, prompt_cache_miss_tokens: 60, }, },
              },),
            ].join('',);
            expect(deepSeekCacheSplitOf({ bodyText: stream, },),).toEqual({ hitTokens: 40, missTokens: 60, },);
            const odd = eventOf({
              chunk: { choices: [], usage: { prompt_cache_hit_tokens: -3, prompt_cache_miss_tokens: 1.5, }, },
            },);
            expect(deepSeekCacheSplitOf({ bodyText: odd, },),).toEqual({},);
            expect(deepSeekCacheSplitOf({ bodyText: 'data: [DONE]\n\n', },),).toEqual({},);
          },
        },),
        it({
          name: 'REFUSES a count written as text, since a price taken from "40" would be a guess at what the '
            + 'provider meant',
          fn: async () => {
            const typed = eventOf({
              chunk: { choices: [], usage: { prompt_cache_hit_tokens: '40', prompt_cache_miss_tokens: 60, }, },
            },);
            expect(deepSeekCacheSplitOf({ bodyText: typed, },),).toEqual({ missTokens: 60, },);
          },
        },),
        it({
          name: 'KEEPS the split of an earlier usage block when the last chunk carries none',
          fn: async () => {
            const stream = [
              eventOf({
                chunk: { choices: [], usage: { prompt_cache_hit_tokens: 40, prompt_cache_miss_tokens: 60, }, },
              },),
              eventOf({ chunk: { choices: [{ index: 0, delta: { content: '', }, finish_reason: 'stop', },], }, },),
              'data: [DONE]\n\n',
            ].join('',);
            expect(deepSeekCacheSplitOf({ bodyText: stream, },),).toEqual({ hitTokens: 40, missTokens: 60, },);
          },
        },),
      ],
    },),
    describe({
      name: systemFingerprintOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the last non-empty system_fingerprint, and nothing where no chunk carried one',
          fn: async () => {
            const stream = [
              eventOf({ chunk: { system_fingerprint: 'fp_a', choices: [], }, },),
              eventOf({ chunk: { system_fingerprint: '', choices: [], }, },),
              eventOf({ chunk: { system_fingerprint: 'fp_b', choices: [], }, },),
            ].join('',);
            expect(systemFingerprintOf({ bodyText: stream, },),).toBe('fp_b',);
            expect(systemFingerprintOf({ bodyText: 'data: [DONE]\n\n', },),).toBe(NOT_REPORTED,);
          },
        },),
      ],
    },),
  ],
},);
