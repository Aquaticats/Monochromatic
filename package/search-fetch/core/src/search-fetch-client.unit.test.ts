/**
 Unit tests for Search Fetch provider routing.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createExaClient,
  createSearchFetchClient,
  exaForwardableBlocklist,
  type GhClient,
  type GhClientFetchOptions,
  planGitHubFetch,
  type FetchLike,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Exa API key fixture.
 */
const EXA_API_KEY = 'exa-secret-key';

/**
 Linkup API key fixture.
 */
const LINKUP_API_KEY = 'linkup-secret-key';

/**
 DeepSeek API key fixture.
 */
const DEEPSEEK_API_KEY = 'deepseek-secret-key';

/**
 DeepSeek base URL fixture.
 */
const DEEPSEEK_BASE_URL = 'https://deepseek.test';

/**
 DeepSeek search response fixture.
 */
const DEEPSEEK_SEARCH_RESPONSE = {
  results: [
    {
      title: 'DeepSeek',
      url: 'https://example.com/deepseek',
      snippet: 'DeepSeek result',
      summary: 'DeepSeek summary',
      content: 'DeepSeek content',
      date: null,
      authority: 3,
      rerank_score: 2.4,
    },
  ],
};

/**
 DeepSeek fetch response fixture.
 */
const DEEPSEEK_FETCH_RESPONSE = {
  title: 'DeepSeek page',
  content: '# DeepSeek page',
};

/**
 Exa base URL fixture.
 */
const EXA_BASE_URL = 'https://exa.test';

/**
 Linkup base URL fixture.
 */
const LINKUP_BASE_URL = 'https://linkup.test/v1';

/**
 Blocklist fixture containing one Exa-incompatible entry.
 */
const BLOCKLIST = [
  'gov',
  'badwikipedia.invalid',
] as const;

/**
 Exa search response fixture.
 */
const EXA_SEARCH_RESPONSE = { results: [{ title: 'Exa', url: 'https://example.com/exa', },], };

/**
 Linkup search response fixture.
 */
const LINKUP_SEARCH_RESPONSE = { results: [{ title: 'Linkup', url: 'https://example.com/linkup', },], };

/**
 Linkup fetch response fixture.
 */
const LINKUP_FETCH_RESPONSE = { markdown: 'Linkup page', };

/**
 Exa fetch response fixture.
 */
const EXA_FETCH_RESPONSE = { results: [{ url: 'https://example.com', text: 'Exa page', },], };

/**
 GitHub repository URL fixture carrying a gh mapping.
 */
const GITHUB_REPOSITORY_URL = 'https://github.com/cli/cli';

/**
 GitHub URL fixture whose shape has no gh mapping.
 */
const UNMAPPED_GITHUB_URL = 'https://github.com/cli/cli/actions/runs/36224089994';

/**
 gh client response fixture.
 */
const GH_FETCH_RESPONSE = { markdown: 'name:\tcli/cli', };

/**
 gh client failure reason fixture.
 */
const GH_FAILURE_REASON = 'gh fetch failed for https://github.com/cli/cli (repository-home)';

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: createExaClient.name,
      children: [
        it({
          name: 'search sends fast mode and only Exa-compatible excluded domains',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: EXA_SEARCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = createExaClient({
              apiKey: EXA_API_KEY,
              blocklist: BLOCKLIST,
              baseUrl: EXA_BASE_URL,
              fetchImpl: mock.fetchImpl,
            },);

            await client.search({
              input: {
                query: 'docs',
                fromDate: '2026-01-01',
                includeDomains: ['example.com',],
                toDate: '2026-12-31',
              },
            },);

            /**
             Local value for requestBody.
             */
            const requestBody = requestJsonBody(firstCall(mock,),);
            expect(firstCall(mock,).url,).toBe(`${EXA_BASE_URL}/search`,);
            expect(requestBody.query,).toBe('docs',);
            expect(requestBody.type,).toBe('fast',);
            expect(requestBody.numResults,).toBe(10,);
            expect(requestBody.excludeDomains,).toEqual(['badwikipedia.invalid',],);
            expect(requestBody.startPublishedDate,).toBe('2026-01-01',);
            expect(requestBody.includeDomains,).toEqual(['example.com',],);
            expect(requestBody.endPublishedDate,).toBe('2026-12-31',);
          },
        },),
        it({
          name: 'filters Exa-incompatible bare suffixes out of forwarded blocklist',
          fn: async () => {
            expect(exaForwardableBlocklist(BLOCKLIST,),).toEqual(['badwikipedia.invalid',],);
          },
        },),
      ],
    },),
    describe({
      name: createSearchFetchClient.name,
      children: [
        it({
          name: 'search uses DeepSeek first when DeepSeek key is configured',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: DEEPSEEK_SEARCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              deepseekApiKey: DEEPSEEK_API_KEY,
              exaApiKey: EXA_API_KEY,
              linkupApiKey: LINKUP_API_KEY,
            },);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query: 'docs', }, },);
            /**
             Local value for requestBody.
             */
            const requestBody = requestJsonBody(firstCall(mock,),);

            expect(result.provider,).toBe('deepseek',);
            expect(result.response,).toEqual(DEEPSEEK_SEARCH_RESPONSE,);
            expect(result.fallbackChain,).toBeUndefined();
            expect(mock.calls,).toHaveLength(1,);
            expect(firstCall(mock,).url,).toBe(`${DEEPSEEK_BASE_URL}/v1/tools/search`,);
            expect(requestBody,).toEqual({
              query: 'docs',
              count: 50,
              need_content: true,
            },);
          },
        },),
        it({
          name: 'search uses Exa when only Exa key is configured',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: EXA_SEARCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({ mock, exaApiKey: EXA_API_KEY, linkupApiKey: LINKUP_API_KEY, },);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query: 'docs', }, },);
            /**
             Local value for firstStep.
             */
            const [firstStep,] = result.fallbackChain ?? [];

            expect(result.provider,).toBe('exa',);
            expect(result.response,).toEqual(EXA_SEARCH_RESPONSE,);
            expect(firstStep?.reason,).toBe('missing DeepSeek API key',);
            expect(mock.calls,).toHaveLength(1,);
            expect(firstCall(mock,).url,).toBe(`${EXA_BASE_URL}/search`,);
          },
        },),
        it({
          name: 'search falls back to Linkup when earlier keys are missing',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: LINKUP_SEARCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({ mock, linkupApiKey: LINKUP_API_KEY, },);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query: 'docs', }, },);
            /**
             Local value for steps.
             */
            const [firstStep, secondStep,] = result.fallbackChain ?? [];

            expect(result.provider,).toBe('linkup',);
            expect(result.fallbackChain,).toHaveLength(2,);
            expect(firstStep?.from,).toBe('deepseek',);
            expect(firstStep?.to,).toBe('exa',);
            expect(firstStep?.reason,).toBe('missing DeepSeek API key',);
            expect(secondStep?.from,).toBe('exa',);
            expect(secondStep?.to,).toBe('linkup',);
            expect(secondStep?.reason,).toBe('missing Exa API key',);
            expect(firstCall(mock,).url,).toBe(`${LINKUP_BASE_URL}/search`,);
          },
        },),
        it({
          name: 'search falls back to Exa when DeepSeek request fails',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [
                { body: { message: 'deepseek down', }, status: 502, statusText: 'Bad Gateway', },
                { body: EXA_SEARCH_RESPONSE, },
              ],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              deepseekApiKey: DEEPSEEK_API_KEY,
              exaApiKey: EXA_API_KEY,
            },);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query: 'docs', }, },);
            /**
             Local value for firstStep.
             */
            const [firstStep,] = result.fallbackChain ?? [];

            expect(result.provider,).toBe('exa',);
            expect(result.fallbackChain,).toHaveLength(1,);
            expect(firstStep?.from,).toBe('deepseek',);
            expect(firstStep?.to,).toBe('exa',);
            expect(firstStep?.reason,).toContain('deepseek down',);
            expect(mock.calls.map(function callUrl(call,) {
              return call.url;
            },),).toEqual([
              `${DEEPSEEK_BASE_URL}/v1/tools/search`,
              `${EXA_BASE_URL}/search`,
            ],);
          },
        },),
        it({
          name: 'search falls back to Linkup when DeepSeek and Exa fail',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [
                { body: { message: 'deepseek down', }, status: 502, statusText: 'Bad Gateway', },
                { body: { message: 'exa down', }, status: 502, statusText: 'Bad Gateway', },
                { body: LINKUP_SEARCH_RESPONSE, },
              ],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              deepseekApiKey: DEEPSEEK_API_KEY,
              exaApiKey: EXA_API_KEY,
              linkupApiKey: LINKUP_API_KEY,
            },);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query: 'docs', }, },);

            expect(result.provider,).toBe('linkup',);
            expect(result.fallbackChain,).toHaveLength(2,);
            expect(result.fallbackChain?.[0]?.from,).toBe('deepseek',);
            expect(result.fallbackChain?.[1]?.from,).toBe('exa',);
            expect(mock.calls,).toHaveLength(3,);
          },
        },),
        it({
          name: 'search skips DeepSeek when the query exceeds the 128 code point limit',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: EXA_SEARCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              deepseekApiKey: DEEPSEEK_API_KEY,
              exaApiKey: EXA_API_KEY,
            },);
            /**
             Long query that DeepSeek would reject.
             */
            const query = 'x'.repeat(129,);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query, }, },);
            /**
             Local value for firstStep.
             */
            const [firstStep,] = result.fallbackChain ?? [];

            expect(result.provider,).toBe('exa',);
            expect(firstStep?.reason,).toContain('128',);
            expect(mock.calls,).toHaveLength(1,);
            expect(firstCall(mock,).url,).toBe(`${EXA_BASE_URL}/search`,);
          },
        },),
        it({
          name: 'search falls back to Linkup when Exa request fails',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [
                { body: { message: 'bad gateway', }, status: 502, statusText: 'Bad Gateway', },
                { body: LINKUP_SEARCH_RESPONSE, },
              ],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({ mock, exaApiKey: EXA_API_KEY, linkupApiKey: LINKUP_API_KEY, },);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query: 'docs', }, },);

            expect(result.provider,).toBe('linkup',);
            expect(result.fallbackChain,).toHaveLength(2,);
            expect(result.fallbackChain?.[0]?.from,).toBe('deepseek',);
            expect(result.fallbackChain?.[1]?.from,).toBe('exa',);
            expect(result.fallbackChain?.[1]?.to,).toBe('linkup',);
            expect(mock.calls.map(function callUrl(call,) {
              return call.url;
            },),).toEqual([
              `${EXA_BASE_URL}/search`,
              `${LINKUP_BASE_URL}/search`,
            ],);
          },
        },),
        it({
          name: 'fetch uses DeepSeek first when DeepSeek key is configured',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: DEEPSEEK_FETCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              deepseekApiKey: DEEPSEEK_API_KEY,
              exaApiKey: EXA_API_KEY,
              linkupApiKey: LINKUP_API_KEY,
            },);

            /**
             Local value for result.
             */
            const result = await client.fetch({ input: { url: 'https://example.com', }, },);
            /**
             Local value for requestBody.
             */
            const requestBody = requestJsonBody(firstCall(mock,),);

            expect(result.provider,).toBe('deepseek',);
            expect(result.response,).toEqual({ markdown: '# DeepSeek page', },);
            expect(result.fallbackChain,).toBeUndefined();
            expect(mock.calls,).toHaveLength(1,);
            expect(firstCall(mock,).url,).toBe(`${DEEPSEEK_BASE_URL}/v1/tools/fetch`,);
            expect(requestBody,).toEqual({ url: 'https://example.com', },);
          },
        },),
        it({
          name: 'fetch falls back to Linkup when DeepSeek fetch fails',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [
                { body: { message: 'deepseek down', }, status: 502, statusText: 'Bad Gateway', },
                { body: LINKUP_FETCH_RESPONSE, },
              ],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              deepseekApiKey: DEEPSEEK_API_KEY,
              linkupApiKey: LINKUP_API_KEY,
            },);

            /**
             Local value for result.
             */
            const result = await client.fetch({ input: { url: 'https://example.com', }, },);

            expect(result.provider,).toBe('linkup',);
            expect(result.response,).toEqual(LINKUP_FETCH_RESPONSE,);
            expect(result.fallbackChain,).toHaveLength(1,);
            expect(result.fallbackChain?.[0]?.from,).toBe('deepseek',);
            expect(result.fallbackChain?.[0]?.to,).toBe('linkup',);
            expect(mock.calls.map(function callUrl(call,) {
              return call.url;
            },),).toEqual([
              `${DEEPSEEK_BASE_URL}/v1/tools/fetch`,
              `${LINKUP_BASE_URL}/fetch`,
            ],);
          },
        },),
        it({
          name: 'fetch falls back to Linkup for a non-GitHub URL when no DeepSeek key is configured',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: LINKUP_FETCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({ mock, exaApiKey: EXA_API_KEY, linkupApiKey: LINKUP_API_KEY, },);

            /**
             Local value for result.
             */
            const result = await client.fetch({ input: { url: 'https://example.com', }, },);
            /**
             Local value for firstStep.
             */
            const [firstStep,] = result.fallbackChain ?? [];

            expect(result.provider,).toBe('linkup',);
            expect(result.response,).toEqual(LINKUP_FETCH_RESPONSE,);
            expect(firstStep?.from,).toBe('deepseek',);
            expect(firstStep?.reason,).toBe('missing DeepSeek API key',);
            expect(mock.calls,).toHaveLength(1,);
            expect(firstCall(mock,).url,).toBe(`${LINKUP_BASE_URL}/fetch`,);
          },
        },),
        it({
          name: 'fetch falls back to Exa contents when earlier keys are missing',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: EXA_FETCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({ mock, exaApiKey: EXA_API_KEY, },);

            /**
             Local value for result.
             */
            const result = await client.fetch({ input: { url: 'https://example.com', }, },);
            /**
             Local value for requestBody.
             */
            const requestBody = requestJsonBody(firstCall(mock,),);
            /**
             Local value for steps.
             */
            const [firstStep, secondStep,] = result.fallbackChain ?? [];

            expect(result.provider,).toBe('exa',);
            expect(result.fallbackChain,).toHaveLength(2,);
            expect(firstStep?.reason,).toBe('missing DeepSeek API key',);
            expect(secondStep?.from,).toBe('linkup',);
            expect(secondStep?.reason,).toBe('missing Linkup API key',);
            expect(firstCall(mock,).url,).toBe(`${EXA_BASE_URL}/contents`,);
            expect(requestBody,).toEqual({
              urls: ['https://example.com',],
              text: true,
            },);
          },
        },),
        it({
          name: 'fetch routes a mapped GitHub URL through gh without any HTTP request',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({ responses: [], },);
            /**
             Local value for gh.
             */
            const gh = recordingGhClient({ response: GH_FETCH_RESPONSE, },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              linkupApiKey: LINKUP_API_KEY,
              ghClient: gh.client,
            },);

            /**
             Local value for result.
             */
            const result = await client.fetch({ input: { url: GITHUB_REPOSITORY_URL, }, },);

            expect(result.provider,).toBe('gh',);
            expect(result.response,).toEqual(GH_FETCH_RESPONSE,);
            expect(result.fallbackChain,).toBeUndefined();
            expect(mock.calls,).toHaveLength(0,);
            expect(gh.requests,).toHaveLength(1,);
            expect(gh.requests[0]?.url,).toBe(GITHUB_REPOSITORY_URL,);
            expect(gh.requests[0]?.kind,).toBe('repository-home',);
            expect(gh.requests[0]?.attempts[0]?.invocations[0]?.args,).toEqual([
              'repo',
              'view',
              'cli/cli',
            ],);
          },
        },),
        it({
          name: 'fetch falls back to Linkup when gh fails and records the gh step',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: LINKUP_FETCH_RESPONSE, },],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              linkupApiKey: LINKUP_API_KEY,
              ghClient: failingGhClient({ reason: GH_FAILURE_REASON, },).client,
            },);

            /**
             Local value for result.
             */
            const result = await client.fetch({ input: { url: GITHUB_REPOSITORY_URL, }, },);

            expect(result.provider,).toBe('linkup',);
            expect(result.response,).toEqual(LINKUP_FETCH_RESPONSE,);
            expect(result.fallbackChain,).toHaveLength(2,);
            expect(result.fallbackChain?.[0]?.from,).toBe('gh',);
            expect(result.fallbackChain?.[0]?.to,).toBe('deepseek',);
            expect(result.fallbackChain?.[0]?.reason,).toBe(GH_FAILURE_REASON,);
            expect(result.fallbackChain?.[1]?.from,).toBe('deepseek',);
            expect(result.fallbackChain?.[1]?.to,).toBe('linkup',);
            expect(result.fallbackChain?.[1]?.reason,).toBe('missing DeepSeek API key',);
          },
        },),
        it({
          name: 'fetch records every earlier step when gh and Linkup fail and Exa answers',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [
                { body: { message: 'bad gateway', }, status: 502, statusText: 'Bad Gateway', },
                { body: EXA_FETCH_RESPONSE, },
              ],
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              exaApiKey: EXA_API_KEY,
              linkupApiKey: LINKUP_API_KEY,
              ghClient: failingGhClient({ reason: GH_FAILURE_REASON, },).client,
            },);

            /**
             Local value for result.
             */
            const result = await client.fetch({ input: { url: GITHUB_REPOSITORY_URL, }, },);

            expect(result.provider,).toBe('exa',);
            expect(result.fallbackChain,).toHaveLength(3,);
            expect(result.fallbackChain?.[0]?.from,).toBe('gh',);
            expect(result.fallbackChain?.[0]?.to,).toBe('deepseek',);
            expect(result.fallbackChain?.[0]?.reason,).toBe(GH_FAILURE_REASON,);
            expect(result.fallbackChain?.[1]?.from,).toBe('deepseek',);
            expect(result.fallbackChain?.[1]?.to,).toBe('linkup',);
            expect(result.fallbackChain?.[1]?.reason,).toBe('missing DeepSeek API key',);
            expect(result.fallbackChain?.[2]?.from,).toBe('linkup',);
            expect(result.fallbackChain?.[2]?.to,).toBe('exa',);
          },
        },),
        it({
          name: 'fetch sends an unmapped GitHub URL to Linkup without calling gh',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: LINKUP_FETCH_RESPONSE, },],
            },);
            /**
             Local value for gh.
             */
            const gh = recordingGhClient({ response: GH_FETCH_RESPONSE, },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              linkupApiKey: LINKUP_API_KEY,
              ghClient: gh.client,
            },);

            /**
             Local value for result.
             */
            const result = await client.fetch({ input: { url: UNMAPPED_GITHUB_URL, }, },);

            expect(result.provider,).toBe('linkup',);
            expect(gh.requests,).toHaveLength(0,);
            expect(planGitHubFetch({ url: UNMAPPED_GITHUB_URL, },).planned,).toBe(false,);
            expect(mock.calls,).toHaveLength(1,);
          },
        },),
        it({
          name: 'fetch rethrows a cancellation instead of falling back to Linkup',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({ responses: [], },);
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            controller.abort();
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              linkupApiKey: LINKUP_API_KEY,
              ghClient: failingGhClient({ reason: 'The operation was aborted', },).client,
            },);

            /**
             Local value for caught.
             */
            let caught: unknown;
            try {
              await client.fetch({
                input: { url: GITHUB_REPOSITORY_URL, },
                signal: controller.signal,
              },);
            }
            catch (error: unknown) {
              caught = error;
            }

            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toBe('This operation was aborted',);
            expect(mock.calls,).toHaveLength(0,);
          },
        },),
        it({
          name: 'fetch rethrows a cancellation arriving during the Linkup call instead of continuing to Exa',
          fn: async () => {
            /**
             Local value for controller.
             */
            const controller = new AbortController();
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [
                { body: { message: 'bad gateway', }, status: 502, statusText: 'Bad Gateway', },
                { body: EXA_FETCH_RESPONSE, },
              ],
              onCall: function cancelFirstCall(): void {
                controller.abort();
              },
            },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              exaApiKey: EXA_API_KEY,
              linkupApiKey: LINKUP_API_KEY,
              ghClient: failingGhClient({ reason: GH_FAILURE_REASON, },).client,
            },);

            /**
             Local value for caught.
             */
            let caught: unknown;
            try {
              await client.fetch({
                input: { url: GITHUB_REPOSITORY_URL, },
                signal: controller.signal,
              },);
            }
            catch (error: unknown) {
              caught = error;
            }

            expect(caught,).toBeInstanceOf(Error,);
            expect(mock.calls,).toHaveLength(1,);
            expect(firstCall(mock,).url,).toBe(`${LINKUP_BASE_URL}/fetch`,);
          },
        },),
        it({
          name: 'search never routes through gh',
          fn: async () => {
            /**
             Local value for mock.
             */
            const mock = mockFetch({
              responses: [{ body: EXA_SEARCH_RESPONSE, },],
            },);
            /**
             Local value for gh.
             */
            const gh = recordingGhClient({ response: GH_FETCH_RESPONSE, },);
            /**
             Local value for client.
             */
            const client = clientWithMock({
              mock,
              exaApiKey: EXA_API_KEY,
              ghClient: gh.client,
            },);

            /**
             Local value for result.
             */
            const result = await client.search({ input: { query: GITHUB_REPOSITORY_URL, }, },);

            expect(result.provider,).toBe('exa',);
            expect(gh.requests,).toHaveLength(0,);
          },
        },),
      ],
    },),
  ],
},);

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
   Fetch implementation passed to client.
   */
  readonly fetchImpl: FetchLike;
  /**
   Recorded fetch calls.
   */
  readonly calls: FetchCall[];
};

/**
 Client mock options.
 */
type ClientWithMockOptions = {
  /**
   Fetch mock.
   */
  readonly mock: FetchMock;
  /**
   Optional DeepSeek API key.
   */
  readonly deepseekApiKey?: string;
  /**
   Optional Exa API key.
   */
  readonly exaApiKey?: string;
  /**
   Optional Linkup API key.
   */
  readonly linkupApiKey?: string;
  /**
   Optional gh client replacing the real child process boundary.
   */
  readonly ghClient?: GhClient;
};

/**
 Recorded gh client harness.
 */
type GhClientFixture = {
  /**
   gh client injected into the routing client.
   */
  readonly client: GhClient;
  /**
   Requests received in call order.
   */
  readonly requests: GhClientFetchOptions[];
};

/**
 Create provider-routing client with common fixtures.
 
 @param mock - fetch mock harness
 
 @param exaApiKey - optional Exa API key
 
 @param linkupApiKey - optional Linkup API key
 
 @returns provider-routing client
 */
function clientWithMock(
  {
    mock,
    deepseekApiKey,
    exaApiKey,
    linkupApiKey,
    ghClient,
  }: ClientWithMockOptions,
) {
  return createSearchFetchClient({
    ...(deepseekApiKey === undefined ? {} : { deepseekApiKey, }),
    ...(exaApiKey === undefined ? {} : { exaApiKey, }),
    ...(linkupApiKey === undefined ? {} : { linkupApiKey, }),
    blocklist: BLOCKLIST,
    deepseekBaseUrl: DEEPSEEK_BASE_URL,
    exaBaseUrl: EXA_BASE_URL,
    linkupBaseUrl: LINKUP_BASE_URL,
    fetchImpl: mock.fetchImpl,
    ...(ghClient === undefined ? {} : { ghClient, }),
  },);
}

/**
 Create a gh client recording requests and returning one scripted response.
 
 @param response - response returned for every request
 
 @returns recording gh client harness
 */
function recordingGhClient({ response, }: { readonly response: unknown; }): GhClientFixture {
  /**
   Recorded requests.
   */
  const requests: GhClientFetchOptions[] = [];
  return {
    requests,
    client: {
      fetch: async function recordFetch(options: GhClientFetchOptions,): Promise<unknown> {
        requests.push(options,);
        return response;
      },
    },
  };
}

/**
 Create a gh client recording requests and failing with one scripted reason.
 
 @param reason - failure message thrown for every request
 
 @returns failing gh client harness
 */
function failingGhClient({ reason, }: { readonly reason: string; }): GhClientFixture {
  /**
   Recorded requests.
   */
  const requests: GhClientFetchOptions[] = [];
  return {
    requests,
    client: {
      fetch: async function failFetch(options: GhClientFetchOptions,): Promise<unknown> {
        requests.push(options,);
        throw new Error(reason,);
      },
    },
  };
}

/**
 Create ordered JSON response fetch mock.
 
 @param responses - responses returned in call order
 
 @returns mock fetch harness
 */
function mockFetch(
  {
    responses,
    onCall,
  }: {
    readonly responses: readonly MockResponse[];
    readonly onCall?: (index: number,) => void;
  },
): FetchMock {
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
    if (onCall !== undefined)
      onCall(calls.length - 1,);
    /**
     Local value for response.
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
 Return first recorded fetch call.
 
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
 Parse recorded JSON request body.
 
 @param call - fetch call
 
 @returns parsed JSON object
 */
function requestJsonBody(call: FetchCall,): Record<string, unknown> {
  if ((typeof call.init.body) !== 'string')
    throw new Error('request body was not a string',);
  return JSON.parse(call.init.body,) as Record<string, unknown>;
}

//endregion Helpers
