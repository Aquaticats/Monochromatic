/**
 Unit tests for shared search and fetch tool execution.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BlockedUrlError,
  collectIgnoredKeys,
  executeWebFetchTool,
  executeWebSearchTool,
  type LinkupConfig,
  type SearchFetchToolClient,
  type SearchFetchToolUpdate,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Blocked host fixture matching the config blocklist below.
 */
const BLOCKED_HOST = 'badwikipedia.invalid';

/**
 Fixed config fixture with one blocked host.
 */
const CONFIG: LinkupConfig = {
  exaApiKey: 'exa-key',
  linkupApiKey: 'linkup-key',
  blocklist: [BLOCKED_HOST,],
  source: {
    path: '/home/test/.config/search-fetch/config.json',
    loaded: true,
  },
};

/**
 Search response fixture with no results.
 */
const EMPTY_SEARCH_RESPONSE = { results: [], };

/**
 Markdown fetch response fixture.
 */
const MARKDOWN_FETCH_RESPONSE = { markdown: '# fetched', };

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: collectIgnoredKeys.name,
      children: [
        it({
          name: 'reports unsupported keys in caller order',
          fn: async () => {
            expect(collectIgnoredKeys({
              input: {
                query: 'docs',
                depth: 'deep',
                limit: 3,
              },
              supportedKeys: ['query',],
            },),).toEqual([
              'depth',
              'limit',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: executeWebSearchTool.name,
      children: [
        it({
          name: 'returns provider response content and details',
          fn: async () => {
            /**
             Stub client capturing the search call.
             */
            const stub = stubClient({
              searchResponse: EMPTY_SEARCH_RESPONSE,
              fetchResponse: MARKDOWN_FETCH_RESPONSE,
            },);
            const result = await executeWebSearchTool({
              client: stub.client,
              config: CONFIG,
              rawParams: { query: 'docs', },
            },);

            expect(result.content,).toHaveLength(1,);
            expect(result.details.provider,).toBe('exa',);
            expect(stub.searchInputs,).toEqual([{ query: 'docs', },],);
          },
        },),
        it({
          name: 'warns about ignored keys before the response',
          fn: async () => {
            /**
             Stub client returning an empty search response.
             */
            const stub = stubClient({
              searchResponse: EMPTY_SEARCH_RESPONSE,
              fetchResponse: MARKDOWN_FETCH_RESPONSE,
            },);
            const result = await executeWebSearchTool({
              client: stub.client,
              config: CONFIG,
              rawParams: {
                query: 'docs',
                depth: 'deep',
              },
            },);

            expect(result.content,).toHaveLength(2,);
            expect(result
              .content[0]
              ?.text
              .startsWith('Warning: ignored extension-unsupported web_search parameters: depth.',),)
              .toBe(true,);
            expect(result.details.ignoredKeys,).toEqual(['depth',],);
          },
        },),
        it({
          name: 'forwards progress updates to the host callback',
          fn: async () => {
            /**
             Stub client returning an empty search response.
             */
            const stub = stubClient({
              searchResponse: EMPTY_SEARCH_RESPONSE,
              fetchResponse: MARKDOWN_FETCH_RESPONSE,
            },);
            /**
             Updates captured from execution.
             */
            const updates: SearchFetchToolUpdate[] = [];
            await executeWebSearchTool({
              client: stub.client,
              config: CONFIG,
              rawParams: { query: 'progress', },
              onUpdate: function captureUpdate(update,) {
                updates.push(update,);
              },
            },);

            expect(updates,).toHaveLength(1,);
            expect(updates[0]
              ?.content[0]
              ?.text,).toBe('Searching web for: progress',);
          },
        },),
      ],
    },),
    describe({
      name: executeWebFetchTool.name,
      children: [
        it({
          name: 'throws for blocklisted URLs before any provider call',
          fn: async () => {
            /**
             Stub client recording whether fetch ran.
             */
            const stub = stubClient({
              searchResponse: EMPTY_SEARCH_RESPONSE,
              fetchResponse: MARKDOWN_FETCH_RESPONSE,
            },);
            /**
             Blocklist failure captured instead of thrown.
             */
            const failureName = (async function runBlockedFetch(): Promise<string> {
              try {
                await executeWebFetchTool({
                  client: stub.client,
                  config: CONFIG,
                  rawParams: { url: `https://${BLOCKED_HOST}/page`, },
                },);
                return 'no failure';
              }
              catch (error: unknown) {
                return error instanceof BlockedUrlError ? error.name : 'wrong error';
              }
            })();

            expect(await failureName,).toBe('BlockedUrlError',);
            expect(stub.fetchInputs,).toEqual([],);
          },
        },),
        it({
          name: 'returns markdown content for markdown-only responses',
          fn: async () => {
            /**
             Stub client returning a markdown fetch response.
             */
            const stub = stubClient({
              searchResponse: EMPTY_SEARCH_RESPONSE,
              fetchResponse: MARKDOWN_FETCH_RESPONSE,
            },);
            /**
             Tool result carrying markdown content and the ignored-key warning.
             */
            const result = await executeWebFetchTool({
              client: stub.client,
              config: CONFIG,
              rawParams: {
                url: 'https://example.com/page',
                raw: true,
              },
            },);

            expect(result.content,).toHaveLength(2,);
            expect(result.content[1]?.text,).toBe('# fetched',);
            expect(result.details.ignoredKeys,).toEqual(['raw',],);
          },
        },),
        it({
          name: 'rejects non-object parameter payloads loudly',
          fn: async () => {
            /**
             Stub client that must never be reached.
             */
            const stub = stubClient({
              searchResponse: EMPTY_SEARCH_RESPONSE,
              fetchResponse: MARKDOWN_FETCH_RESPONSE,
            },);
            /**
             Validation failure captured instead of thrown.
             */
            const rejected = (async function runNonObjectParams(): Promise<boolean> {
              try {
                await executeWebFetchTool({
                  client: stub.client,
                  config: CONFIG,
                  rawParams: 'not an object',
                },);
                return false;
              }
              catch (error: unknown) {
                return true;
              }
            })();

            expect(await rejected,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);

//region Helpers

/**
 Stub client harness recording calls.

 */
type StubClient = {
  /**
   Client passed to executors.
   */
  readonly client: SearchFetchToolClient;
  /**
   Sanitized search inputs seen by the stub.
   */
  readonly searchInputs: readonly unknown[];
  /**
   Sanitized fetch inputs seen by the stub.
   */
  readonly fetchInputs: readonly unknown[];
};

/**
 Build a stub provider client recording its calls.

 @param responses - canned provider responses

 @returns stub harness with recorded inputs

 @example
 ```ts
 stubClient({ searchResponse: { results: [] }, fetchResponse: { markdown: '' } });
 ```
 */
function stubClient(
  {
    searchResponse,
    fetchResponse,
  }: {
    /**
     Canned search response.
     */
    readonly searchResponse: unknown;
    /**
     Canned fetch response.
     */
    readonly fetchResponse: unknown;
  },
): StubClient {
  /**
   Recorded sanitized search inputs.
   */
  const searchInputs: unknown[] = [];
  /**
   Recorded sanitized fetch inputs.
   */
  const fetchInputs: unknown[] = [];
  return {
    client: {
      async search(options,) {
        searchInputs.push(options.input,);
        return {
          provider: 'exa',
          response: searchResponse,
        };
      },
      async fetch(options,) {
        fetchInputs.push(options.input,);
        return {
          provider: 'linkup',
          response: fetchResponse,
        };
      },
    },
    searchInputs,
    fetchInputs,
  };
}

//endregion Helpers
