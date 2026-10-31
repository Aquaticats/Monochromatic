/**
 Unit tests for the whole-call deadline bounding one complete search or fetch call.

 @module
 */

import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createSearchFetchClient,
  DEEPSEEK_FETCH_ENDPOINT,
  DEEPSEEK_SEARCH_ENDPOINT,
  WholeCallDeadlineError,
  WHOLE_CALL_DEADLINE_MILLISECONDS,
  withWholeCallDeadline,
  type FetchLike,
  type GhClient,
  type GhClientFetchOptions,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Short deadline keeping every expiry case fast.
 */
const SHORT_DEADLINE_MILLISECONDS = 50;

/**
 Long deadline that outlives the caller abort used in the caller-first cases.
 */
const LONG_DEADLINE_MILLISECONDS = 2_000;

/**
 Delay before a caller abort in the caller-first cases.
 */
const CALLER_ABORT_DELAY_MILLISECONDS = 10;

/**
 DeepSeek base URL for routed fetch assertions.
 */
const DEEPSEEK_BASE_URL = 'https://deepseek.test';

/**
 Exa base URL for routed fetch assertions.
 */
const EXA_BASE_URL = 'https://exa.test';

/**
 Linkup base URL for routed fetch assertions.
 */
const LINKUP_BASE_URL = 'https://linkup.test';

/**
 Search query sent through the routed search cases.
 */
const SEARCH_QUERY = 'docs';

/**
 GitHub repository URL routed through gh in the mapped fetch cases.
 */
const GITHUB_REPOSITORY_URL = 'https://github.com/cli/cli';

/**
 Failure text carried by fast-failing provider hops.
 */
const PROVIDER_FAILURE_BODY = { message: 'bad gateway', };

/**
 Caller abort reason used by the caller-first cases.
 */
const CALLER_REASON = 'caller cancelled the search';

//endregion Fixtures

//region Types

/**
 One scripted behavior for a URL substring.
 */
type FetchRoute = {
  /**
   URL substring selecting this route.
   */
  readonly urlPart: string;
  /**
   Behavior run once the route is selected.
   */
  readonly respond: (init: RequestInit,) => Promise<Response>;
};

/**
 Fetch harness recording calls and routing them to scripted behaviors.
 */
type RoutedFetch = {
  /**
   Fetch implementation injected into the client under test.
   */
  readonly fetchImpl: FetchLike;
  /**
   Recorded call URLs in call order.
   */
  readonly urls: string[];
};

/**
 Mutable record of one observed in-flight abort.
 */
type AbortObservation = {
  /**
   Whether the provider hop saw its request signal abort.
   */
  aborted: boolean;
  /**
   Abort reason the provider hop observed.
   */
  reason: unknown;
};

//endregion Types

//region Helpers

/**
 Build a fetch harness routing each call by URL substring.

 @param routes - scripted behaviors keyed by URL substring

 @returns fetch harness with recorded call URLs

 @example
 ```ts
 routedFetch({ routes: [{ urlPart: 'deepseek.test', respond: () => neverSettles() }] });
 ```
 */
function routedFetch({ routes, }: { readonly routes: readonly FetchRoute[]; }): RoutedFetch {
  /**
   Recorded call URLs.
   */
  const urls: string[] = [];
  return {
    urls,
    async fetchImpl(input: RequestInfo | URL, init?: RequestInit,): Promise<Response> {
      /**
       URL text of the routed request.
       */
      const url = (typeof input) === 'string'
        ? input
        : input instanceof URL
          ? input.href
          : input.url;
      urls.push(url,);
      /**
       Matching scripted behavior.
       */
      const matched = routes
        .find(function matchesUrl(candidate,) {
          return url.includes(candidate.urlPart,);
        },);
      if (matched === undefined)
        throw new Error(`unexpected fetch call: ${url}`,);
      return matched.respond(init ?? {},);
    },
  };
}

/**
 Build a promise that never settles, like a provider that ignores cancellation.

 @returns pending response promise

 @example
 ```ts
 neverSettles();
 ```
 */
function neverSettles(): Promise<Response> {
  return Promise
    .withResolvers<Response>()
    .promise;
}

/**
 Build a fast failing provider hop.

 @returns behavior answering with a 502 response

 @example
 ```ts
 httpFailure();
 ```
 */
function httpFailure(): () => Promise<Response> {
  return async function failFast(): Promise<Response> {
    return Response.json(
      PROVIDER_FAILURE_BODY,
      {
        status: 502,
        statusText: 'Bad Gateway',
      },
    );
  };
}

/**
 Build a hanging provider hop that records its request signal abort.

 @param observation - record filled when the hop sees its abort

 @returns behavior hanging until its request signal aborts

 @example
 ```ts
 abortRecordingHang({ observation });
 ```
 */
function abortRecordingHang(
  { observation, }: {
    readonly observation: AbortObservation;
  },
): (init: RequestInit,) => Promise<Response> {
  return function hangUntilAborted(init: RequestInit,): Promise<Response> {
    /**
     Pending response settled by rejection when the request signal aborts.
     */
    const { promise, reject, } = Promise.withResolvers<Response>();
    /**
     Request signal carried by the provider hop.
     */
    const requestSignal = init.signal;
    if (!(requestSignal instanceof AbortSignal))
      return promise;
    /**
     Narrowed abort signal used inside the abort listener.
     */
    const abortSignal: AbortSignal = requestSignal;
    /**
     Record the abort and reject with its reason.
     */
    function recordAbort(): void {
      observation.aborted = true;
      observation.reason = abortSignal.reason;
      reject(abortSignal.reason,);
    }
    if (abortSignal.aborted) {
      recordAbort();
      return promise;
    }
    abortSignal.addEventListener(
      'abort',
      recordAbort,
      { once: true, },
    );
    return promise;
  };
}

/**
 Build a gh client whose fetch hangs without settling.

 @param requests - record of received gh fetch options

 @returns hanging gh client harness

 @example
 ```ts
 hangingGhClient({ requests: [] });
 ```
 */
function hangingGhClient(
  { requests, }: {
    readonly requests: GhClientFetchOptions[];
  },
): GhClient {
  return {
    fetch: async function hangGh(options: GhClientFetchOptions,): Promise<unknown> {
      requests.push(options,);
      return neverSettles();
    },
  };
}

/**
 Build a gh client failing every fetch with the given reason while recording requests.

 @param reason - failure message thrown for every request

 @param requests - record of received gh fetch options

 @returns failing gh client harness

 @example
 ```ts
 failingGhClient({ reason: 'gh is unavailable', requests: [] });
 ```
 */
function failingGhClient(
  {
    reason,
    requests,
  }: {
    readonly reason: string;
    readonly requests: GhClientFetchOptions[];
  },
): GhClient {
  return {
    fetch: async function failGh(options: GhClientFetchOptions,): Promise<unknown> {
      requests.push(options,);
      throw new Error(reason,);
    },
  };
}

/**
 Build a routed client with all three provider keys configured.

 @param options - fetch harness, optional gh client, and deadline override

 @returns provider-routing client under test

 @example
 ```ts
 routedClient({ fetchImpl, wholeCallDeadlineMs: 50 });
 ```
 */
function routedClient(
  {
    fetchImpl,
    ghClient,
    wholeCallDeadlineMs = SHORT_DEADLINE_MILLISECONDS,
  }: {
    readonly fetchImpl: FetchLike;
    readonly ghClient?: GhClient;
    readonly wholeCallDeadlineMs?: number;
  },
) {
  return createSearchFetchClient({
    deepseekApiKey: 'deepseek-key',
    exaApiKey: 'exa-key',
    linkupApiKey: 'linkup-key',
    blocklist: [],
    deepseekBaseUrl: DEEPSEEK_BASE_URL,
    exaBaseUrl: EXA_BASE_URL,
    linkupBaseUrl: LINKUP_BASE_URL,
    fetchImpl,
    wholeCallDeadlineMs,
    ...(ghClient === undefined ? {} : { ghClient, }),
  },);
}

/**
 Await a call expected to fail and return the caught value.

 @param run - call expected to reject

 @returns caught rejection value

 @example
 ```ts
 caughtFrom(async () => client.search({ input: { query: 'docs' } }));
 ```
 */
async function caughtFrom(run: () => Promise<unknown>,): Promise<unknown> {
  try {
    await run();
  }
  catch (error: unknown) {
    return error;
  }
  throw new Error('expected the call to fail',);
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: 'WHOLE_CALL_DEADLINE_MILLISECONDS',
      children: [
        it({
          name: 'is 120 s so no complete call exceeds the default',
          fn: async () => {
            expect(WHOLE_CALL_DEADLINE_MILLISECONDS,).toBe(120_000,);
          },
        },),
      ],
    },),
    describe({
      name: WholeCallDeadlineError.name,
      children: [
        it({
          name: 'names the budget and explains the whole-call deadline',
          fn: async () => {
            /**
             Local value for error.
             */
            const error = new WholeCallDeadlineError({
              operation: 'search',
              deadlineMs: WHOLE_CALL_DEADLINE_MILLISECONDS,
            },);

            expect(error.message,).toContain('Search Fetch search call',);
            expect(error.message,).toContain('whole-call deadline of 120000 ms',);
            expect(error.message,).toContain('120 s by default',);
            expect(error.message,).toContain('including every fallback hop',);
            expect(error.name,).toBe('WholeCallDeadlineError',);
          },
        },),
        it({
          name: 'exposes the effective deadline the call exceeded',
          fn: async () => {
            /**
             Local value for error.
             */
            const error = new WholeCallDeadlineError({
              operation: 'fetch',
              deadlineMs: SHORT_DEADLINE_MILLISECONDS,
            },);

            expect(error.deadlineMs,).toBe(SHORT_DEADLINE_MILLISECONDS,);
            expect(error.message,).toContain('Search Fetch fetch call',);
          },
        },),
      ],
    },),
    describe({
      name: withWholeCallDeadline.name,
      children: [
        it({
          name: 'resolves the call result when the call settles inside its deadline',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await withWholeCallDeadline({
              operation: 'search',
              deadlineMs: SHORT_DEADLINE_MILLISECONDS,
              call: async function quickCall() {
                return 'answer';
              },
            },);

            expect(result,).toBe('answer',);
          },
        },),
        it({
          name: 'rejects with a deadline error naming the budget when the call outlives its deadline',
          fn: async () => {
            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function hangToExpiry() {
              await withWholeCallDeadline({
                operation: 'search',
                deadlineMs: SHORT_DEADLINE_MILLISECONDS,
                call: async function hang() {
                  return neverSettles();
                },
              },);
            },);

            expect(caught,).toBeInstanceOf(WholeCallDeadlineError,);
            expect((caught as WholeCallDeadlineError).message,).toContain('whole-call deadline of 50 ms',);
            expect((caught as WholeCallDeadlineError).message,).toContain('120 s by default',);
            expect((caught as WholeCallDeadlineError).deadlineMs,).toBe(SHORT_DEADLINE_MILLISECONDS,);
          },
        },),
        it({
          name: 'aborts the combined signal with the deadline error when the deadline fires first',
          fn: async () => {
            /**
             Mutable record of the in-flight abort.
             */
            const observation: AbortObservation = {
              aborted: false,
              reason: undefined,
            };
            /**
             Local value for controller.
             */
            const controller = new AbortController();

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function hangToExpiry() {
              await withWholeCallDeadline({
                operation: 'fetch',
                deadlineMs: SHORT_DEADLINE_MILLISECONDS,
                signal: controller.signal,
                call: async function hangObserving({ signal, }: { readonly signal: AbortSignal; }) {
                  signal.addEventListener(
                    'abort',
                    function recordAbort() {
                      observation.aborted = true;
                      observation.reason = signal.reason;
                    },
                    { once: true, },
                  );
                  return neverSettles();
                },
              },);
            },);

            expect(caught,).toBeInstanceOf(WholeCallDeadlineError,);
            expect(observation.aborted,).toBe(true,);
            expect(observation.reason,).toBe(caught,);
          },
        },),
        it({
          name: 'surfaces the caller abort reason when the caller signal fires first',
          fn: async () => {
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            /**
             Local value for aborting.
             */
            const aborting = (async function abortLater() {
              await delay(CALLER_ABORT_DELAY_MILLISECONDS,);
              controller.abort(new Error(CALLER_REASON,),);
            })();

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function hangUntilCallerAbort() {
              await withWholeCallDeadline({
                operation: 'search',
                deadlineMs: LONG_DEADLINE_MILLISECONDS,
                signal: controller.signal,
                call: async function hang() {
                  return neverSettles();
                },
              },);
            },);
            await aborting;

            expect(caught,).toBeInstanceOf(Error,);
            expect(caught,).not.toBeInstanceOf(WholeCallDeadlineError,);
            expect((caught as Error).message,).toBe(CALLER_REASON,);
          },
        },),
        it({
          name: 'surfaces the caller abort reason when the signal is already aborted at entry',
          fn: async () => {
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            controller.abort(new Error(CALLER_REASON,),);

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function hangPastEntry() {
              await withWholeCallDeadline({
                operation: 'search',
                deadlineMs: LONG_DEADLINE_MILLISECONDS,
                signal: controller.signal,
                call: async function hang() {
                  return neverSettles();
                },
              },);
            },);

            expect((caught as Error).message,).toBe(CALLER_REASON,);
          },
        },),
      ],
    },),
    describe({
      name: createSearchFetchClient.name,
      children: [
        it({
          name: 'expires during the first fallback hop when the DeepSeek search request hangs',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = routedFetch({
              routes: [
                {
                  urlPart: DEEPSEEK_BASE_URL,
                  respond: () => neverSettles(),
                },
              ],
            },);
            /**
             Local value for client.
             */
            const client = routedClient({ fetchImpl: harness.fetchImpl, },);

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function searchToExpiry() {
              await client.search({ input: { query: SEARCH_QUERY, }, },);
            },);

            expect(caught,).toBeInstanceOf(WholeCallDeadlineError,);
            expect((caught as WholeCallDeadlineError).message,).toContain('search call',);
            expect(harness.urls,).toHaveLength(1,);
            expect(harness.urls[0],).toBe(`${DEEPSEEK_BASE_URL}${DEEPSEEK_SEARCH_ENDPOINT}`,);
          },
        },),
        it({
          name: 'expires during a later fallback hop when the Linkup search request hangs after earlier hops fail',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = routedFetch({
              routes: [
                {
                  urlPart: DEEPSEEK_BASE_URL,
                  respond: httpFailure(),
                },
                {
                  urlPart: EXA_BASE_URL,
                  respond: httpFailure(),
                },
                {
                  urlPart: LINKUP_BASE_URL,
                  respond: () => neverSettles(),
                },
              ],
            },);
            /**
             Local value for client.
             */
            const client = routedClient({ fetchImpl: harness.fetchImpl, },);

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function searchToExpiry() {
              await client.search({ input: { query: SEARCH_QUERY, }, },);
            },);

            expect(caught,).toBeInstanceOf(WholeCallDeadlineError,);
            expect(harness.urls,).toHaveLength(3,);
            expect(harness.urls[0],).toBe(`${DEEPSEEK_BASE_URL}${DEEPSEEK_SEARCH_ENDPOINT}`,);
            expect(harness.urls[1],).toBe(`${EXA_BASE_URL}/search`,);
            expect(harness.urls[2],).toBe(`${LINKUP_BASE_URL}/search`,);
          },
        },),
        it({
          name: 'expires during the gh hop of a mapped GitHub fetch when the gh child hangs',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = routedFetch({ routes: [], },);
            /**
             Requests received by the hanging gh client.
             */
            const ghRequests: GhClientFetchOptions[] = [];
            /**
             Local value for client.
             */
            const client = routedClient({
              fetchImpl: harness.fetchImpl,
              ghClient: hangingGhClient({ requests: ghRequests, },),
            },);

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function fetchToExpiry() {
              await client.fetch({ input: { url: GITHUB_REPOSITORY_URL, }, },);
            },);

            expect(caught,).toBeInstanceOf(WholeCallDeadlineError,);
            expect((caught as WholeCallDeadlineError).message,).toContain('fetch call',);
            expect(ghRequests,).toHaveLength(1,);
            expect(harness.urls,).toHaveLength(0,);
          },
        },),
        it({
          name: 'expires during a later fallback hop when the Linkup fetch hangs after gh and DeepSeek fail',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = routedFetch({
              routes: [
                {
                  urlPart: DEEPSEEK_BASE_URL,
                  respond: httpFailure(),
                },
                {
                  urlPart: LINKUP_BASE_URL,
                  respond: () => neverSettles(),
                },
              ],
            },);
            /**
             Requests received by the failing gh client.
             */
            const ghRequests: GhClientFetchOptions[] = [];
            /**
             Local value for client.
             */
            const client = routedClient({
              fetchImpl: harness.fetchImpl,
              ghClient: failingGhClient({
                reason: 'gh is unavailable',
                requests: ghRequests,
              },),
            },);

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function fetchToExpiry() {
              await client.fetch({ input: { url: GITHUB_REPOSITORY_URL, }, },);
            },);

            expect(caught,).toBeInstanceOf(WholeCallDeadlineError,);
            expect(ghRequests,).toHaveLength(1,);
            expect(harness.urls,).toHaveLength(2,);
            expect(harness.urls[0],).toBe(`${DEEPSEEK_BASE_URL}${DEEPSEEK_FETCH_ENDPOINT}`,);
            expect(harness.urls[1],).toBe(`${LINKUP_BASE_URL}/fetch`,);
          },
        },),
        it({
          name: 'aborts the in-flight provider hop with the deadline error when the deadline fires first',
          fn: async () => {
            /**
             Mutable record of the in-flight provider abort.
             */
            const observation: AbortObservation = {
              aborted: false,
              reason: undefined,
            };
            /**
             Local value for harness.
             */
            const harness = routedFetch({
              routes: [
                {
                  urlPart: DEEPSEEK_BASE_URL,
                  respond: abortRecordingHang({ observation, }),
                },
              ],
            },);
            /**
             Local value for client.
             */
            const client = routedClient({ fetchImpl: harness.fetchImpl, },);

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function searchToExpiry() {
              await client.search({ input: { query: SEARCH_QUERY, }, },);
            },);

            expect(caught,).toBeInstanceOf(WholeCallDeadlineError,);
            expect(observation.aborted,).toBe(true,);
            expect(observation.reason,).toBe(caught,);
          },
        },),
        it({
          name: 'surfaces the caller abort reason and stops the chain when the caller signal fires first',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = routedFetch({
              routes: [
                {
                  urlPart: DEEPSEEK_BASE_URL,
                  respond: () => neverSettles(),
                },
              ],
            },);
            /**
             Local value for client.
             */
            const client = routedClient({
              fetchImpl: harness.fetchImpl,
              wholeCallDeadlineMs: LONG_DEADLINE_MILLISECONDS,
            },);
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            /**
             Local value for aborting.
             */
            const aborting = (async function abortLater() {
              await delay(CALLER_ABORT_DELAY_MILLISECONDS,);
              controller.abort(new Error(CALLER_REASON,),);
            })();

            /**
             Local value for caught.
             */
            const caught = await caughtFrom(async function searchUntilCallerAbort() {
              await client.search({
                input: { query: SEARCH_QUERY, },
                signal: controller.signal,
              },);
            },);
            await aborting;

            expect(caught,).toBeInstanceOf(Error,);
            expect(caught,).not.toBeInstanceOf(WholeCallDeadlineError,);
            expect((caught as Error).message,).toBe(CALLER_REASON,);
            expect(harness.urls,).toHaveLength(1,);
          },
        },),
        it({
          name: 'resolves the provider response when the chain answers inside the deadline',
          fn: async () => {
            /**
             Local value for harness.
             */
            const harness = routedFetch({
              routes: [
                {
                  urlPart: DEEPSEEK_BASE_URL,
                  respond: async function answer() {
                    return Response.json(
                      {
                        results: [
                          {
                            title: 'Docs',
                            url: 'https://example.com/docs',
                          },
                        ],
                      },
                    );
                  },
                },
              ],
            },);
            /**
             Local value for client.
             */
            const client = routedClient({ fetchImpl: harness.fetchImpl, },);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query: SEARCH_QUERY, }, },);

            expect(result.provider,).toBe('deepseek',);
            expect(harness.urls,).toHaveLength(1,);
          },
        },),
      ],
    },),
  ],
},);
