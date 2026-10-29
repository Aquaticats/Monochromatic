/**
 Unit tests for the DeepSeek Tools API client.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  DEFAULT_DEEPSEEK_BASE_URL,
  createDeepseekClient,
  type FetchLike,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 DeepSeek API key fixture.
 */
const DEEPSEEK_API_KEY = 'deepseek-secret-key';

/**
 DeepSeek base URL fixture.
 */
const DEEPSEEK_BASE_URL = 'https://deepseek.test';

/**
 Allowed deep-domain result fixture.
 */
const ALLOWED_RESULT = {
  title: 'Allowed',
  url: 'https://allowed.example.com/page#fragment',
  snippet: 'Allowed snippet',
  summary: 'Allowed summary',
  content: 'Allowed content',
  site_name: 'Allowed',
  site_icon: null,
  date: '2026-05-01T00:00:00+00:00',
  authority: 3,
  rerank_score: 2.4,
};

/**
 Duplicate of the allowed result differing only in query and fragment.
 */
const DUPLICATE_ALLOWED_RESULT = {
  ...ALLOWED_RESULT,
  url: 'https://allowed.example.com/page?tracking=1',
};

/**
 Result on a host outside the include-domain filter.
 */
const OTHER_HOST_RESULT = {
  ...ALLOWED_RESULT,
  title: 'Other',
  url: 'https://other.example.com/page',
};

/**
 Allowed-host result outside the requested date window.
 */
const OLD_RESULT = {
  ...ALLOWED_RESULT,
  title: 'Old',
  url: 'https://allowed.example.com/old',
  date: '2024-01-01T00:00:00+00:00',
};

/**
 Allowed-host result with no publication date.
 */
const UNDATED_RESULT = {
  ...ALLOWED_RESULT,
  title: 'Undated',
  url: 'https://allowed.example.com/undated',
  date: null,
};

//endregion Fixtures

//region Helpers

/**
 Mock response fixture.
 */
type MockResponse = {
  /**
   Response body object.
   */
  readonly body: unknown;
  /**
   HTTP status.
   */
  readonly status?: number;
  /**
   HTTP status text.
   */
  readonly statusText?: string;
};

/**
 Recorded fetch call.
 */
type FetchCall = {
  /**
   Request URL.
   */
  readonly url: string;
  /**
   Request init.
   */
  readonly init: RequestInit;
};

/**
 Mock fetch harness.
 */
type FetchMock = {
  /**
   Fetch implementation passed to the client.
   */
  readonly fetchImpl: FetchLike;
  /**
   Recorded fetch calls.
   */
  readonly calls: FetchCall[];
};

/**
 Create an ordered JSON response fetch mock.
 
 @param responses - responses returned in call order
 
 @returns mock fetch harness
 */
function mockFetch({ responses, }: { readonly responses: readonly MockResponse[]; }): FetchMock {
  /**
   Recorded calls.
   */
  const calls: FetchCall[] = [];
  /**
   Fetch implementation.
   */
  async function fetchImpl(input: RequestInfo | URL, init?: RequestInit,): Promise<Response> {
    calls.push({
      url: fetchInputUrl(input,),
      init: init ?? {},
    },);
    /**
     Scripted response for this call.
     */
    const response = responses[calls.length - 1];
    if (response === undefined)
      throw new Error('unexpected fetch call',);
    return Response.json(
      response.body,
      {
        status: response.status ?? 200,
        statusText: response.statusText ?? 'OK',
      },
    );
  }
  return {
    fetchImpl,
    calls,
  };
}

/**
 Return URL text for fetch input.
 
 @param input - fetch input
 
 @returns URL text
 */
function fetchInputUrl(input: RequestInfo | URL,): string {
  if ((typeof input) === 'string')
    return input;
  if (input instanceof URL)
    return input.href;
  return input.url;
}

/**
 Return the first recorded fetch call.
 
 @param mock - mock fetch harness
 
 @returns first fetch call
 */
function firstCall(mock: FetchMock,): FetchCall {
  /**
   First recorded call.
   */
  const [call,] = mock.calls;
  if (call === undefined)
    throw new Error('missing fetch call',);
  return call;
}

/**
 Parse a recorded JSON request body.
 
 @param call - fetch call
 
 @returns parsed JSON object
 */
function requestJsonBody(call: FetchCall,): Record<string, unknown> {
  if ((typeof call.init.body) !== 'string')
    throw new Error('request body was not a string',);
  return JSON.parse(call.init.body,) as Record<string, unknown>;
}

/**
 Read one request header.
 
 @param call - fetch call
 
 @param name - header name
 
 @returns header value
 */
function requestHeader(call: FetchCall, name: string,): string {
  /**
   Header value from the recorded request init.
   */
  const value = new Headers(call.init.headers,).get(name,);
  if (value === null)
    throw new Error(`missing request header ${name}`,);
  return value;
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: createDeepseekClient.name,
      children: [
        it({
          name: 'search sends the maximum count, page content, filters, and bearer auth',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: { results: [ALLOWED_RESULT,], }, },],
            },);
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mock.fetchImpl,
            },);

            await client.search({
              input: {
                query: 'docs',
                fromDate: '2026-01-01',
                includeDomains: ['https://Allowed.Example.com/docs',],
                toDate: '2026-12-31',
              },
            },);

            expect(firstCall(mock,).url,).toBe(`${DEEPSEEK_BASE_URL}/v1/tools/search`,);
            expect(requestHeader(firstCall(mock,), 'Authorization',),).toBe(`Bearer ${DEEPSEEK_API_KEY}`,);
            /**
             Recorded search request body.
             */
            const searchBody = requestJsonBody(firstCall(mock,),);
            expect(searchBody,).toEqual({
              query: 'docs site:allowed.example.com after:2026-01-01 before:2026-12-31',
              count: 50,
              need_content: true,
            },);
          },
        },),
        it({
          name: 'search post-filters include domains, dates, and duplicate URLs',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{
                body: { results: [ALLOWED_RESULT, DUPLICATE_ALLOWED_RESULT, OTHER_HOST_RESULT, OLD_RESULT, UNDATED_RESULT,], },
              },],
            },);
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mock.fetchImpl,
            },);

            /**
             Local value for response.
             */
            const response = await client.search({
              input: {
                query: 'docs',
                fromDate: '2026-01-01',
                includeDomains: ['allowed.example.com',],
                toDate: '2026-12-31',
              },
            },) as { readonly results: readonly { readonly url: string; }[]; };

            expect(response.results.map(function resultUrl(result,) {
              return result.url;
            },),).toEqual([
              ALLOWED_RESULT.url,
              UNDATED_RESULT.url,
            ],);
          },
        },),
        it({
          name: 'search drops operators that would cross the 128 code point limit and filters locally',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: { results: [ALLOWED_RESULT, OTHER_HOST_RESULT,], }, },],
            },);
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mock.fetchImpl,
            },);
            /**
             Base query long enough that the domain operator cannot fit.
             */
            const query = 'x'.repeat(120,);

            /**
             Local value for response.
             */
            const response = await client.search({
              input: {
                query,
                includeDomains: ['allowed.example.com',],
              },
            },) as { readonly results: readonly { readonly url: string; }[]; };

            expect(requestJsonBody(firstCall(mock,),).query,).toBe(query,);
            expect(response.results,).toHaveLength(1,);
            expect(response.results[0]?.url,).toBe(ALLOWED_RESULT.url,);
          },
        },),
        it({
          name: 'search rejects a base query longer than 128 code points',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({ responses: [], },);
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mock.fetchImpl,
            },);

            await expect(
              client.search({ input: { query: 'x'.repeat(129,), }, },),
            ).rejects.toThrow('128',);
            expect(mock.calls,).toHaveLength(0,);
          },
        },),
        it({
          name: 'search rejects an unexpected response shape',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({ responses: [{ body: { nope: true, }, },], },);
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mock.fetchImpl,
            },);

            await expect(
              client.search({ input: { query: 'docs', }, },),
            ).rejects.toThrow('unexpected response shape',);
          },
        },),
        it({
          name: 'search names the API message without leaking the key on HTTP errors',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{
                body: { error: { message: 'invalid key', }, },
                status: 401,
                statusText: 'Unauthorized',
              },],
            },);
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mock.fetchImpl,
            },);

            /**
             Local value for caught.
             */
            let caught: unknown;
            try {
              await client.search({ input: { query: 'docs', }, },);
            }
            catch (error: unknown) {
              caught = error;
            }

            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('DeepSeek',);
            expect((caught as Error).message,).toContain('invalid key',);
            expect((caught as Error).message,).not.toContain(DEEPSEEK_API_KEY,);
          },
        },),
        it({
          name: 'search names the missing key and the store hint',
          fn: async () => {
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mockFetch({ responses: [], },).fetchImpl,
              credentialHint: 'store is missing',
            },);

            await expect(
              client.search({ input: { query: 'docs', }, },),
            ).rejects.toThrow('DEEPSEEK_API_KEY',);
          },
        },),
        it({
          name: 'search rethrows a caller cancellation',
          fn: async () => {
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            controller.abort();
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mockFetch({ responses: [], },).fetchImpl,
            },);

            await expect(
              client.search({
                input: { query: 'docs', },
                signal: controller.signal,
              },),
            ).rejects.toThrow('aborted',);
          },
        },),
        it({
          name: 'fetch sends one URL and normalizes Markdown content',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: { title: 'Page title', content: '# Page\n\nBody', }, },],
            },);
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mock.fetchImpl,
            },);

            /**
             Local value for response.
             */
            const response = await client.fetch({ input: { url: 'https://example.com/page', }, },);

            expect(firstCall(mock,).url,).toBe(`${DEEPSEEK_BASE_URL}/v1/tools/fetch`,);
            /**
             Recorded fetch request body.
             */
            const fetchBody = requestJsonBody(firstCall(mock,),);
            expect(fetchBody,).toEqual({
              url: 'https://example.com/page',
            },);
            expect(response,).toEqual({ markdown: '# Page\n\nBody', },);
          },
        },),
        it({
          name: 'fetch rejects a response without page content',
          fn: async () => {
            /**
             Local value for client.
             */
            const client = createDeepseekClient({
              apiKey: DEEPSEEK_API_KEY,
              baseUrl: DEEPSEEK_BASE_URL,
              fetchImpl: mockFetch({ responses: [{ body: { title: 'Empty', content: '   ', }, },], },).fetchImpl,
            },);

            await expect(
              client.fetch({ input: { url: 'https://example.com/page', }, },),
            ).rejects.toThrow('no page content',);
          },
        },),
      ],
    },),
    describe({
      name: DEFAULT_DEEPSEEK_BASE_URL,
      children: [
        it({
          name: 'is the public DeepSeek API origin',
          fn: async () => {
            expect(DEFAULT_DEEPSEEK_BASE_URL,).toBe('https://api.deepseek.com',);
          },
        },),
      ],
    },),
  ],
},);
