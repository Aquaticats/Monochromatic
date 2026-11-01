/**
 Unit tests for Pi Search Fetch tool definitions.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  fakeExtensionToolContext,
} from './extension-tool-context-fixture.ts';
import {
  createLinkupTools,
  LINKUP_WEB_FETCH_TOOL_NAME,
  LINKUP_WEB_SEARCH_TOOL_NAME,
  type LinkupConfig,
  type SearchFetchToolClient,
  type LinkupWebFetchInput,
  type LinkupWebSearchInput,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Blocked host fixture.
 */
const BLOCKED_HOST = 'badwikipedia.invalid';

/**
 Good search result URL fixture.
 */
const GOOD_RESULT_URL = 'https://example.com/good';

/**
 Fixed config fixture.
 */
const CONFIG: LinkupConfig = {
  exaApiKey: 'exa-key',
  linkupApiKey: 'linkup-key',
  blocklist: [BLOCKED_HOST,],
  source: {
    path: '/home/test/.pi/agent/extensions/pi-search-fetch.json',
    loaded: true,
  },
};

/**
 Empty Linkup response fixture.
 */
const EMPTY_RESPONSE = { results: [], };

/**
 Exact metadata search response fixture expected to render its object results as JSONL.
 */
const METADATA_SEARCH_RESPONSE = {
  costDollars: {},
  requestId: 'search-request',
  resolvedSearchType: '',
  results: [{
    name: 'allowed',
    url: GOOD_RESULT_URL,
  },],
  searchTime: 1,
} as const;

//endregion Fixtures

await describe({
  name: createLinkupTools.name,
  children: [
    it({
      name: 'creates only web_search and web_fetch tools',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockClient({
          searchResponse: EMPTY_RESPONSE,
          fetchResponse: { markdown: 'ok', },
        },);

        /**
         Local value for tools.
         */
        const tools = createLinkupTools({
          config: CONFIG,
          client: mock.client,
        },);

        expect(tools.map(function toolName(tool,) {
          return tool.name;
        },),).toEqual([
          LINKUP_WEB_SEARCH_TOOL_NAME,
          LINKUP_WEB_FETCH_TOOL_NAME,
        ],);
      },
    },),
    it({
      name: 'search ignored params produce model-visible warnings and do not reach client',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockClient({
          searchResponse: EMPTY_RESPONSE,
          fetchResponse: { markdown: 'ok', },
        },);
        /**
         Local value for searchTool.
         */
        const searchTool = searchToolFrom(mock.client,);

        /**
         Local value for result.
         */
        const result = await searchTool.execute(
          'tool-call-search-warning',
          {
            query: 'ignored params',
            depth: 'deep',
            limit: 2,
            maxResults: 5,
          },
          undefined,
          undefined,
          await fakeExtensionToolContext(),
        );

        expect(result.content,).toHaveLength(2,);
        /**
         Local value for warning.
         */
        const warning = textContentAt({ result, index: 0, },);
        expect(warning,).toContain('depth, limit, maxResults',);
        expect(warning,).toContain('DeepSeek search first',);
        expect(mock.searchCalls,).toHaveLength(1,);
        /**
         Local value for searchInput.
         */
        const searchInput = firstSearchInput(mock,);
        expect('depth' in searchInput,).toBe(false,);
        expect('maxResults' in searchInput,).toBe(false,);
      },
    },),
    it({
      name: 'search removes blocked result URLs from model-visible output and preserves raw response in details',
      fn: async () => {
        /**
         Local value for rawResponse.
         */
        const rawResponse = {
          results: [
            {
              name: 'blocked exact',
              url: `https://${BLOCKED_HOST}/a`,
            },
            {
              name: 'allowed',
              url: GOOD_RESULT_URL,
            },
            {
              name: 'blocked subdomain',
              url: `https://www.${BLOCKED_HOST}/b`,
            },
          ],
        };
        /**
         Local value for mock.
         */
        const mock = mockClient({
          searchResponse: rawResponse,
          fetchResponse: { markdown: 'ok', },
        },);
        /**
         Local value for searchTool.
         */
        const searchTool = searchToolFrom(mock.client,);

        /**
         Local value for result.
         */
        const result = await searchTool.execute(
          'tool-call-search-filter',
          { query: 'filter', },
          undefined,
          undefined,
          await fakeExtensionToolContext(),
        );

        /**
         Local value for visible.
         */
        const visible = parseVisibleJsonl({ result, index: 0, },);
        expect(visible,).toHaveLength(1,);
        expect(resultUrl(visible[0],),).toBe(GOOD_RESULT_URL,);
        expect(resultsArray(result.details.rawLinkupResponse,),).toHaveLength(3,);
        expect(result.details.removedBlockedUrls,).toEqual([
          `https://${BLOCKED_HOST}/a`,
          `https://www.${BLOCKED_HOST}/b`,
        ],);
        expect(result.details.provider,).toBe('exa',);
      },
    },),
    it({
      name: 'search renders exact metadata response results as JSONL',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockClient({
          searchResponse: METADATA_SEARCH_RESPONSE,
          fetchResponse: { markdown: 'ok', },
        },);
        /**
         Local value for searchTool.
         */
        const searchTool = searchToolFrom(mock.client,);

        /**
         Local value for result.
         */
        const result = await searchTool.execute(
          'tool-call-search-metadata',
          { query: 'metadata', },
          undefined,
          undefined,
          await fakeExtensionToolContext(),
        );

        expect(parseVisibleJsonl({ result, index: 0, },),).toEqual(METADATA_SEARCH_RESPONSE.results,);
        expect(textContentAt({ result, index: 0, },),).not.toContain('requestId',);
        expect(result.details.linkupResponse,).toBe(METADATA_SEARCH_RESPONSE,);
      },
    },),
    it({
      name: 'fetch ignored params produce model-visible warnings',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockClient({
          searchResponse: EMPTY_RESPONSE,
          fetchResponse: { markdown: 'ok', },
        },);
        /**
         Local value for fetchTool.
         */
        const fetchTool = fetchToolFrom(mock.client,);

        /**
         Local value for result.
         */
        const result = await fetchTool.execute(
          'tool-call-fetch-warning',
          {
            url: 'https://example.com',
            renderJs: false,
            includeRawHtml: true,
          },
          undefined,
          undefined,
          await fakeExtensionToolContext(),
        );

        expect(result.content,).toHaveLength(2,);
        expect(textContentAt({ result, index: 0, },),).toContain('renderJs, includeRawHtml',);
        expect(mock.fetchCalls,).toHaveLength(1,);
        /**
         Local value for fetchInput.
         */
        const fetchInput = firstFetchInput(mock,);
        expect('renderJs' in fetchInput,).toBe(false,);
      },
    },),
    it({
      name: 'blocked fetch throws before mocked client fetch is called',
      fn: async () => {
        /**
         Local value for mock.
         */
        const mock = mockClient({
          searchResponse: EMPTY_RESPONSE,
          fetchResponse: { markdown: 'ok', },
        },);
        /**
         Local value for fetchTool.
         */
        const fetchTool = fetchToolFrom(mock.client,);

        let caught: unknown;
        try {
          await fetchTool.execute(
            'tool-call-blocked-fetch',
            { url: `https://www.${BLOCKED_HOST}/page`, },
            undefined,
            undefined,
            await fakeExtensionToolContext(),
          );
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain(`Blocked by pi-search-fetch blocklist: ${BLOCKED_HOST}`,);
        expect(mock.fetchCalls,).toHaveLength(0,);
      },
    },),
    it({
      name: 'fetch stores markdown-only response in details and visible output is markdown',
      fn: async () => {
        /**
         Local value for fetchResponse.
         */
        const fetchResponse = { markdown: 'Fetched page', };
        /**
         Local value for mock.
         */
        const mock = mockClient({
          searchResponse: EMPTY_RESPONSE,
          fetchResponse,
        },);
        /**
         Local value for fetchTool.
         */
        const fetchTool = fetchToolFrom(mock.client,);

        /**
         Local value for result.
         */
        const result = await fetchTool.execute(
          'tool-call-fetch-markdown',
          { url: 'https://example.com', },
          undefined,
          undefined,
          await fakeExtensionToolContext(),
        );

        expect(result.details.linkupResponse,).toBe(fetchResponse,);
        expect(textContentAt({ result, index: 0, },),).toBe(fetchResponse.markdown,);
      },
    },),
    it({
      name: 'fetch filters base64 Markdown images while preserving raw response details',
      fn: async () => {
        /**
         Fetch response containing line-wrapped linked image data.
         */
        const fetchResponse = {
          markdown: [
            'Before',
            '[![Microsoft',
            ' Logo](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA',
            ' ALgAAAAmCAYAAAB3X1H0AAAA)]()',
            'After',
          ].join('\n',),
        };
        /**
         Mock client returning unfiltered provider response.
         */
        const mock = mockClient({
          searchResponse: EMPTY_RESPONSE,
          fetchResponse,
        },);
        /**
         Fetch tool under test.
         */
        const fetchTool = fetchToolFrom(mock.client,);

        /**
         Final filtered tool result.
         */
        const result = await fetchTool.execute(
          'tool-call-fetch-data-image-filter',
          { url: 'https://example.com', },
          undefined,
          undefined,
          await fakeExtensionToolContext(),
        );

        expect(textContentAt({ result, index: 0, },),).toBe('Before\nAfter',);
        expect(result.details.rawLinkupResponse,).toBe(fetchResponse,);
        expect(result.details.linkupResponse,).toEqual({ markdown: 'Before\nAfter', },);
      },
    },),
    it({
      name: 'fetch stores non-exact markdown response in details and visible output is JSON',
      fn: async () => {
        /**
         Local value for fetchResponse.
         */
        const fetchResponse = {
          markdown: 'Fetched page',
          title: 'Fetched title',
        };
        /**
         Local value for mock.
         */
        const mock = mockClient({
          searchResponse: EMPTY_RESPONSE,
          fetchResponse,
        },);
        /**
         Local value for fetchTool.
         */
        const fetchTool = fetchToolFrom(mock.client,);

        /**
         Local value for result.
         */
        const result = await fetchTool.execute(
          'tool-call-fetch-json',
          { url: 'https://example.com', },
          undefined,
          undefined,
          await fakeExtensionToolContext(),
        );

        expect(result.details.linkupResponse,).toBe(fetchResponse,);
        expect(parseVisibleJson({ result, index: 0, },),).toEqual(fetchResponse,);
      },
    },),
  ],
},);

//region Helpers

/**
 Search call record.
 */
type SearchCall = {
  /**
   Search input passed to client.
   */
  readonly input: LinkupWebSearchInput;
};

/**
 Fetch call record.
 */
type FetchCall = {
  /**
   Fetch input passed to client.
   */
  readonly input: LinkupWebFetchInput;
};

/**
 Mock client harness.
 */
type MockClient = {
  /**
   Client implementation.
   */
  readonly client: SearchFetchToolClient;
  /**
   Recorded search calls.
   */
  readonly searchCalls: SearchCall[];
  /**
   Recorded fetch calls.
   */
  readonly fetchCalls: FetchCall[];
};

/**
 Build mock Linkup client.
 
 @param searchResponse - response returned by search
 
 @param fetchResponse - response returned by fetch
 
 @returns mock client harness
 */
function mockClient(
  {
    searchResponse,
    fetchResponse,
  }: {
    readonly searchResponse: unknown;
    readonly fetchResponse: unknown;
  },
): MockClient {
  /**
   Local value for searchCalls.
   */
  const searchCalls: SearchCall[] = [];
  /**
   Local value for fetchCalls.
   */
  const fetchCalls: FetchCall[] = [];
  return {
    client: {
      async search(options,) {
        searchCalls.push({ input: options.input, },);
        return {
          provider: 'exa',
          response: searchResponse,
        };
      },
      async fetch(options,) {
        fetchCalls.push({ input: options.input, },);
        return {
          provider: 'linkup',
          response: fetchResponse,
        };
      },
    },
    searchCalls,
    fetchCalls,
  };
}

/**
 Return first recorded search input.
 
 @param mock - mock client harness
 
 @returns first search input
 */
function firstSearchInput(mock: MockClient,): LinkupWebSearchInput {
  /**
   First recorded search call.
   */
  const [call,] = mock.searchCalls;
  if (call === undefined)
    throw new Error('missing search call',);
  return call.input;
}

/**
 Return first recorded fetch input.
 
 @param mock - mock client harness
 
 @returns first fetch input
 */
function firstFetchInput(mock: MockClient,): LinkupWebFetchInput {
  /**
   First recorded fetch call.
   */
  const [call,] = mock.fetchCalls;
  if (call === undefined)
    throw new Error('missing fetch call',);
  return call.input;
}

/**
 Return search tool from a fresh tool set.
 
 @param client - mock client
 
 @returns search tool
 */
function searchToolFrom(client: SearchFetchToolClient,) {
  /**
   Local value for tool.
   */
  const tool = createLinkupTools({
    config: CONFIG,
    client,
  },).find(function isSearchTool(candidate,) {
    return candidate.name === LINKUP_WEB_SEARCH_TOOL_NAME;
  },);
  if (tool === undefined)
    throw new Error('missing search tool',);
  return tool;
}

/**
 Return fetch tool from a fresh tool set.
 
 @param client - mock client
 
 @returns fetch tool
 */
function fetchToolFrom(client: SearchFetchToolClient,) {
  /**
   Local value for tool.
   */
  const tool = createLinkupTools({
    config: CONFIG,
    client,
  },).find(function isFetchTool(candidate,) {
    return candidate.name === LINKUP_WEB_FETCH_TOOL_NAME;
  },);
  if (tool === undefined)
    throw new Error('missing fetch tool',);
  return tool;
}

/**
 Return text content at index from a tool result.
 
 @param result - tool result
 
 @param index - content item index
 
 @returns text content
 */
function textContentAt(
  {
    result,
    index,
  }: {
    readonly result: { readonly content: readonly unknown[]; };
    readonly index: number;
  },
): string {
  /**
   Local value for item.
   */
  const item = result.content[index];
  if ((!isRecord(item,)) || (item.type !== 'text') || ((typeof item.text) !== 'string'))
    throw new Error(`content item ${String(index,)} was not text`,);
  return item.text;
}

/**
 Parse visible JSON content at index.
 
 @param result - tool result
 
 @param index - content item index
 
 @returns parsed JSON
 */
function parseVisibleJson(
  {
    result,
    index,
  }: {
    readonly result: { readonly content: readonly unknown[]; };
    readonly index: number;
  },
): unknown {
  return JSON.parse(textContentAt({ result, index, },),) as unknown;
}

/**
 Parse visible JSONL content at index.
 
 @param result - tool result
 
 @param index - content item index
 
 @returns parsed JSONL items
 */
function parseVisibleJsonl(
  {
    result,
    index,
  }: {
    readonly result: { readonly content: readonly unknown[]; };
    readonly index: number;
  },
): readonly unknown[] {
  /**
   Local value for text.
   */
  const text = textContentAt({ result, index, },);
  if (text === '')
    return [];
  return text
    .split('\n',)
    .map(function parseJsonLine(line,) {
      return JSON.parse(line,) as unknown;
    },);
}

/**
 Extract Linkup results array from a response.
 
 @param value - Linkup response value
 
 @returns results array
 */
function resultsArray(value: unknown,): readonly unknown[] {
  if ((!isRecord(value,)) || (!Array.isArray(value.results,)))
    throw new Error('response did not contain results array',);
  return value.results;
}

/**
 Extract result URL from a result object.
 
 @param value - result value
 
 @returns URL string
 */
function resultUrl(value: unknown,): string {
  if ((!isRecord(value,)) || ((typeof value.url) !== 'string'))
    throw new Error('result did not contain url',);
  return value.url;
}

/**
 Return whether value is a non-null object record.
 
 @param value - unknown value
 
 @returns whether value can be read by string keys
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

//endregion Helpers
