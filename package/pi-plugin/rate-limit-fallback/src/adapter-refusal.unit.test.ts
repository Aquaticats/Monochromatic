/**
 Compare test fixture text with the installed adapters using an entirely in-memory transport.
 @module
 */

import { streamSimple as streamCompletions, } from '@earendil-works/pi-ai/api/openai-completions';
import { streamSimple as streamResponses, } from '@earendil-works/pi-ai/api/openai-responses';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { POLICY_STOP_TEXT, policyStopStream, } from './policy-stop-fixture.ts';
import {
  collect,
  MEMBER_KEY,
  PHYSICAL_MODELS,
  rateLimitRefusal,
  RATE_LIMIT_STATUS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/** The fixture representatives for the two adapters used by this pool. */
const [OPENAI, QWEN,] = PHYSICAL_MODELS;

/** Body message common to both synthetic HTTP refusals. */
const MESSAGE = 'Request allowance exhausted for the current window';

await describe({
  name: 'installed adapter refusal text',
  children: [
    ...['openai-responses', 'openai-completions',].map(function adapter(api,) {
      return it({
        name: `matches the ${api} 429 fixture without a response hook`,
        fn: async function actualAdapterRefusal() {
          const counts = { fetch: 0, response: 0, };
          const options = {
            apiKey: MEMBER_KEY,
            maxRetries: 0,
            async fetch() {
              counts.fetch += 1;
              return Response.json({ error: { message: MESSAGE, }, }, { status: RATE_LIMIT_STATUS, },);
            },
            onResponse() { counts.response += 1; },
          };
          const isResponses = api === 'openai-responses';
          const physical = isResponses ? OPENAI : QWEN;
          if (physical === undefined)
            throw new Error(`No fixture model for ${api}`,);
          const model = { ...physical, baseUrl: 'http://127.0.0.1:0/v1', };
          const stream = isResponses
            ? streamResponses({ ...model, api: 'openai-responses', }, TRANSCRIPT, options,)
            : streamCompletions({ ...model, api: 'openai-completions', }, TRANSCRIPT, options,);
          const { result, } = await collect(stream,);
          expect(counts,).toEqual({ fetch: 1, response: 0, },);
          expect(result.errorMessage,).toBe(rateLimitRefusal({ model: physical, message: MESSAGE, },),);
        },
      },);
    },),
    it({
      name: 'reports OpenAI’s policy stop verbatim, with no status prefix',
      fn: async function policyStop() {
        if (OPENAI === undefined)
          throw new Error('No fixture model for openai-responses',);
        const { result, } = await collect(policyStopStream({
          model: { ...OPENAI, baseUrl: 'http://127.0.0.1:0/v1', },
          context: TRANSCRIPT,
        },),);
        expect(result.stopReason,).toBe('error',);
        expect(result.errorMessage,).toBe(POLICY_STOP_TEXT,);
      },
    },),
  ],
},);
