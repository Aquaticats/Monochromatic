// Dispatches tools/call requests to registered tool handlers.

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  JSON_RPC_INVALID_PARAMS,
  type JsonRpcOutbound,
  type JsonRpcRequest,
} from './json-rpc.ts';

import type { ToolCallResult, } from './protocol-tool.ts';

import type { Implementation, } from './protocol-meta.ts';

import { buildToolCallResult, } from './server-result.ts';
import {
  respondError,
  respondSuccess,
} from './server-response.ts';
import type { RegisteredTool, } from './server-types.ts';
import {
  TOOL_ARGUMENTS_VALID,
  validateToolArguments,
} from './tool-schema.ts';

/**
 Dispatches a `tools/call` request to the registered handler.
 Validates tool name and arguments from untrusted client input before dispatch.
 
 @param toolMap - Immutable map of registered tools keyed by name.
 
 @param request - Request containing tool `name` and `arguments` in `params`.
 
 @param serverInfo - Identity stamped into result metadata.
 
 @returns Tool result wrapped in a JSON-RPC response via {@link respondSuccess}, or an error via {@link respondError} if the tool is unknown.
 
 @example
 ```ts
 const response = await handleToolCall({
   toolMap,
   request: { jsonrpc: '2.0', id: 1, method: 'tools/call', params: { name: 'get_diagnostics' } },
   serverInfo: { name: 'demo', version: '0.1.0' },
 });
 ```
 */
export async function handleToolCall(
  {
    toolMap,
    request,
    serverInfo,
  }: {
    readonly toolMap: ReadonlyMap<string, RegisteredTool>;
    readonly request: JsonRpcRequest;
    readonly serverInfo: Implementation;
  },
): Promise<JsonRpcOutbound> {
  /**
   Echoed back in the response so the client can correlate the call with the result.
   */
  const {
    id,
    params,
  } = request;

  /**
   Tool name extracted from untrusted params; `undefined` triggers an invalid-params error below.
   */
  const toolName = ((typeof params?.name) === 'string') ? params.name : undefined;
  if (toolName === undefined) {
    return respondError({
      id,
      code: JSON_RPC_INVALID_PARAMS,
      message: 'Missing or non-string tool name in tools/call',
    },);
  }

  /**
   Raw `arguments` value from params, validated below before being handed to the tool.
   */
  const rawArgs = params?.arguments;
  // A null, array, or primitive `arguments` value is a malformed call, not an empty one:
  // silently substituting `{}` would run the tool with every argument missing and report
  // the resulting failure as if the caller had asked for it.
  if ((rawArgs !== undefined)
    && ((rawArgs === null)
      || ((typeof rawArgs) !== 'object')
      || Array.isArray(rawArgs,))) {
    return respondError({
      id,
      code: JSON_RPC_INVALID_PARAMS,
      message: `Arguments for tool "${toolName}" must be a JSON object when present`,
    },);
  }
  /**
   Validated argument bag; empty when the client sent no `arguments` at all.
   */
  const toolArgs: Record<string, unknown> = (rawArgs === undefined)
    ? {}
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowed from unknown to non-array, non-null object by the guard above
    : (rawArgs as Record<string, unknown>);

  /**
   Tool entry resolved from the registry; `undefined` means the client requested an unknown tool.
   */
  const registered = toolMap.get(toolName,);
  if (registered === undefined) {
    return respondError({
      id,
      code: JSON_RPC_INVALID_PARAMS,
      message: `Unknown tool: ${toolName}`,
    },);
  }

  /**
   Verdict from the same schema advertised for this tool.
   */
  const validation = validateToolArguments({
    schema: registered.schema,
    args: toolArgs,
  },);
  // Arguments contradicting the advertised schema are a caller error, so they answer with
  // invalid-params rather than reaching the handler and failing partway through its work.
  if (validation !== TOOL_ARGUMENTS_VALID) {
    return respondError({
      id,
      code: JSON_RPC_INVALID_PARAMS,
      message: `Invalid arguments for tool "${toolName}": ${validation}`,
    },);
  }

  // Deliberate catch-and-return: a failure originating inside a tool belongs in the result
  // with `isError` set, not in a JSON-RPC error response. A protocol error hides the failure
  // from the model, which then cannot see what went wrong and correct its next call.
  try {
    /**
     Tool handler output, wrapped below into a JSON-RPC success response.
     */
    const result: ToolCallResult = await registered.handler(toolArgs,);
    return respondSuccess({
      id,
      result: buildToolCallResult({
        result,
        serverInfo,
      },),
    },);
  }
  catch (error: unknown) {
    /**
     Human-readable error text; falls back to `String(error)` when the thrown value is not an `Error`.
     */
    const message = caughtValueText(error,);
    console.error(
      `[mcp-stdio] tool "${toolName}" threw:`,
      error,
    );
    return respondSuccess({
      id,
      result: buildToolCallResult({
        result: {
          content: [{
            type: 'text',
            text: `Tool execution failed: ${message}`,
          },],
          isError: true,
        },
        serverInfo,
      },),
    },);
  }
}
