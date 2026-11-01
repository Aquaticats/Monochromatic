// JSON-RPC response builders, protocol-version error responses, and the notification handler.

import {
  JSON_RPC_INVALID_PARAMS,
  JSON_RPC_METHOD_NOT_FOUND,
  JSON_RPC_UNSUPPORTED_PROTOCOL_VERSION,
  type JsonRpcErrorResponse,
  type JsonRpcNotification,
  type JsonRpcRequest,
  type JsonRpcResponse,
} from './json-rpc.ts';

import { SUPPORTED_PROTOCOL_VERSIONS, } from './protocol.ts';

import {
  type DispatchResult,
  NO_RESPONSE,
} from './server-types.ts';

//region Generic builders: success and error envelopes

/**
 Constructs a JSON-RPC success response.
 
 @param id - Request id to echo back.
 
 @param result - Payload for `result` field, already carrying its protocol envelope.
 
 @returns Formatted JSON-RPC response.
 
 @example
 ```ts
 respondSuccess({ id: 1, result: { resultType: 'complete', tools: [] } });
 // { jsonrpc: '2.0', id: 1, result: { resultType: 'complete', tools: [] } }
 ```
 */
export function respondSuccess(
  {
    id,
    result,
  }: {
    readonly id: JsonRpcRequest['id'];
    readonly result: unknown;
  },
): JsonRpcResponse {
  return {
    jsonrpc: '2.0',
    id,
    result,
  };
}

/**
 Constructs a JSON-RPC error response.
 
 @param id - Request id to echo back.
 
 @param code - JSON-RPC or MCP error code.
 
 @param message - Human-readable error description.
 
 @param data - Structured detail the client can act on, omitted when absent.
 
 @returns Formatted JSON-RPC error response.
 
 @example
 ```ts
 respondError({ id: 1, code: -32601, message: 'Method not found' });
 // { jsonrpc: '2.0', id: 1, error: { code: -32601, message: 'Method not found' } }
 ```
 */
export function respondError(
  {
    id,
    code,
    message,
    data,
  }: {
    readonly id: JsonRpcRequest['id'];
    readonly code: number;
    readonly message: string;
    readonly data?: unknown;
  },
): JsonRpcErrorResponse {
  return {
    jsonrpc: '2.0',
    id,
    error: {
      code,
      message,
      ...((data === undefined) ? {} : { data, }),
    },
  };
}

//endregion

//region Protocol-version errors: the two ways request validation refuses to serve

/**
 Builds the `UnsupportedProtocolVersionError` response defined by MCP revision 2026-07-28.
 Its `data` names every revision on offer so the client can retry on one of them.
 
 @param id - Request id to echo back.
 
 @param requested - Revision the client declared and this server refused.
 
 @param supported - Revisions this server implements, for the client to retry on.
 
 @returns Error response with code -32022 and its mandated `data` payload.
 
 @example
 ```ts
 respondUnsupportedProtocolVersion({
   id: 1,
   requested: '2025-06-18',
   supported: ['2026-07-28'],
 });
 ```
 */
export function respondUnsupportedProtocolVersion(
  {
    id,
    requested,
    supported,
  }: {
    readonly id: JsonRpcRequest['id'];
    readonly requested: string;
    readonly supported: readonly string[];
  },
): JsonRpcErrorResponse {
  return respondError({
    id,
    code: JSON_RPC_UNSUPPORTED_PROTOCOL_VERSION,
    message: 'Unsupported protocol version',
    data: {
      supported,
      requested,
    },
  },);
}

/**
 Builds the response for a request whose `_meta` omitted the mandatory revision key.
 Invalid params rather than an unsupported version: nothing was requested to refuse.
 
 @param id - Request id to echo back.
 
 @param message - Description naming both missing key and available revisions.
 
 @returns Error response with code -32602 listing supported revisions.
 
 @example
 ```ts
 respondMissingProtocolVersion({ id: 1, message: 'Request is missing ...' });
 ```
 */
export function respondMissingProtocolVersion(
  {
    id,
    message,
  }: {
    readonly id: JsonRpcRequest['id'];
    readonly message: string;
  },
): JsonRpcErrorResponse {
  return respondError({
    id,
    code: JSON_RPC_INVALID_PARAMS,
    message,
    data: { supported: SUPPORTED_PROTOCOL_VERSIONS, },
  },);
}

/**
 Builds the response for the removed `initialize` handshake.
 A handshake-era client cannot fall forward, so this message is the only diagnostic it can
 show its user; the spec asks a modern-only server to name its revisions here.
 
 @param id - Request id to echo back.
 
 @returns Error response explaining that discovery replaced the handshake.
 
 @example
 ```ts
 respondInitializeRemoved({ id: 1 });
 ```
 */
export function respondInitializeRemoved(
  { id, }: { readonly id: JsonRpcRequest['id']; },
): JsonRpcErrorResponse {
  return respondError({
    id,
    code: JSON_RPC_METHOD_NOT_FOUND,
    message:
      `Method not found: initialize. This server implements MCP revision `
        + `${SUPPORTED_PROTOCOL_VERSIONS.join(', ',)}, which removed the initialize handshake: `
        + `call server/discover and declare the revision in each request's params._meta instead`,
    data: { supported: SUPPORTED_PROTOCOL_VERSIONS, },
  },);
}

//endregion

//region Notifications: consumed, never answered

/**
 Method name of the cancellation notification a client may send for an in-flight request.
 Handlers here run to completion, so the notification is accepted and dropped without a reply.
 
 The transport intercepts this method before dispatch so a cancellation takes effect while a
 tool is still running; reaching {@link handleNotification} means it arrived by some other
 path, such as a direct handle caller.
 */
export const CANCELLED_NOTIFICATION = 'notifications/cancelled';

/**
 Processes notifications. Logs unexpected notification methods for protocol debugging.
 
 @param notification - Inbound notification (consumed but not acted upon).
 
 @returns {@link NO_RESPONSE} sentinel since notifications produce no reply to send.
 
 @example
 ```ts
 handleNotification({ jsonrpc: '2.0', method: 'notifications/cancelled' });
 // NO_RESPONSE
 ```
 */
export function handleNotification(notification: JsonRpcNotification,): DispatchResult {
  if (notification.method !== CANCELLED_NOTIFICATION)
    console.error(`[mcp-stdio] unexpected notification method: ${notification.method}`,);
  return NO_RESPONSE;
}

//endregion
