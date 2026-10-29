import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createMcpServer,
  defineTool,
  JSON_RPC_INVALID_PARAMS,
  JSON_RPC_METHOD_NOT_FOUND,
  JSON_RPC_UNSUPPORTED_PROTOCOL_VERSION,
  type JsonRpcErrorResponse,
  type JsonRpcId,
  type JsonRpcInbound,
  type JsonRpcResponse,
  META_PROTOCOL_VERSION,
  META_SERVER_INFO,
  NO_RESPONSE,
  PROTOCOL_VERSION,
  registerTools,
  strictArguments,
  RESULT_TYPE_COMPLETE,
  SUPPORTED_PROTOCOL_VERSIONS,
  type ToolContent,
  type ToolEntry,
} from '@monochromatic-dev/mcp-stdio';
import * as v from 'valibot';

/**
 Permissive argument schema for tools whose cases exercise dispatch rather than validation.
 
 `object` ignores unknown keys where `strictObject` would reject them, so adding it to an
 existing case changes nothing about what that case asserts.
 */
const ANY_ARGUMENTS = v.object({},);

/**
 Envelope every derived `inputSchema` carries regardless of a tool's arguments.
 
 `$schema` names 2020-12 because that is the draft revision 2026-07-28 expects; the
 converter would otherwise default to draft-07.
 */
const DERIVED_ROOT = {
  type: 'object',
  required: [],
  $schema: 'https://json-schema.org/draft/2020-12/schema',
} as const;

/** Reusable test tool that echoes arguments back as text content. */
const echoTool: ToolEntry = {
  name: 'echo',
  description: 'Echoes arguments.',
  schema: v.object({ text: v.optional(v.string(),), },),
  handler: (args: Readonly<Record<string, unknown>>,) => ({
    content: [{ type: 'text', text: JSON.stringify(args,), },],
  }),
};

/** Identity every server in this file reports, so result metadata assertions stay short. */
const serverIdentity = { name: 'srv', version: '0.1.0', };

/**
 Builds a request carrying the protocol revision metadata every served request needs.
 
 @param id - Request id echoed back in the response.
 
 @param method - MCP method to invoke.
 
 @param params - Method params merged alongside the generated `_meta`.
 
 @param version - Revision to declare, defaulting to the one this server implements.
 
 @returns Inbound request ready for `handleMessage`.
 */
const modernRequest = ({
  id,
  method,
  params = {},
  version = PROTOCOL_VERSION,
}: {
  readonly id: JsonRpcId;
  readonly method: string;
  readonly params?: Readonly<Record<string, unknown>>;
  readonly version?: string;
},): JsonRpcInbound => ({
  jsonrpc: '2.0',
  id,
  method,
  params: {
    ...params,
    _meta: { [META_PROTOCOL_VERSION]: version, },
  },
});

//region defineTool: bundles name with tool entry options

await describe({
  name: '',
  children: [
    describe({
      name: defineTool.name,
      children: [
        it({
          name: 'returns a ToolEntry with the given name and options',
          fn: async () => {
            const entry = defineTool({
              name: 'greet',
              entry: {
                description: 'Greets by name.',
                schema: ANY_ARGUMENTS,
                handler: () => ({ content: [{ type: 'text', text: 'hello', },], }),
              },
            },);
            expect(entry.name,).toBe('greet',);
            expect(entry.description,).toBe('Greets by name.',);
          },
        },),
        it({
          name: 'carries the declared schema through unchanged',
          fn: async () => {
            const schema = v.strictObject({ name: v.string(), },);
            const entry = defineTool({
              name: 'test',
              entry: {
                description: 'Test tool.',
                schema,
                handler: () => ({ content: [{ type: 'text', text: 'ok', },], }),
              },
            },);
            // The entry keeps the valibot schema itself; conversion to the advertised JSON
            // Schema happens at registration, so one declaration drives both.
            expect(entry.schema,).toBe(schema,);
          },
        },),
      ],
    },),

    //endregion defineTool

    //region registerTools: normalizes entries and rejects colliding names

    describe({
      name: registerTools.name,
      children: [
        it({
          name: 'carries title, outputSchema, and annotations into the definition',
          fn: async () => {
            const registry = registerTools({
              tools: [{
                name: 'rich',
                title: 'Rich Tool',
                description: 'Declares every optional field.',
                outputSchema: { type: 'object', },
                annotations: { readOnlyHint: true, },
                schema: ANY_ARGUMENTS,
                handler: () => ({ content: [], }),
              },],
            },);
            expect(registry.get('rich',)?.definition,).toEqual({
              name: 'rich',
              description: 'Declares every optional field.',
              inputSchema: { ...DERIVED_ROOT, properties: {}, },
              title: 'Rich Tool',
              outputSchema: { type: 'object', },
              annotations: { readOnlyHint: true, },
            },);
          },
        },),
        it({
          name: 'omits optional definition fields that were not declared',
          fn: async () => {
            const registry = registerTools({ tools: [echoTool,], },);
            expect(registry.get('echo',)?.definition,).toEqual({
              name: 'echo',
              description: 'Echoes arguments.',
              inputSchema: {
                ...DERIVED_ROOT,
                properties: { text: { type: 'string', }, },
              },
            },);
          },
        },),
        it({
          name: 'throws at construction when a schema cannot be advertised',
          fn: async () => {
            // errorMode 'throw' exists so an unconvertible schema fails here rather than
            // being advertised in a degraded form that no longer matches what is enforced.
            expect(() => registerTools({
              tools: [{
                name: 'unconvertible',
                description: 'Declares a schema JSON Schema cannot express.',
                schema: v.custom(() => true,),
                handler: () => ({ content: [], }),
              },],
            },),).toThrow('unconvertible',);
          },
        },),
        it({
          name: 'throws at construction when a schema declares a non-object root',
          fn: async () => {
            // Revision 2026-07-28: "Tool arguments are always JSON objects, so
            // `type: \"object\"` is required at the root."
            expect(() => registerTools({
              tools: [{
                name: 'scalar',
                description: 'Declares a string where an object belongs.',
                schema: v.string(),
                handler: () => ({ content: [], }),
              },],
            },),).toThrow('object root',);
          },
        },),
        it({
          name: 'throws at construction when a union mixes object and scalar branches',
          fn: async () => {
            // Restoring an object root over these branches would advertise a schema
            // nothing can satisfy: an argument bag would have to be an object and also a
            // string. Refusing beats shipping a tool no client can call.
            expect(() => registerTools({
              tools: [{
                name: 'mixed',
                description: 'Declares a scalar union.',
                schema: v.union([v.string(), v.number(),],),
                handler: () => ({ content: [], }),
              },],
            },),).toThrow('not all objects',);
          },
        },),
        it({
          name: 'throws when two entries share a name',
          fn: async () => {
            expect(() => registerTools({
              tools: [
                echoTool,
                { ...echoTool, description: 'Shadowing duplicate.', },
              ],
            },),).toThrow('Duplicate tool names registered: echo',);
          },
        },),
      ],
    },),

    //endregion registerTools

    //region createMcpServer: builds immutable server and dispatches messages

    describe({
      name: createMcpServer.name,
      children: [
        //region server/discover: advertises revisions, capabilities, and cache policy

        describe({
          name: 'server/discover',
          children: [
            it({
              name: 'responds with supported revisions, capabilities, and cache hints',
              fn: async () => {
                const server = createMcpServer({
                  config: { name: 'test-server', version: '1.0.0', },
                  tools: [],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 1, method: 'server/discover', },),
                ) as JsonRpcResponse;
                expect(response,).toEqual({
                  jsonrpc: '2.0',
                  id: 1,
                  result: {
                    resultType: RESULT_TYPE_COMPLETE,
                    supportedVersions: SUPPORTED_PROTOCOL_VERSIONS,
                    capabilities: { tools: {}, },
                    ttlMs: 0,
                    cacheScope: 'private',
                    _meta: {
                      [META_SERVER_INFO]: { name: 'test-server', version: '1.0.0', },
                    },
                  },
                },);
              },
            },),
            it({
              name: 'includes instructions and title when configured',
              fn: async () => {
                const server = createMcpServer({
                  config: {
                    ...serverIdentity,
                    title: 'Test Server',
                    instructions: 'Call echo first.',
                  },
                  tools: [],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 2, method: 'server/discover', },),
                ) as JsonRpcResponse;
                const result = response.result as {
                  instructions?: string;
                  _meta: Record<string, unknown>;
                };
                expect(result.instructions,).toBe('Call echo first.',);
                expect(result._meta[META_SERVER_INFO],).toEqual({
                  name: 'srv',
                  version: '0.1.0',
                  title: 'Test Server',
                },);
              },
            },),
            it({
              name: 'honors a configured cache hint and capability set',
              fn: async () => {
                const server = createMcpServer({
                  config: {
                    ...serverIdentity,
                    capabilities: { tools: { listChanged: true, }, },
                    discoverCache: { ttlMs: 3_600_000, cacheScope: 'public', },
                  },
                  tools: [],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 3, method: 'server/discover', },),
                ) as JsonRpcResponse;
                const result = response.result as {
                  ttlMs: number;
                  cacheScope: string;
                  capabilities: unknown;
                };
                expect(result.ttlMs,).toBe(3_600_000,);
                expect(result.cacheScope,).toBe('public',);
                expect(result.capabilities,).toEqual({ tools: { listChanged: true, }, },);
              },
            },),
            it({
              name: 'omits instructions when not configured',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 4, method: 'server/discover', },),
                ) as JsonRpcResponse;
                expect(
                  Object.hasOwn(response.result as object, 'instructions',),
                ).toBe(false,);
              },
            },),
          ],
        },),

        //endregion server/discover

        //region protocol version validation: every request declares its revision

        describe({
          name: 'protocol version validation',
          children: [
            it({
              name: 'rejects a request whose params carry no _meta',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const request: JsonRpcInbound = { jsonrpc: '2.0', id: 5,
                  method: 'tools/list', };
                const response = await server.handleMessage(request,) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
                expect(response.error.message,).toContain(META_PROTOCOL_VERSION,);
                expect(response.error.data,).toEqual({
                  supported: SUPPORTED_PROTOCOL_VERSIONS,
                },);
              },
            },),
            it({
              name: 'rejects a request whose _meta omits the revision key',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const request: JsonRpcInbound = {
                  jsonrpc: '2.0',
                  id: 6,
                  method: 'tools/list',
                  params: { _meta: { other: 'value', }, },
                };
                const response = await server.handleMessage(request,) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
              },
            },),
            it({
              name: 'rejects a request whose declared revision is not implemented',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const response = await server.handleMessage(
                  modernRequest({ id: 7, method: 'tools/list', version: '2025-06-18', },),
                ) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_UNSUPPORTED_PROTOCOL_VERSION,);
                expect(response.error.message,).toBe('Unsupported protocol version',);
                expect(response.error.data,).toEqual({
                  supported: SUPPORTED_PROTOCOL_VERSIONS,
                  requested: '2025-06-18',
                },);
              },
            },),
            it({
              name: 'rejects a request whose revision is not a string',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const request: JsonRpcInbound = {
                  jsonrpc: '2.0',
                  id: 8,
                  method: 'tools/list',
                  params: { _meta: { [META_PROTOCOL_VERSION]: 20_260_728, }, },
                };
                const response = await server.handleMessage(request,) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
              },
            },),
            it({
              name: 'rejects a request whose _meta is an array rather than an object',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const request: JsonRpcInbound = {
                  jsonrpc: '2.0',
                  id: 9,
                  method: 'tools/list',
                  params: { _meta: [PROTOCOL_VERSION,], },
                };
                const response = await server.handleMessage(request,) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
              },
            },),
          ],
        },),

        //endregion protocol version validation

        //region initialize: removed in this revision, answered with a diagnostic

        describe({
          name: 'initialize',
          children: [
            it({
              name: 'reports the handshake removed and names supported revisions',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const request: JsonRpcInbound = {
                  jsonrpc: '2.0',
                  id: 10,
                  method: 'initialize',
                  params: { protocolVersion: '2025-11-25', },
                };
                const response = await server.handleMessage(request,) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_METHOD_NOT_FOUND,);
                expect(response.error.message,).toContain(PROTOCOL_VERSION,);
                expect(response.error.message,).toContain('server/discover',);
                expect(response.error.data,).toEqual({
                  supported: SUPPORTED_PROTOCOL_VERSIONS,
                },);
              },
            },),
            it({
              name: 'answers without requiring the revision metadata it predates',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const request: JsonRpcInbound = { jsonrpc: '2.0', id: 'string-id',
                  method: 'initialize', };
                const response = await server.handleMessage(request,) as JsonRpcErrorResponse;
                expect(response.id,).toBe('string-id',);
                expect(response.error.code,).toBe(JSON_RPC_METHOD_NOT_FOUND,);
              },
            },),
          ],
        },),

        //endregion initialize

        //region tools/list: returns registered tool definitions

        describe({
          name: 'tools/list',
          children: [
            it({
              name: 'returns an empty listing with result envelope and cache hints',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const response = await server.handleMessage(
                  modernRequest({ id: 11, method: 'tools/list', },),
                ) as JsonRpcResponse;
                expect(response.result,).toEqual({
                  resultType: RESULT_TYPE_COMPLETE,
                  tools: [],
                  ttlMs: 0,
                  cacheScope: 'private',
                  _meta: { [META_SERVER_INFO]: serverIdentity, },
                },);
              },
            },),
            it({
              name: 'returns all registered tools with definitions',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [echoTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 12, method: 'tools/list', },),
                ) as JsonRpcResponse;
                expect((response.result as { tools: unknown; }).tools,).toEqual([
                  {
                    name: 'echo',
                    description: 'Echoes arguments.',
                    inputSchema: {
                      ...DERIVED_ROOT,
                      properties: { text: { type: 'string', }, },
                    },
                  },
                ],);
              },
            },),
            it({
              name: 'derives an empty object schema for a tool taking no arguments',
              fn: async () => {
                const tool: ToolEntry = {
                  name: 'no-schema',
                  description: 'No explicit schema.',
                  schema: ANY_ARGUMENTS,
                  handler: () => ({ content: [{ type: 'text', text: 'ok', },], }),
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [tool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 13, method: 'tools/list', },),
                ) as JsonRpcResponse;
                const { tools, } = response.result as {
                  tools: readonly { inputSchema: unknown; }[];
                };
                expect(tools[0]?.inputSchema,).toEqual({ ...DERIVED_ROOT, properties: {}, },);
              },
            },),
            it({
              name: 'lists multiple tools in registration order',
              fn: async () => {
                const toolA: ToolEntry = {
                  name: 'alpha',
                  description: 'First tool.',
                  schema: ANY_ARGUMENTS,
                  handler: () => ({ content: [{ type: 'text', text: 'a', },], }),
                };
                const toolB: ToolEntry = {
                  name: 'beta',
                  description: 'Second tool.',
                  schema: ANY_ARGUMENTS,
                  handler: () => ({ content: [{ type: 'text', text: 'b', },], }),
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [toolA, toolB,],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 14, method: 'tools/list', },),
                ) as JsonRpcResponse;
                const names =
                  (response.result as { tools: readonly { name: string; }[]; })
                    .tools
                    .map(
                    tool => tool.name,
                    );
                expect(names,).toEqual(['alpha', 'beta',],);
              },
            },),
            it({
              name: 'honors a configured tools cache hint',
              fn: async () => {
                const server = createMcpServer({
                  config: {
                    ...serverIdentity,
                    toolsCache: { ttlMs: 300_000, cacheScope: 'public', },
                  },
                  tools: [],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 15, method: 'tools/list', },),
                ) as JsonRpcResponse;
                const result = response.result as { ttlMs: number; cacheScope: string; };
                expect(result.ttlMs,).toBe(300_000,);
                expect(result.cacheScope,).toBe('public',);
              },
            },),
          ],
        },),

        //endregion tools/list

        //region tools/call: dispatches to registered tool handlers

        describe({
          name: 'tools/call',
          children: [
            it({
              name: 'calls the correct tool handler and stamps the result envelope',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [echoTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 16,
                    method: 'tools/call',
                    params: { name: 'echo', arguments: { text: 'hello', }, },
                  },),
                ) as JsonRpcResponse;
                expect(response.result,).toEqual({
                  resultType: RESULT_TYPE_COMPLETE,
                  content: [{ type: 'text', text: '{"text":"hello"}', },],
                  _meta: { [META_SERVER_INFO]: serverIdentity, },
                },);
              },
            },),
            it({
              name: 'carries structuredContent through to the client',
              fn: async () => {
                const structuredTool: ToolEntry = {
                  name: 'structured',
                  description: 'Returns structured output.',
                  outputSchema: { type: 'object', },
                  schema: ANY_ARGUMENTS,
                  handler: () => ({
                    content: [{ type: 'text', text: '{"exitCode":0}', },],
                    structuredContent: { exitCode: 0, },
                  }),
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [structuredTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 17,
                    method: 'tools/call',
                    params: { name: 'structured', },
                  },),
                ) as JsonRpcResponse;
                expect(
                  (response.result as { structuredContent: unknown; }).structuredContent,
                ).toEqual({ exitCode: 0, },);
              },
            },),
            it({
              name: 'passes every spec content block type through untouched',
              fn: async () => {
                /** One item per ContentBlock variant the 2026-07-28 schema defines. */
                const blocks: readonly ToolContent[] = [
                  { type: 'text', text: 'plain', },
                  { type: 'image', data: 'aW1n', mimeType: 'image/png', },
                  { type: 'audio', data: 'c25k', mimeType: 'audio/mpeg', },
                  {
                    type: 'resource_link',
                    uri: 'file:///var/log/build.log',
                    name: 'build log',
                    mimeType: 'text/plain',
                  },
                  {
                    type: 'resource',
                    resource: { uri: 'file:///etc/hosts', text: '127.0.0.1 localhost', },
                  },
                ];
                const everyBlockTool: ToolEntry = {
                  name: 'every-block',
                  description: 'Returns one of each content block.',
                  schema: ANY_ARGUMENTS,
                  handler: () => ({ content: blocks, }),
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [everyBlockTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 29,
                    method: 'tools/call',
                    params: { name: 'every-block', },
                  },),
                ) as JsonRpcResponse;
                expect((response.result as { content: unknown; }).content,).toEqual(blocks,);
              },
            },),
            it({
              name: 'carries content annotations through to the client',
              fn: async () => {
                const annotatedTool: ToolEntry = {
                  name: 'annotated',
                  description: 'Returns annotated content.',
                  schema: ANY_ARGUMENTS,
                  handler: () => ({
                    content: [{
                      type: 'text',
                      text: 'for the user',
                      annotations: { audience: ['user',], priority: 1, },
                    },],
                  }),
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [annotatedTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 30,
                    method: 'tools/call',
                    params: { name: 'annotated', },
                  },),
                ) as JsonRpcResponse;
                const { content, } = response.result as {
                  content: readonly { annotations: unknown; }[];
                };
                expect(content[0]?.annotations,).toEqual({
                  audience: ['user',],
                  priority: 1,
                },);
              },
            },),
            it({
              name: 'returns error for unknown tool name',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [echoTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 18,
                    method: 'tools/call',
                    params: { name: 'nonexistent', },
                  },),
                ) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
                expect(response.error.message,).toContain('nonexistent',);
              },
            },),
            it({
              name: 'returns error when tool name is missing',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [echoTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 19, method: 'tools/call', },),
                ) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
              },
            },),
            it({
              name: 'returns error when tool name is not a string',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [echoTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({ id: 20, method: 'tools/call', params: { name: 42, }, },),
                ) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
              },
            },),
            it({
              name: 'defaults arguments to empty object when not provided',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [echoTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 21,
                    method: 'tools/call',
                    params: { name: 'echo', },
                  },),
                ) as JsonRpcResponse;
                expect((response.result as { content: unknown; }).content,).toEqual([
                  { type: 'text', text: '{}', },
                ],);
              },
            },),
            it({
              name: 'rejects null arguments rather than treating them as empty',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [echoTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 22,
                    method: 'tools/call',
                    params: { name: 'echo', arguments: null, },
                  },),
                ) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
                expect(response.error.message,).toContain('must be a JSON object',);
              },
            },),
            it({
              name: 'rejects array arguments rather than treating them as empty',
              fn: async () => {
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [echoTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 23,
                    method: 'tools/call',
                    params: { name: 'echo', arguments: [1, 2,], },
                  },),
                ) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
              },
            },),
            it({
              name: 'refuses a call omitting a required argument before reaching the handler',
              fn: async () => {
                /** Records whether the handler ran, proving the refusal happened earlier. */
                const dispatched: string[] = [];
                const strictTool: ToolEntry = {
                  name: 'strict',
                  description: 'Needs a name.',
                  schema: v.strictObject({ name: v.string(), },),
                  handler: () => {
                    dispatched.push('ran',);
                    return { content: [], };
                  },
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [strictTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 24,
                    method: 'tools/call',
                    params: { name: 'strict', arguments: {}, },
                  },),
                ) as JsonRpcErrorResponse;

                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
                // Naming the argument is the point: a bare "invalid arguments" leaves the
                // caller guessing which of several it got wrong.
                expect(response.error.message.includes('name',),).toBe(true,);
                expect(dispatched,).toEqual([],);
              },
            },),
            it({
              name: 'refuses an argument the declared schema does not admit',
              fn: async () => {
                const strictTool: ToolEntry = {
                  name: 'strict',
                  description: 'Admits nothing.',
                  schema: v.strictObject({},),
                  handler: () => ({ content: [], }),
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [strictTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 25,
                    method: 'tools/call',
                    params: { name: 'strict', arguments: { typo: 1, }, },
                  },),
                ) as JsonRpcErrorResponse;

                // A misspelled argument is louder as a refusal than as a silent omission,
                // which would run the tool as though the caller had asked for the default.
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
                expect(response.error.message.includes('typo',),).toBe(true,);
              },
            },),
            it({
              name: 'hands the handler the arguments as sent, not a schema-stripped copy',
              fn: async () => {
                /** Argument bag the handler actually received. */
                const seen: Record<string, unknown>[] = [];
                const looseTool: ToolEntry = {
                  name: 'loose',
                  description: 'Ignores extras.',
                  schema: v.object({ text: v.optional(v.string(),), },),
                  handler: (args: Readonly<Record<string, unknown>>,) => {
                    seen.push({ ...args, },);
                    return { content: [], };
                  },
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [looseTool,],
                },);
                await server.handleMessage(
                  modernRequest({
                    id: 26,
                    method: 'tools/call',
                    params: { name: 'loose', arguments: { text: 'hi', extra: 7, }, },
                  },),
                );

                // Validation is a gate, not a transform: valibot's object schemas drop keys
                // they do not declare, so passing its output would silently change what every
                // handler receives.
                expect(seen,).toEqual([{ text: 'hi', extra: 7, },],);
              },
            },),
            it({
              name: 'refuses an argument named after an inherited property',
              fn: async () => {
                /** Argument bags the handler received, if any. */
                const seen: Record<string, unknown>[] = [];
                const strictTool: ToolEntry = {
                  name: 'strict',
                  description: 'Declares only name.',
                  schema: strictArguments({ name: v.string(), },),
                  handler: (args: Readonly<Record<string, unknown>>,) => {
                    seen.push({ ...args, },);
                    return { content: [], };
                  },
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [strictTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 27,
                    method: 'tools/call',
                    // Parsed from text so `__proto__` lands as an own property, which an
                    // object literal would not produce.
                    params: JSON.parse(
                      '{"name":"strict","arguments":{"name":"vm1","__proto__":{"polluted":true},"constructor":1}}',
                    ) as Record<string, unknown>,
                  },),
                ) as JsonRpcErrorResponse;

                // Valibot decides declaredness with `key in entries`, so a plain entries
                // object would treat every Object.prototype name as declared and let these
                // through, while the advertised schema says additionalProperties false.
                expect(response.error.code,).toBe(JSON_RPC_INVALID_PARAMS,);
                expect(seen,).toEqual([],);
              },
            },),
            it({
              name: 'reports a thrown handler failure as an isError result, not a protocol error',
              fn: async () => {
                const failingTool: ToolEntry = {
                  name: 'fail',
                  description: 'Always fails.',
                  schema: ANY_ARGUMENTS,
                  handler: () => {
                    throw new Error('deliberate failure',);
                  },
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [failingTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 24,
                    method: 'tools/call',
                    params: { name: 'fail', },
                  },),
                ) as JsonRpcResponse;
                expect(response.result,).toEqual({
                  resultType: RESULT_TYPE_COMPLETE,
                  content: [
                    { type: 'text', text: 'Tool execution failed: deliberate failure', },
                  ],
                  isError: true,
                  _meta: { [META_SERVER_INFO]: serverIdentity, },
                },);
              },
            },),
            it({
              name: 'handles non-Error thrown values',
              fn: async () => {
                const throwStringTool: ToolEntry = {
                  name: 'throw-string',
                  description: 'Throws a string.',
                  schema: ANY_ARGUMENTS,
                  handler: () => {
                    // oxlint-disable-next-line eslint/no-throw-literal, typescript/only-throw-error -- testing non-Error throw
                    throw 'string-error';
                  },
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [throwStringTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 25,
                    method: 'tools/call',
                    params: { name: 'throw-string', },
                  },),
                ) as JsonRpcResponse;
                const result = response.result as {
                  content: readonly { text: string; }[];
                  isError: boolean;
                };
                expect(result.isError,).toBe(true,);
                expect(result.content[0]?.text,).toContain('string-error',);
              },
            },),
            it({
              name: 'preserves an isError result the handler produced itself',
              fn: async () => {
                const reportingTool: ToolEntry = {
                  name: 'reports',
                  description: 'Reports its own failure.',
                  schema: ANY_ARGUMENTS,
                  handler: () => ({
                    content: [{ type: 'text', text: 'exit code 1', },],
                    isError: true,
                  }),
                };
                const server = createMcpServer({
                  config: serverIdentity,
                  tools: [reportingTool,],
                },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 26,
                    method: 'tools/call',
                    params: { name: 'reports', },
                  },),
                ) as JsonRpcResponse;
                expect((response.result as { isError: boolean; }).isError,).toBe(true,);
              },
            },),
          ],
        },),

        //endregion tools/call

        //region unknown method: returns method not found error

        describe({
          name: 'unknown method',
          children: [
            it({
              name: 'returns method not found error',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const response = await server.handleMessage(
                  modernRequest({ id: 27, method: 'unknown/method', },),
                ) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_METHOD_NOT_FOUND,);
                expect(response.error.message,).toContain('unknown/method',);
              },
            },),
            it({
              name: 'checks the declared revision before the method name',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const response = await server.handleMessage(
                  modernRequest({
                    id: 28,
                    method: 'unknown/method',
                    version: '1900-01-01',
                  },),
                ) as JsonRpcErrorResponse;
                expect(response.error.code,).toBe(JSON_RPC_UNSUPPORTED_PROTOCOL_VERSION,);
              },
            },),
          ],
        },),

        //endregion unknown method

        //region notifications: returns the no-response sentinel for notifications

        describe({
          name: 'notifications',
          children: [
            it({
              name: 'returns NO_RESPONSE for notifications/cancelled',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const notification: JsonRpcInbound = { jsonrpc: '2.0',
                  method: 'notifications/cancelled', };
                const result = await server.handleMessage(notification,);
                expect(result,).toBe(NO_RESPONSE,);
              },
            },),
            it({
              name: 'returns NO_RESPONSE for unexpected notification methods',
              fn: async () => {
                const server = createMcpServer({ config: serverIdentity, tools: [], },);
                const notification: JsonRpcInbound = { jsonrpc: '2.0',
                  method: 'notifications/unknown', };
                const result = await server.handleMessage(notification,);
                expect(result,).toBe(NO_RESPONSE,);
              },
            },),
          ],
        },),
        //endregion notifications
      ],
    },),
    //endregion createMcpServer
  ],
},);
