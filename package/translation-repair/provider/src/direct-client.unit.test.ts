/**
 Tests for the chat body DeepSeek's and Xiaomi's clients share, run over both routes: the stream that stops before its
 terminator and the measured completion cap.

 NOTHING HERE REACHES A PROVIDER: every transport is an in-memory script, and every key is invented. The route-specific
 cases sit beside their clients in `deepseek-client.unit.test.ts` and `xiaomi-client.unit.test.ts`. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  exchangeFailureLogText,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  createDeepSeekClient,
  createXiaomiClient,
  type ModelCaller,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_OPENROUTER_ONLY_CHECKER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  askCat,
  DEEPSEEK_USAGE_CHUNK,
  errorChainText,
  firstBodyOf,
  ONE_QUICK_RETRY,
  scripted,
  streamOf,
  XIAOMI_USAGE_CHUNK,
} from './direct-client.test-fixture.ts';

/**
 One route's client over a script, and the model it is granted for.

 @example
 ```ts
 const route: RouteUnderTest = { name: 'deepseek', make, modelId, usageChunk, };
 ```
 */
type RouteUnderTest = {
  /**
   Route's name, for the failure message.
   */
  readonly name: string;

  /**
   Builds the route's client over a recorded transport.
   */
  readonly make: (
    args: Pick<Parameters<typeof createXiaomiClient>[0], 'transport' | 'apiKey'>,
  ) => Pick<ModelCaller, 'chatText'>;

  /**
   The one roster model the route's key is granted for.
   */
  readonly modelId: typeof SEAT_HYPER_OPENROUTER_UNMEASURED | typeof SEAT_OPENROUTER_ONLY_CHECKER;

  /**
   The closing usage chunk the route's provider writes.
   */
  readonly usageChunk: string;
};

/**
 Key every case here sends. Its last four units are what a provider's refusal text might echo back.
 */
const CAT_KEY = 'cat-key-whisker-9f3c7a-Qx4d';

/**
 Both direct routes, each over a client that retries once a millisecond later.
 */
const ROUTES: readonly RouteUnderTest[] = [
  {
    name: 'deepseek',
    make: ({ transport, apiKey, },) => createDeepSeekClient({ apiKey, transport, retryPolicy: ONE_QUICK_RETRY, },),
    modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
    usageChunk: DEEPSEEK_USAGE_CHUNK,
  },
  {
    name: 'xiaomi',
    make: ({ transport, apiKey, },) => createXiaomiClient({ apiKey, transport, retryPolicy: ONE_QUICK_RETRY, },),
    modelId: SEAT_OPENROUTER_ONLY_CHECKER,
    usageChunk: XIAOMI_USAGE_CHUNK,
  },
];

await describe({
  name: 'the chat body both direct routes share',
  // SEQUENTIAL: the run spend meter is process-wide, so cases that read its change must not overlap.
  concurrency: 1,
  children: ROUTES.map((route,) =>
    describe({
      name: route.name,
      concurrency: 1,
      children: [
        it({
          name: 'RETRIES a stream that stopped before [DONE] as the transport failure it is, and answers from the '
            + 'whole one',
          fn: async () => {
            const { transport, exchanges, } = scripted({
              replies: [
                { status: 200, bodyText: streamOf({ usageChunk: route.usageChunk, done: false, }), },
                { status: 200, bodyText: streamOf({ usageChunk: route.usageChunk, done: true, }), },
              ],
            },);

            const reply = await askCat({
              client: route.make({ transport, apiKey: CAT_KEY, },),
              modelId: route.modelId,
            },);

            expect(reply.text,).toBe('{"spot":"sunbeam"}',);
            expect(exchanges.length,).toBe(2,);
          },
        },),

        it({
          name: 'DROPS the provider\'s words from a 401, which may echo a short tail of the key that no mask of the '
            + 'whole '
            + 'key would find, and still fails as a 401 after one request',
          fn: async () => {
            const { transport, exchanges, } = scripted({
              replies: [{
                status: 401,
                bodyText: '{"error":{"message":"Authentication Fails, Your api key: ****Qx4d is invalid"}}',
              },],
            },);

            const failure: unknown = await rejectionOf(
              () => askCat({ client: route.make({ transport, apiKey: CAT_KEY, },), modelId: route.modelId, },),
            );

            expect(failure,).toBeInstanceOf(ProviderHttpError,);
            expect((failure as ProviderHttpError).status,).toBe(401,);
            expect(exchangeFailureLogText({ error: failure, },),).not.toContain('Qx4d',);
            expect(errorChainText({ error: failure, },),).not.toContain('Qx4d',);
            expect(exchanges.length,).toBe(1,);
          },
        },),

        it({
          name: 'SENDS the measured completion cap as a whole number of tokens on every call',
          fn: async () => {
            const { transport, exchanges, } = scripted({
              replies: [{ status: 200, bodyText: streamOf({ usageChunk: route.usageChunk, done: true, }), },],
            },);

            await askCat({ client: route.make({ transport, apiKey: CAT_KEY, },), modelId: route.modelId, },);

            const { max_tokens: cap, } = firstBodyOf({ exchanges, },);
            expect(Number.isSafeInteger(cap,),).toBe(true,);
            expect(cap as number,).toBeGreaterThan(0,);
          },
        },),
      ],
    },)
  ),
},);
