/**
 Read the transport-failure texts back from the two installed adapters, in memory.
 @module
 */

import { streamSimple as streamCompletions, } from '@earendil-works/pi-ai/api/openai-completions';
import { streamSimple as streamResponses, } from '@earendil-works/pi-ai/api/openai-responses';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  collect,
  MEMBER_KEY,
  PHYSICAL_MODELS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';
import { TRANSPORT_FAILURES, } from './transport-failure-fixture.ts';

/**
 Fixture models standing for the two adapter protocols this pool dispatches through: the Responses
 adapter's OpenAI representative and the Completions adapter's representative.
 */
const [OPENAI, QWEN,] = PHYSICAL_MODELS;

/**
 The adapters, named by the API each one implements so a failure names the protocol it arrived over.
 */
const ADAPTERS = ['openai-completions', 'openai-responses',] as const;

await describe({
  name: 'installed adapter transport failure text',
  children: [
    ...ADAPTERS.flatMap(function adapter(api,) {
      return Object.entries(TRANSPORT_FAILURES,)
        .map(function failure([name, failure,],) {
          return it({
            name: `reports ${name} (${failure.code}) over ${api}`,
            fn: async function transportFailure() {
              /**
               Physical model whose protocol this case drives.
               */
              const physical = api === 'openai-responses' ? OPENAI : QWEN;
              if (physical === undefined)
                throw new Error(`No fixture model for ${api}`,);
              /**
               Responses the response hook saw, which no transport failure may produce.
               */
              const statuses: number[] = [];
              const stream = api === 'openai-responses'
                ? streamResponses(
                  { ...physical, api: 'openai-responses', baseUrl: 'http://127.0.0.1:0/v1', },
                  TRANSCRIPT,
                  {
                    apiKey: MEMBER_KEY,
                    maxRetries: 0,
                    fetch: failure.respond,
                    onResponse(response,) { statuses.push(response.status,); },
                  },
                )
                : streamCompletions(
                  { ...physical, api: 'openai-completions', baseUrl: 'http://127.0.0.1:0/v1', },
                  TRANSCRIPT,
                  {
                    apiKey: MEMBER_KEY,
                    maxRetries: 0,
                    fetch: failure.respond,
                    onResponse(response,) { statuses.push(response.status,); },
                  },
                );
              const { result, } = await collect(stream,);
              expect(result.stopReason,).toBe('error',);
              expect(result.errorMessage,).toBe(failure.text,);
              // Nothing was shown to Pi: the pool may still answer this request elsewhere.
              expect(result.content,).toEqual([],);
              if (failure.code === 'UND_ERR_SOCKET') {
                // A reset after a successful header is the one failure whose member did answer,
                // with a response whose body died before a single token.
                expect(statuses,).toEqual([200,],);
                return;
              }
              expect(statuses,).toEqual([],);
            },
          },);
        },);
    },),
  ],
},);
