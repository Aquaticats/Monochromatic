/**
 Request builder for the MCP test suites, which live in this package and in the
 packages that serve over it.

 It sits beside the product modules and is exported from the package entry, so a
 suite in another package reaches it through the sanctioned `/ts` source channel
 and this package's own suites reach it through the shipped entry.

 @module
 */

import type {
  JsonRpcId,
  JsonRpcInbound,
} from './json-rpc.ts';
import { PROTOCOL_VERSION, } from './protocol.ts';
import { META_PROTOCOL_VERSION, } from './protocol-meta.ts';

/**
 Builds a request carrying the protocol revision metadata every served request
 needs, so no suite has to spell the envelope out again.

 @param id - Request id echoed back in the response.

 @param method - MCP method to invoke.

 @param params - Method params merged alongside the generated `_meta`.

 @param version - Revision to declare, defaulting to the one this package implements.

 @returns Inbound request ready for `handleMessage`.

 @example
 ```ts
 protocolVersionRequest({ id: 1, method: 'tools/list' });
 ```

 @internal
 */
export function protocolVersionRequest(
  {
    id,
    method,
    params = {},
    version = PROTOCOL_VERSION,
  }: {
    readonly id: JsonRpcId;
    readonly method: string;
    readonly params?: Readonly<Record<string, unknown>>;
    readonly version?: string;
  },
): JsonRpcInbound {
  return {
    jsonrpc: '2.0',
    id,
    method,
    params: {
      ...params,
      _meta: { [META_PROTOCOL_VERSION]: version, },
    },
  };
}
