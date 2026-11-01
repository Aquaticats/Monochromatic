/**
 Unit tests for the dynamic resource surface: `resources/list` and `resources/read`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createMcpServer,
  JSON_RPC_INVALID_PARAMS,
  JSON_RPC_METHOD_NOT_FOUND,
  type JsonRpcErrorResponse,
  type JsonRpcId,
  type JsonRpcInbound,
  type JsonRpcResponse,
  META_PROTOCOL_VERSION,
  PROTOCOL_VERSION,
  type ResourceContents,
  type ResourceDescriptor,
  type ResourceProvider,
  RESULT_TYPE_COMPLETE,
} from '@monochromatic-dev/mcp-stdio';

//region Fixtures

/**
 Identity every server in this file reports.
 */
const SERVER_IDENTITY = {
  name: 'resources-test',
  version: '0.1.0',
} as const;

/**
 One descriptor advertising the readable fixture resource.
 */
const FIXTURE_DESCRIPTOR: ResourceDescriptor = {
  uri: 'search-fetch://response/fixture',
  name: 'full response',
};

/**
 Contents of the readable fixture resource.
 */
const FIXTURE_CONTENTS: ResourceContents = {
  uri: 'search-fetch://response/fixture',
  text: 'full response text',
};

/**
 Resource provider serving exactly one fixture resource.
 */
const FIXTURE_PROVIDER: ResourceProvider = {
  list: function listFixture() {
    return [FIXTURE_DESCRIPTOR,];
  },
  read: async function readFixture(uri,) {
    if (uri !== FIXTURE_CONTENTS.uri)
      throw new Error(`unknown resource: ${uri}`);
    return FIXTURE_CONTENTS;
  },
};

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
 resourceRequest(1, 'resources/list');
 ```
 */
function resourceRequest(
  id: JsonRpcId,
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

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: 'resources capability',
      children: [
        it({
          name: 'advertises resources in discovery when a provider is configured',
          fn: async () => {
            /**
             Server exposing the fixture provider.
             */
            const server = createMcpServer({
              config: SERVER_IDENTITY,
              tools: [],
              resources: FIXTURE_PROVIDER,
            },);
            /**
             Discovery response declaring capabilities.
             */
            const response = await server
              .handleMessage(resourceRequest(1, 'server/discover',),) as JsonRpcResponse;
            /**
             Capabilities object read from the discovery result.
             */
            const { capabilities, } = response.result as { capabilities: Record<string, unknown>; };
            expect('resources' in capabilities,).toBe(true,);
          },
        },),
        it({
          name: 'omits resources in discovery when no provider is configured',
          fn: async () => {
            /**
             Server without a resource provider.
             */
            const server = createMcpServer({
              config: SERVER_IDENTITY,
              tools: [],
            },);
            /**
             Discovery response declaring capabilities.
             */
            const response = await server
              .handleMessage(resourceRequest(1, 'server/discover',),) as JsonRpcResponse;
            /**
             Capabilities object read from the discovery result.
             */
            const { capabilities, } = response.result as { capabilities: Record<string, unknown>; };
            expect('resources' in capabilities,).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'resources/list',
      children: [
        it({
          name: 'returns provider descriptors in the protocol envelope',
          fn: async () => {
            /**
             Server exposing the fixture provider.
             */
            const server = createMcpServer({
              config: SERVER_IDENTITY,
              tools: [],
              resources: FIXTURE_PROVIDER,
            },);
            /**
             Listing response carrying descriptors.
             */
            const response = await server
              .handleMessage(resourceRequest(1, 'resources/list',),) as JsonRpcResponse;
            /**
             Listing payload read from the response.
             */
            const result = response.result as {
              resultType: string;
              resources: readonly ResourceDescriptor[];
            };
            expect(result.resultType,).toBe(RESULT_TYPE_COMPLETE,);
            expect(result.resources,).toEqual([FIXTURE_DESCRIPTOR,],);
          },
        },),
        it({
          name: 'reports method-not-found without a provider',
          fn: async () => {
            /**
             Server without a resource provider.
             */
            const server = createMcpServer({
              config: SERVER_IDENTITY,
              tools: [],
            },);
            /**
             Error response read from dispatch.
             */
            const response = await server
              .handleMessage(resourceRequest(1, 'resources/list',),) as JsonRpcErrorResponse;
            expect(response.error.code,).toBe(JSON_RPC_METHOD_NOT_FOUND,);
          },
        },),
      ],
    },),
    describe({
      name: 'resources/read',
      children: [
        it({
          name: 'returns contents for a known resource URI',
          fn: async () => {
            /**
             Server exposing the fixture provider.
             */
            const server = createMcpServer({
              config: SERVER_IDENTITY,
              tools: [],
              resources: FIXTURE_PROVIDER,
            },);
            /**
             Read response carrying contents.
             */
            const response = await server
              .handleMessage(resourceRequest(
                1,
                'resources/read',
                { uri: FIXTURE_CONTENTS.uri, },
              ),) as JsonRpcResponse;
            /**
             Read payload read from the response.
             */
            const result = response.result as {
              contents: readonly ResourceContents[];
            };
            expect(result.contents,).toEqual([FIXTURE_CONTENTS,],);
          },
        },),
        it({
          name: 'reports a request error for an unknown resource URI instead of failing',
          fn: async () => {
            /**
             Server exposing the fixture provider.
             */
            const server = createMcpServer({
              config: SERVER_IDENTITY,
              tools: [],
              resources: FIXTURE_PROVIDER,
            },);
            /**
             Error response read from dispatch.
             */
            const response = await server
              .handleMessage(resourceRequest(
                1,
                'resources/read',
                { uri: 'search-fetch://response/missing', },
              ),) as JsonRpcErrorResponse;
            expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
            expect(response.error.message.includes('search-fetch://response/missing',),).toBe(true,);
          },
        },),
        it({
          name: 'rejects requests without a string uri',
          fn: async () => {
            /**
             Server exposing the fixture provider.
             */
            const server = createMcpServer({
              config: SERVER_IDENTITY,
              tools: [],
              resources: FIXTURE_PROVIDER,
            },);
            /**
             Error response read from dispatch.
             */
            const response = await server
              .handleMessage(resourceRequest(1, 'resources/read',),) as JsonRpcErrorResponse;
            expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
          },
        },),
        it({
          name: 'reports method-not-found without a provider',
          fn: async () => {
            /**
             Server without a resource provider.
             */
            const server = createMcpServer({
              config: SERVER_IDENTITY,
              tools: [],
            },);
            /**
             Error response read from dispatch.
             */
            const response = await server
              .handleMessage(resourceRequest(
                1,
                'resources/read',
                { uri: FIXTURE_CONTENTS.uri, },
              ),) as JsonRpcErrorResponse;
            expect(response.error.code,).toBe(JSON_RPC_METHOD_NOT_FOUND,);
          },
        },),
      ],
    },),
  ],
},);
