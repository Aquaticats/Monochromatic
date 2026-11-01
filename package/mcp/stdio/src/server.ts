// MCP server: immutable tool registry and JSON-RPC dispatch for spec revision 2026-07-28.

import {
  JSON_RPC_INVALID_PARAMS,
  JSON_RPC_METHOD_NOT_FOUND,
  type JsonRpcId,
  type JsonRpcInbound,
  type JsonRpcOutbound,
  type JsonRpcRequest,
} from './json-rpc.ts';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { DEFAULT_CACHE_HINT, } from './protocol.ts';

import type { Implementation, } from './protocol-meta.ts';

import {
  buildDiscoverResult,
  buildListResourcesResult,
  buildListToolsResult,
  buildReadResourceResult,
} from './server-result.ts';

import {
  handleNotification,
  respondError,
  respondInitializeRemoved,
  respondMissingProtocolVersion,
  respondSuccess,
  respondUnsupportedProtocolVersion,
} from './server-response.ts';
import { handleToolCall, } from './server-tool-call.ts';
import { registerTools, } from './server-tool-registry.ts';
import type {
  DispatchResult,
  McpServerConfig,
  McpServerHandle,
  ResourceProvider,
  ToolEntry,
} from './server-types.ts';
import { requireProtocolVersion, } from './server-request-version.ts';
import {
  MissingProtocolVersionError,
  UnsupportedProtocolVersionError,
} from './server-protocol-error.ts';

//region createMcpServer: builds an immutable server from config and tool entries

/**
 Creates an immutable MCP server that dispatches JSON-RPC messages.
 Tools are registered at creation time; no mutation after construction.
 
 @param config - Server identity and discovery payload.
 
 @param tools - Tool entries to register, typically created via {@link defineTool}.
 
 @returns Server handle with a `handleMessage` function for the transport layer.
 
 @example
 ```ts
 import { createMcpServer, defineTool, serve } from '\@monochromatic-dev/mcp-stdio';
 
 const server = createMcpServer({
   config: { name: 'demo', version: '0.1.0' },
   tools: [
     defineTool({
       name: 'greet',
       entry: {
         description: 'Greets by name.',
         schema: v.strictObject({ name: v.string() }),
         handler: async (args) => ({
           content: [{ type: 'text', text: `Hello, ${args.name}!` }],
         }),
       },
     }),
   ],
 });
 await serve({ server });
 ```
 */
export function createMcpServer(
  {
    config,
    tools,
    resources,
  }: {
    readonly config: McpServerConfig;
    readonly tools: readonly ToolEntry[];
    /**
     Optional dynamic resource surface answering `resources/list` and `resources/read`.
     */
    readonly resources?: ResourceProvider;
  },
): McpServerHandle {
  /**
   Immutable lookup of registered tools keyed by name; built once at construction so
   later dispatch is O(1) without exposing a mutation surface.
   */
  const toolMap = registerTools({ tools, },);

  /**
   Identity stamped into the `_meta` of every result this server sends.
   */
  const serverInfo: Implementation = {
    name: config.name,
    version: config.version,
    ...((config.title === undefined) ? {} : { title: config.title, }),
  };

  //region Request dispatch: routes JSON-RPC methods to handlers

  /**
   Routes a version-checked request to the matching method handler.
   
   @param request - Inbound request whose declared revision this server implements.
   
   @returns JSON-RPC success or error response.
   */
  function routeRequest(request: JsonRpcRequest,): Promise<JsonRpcOutbound> {
    /**
     Request `id` is echoed in the response; `method` selects the branch below.
     */
    const {
      id,
      method,
    } = request;

    if (method === 'server/discover') {
      return Promise.resolve(respondSuccess({
        id,
        result: buildDiscoverResult({
          serverInfo,
          capabilities: config.capabilities ?? {
            tools: {},
            ...(resources === undefined ? {} : { resources: {}, }),
          },
          cache: config.discoverCache ?? DEFAULT_CACHE_HINT,
          ...((config.instructions === undefined) ? {} : { instructions: config.instructions, }),
        },),
      },),);
    }
    if (method === 'tools/list') {
      return Promise.resolve(respondSuccess({
        id,
        result: buildListToolsResult({
          tools: [...toolMap.values(),].map(function getDefinition(registered,) {
            return registered.definition;
          },),
          serverInfo,
          cache: config.toolsCache ?? DEFAULT_CACHE_HINT,
        },),
      },),);
    }
    if (method === 'tools/call') {
      return handleToolCall({
        toolMap,
        request,
        serverInfo,
      },);
    }
    if (method === 'resources/list') {
      return respondResourcesList({ id, },);
    }
    if (method === 'resources/read') {
      return respondResourcesRead({
        id,
        ...(request.params === undefined ? {} : { params: request.params, }),
      },);
    }
    return Promise.resolve(
      respondError({
        id,
        code: JSON_RPC_METHOD_NOT_FOUND,
        message: `Method not found: ${method}`,
      },),
    );
  }

  //region Resource dispatch: answers the dynamic resource surface when configured

  /**
   Answers `resources/list` through the configured resource provider.
   
   @param id - Request `id` echoed in the response.
   
   @returns JSON-RPC success or method-not-found response.
   */
  async function respondResourcesList(
    { id, }: {
      /**
       Request id echoed in the response.
       */
      readonly id: JsonRpcId;
    },
  ): Promise<JsonRpcOutbound> {
    if (resources === undefined)
      return respondError({
        id,
        code: JSON_RPC_METHOD_NOT_FOUND,
        message: 'Method not found: resources/list',
      },);

    return respondSuccess({
      id,
      result: buildListResourcesResult({
        resources: await resources
          .list(),
        serverInfo,
        cache: config.resourcesCache ?? DEFAULT_CACHE_HINT,
      },),
    },);
  }

  /**
   Answers `resources/read` through the configured resource provider.
   
   @param id - Request `id` echoed in the response.
   
   @param params - Raw request params whose `uri` selects the resource.
   
   @returns JSON-RPC success, method-not-found, or invalid-params response.
   */
  async function respondResourcesRead(
    {
      id,
      params,
    }: {
      /**
       Request id echoed in the response.
       */
      readonly id: JsonRpcId;
      /**
       Raw request params whose `uri` selects the resource.
       */
      readonly params?: Readonly<Record<string, unknown>>;
    },
  ): Promise<JsonRpcOutbound> {
    if (resources === undefined)
      return respondError({
        id,
        code: JSON_RPC_METHOD_NOT_FOUND,
        message: 'Method not found: resources/read',
      },);

    /**
     Resource URI taken from raw params, validated before it reaches the provider.
     */
    const uri = (params === undefined) ? undefined : params.uri;
    if ((typeof uri) !== 'string')
      return respondError({
        id,
        code: JSON_RPC_INVALID_PARAMS,
        message: 'resources/read requires a string "uri" parameter',
      },);

    // Deliberate catch-and-return: a provider that cannot serve a URI reports a request
    // error to the client rather than crashing the server process.
    try {
      /**
       Contents served by the provider for this URI.
       */
      const contents = await resources
        .read(uri,);
      return respondSuccess({
        id,
        result: buildReadResourceResult({
          contents: [contents,],
          serverInfo,
          cache: config.resourcesCache ?? DEFAULT_CACHE_HINT,
        },),
      },);
    }
    catch (error: unknown) {
      console.error(
        `[mcp-stdio] resource read failed for "${uri}":`,
        error,
      );
      return respondError({
        id,
        code: JSON_RPC_INVALID_PARAMS,
        message: `Resource not readable: ${uri}: ${caughtValueText(error,)}`,
      },);
    }
  }

  //endregion

  /**
   Validates the request's declared protocol revision, then routes it.
   `initialize` short-circuits ahead of validation: a handshake-era client never sends
   the `_meta` this revision requires, and its error message is its only diagnostic.
   
   @param request - Inbound request with an `id` that must be echoed in the response.
   
   @returns JSON-RPC success or error response.
   */
  function handleRequest(request: JsonRpcRequest,): Promise<JsonRpcOutbound> {
    if (request.method === 'initialize')
      return Promise.resolve(respondInitializeRemoved({ id: request.id, },),);

    // Deliberate catch-and-return: version validation reports refusal to the client as a
    // JSON-RPC error response rather than crashing the server process.
    try {
      requireProtocolVersion({ request, },);
    }
    catch (error: unknown) {
      if (error instanceof UnsupportedProtocolVersionError) {
        console.error(`[mcp-stdio] refused request: ${error.message}`,);
        return Promise.resolve(
          respondUnsupportedProtocolVersion({
            id: request.id,
            requested: error.requested,
            supported: error.supported,
          },),
        );
      }
      if (error instanceof MissingProtocolVersionError) {
        console.error(`[mcp-stdio] refused request: ${error.message}`,);
        return Promise.resolve(
          respondMissingProtocolVersion({
            id: request.id,
            message: error.message,
          },),
        );
      }
      throw error;
    }

    return routeRequest(request,);
  }

  //endregion

  //region Public handle: single dispatch function exposed to the transport

  /**
   Dispatches a parsed JSON-RPC message to the appropriate handler.
   Returns a response for requests, or delegates to {@link handleNotification} for notifications.
   
   @param message - Parsed inbound JSON-RPC request or notification.
   
   @returns JSON-RPC response for requests; the {@link NO_RESPONSE} sentinel for notifications.
   */
  function handleMessage(message: JsonRpcInbound,): Promise<DispatchResult> {
    if (!('id' in message)) {
      return Promise.resolve(handleNotification(message,),);
    }
    return handleRequest(message,);
  }

  return { handleMessage, };

  //endregion
}

//endregion
