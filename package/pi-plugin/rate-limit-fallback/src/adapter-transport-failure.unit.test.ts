/**
 Read the transport-failure texts back from the two installed adapters, in memory.
 @module
 */

import type { AssistantMessage, } from '@earendil-works/pi-ai';
import { streamSimple as streamCompletions, } from '@earendil-works/pi-ai/api/openai-completions';
import { streamSimple as streamResponses, } from '@earendil-works/pi-ai/api/openai-responses';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  collect,
  MEMBER_KEY,
  PHYSICAL_MODELS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';
import {
  type TransportFailure,
  TRANSPORT_FAILURES,
} from './transport-failure-fixture.ts';

/**
 Fixture models standing for the two adapter protocols this pool dispatches through: the Responses
 adapter's OpenAI representative and the Completions adapter's representative.
 */
const [OPENAI, QWEN,] = PHYSICAL_MODELS;

/**
 The two adapters, named by the API each one implements so a failure names the protocol it arrived over.
 */
const ADAPTERS = ['openai-completions', 'openai-responses',] as const;

/**
 One adapter protocol a case drives.
 */
type AdapterApi = (typeof ADAPTERS)[number];

/**
 Drive one failure through the installed adapter that speaks this protocol.

 @param api - protocol whose adapter reports the failure

 @param transport - failure the member's provider stands behind

 @returns terminal message Pi's adapter produced, and the statuses its response hook saw
 */
async function adapterFailure({
  api,
  transport,
}: {
  readonly api: AdapterApi;
  readonly transport: TransportFailure;
},): Promise<{ readonly result: AssistantMessage; readonly statuses: readonly number[]; }> {
  /**
   Physical model whose protocol this case drives.
   */
  const physical = api === 'openai-responses' ? OPENAI : QWEN;
  if (physical === undefined)
    throw new Error(`No fixture model for ${api}`,);
  /**
   Statuses the response hook saw; a refusal the adapter never answered exposes none.
   */
  const statuses: number[] = [];
  /**
   Model record the adapter streams for, at a base URL no request reaches.
   */
  const model = {
    ...physical,
    api,
    baseUrl: 'http://127.0.0.1:0/v1',
  };
  /**
   Options every adapter case shares: one attempt, and the failing transport.
   */
  const options = {
    apiKey: MEMBER_KEY,
    maxRetries: 0,
    fetch: transport.respond,
    onResponse(response: { readonly status: number; },) { statuses.push(response.status,); },
  };
  const stream = api === 'openai-responses'
    ? streamResponses(
      { ...model, api: 'openai-responses', },
      TRANSCRIPT,
      options,
    )
    : streamCompletions(
      { ...model, api: 'openai-completions', },
      TRANSCRIPT,
      options,
    );
  const { result, } = await collect(stream,);
  return {
    result,
    statuses,
  };
}

await describe({
  name: 'installed adapter transport failure text',
  children: ADAPTERS.map(function adapterSuite(api,) {
    return describe({
      name: `over ${api}`,
      children: TRANSPORT_FAILURES.map(function caseFor(transport,) {
        return it({
          name: `reports ${transport.name} (${transport.code})`,
          fn: async function transportFailure() {
            const { result, statuses, } = await adapterFailure({
              api,
              transport,
            },);
            expect(result.stopReason,).toBe('error',);
            expect(result.errorMessage,).toBe(transport.text,);
            // A reset after a successful header is the one failure whose member did answer, with a
            // response whose body died before a single token; every other failure shows neither.
            expect(statuses,).toEqual(transport.code === 'UND_ERR_SOCKET' ? [200,] : [],);
          },
        },);
      },),
    },);
  },),
},);
