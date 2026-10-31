/**
 Unit tests for the search-fetch MCP server over its built artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  JSON_RPC_INVALID_PARAMS,
  type JsonRpcErrorResponse,
  type JsonRpcInbound,
  type JsonRpcResponse,
  META_PROTOCOL_VERSION,
  PROTOCOL_VERSION,
  type ToolContent,
} from '@monochromatic-dev/mcp-stdio/ts';
import {
  createSearchFetchServer,
  createResponseStore,
  type LinkupConfig,
  type SearchFetchClient,
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
 Search response fixture with one result.
 */
const SEARCH_RESPONSE = {
  results: [
    { url: 'https://example.com/good', title: 'good', },
  ],
};

/**
 Markdown fetch response fixture.
 */
const MARKDOWN_FETCH_RESPONSE = { markdown: '# fetched', };

/**
 Oversized markdown response forcing the truncation path without touching the network.
 */
const OVERSIZED_FETCH_RESPONSE = { markdown: 'x'.repeat(200_000,) };

//endregion Fixtures

//region Helpers

/**
 Build a request carrying the protocol revision metadata every served request needs.

 @param id - request id echoed back in the response

 @param method - MCP method to invoke

 @param params - method params merged alongside the generated `_meta`

 @returns inbound request ready for `handleMessage`

 @example
 ```ts
 serverRequest(1, 'tools/list');
 ```
 */
function serverRequest(
  id: number,
  method: string,
  params: Readonly<Record<string, unknown>> = {},
): JsonRpcInbound {
  return {
    jsonrpc: '2.0',
    id,
    method,
    params: {
      ...params,
      _meta: { [META_PROTOCOL_VERSION]: PROTOCOL_VERSION, },
    },
  };
}

/**
 Build a server over stub provider responses.

 @param fetchResponse - canned fetch response the stub client returns

 @returns server handle with its response store

 @example
 ```ts
 stubServer(MARKDOWN_FETCH_RESPONSE);
 ```
 */
function stubServer(fetchResponse: unknown,): {
  /**
   Server handle dispatching JSON-RPC messages.
   */
  readonly server: ReturnType<typeof createSearchFetchServer>;
  /**
   Response store recording truncated full responses.
   */
  readonly store: ReturnType<typeof createResponseStore>;
} {
  /**
   Store recording truncated full responses.
   */
  const store = createResponseStore({ capacity: 4, },);
  /**
   Stub provider client answering with canned responses.
   */
  const client: SearchFetchClient = {
    async search() {
      return {
        provider: 'exa',
        response: SEARCH_RESPONSE,
      };
    },
    async fetch() {
      return {
        provider: 'linkup',
        response: fetchResponse,
      };
    },
  };
  return {
    server: createSearchFetchServer({
      config: CONFIG,
      client,
      store,
    },),
    store,
  };
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: createSearchFetchServer.name,
      children: [
        it({
          name: 'advertises both tools and the resource surface in discovery',
          fn: async () => {
            /**
             Server over stub responses.
             */
            const { server, } = stubServer(MARKDOWN_FETCH_RESPONSE,);
            /**
             Discovery response declaring capabilities.
             */
            const response = await server
              .handleMessage(serverRequest(1, 'server/discover',),) as JsonRpcResponse;
            /**
             Discovery payload read from the response.
             */
            const result = response.result as {
              capabilities: Record<string, unknown>;
              instructions?: string;
            };
            expect(Object.keys(result.capabilities,),).toEqual([
              'tools',
              'resources',
            ],);
            expect(result.instructions?.includes('web_search',),).toBe(true,);
          },
        },),
        it({
          name: 'lists web_search and web_fetch with their shared schemas',
          fn: async () => {
            /**
             Server over stub responses.
             */
            const { server, } = stubServer(MARKDOWN_FETCH_RESPONSE,);
            /**
             Listing response carrying tool definitions.
             */
            const response = await server
              .handleMessage(serverRequest(1, 'tools/list',),) as JsonRpcResponse;
            /**
             Tool definitions read from the response.
             */
            const result = response.result as {
              tools: readonly { name: string; description: string; inputSchema: unknown; }[];
            };
            expect(result
              .tools
              .map(function toName(tool,) {
                return tool.name;
              },),).toEqual([
                'web_search',
                'web_fetch',
              ],);
            expect(JSON.stringify(result.tools[0]?.inputSchema,),).toContain('"query"',);
          },
        },),
        it({
          name: 'warns about ignored keys and returns the provider response',
          fn: async () => {
            /**
             Server over stub responses.
             */
            const { server, } = stubServer(MARKDOWN_FETCH_RESPONSE,);
            /**
             Tool call response carrying warning and response content.
             */
            const response = await server
              .handleMessage(serverRequest(
                1,
                'tools/call',
                {
                  name: 'web_search',
                  arguments: {
                    query: 'docs',
                    depth: 'deep',
                  },
                },
              ),) as JsonRpcResponse;
            /**
             Content items read from the tool result.
             */
            const result = response.result as {
              content: readonly ToolContent[];
            };
            expect(result.content,).toHaveLength(2,);
            expect(JSON.stringify(result.content[0],).includes('depth',),).toBe(true,);
          },
        },),
        it({
          name: 'reports blocklisted fetches as tool errors before any provider call',
          fn: async () => {
            /**
             Server over stub responses.
             */
            const { server, } = stubServer(MARKDOWN_FETCH_RESPONSE,);
            /**
             Tool call response carrying the blocklist failure.
             */
            const response = await server
              .handleMessage(serverRequest(
                1,
                'tools/call',
                {
                  name: 'web_fetch',
                  arguments: { url: `https://${BLOCKED_HOST}/page`, },
                },
              ),) as JsonRpcResponse;
            /**
             Tool result read from the response.
             */
            const result = response.result as {
              isError?: boolean;
              content: readonly ToolContent[];
            };
            expect(result.isError,).toBe(true,);
            expect(JSON.stringify(result.content,).includes(BLOCKED_HOST,),).toBe(true,);
          },
        },),
        it({
          name: 'delivers truncated full responses as resource links and readable resources',
          fn: async () => {
            /**
             Server over an oversized stub response.
             */
            const { server, store, } = stubServer(OVERSIZED_FETCH_RESPONSE,);
            /**
             Tool call response carrying truncated content.
             */
            const response = await server
              .handleMessage(serverRequest(
                1,
                'tools/call',
                {
                  name: 'web_fetch',
                  arguments: { url: 'https://example.com/big', },
                },
              ),) as JsonRpcResponse;
            /**
             Content items read from the tool result.
             */
            const result = response.result as {
              content: readonly ToolContent[];
            };
            /**
             Resource link block carried after the truncated text.
             */
            const resourceLink = result
              .content
              .find(function isResourceLink(content,) {
                return content.type === 'resource_link';
              },);
            if ((resourceLink === undefined) || (resourceLink.type !== 'resource_link'))
              throw new Error('expected a resource_link content block',);
            expect(resourceLink.uri.startsWith('search-fetch://response/',),).toBe(true,);
            expect(store.list(),).toHaveLength(1,);

            /**
             Resource read response carrying the full text.
             */
            const readResponse = await server
              .handleMessage(serverRequest(
                2,
                'resources/read',
                { uri: resourceLink.uri, },
              ),) as JsonRpcResponse;
            /**
             Resource contents read from the response.
             */
            const readResult = readResponse.result as {
              contents: readonly { text?: string; }[];
            };
            expect(readResult.contents[0]?.text,).toBe(OVERSIZED_FETCH_RESPONSE.markdown,);
          },
        },),
        it({
          name: 'rejects resource reads without a string uri',
          fn: async () => {
            /**
             Server over stub responses.
             */
            const { server, } = stubServer(MARKDOWN_FETCH_RESPONSE,);
            /**
             Error response read from dispatch.
             */
            const response = await server
              .handleMessage(serverRequest(1, 'resources/read',),) as JsonRpcErrorResponse;
            expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
          },
        },),
      ],
    },),
  ],
},);
