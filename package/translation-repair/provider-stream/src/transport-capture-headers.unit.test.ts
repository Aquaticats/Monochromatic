/**
 Tests the response headers a caller can ask the fetch-backed transport to hand back. The Claude route needs two
 kinds: `retry-after`, which Anthropic's 429 and 529 replies carry and the retry ladder honours, and the
 `anthropic-ratelimit-*` family, which the client logs once on the first reply of a run. A reply that was not asked
 for headers keeps the shape it always had, so every older caller and test reads it unchanged.

 The global fetch is stubbed per test; children run sequentially so stubs never overlap. Fixtures are cat-themed
 invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { fetchTransport, } from '../dist/final/node/index.mjs';

/**
 Exchange every case sends, asking for the headers it is given.

 @param captureHeaders - header names, or prefixes ending in an asterisk, the reply should carry

 @returns Exchange the transport performs

 @example
 ```ts
 const exchange = exchangeAsking({ captureHeaders: ['retry-after',], },);
 ```
 */
function exchangeAsking(
  { captureHeaders, }: { readonly captureHeaders?: readonly string[]; },
) {
  return {
    url: 'https://example.org/cat-chat',
    label: 'hf:whiskers',
    method: 'POST' as const,
    headers: {},
    bodyJson: '{}',
    signal: new AbortController().signal,
    // Conditional spread keeps the member absent instead of undefined.
    ...((captureHeaders === undefined) ? {} : { captureHeaders, }),
  };
}

await describe({
  name: 'captured response headers',
  concurrency: 1,
  children: [
    it({
      name: 'HANDS BACK THE HEADERS ASKED FOR, by exact name or by prefix, in lower case, and no others',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            'upstream napping',
            {
              status: 429,
              headers: {
                'Retry-After': '3',
                'anthropic-ratelimit-requests-remaining': '998',
                'Anthropic-RateLimit-Input-Tokens-Limit': '2000000',
                'content-type': 'text/plain',
                'x-cat': 'purr',
              },
            },
          ),);

        const reply = await fetchTransport(exchangeAsking({
          captureHeaders: ['retry-after', 'anthropic-ratelimit-*',],
        },),);

        expect(reply.status,).toBe(429,);
        expect(reply.headers,).toEqual({
          'retry-after': '3',
          'anthropic-ratelimit-requests-remaining': '998',
          'anthropic-ratelimit-input-tokens-limit': '2000000',
        },);
      },
    },),

    it({
      name: 'LEAVES THE REPLY AS IT WAS when no header was asked for, whatever the provider sent',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            '{}',
            {
              status: 200,
              headers: { 'retry-after': '3', },
            },
          ),);

        expect(
          await fetchTransport(exchangeAsking({}),),
        ).toEqual({
          status: 200,
          bodyText: '{}',
        },);
      },
    },),

    it({
      name: 'CARRIES NO HEADERS MEMBER when none of the headers asked for came back',
      fn: async ctx => {
        ctx.sinon
          .stub(
          globalThis,
          'fetch',
          )
          .resolves(new Response(
            '{}',
            { status: 200, },
          ),);

        expect(
          await fetchTransport(exchangeAsking({ captureHeaders: ['retry-after',], },),),
        ).toEqual({
          status: 200,
          bodyText: '{}',
        },);
      },
    },),
  ],
},);
