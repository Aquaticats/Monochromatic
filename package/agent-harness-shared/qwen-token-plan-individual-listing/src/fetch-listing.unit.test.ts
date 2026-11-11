import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fetchQwenTokenPlanIndividualListing,
  QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS,
  QwenTokenPlanListingError,
} from '../dist/final/node/index.mjs';

/**
 Base URL the cases point the fetch at; the fetch appends the listing path.
 */
const BASE_URL = 'https://example.invalid/compatible-mode/v1';

/**
 Bearer key the cases hand the fetch, standing in for the credential a
 harness resolves.
 */
const API_KEY = 'test-key';

await describe({
  name: fetchQwenTokenPlanIndividualListing.name,
  children: [
    //region Success

    it({
      name: 'returns the listing entries the endpoint sent, narrowed to id and created',
      fn: async () => {
        const listing = await fetchQwenTokenPlanIndividualListing({
          baseUrl: BASE_URL,
          apiKey: API_KEY,
          fetcher: async (options) => {
            expect(options.baseUrl).toBe(BASE_URL);
            expect(options.apiKey).toBe(API_KEY);
            expect(options.signal).toBeInstanceOf(AbortSignal);
            return {
              ok: true,
              status: 200,
              json: async () => ({
                data: [
                  {
                    id: 'a',
                    created: 5,
                    owned_by: 'system',
                    object: 'model',
                  },
                  { id: 'b', },
                  {
                    id: '',
                    created: 1,
                  },
                  { created: 2, },
                  5,
                  null,
                  'x',
                  {
                    id: 'c',
                    created: 2.5,
                  },
                ],
              }),
            };
          },
        });
        expect(listing).toEqual([
          {
            id: 'a',
            created: 5,
          },
          { id: 'b', },
          { id: 'c', },
        ]);
      },
    }),

    it({
      name: 'reads the listing endpoint with the bearer key through the default fetcher',
      fn: async ctx => {
        const fake = ctx.sinon.stub(globalThis, 'fetch').callsFake(
          async () => new Response('{"data": []}', { status: 200, }),
        );
        const listing = await fetchQwenTokenPlanIndividualListing({
          baseUrl: BASE_URL,
          apiKey: API_KEY,
        });
        expect(fake).toHaveBeenCalledTimes(1);
        expect(fake.firstCall?.args[0]).toBe(`${BASE_URL}/models`);
        expect(fake.firstCall?.args[1]?.signal).toBeInstanceOf(AbortSignal);
        expect(fake.firstCall?.args[1]?.headers)
          .toEqual({ Authorization: `Bearer ${API_KEY}`, });
        expect(listing).toEqual([]);
      },
    }),

    it({
      name: 'sends no Authorization header when the caller holds no key',
      fn: async ctx => {
        const fake = ctx.sinon.stub(globalThis, 'fetch').callsFake(
          async () => new Response('{"data": []}', { status: 200, }),
        );
        await fetchQwenTokenPlanIndividualListing({
          baseUrl: BASE_URL,
          apiKey: undefined,
        });
        expect(fake.firstCall?.args[1]?.headers).toBeUndefined();
      },
    }),

    //endregion Success

    //region Failures

    it({
      name: 'bounds the read with the pinned timeout, aborting the transport',
      fn: async ctx => {
        const aborted = AbortSignal.abort();
        const timeoutStub = ctx.sinon.stub(AbortSignal, 'timeout')
          .returns(aborted);
        let caught: unknown;
        try {
          await fetchQwenTokenPlanIndividualListing({
            baseUrl: BASE_URL,
            apiKey: API_KEY,
            fetcher: ({ signal, }) => new Promise((_resolve, reject) => {
              function rejectOnAbort() {
                reject(new Error('aborted by the bound',),);
              }
              if (signal.aborted) {
                rejectOnAbort();
                return;
              }
              signal.addEventListener('abort', rejectOnAbort,);
            }),
          });
        }
        catch (error) {
          caught = error;
        }
        expect(timeoutStub).toHaveBeenCalledWith(
          QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS,
        );
        expect(caught).toBeInstanceOf(QwenTokenPlanListingError);
        expect((caught as Error).message).toBe(
          `Qwen Token Plan individual listing GET ${BASE_URL}/models failed at the transport step`,
        );
      },
    }),

    it({
      name: 'wraps the transport failure when the caller aborts',
      fn: async () => {
        const controller = new AbortController();
        setTimeout(function abortSoon() {
          controller.abort();
        }, 0);
        let caught: unknown;
        try {
          await fetchQwenTokenPlanIndividualListing({
            baseUrl: BASE_URL,
            apiKey: API_KEY,
            signal: controller.signal,
            fetcher: ({ signal, }) => new Promise((_resolve, reject) => {
              signal.addEventListener('abort', () => {
                reject(new Error('aborted by the caller',),);
              },);
            }),
          });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBeInstanceOf(QwenTokenPlanListingError);
        expect((caught as Error).message).toBe(
          `Qwen Token Plan individual listing GET ${BASE_URL}/models failed at the transport step`,
        );
      },
    }),

    it({
      name: 'throws a listing error naming the HTTP status when the endpoint answers non-OK',
      fn: async () => {
        let caught: unknown;
        try {
          await fetchQwenTokenPlanIndividualListing({
            baseUrl: BASE_URL,
            apiKey: API_KEY,
            fetcher: async () => ({
              ok: false,
              status: 503,
              json: async () => ({}),
            }),
          });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBeInstanceOf(QwenTokenPlanListingError);
        expect((caught as Error).message).toBe(
          `Qwen Token Plan individual listing GET ${BASE_URL}/models failed: HTTP 503`,
        );
      },
    }),

    it({
      name: 'throws a listing error when the body is not a record',
      fn: async () => {
        let caught: unknown;
        try {
          await fetchQwenTokenPlanIndividualListing({
            baseUrl: BASE_URL,
            apiKey: API_KEY,
            fetcher: async () => ({
              ok: true,
              status: 200,
              json: async () => 5,
            }),
          });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBeInstanceOf(QwenTokenPlanListingError);
        expect((caught as Error).message).toBe(
          `Qwen Token Plan individual listing GET ${BASE_URL}/models failed: response carries no model list`,
        );
      },
    }),

    it({
      name: 'throws a listing error when the body carries no model list',
      fn: async () => {
        let caught: unknown;
        try {
          await fetchQwenTokenPlanIndividualListing({
            baseUrl: BASE_URL,
            apiKey: API_KEY,
            fetcher: async () => ({
              ok: true,
              status: 200,
              json: async () => ({ data: 'not a list', }),
            }),
          });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBeInstanceOf(QwenTokenPlanListingError);
        expect((caught as Error).message).toBe(
          `Qwen Token Plan individual listing GET ${BASE_URL}/models failed: response carries no model list`,
        );
      },
    }),

    it({
      name: 'wraps a transport rejection with its cause',
      fn: async () => {
        const transportFailure = new Error('connection refused',);
        let caught: unknown;
        try {
          await fetchQwenTokenPlanIndividualListing({
            baseUrl: BASE_URL,
            apiKey: API_KEY,
            fetcher: async () => {
              throw transportFailure;
            },
          });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBeInstanceOf(QwenTokenPlanListingError);
        expect((caught as Error).cause).toBe(transportFailure);
        expect((caught as Error).message).toBe(
          `Qwen Token Plan individual listing GET ${BASE_URL}/models failed at the transport step`,
        );
      },
    }),

    it({
      name: 'wraps a body parse rejection with its cause',
      fn: async () => {
        const parseFailure = new Error('invalid json',);
        let caught: unknown;
        try {
          await fetchQwenTokenPlanIndividualListing({
            baseUrl: BASE_URL,
            apiKey: API_KEY,
            fetcher: async () => ({
              ok: true,
              status: 200,
              json: async () => {
                throw parseFailure;
              },
            }),
          });
        }
        catch (error) {
          caught = error;
        }
        expect(caught).toBeInstanceOf(QwenTokenPlanListingError);
        expect((caught as Error).cause).toBe(parseFailure);
      },
    }),

    //endregion Failures
  ],
});
