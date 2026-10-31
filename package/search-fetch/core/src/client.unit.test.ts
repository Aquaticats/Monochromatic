/**
 Unit tests for Linkup HTTP client request shaping and errors.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createLinkupClient,
  type FetchLike,
  type LinkupClient,
  type LinkupSearchRequestBody,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 API key fixture that must never appear in errors.
 */
const SECRET_API_KEY = 'super-secret-linkup-key';

/**
 Search response fixture.
 */
const SEARCH_RESPONSE = { results: [], };

/**
 Fetch response fixture.
 */
const FETCH_RESPONSE = { markdown: 'hello', };

/**
 Blocklist fixture.
 */
const BLOCKLIST = ['badwikipedia.invalid',] as const;

/**
 Base URL fixture.
 */
const BASE_URL = 'https://linkup.test/v1';

//endregion Fixtures

await describe({
  name: createLinkupClient.name,
  children: [
    it({
      name: 'search sends q, fixed standard depth, fixed searchResults output, and global excludeDomains',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockFetch({ body: SEARCH_RESPONSE, },);
        /**
         Local value for client.
         */
        const client = clientWithMock(mock.fetchImpl,);

        await client.search({
          input: { query: 'What is Linkup?', },
        },);

        /**
         Local value for requestBody.
         */
        const requestBody = requestJsonBody(firstFetchCall(mock,),) as LinkupSearchRequestBody;
        expect(requestBody.q,).toBe('What is Linkup?',);
        expect(requestBody.depth,).toBe('standard',);
        expect(requestBody.outputType,).toBe('searchResults',);
        expect(requestBody.excludeDomains,).toEqual(BLOCKLIST,);
      },
    },),
    it({
      name: 'search includes fromDate includeDomains and toDate when provided',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockFetch({ body: SEARCH_RESPONSE, },);
        /**
         Local value for client.
         */
        const client = clientWithMock(mock.fetchImpl,);

        await client.search({
          input: {
            query: 'Microsoft revenue',
            fromDate: '2025-01-01',
            includeDomains: ['microsoft.com',],
            toDate: '2025-12-31',
          },
        },);

        /**
         Local value for requestBody.
         */
        const requestBody = requestJsonBody(firstFetchCall(mock,),);
        expect(requestBody,).toHaveProperty('fromDate', '2025-01-01',);
        expect(requestBody.includeDomains,).toEqual(['microsoft.com',],);
        expect(requestBody,).toHaveProperty('toDate', '2025-12-31',);
      },
    },),
    it({
      name: 'search does not send extension-unsupported per-call options',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockFetch({ body: SEARCH_RESPONSE, },);
        /**
         Local value for client.
         */
        const client = clientWithMock(mock.fetchImpl,);

        await client.search({
          input: {
            query: 'ignored options',
            depth: 'deep',
            maxResults: 5,
            limit: 2,
          } as never,
        },);

        /**
         Local value for requestBody.
         */
        const requestBody = requestJsonBody(firstFetchCall(mock,),);
        expect('maxResults' in requestBody,).toBe(false,);
        expect('limit' in requestBody,).toBe(false,);
        expect(requestBody.depth,).toBe('standard',);
      },
    },),
    it({
      name: 'fetch sends fixed renderJs extractImages and includeRawHtml flags',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockFetch({ body: FETCH_RESPONSE, },);
        /**
         Local value for client.
         */
        const client = clientWithMock(mock.fetchImpl,);

        await client.fetch({
          input: { url: 'https://example.com', },
        },);

        /**
         Local value for requestBody.
         */
        const requestBody = requestJsonBody(firstFetchCall(mock,),);
        expect(requestBody,).toEqual({
          url: 'https://example.com',
          renderJs: true,
          extractImages: false,
          includeRawHtml: false,
        },);
      },
    },),
    it({
      name: 'non-2xx response throws without leaking API key',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockFetch({
          body: { error: { message: 'forbidden', }, },
          status: 403,
          statusText: 'Forbidden',
        },);
        /**
         Local value for client.
         */
        const client = clientWithMock(mock.fetchImpl,);

        let caught: unknown;
        try {
          await client.search({
            input: { query: 'secret safety', },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('/search',);
        expect((caught as Error).message.includes(SECRET_API_KEY,),).toBe(false,);
        expect((caught as Error).message.includes('Authorization',),).toBe(false,);
      },
    },),
    it({
      name: 'missing API key throws clear endpoint error',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockFetch({ body: SEARCH_RESPONSE, },);
        /**
         Local value for client.
         */
        const client = createLinkupClient({
          blocklist: [],
          baseUrl: BASE_URL,
          fetchImpl: mock.fetchImpl,
        },);

        let caught: unknown;
        try {
          await client.search({
            input: { query: 'missing key', },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('missing API key',);
        expect(mock.calls,).toHaveLength(0,);
      },
    },),
    it({
      name: 'invalid JSON response throws endpoint error',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockFetchText({ text: 'not json', },);
        /**
         Local value for client.
         */
        const client = clientWithMock(mock.fetchImpl,);

        let caught: unknown;
        try {
          await client.fetch({
            input: { url: 'https://example.com', },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('invalid JSON',);
        expect((caught as Error).message,).toContain('/fetch',);
      },
    },),
    it({
      name: 'aborted request throws endpoint abort error',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockAbortFetch();
        /**
         Local value for client.
         */
        const client = clientWithMock(mock.fetchImpl,);

        let caught: unknown;
        try {
          await client.search({
            input: { query: 'abort', },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('request aborted',);
      },
    },),
  ],
},);

//region Helpers

/**
 Recorded fetch call.
 */
type FetchCall = {
  /**
   Request URL.
   */
  readonly input: RequestInfo | URL;
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
   Fetch implementation passed to client.
   */
  readonly fetchImpl: FetchLike;
  /**
   Recorded fetch calls.
   */
  readonly calls: FetchCall[];
};

/**
 Create client with common fixtures.
 
 @param fetchImpl - mocked fetch implementation
 
 @returns Linkup client
 */
function clientWithMock(fetchImpl: FetchLike,): LinkupClient {
  return createLinkupClient({
    apiKey: SECRET_API_KEY,
    blocklist: BLOCKLIST,
    baseUrl: BASE_URL,
    fetchImpl,
  },);
}

/**
 Create JSON response fetch mock.
 
 @param body - response JSON body
 
 @param status - HTTP status
 
 @param statusText - HTTP status text
 
 @returns mock fetch harness
 */
function mockFetch(
  {
    body,
    status = 200,
    statusText = 'OK',
  }: {
    readonly body: unknown;
    readonly status?: number;
    readonly statusText?: string;
  },
): FetchMock {
  return mockFetchText({
    text: JSON.stringify(body,),
    status,
    statusText,
  },);
}

/**
 Create text response fetch mock.
 
 @param text - response body text
 
 @param status - HTTP status
 
 @param statusText - HTTP status text
 
 @returns mock fetch harness
 */
function mockFetchText(
  {
    text,
    status = 200,
    statusText = 'OK',
  }: {
    readonly text: string;
    readonly status?: number;
    readonly statusText?: string;
  },
): FetchMock {
  /**
   Local value for calls.
   */
  const calls: FetchCall[] = [];
  /**
   Fetch implementation returning configured response text.
   
   @param input - fetch input recorded for assertions
   
   @param init - fetch init recorded for assertions
   
   @returns configured response
   */
  async function fetchImpl(
    input: Parameters<FetchLike>[0],
    init: Parameters<FetchLike>[1],
  ): ReturnType<FetchLike> {
    calls.push({
      input,
      init: init ?? {},
    },);
    return new Response(text, {
      status,
      statusText,
    },);
  }
  return {
    fetchImpl,
    calls,
  };
}

/**
 Create fetch mock that throws AbortError.
 
 @returns mock fetch harness
 */
function mockAbortFetch(): FetchMock {
  /**
   Local value for calls.
   */
  const calls: FetchCall[] = [];
  /**
   Fetch implementation throwing AbortError after recording input.
   
   @param input - fetch input recorded for assertions
   
   @param init - fetch init recorded for assertions
   
   @returns never resolves because it throws AbortError
   */
  async function fetchImpl(
    input: Parameters<FetchLike>[0],
    init: Parameters<FetchLike>[1],
  ): ReturnType<FetchLike> {
    calls.push({
      input,
      init: init ?? {},
    },);
    /**
     Local value for error.
     */
    const error = new Error('aborted by test');
    error.name = 'AbortError';
    throw error;
  }
  return {
    fetchImpl,
    calls,
  };
}

/**
 Return first recorded fetch call.
 
 @param mock - fetch mock harness
 
 @returns first recorded fetch call
 */
function firstFetchCall(mock: FetchMock,): FetchCall {
  /**
   First recorded fetch call.
   */
  const [call,] = mock.calls;
  if (call === undefined)
    throw new Error('missing fetch call',);
  return call;
}

/**
 Parse JSON request body from recorded fetch call.
 
 @param call - recorded fetch call
 
 @returns parsed body record
 */
function requestJsonBody(call: FetchCall,): Record<string, unknown> {
  if ((typeof call.init.body) !== 'string')
    throw new Error('fetch body was not a string',);
  /**
   Local value for parsed.
   */
  const parsed = JSON.parse(call.init.body,) as unknown;
  if ((parsed === null) || ((typeof parsed) !== 'object') || Array.isArray(parsed,))
    throw new Error('fetch body was not an object',);
  return parsed as Record<string, unknown>;
}

//endregion Helpers
