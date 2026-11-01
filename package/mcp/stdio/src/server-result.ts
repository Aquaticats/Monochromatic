// Builders for the three result payloads this server returns, each stamped with the
// envelope fields MCP revision 2026-07-28 requires: `resultType`, server identity, cache hints.

import {
  type CacheHint,
  type DiscoverResult,
  type ListResourcesResult,
  type ListToolsResult,
  type McpResult,
  type ReadResourceResult,
  type ResourceDescriptor,
  RESULT_TYPE_COMPLETE,
  type ServerCapabilities,
  SUPPORTED_PROTOCOL_VERSIONS,
} from './protocol.ts';

import {
  type Implementation,
  META_SERVER_INFO,
  type ResultMeta,
} from './protocol-meta.ts';

import type {
  ResourceContents,
  ToolCallResult,
  ToolDefinition,
} from './protocol-tool.ts';

//region Server identity: stamped onto every result

/**
 Wraps server identity in the `_meta` shape that clients read it from.
 
 @param serverInfo - Identity of this server, taken from its configuration.
 
 @returns Metadata object carrying identity under its reserved key.
 
 @example
 ```ts
 serverInfoMeta({ serverInfo: { name: 'mvm', version: '0.1.0' } });
 // { 'io.modelcontextprotocol/serverInfo': { name: 'mvm', version: '0.1.0' } }
 ```
 */
export function serverInfoMeta(
  { serverInfo, }: { readonly serverInfo: Implementation; },
): ResultMeta {
  return { [META_SERVER_INFO]: serverInfo, };
}

//endregion

//region Discovery result: answers `server/discover`

/**
 Builds the payload for `server/discover`, the RPC every 2026-07-28 server must implement.
 
 @param serverInfo - Identity stamped into result metadata.
 
 @param capabilities - Optional features this server offers.
 
 @param cache - Freshness hint telling clients how long to reuse this payload.
 
 @param instructions - Natural-language guidance about this server for the model.
 
 @returns Discovery payload listing supported revisions and capabilities.
 
 @example
 ```ts
 buildDiscoverResult({
   serverInfo: { name: 'mvm', version: '0.1.0' },
   capabilities: { tools: {} },
   cache: { ttlMs: 0, cacheScope: 'private' },
 });
 ```
 */
export function buildDiscoverResult(
  {
    serverInfo,
    capabilities,
    cache,
    instructions,
  }: {
    readonly serverInfo: Implementation;
    readonly capabilities: ServerCapabilities;
    readonly cache: CacheHint;
    readonly instructions?: string;
  },
): DiscoverResult {
  return {
    resultType: RESULT_TYPE_COMPLETE,
    supportedVersions: SUPPORTED_PROTOCOL_VERSIONS,
    capabilities,
    ttlMs: cache.ttlMs,
    cacheScope: cache.cacheScope,
    ...((instructions === undefined) ? {} : { instructions, }),
    _meta: serverInfoMeta({ serverInfo, },),
  };
}

//endregion

//region Tool listing result: answers `tools/list`

/**
 Builds the payload for `tools/list`.
 Emits no `nextCursor`: a stdio server registers every tool at construction,
 so one page always holds the whole registry.
 
 @param tools - Wire-format definitions of every registered tool.
 
 @param serverInfo - Identity stamped into result metadata.
 
 @param cache - Freshness hint telling clients how long to reuse this listing.
 
 @returns Listing payload carrying every registered tool.
 
 @example
 ```ts
 buildListToolsResult({
   tools: [],
   serverInfo: { name: 'mvm', version: '0.1.0' },
   cache: { ttlMs: 0, cacheScope: 'private' },
 });
 ```
 */
export function buildListToolsResult(
  {
    tools,
    serverInfo,
    cache,
  }: {
    readonly tools: readonly ToolDefinition[];
    readonly serverInfo: Implementation;
    readonly cache: CacheHint;
  },
): ListToolsResult {
  return {
    resultType: RESULT_TYPE_COMPLETE,
    tools,
    ttlMs: cache.ttlMs,
    cacheScope: cache.cacheScope,
    _meta: serverInfoMeta({ serverInfo, },),
  };
}

//endregion

//region Resource listing result: answers `resources/list`

/**
 Builds the payload for `resources/list`.
 Emits no `nextCursor`: a stdio server lists every resource in one page.
 
 @param resources - Descriptors of every currently readable resource.
 
 @param serverInfo - Identity stamped into result metadata.
 
 @param cache - Freshness hint telling clients how long to reuse this listing.
 
 @returns Listing payload carrying every readable resource.
 
 @example
 ```ts
 buildListResourcesResult({
   resources: [],
   serverInfo: { name: 'search-fetch-mcp', version: '0.1.0' },
   cache: { ttlMs: 0, cacheScope: 'private' },
 });
 ```
 */
export function buildListResourcesResult(
  {
    resources,
    serverInfo,
    cache,
  }: {
    readonly resources: readonly ResourceDescriptor[];
    readonly serverInfo: Implementation;
    readonly cache: CacheHint;
  },
): ListResourcesResult {
  return {
    resultType: RESULT_TYPE_COMPLETE,
    resources,
    ttlMs: cache.ttlMs,
    cacheScope: cache.cacheScope,
    _meta: serverInfoMeta({ serverInfo, },),
  };
}

/**
 Builds the payload for `resources/read`.
 
 @param contents - Contents of the one resource that was read.
 
 @param serverInfo - Identity stamped into result metadata.
 
 @param cache - Freshness hint telling clients how long to reuse this payload.
 
 @returns Read payload carrying the resource contents.
 
 @example
 ```ts
 buildReadResourceResult({
   contents: [{ uri: 'search-fetch://response/abc', text: 'full response text' }],
   serverInfo: { name: 'search-fetch-mcp', version: '0.1.0' },
   cache: { ttlMs: 0, cacheScope: 'private' },
 });
 ```
 */
export function buildReadResourceResult(
  {
    contents,
    serverInfo,
    cache,
  }: {
    readonly contents: readonly ResourceContents[];
    readonly serverInfo: Implementation;
    readonly cache: CacheHint;
  },
): ReadResourceResult {
  return {
    resultType: RESULT_TYPE_COMPLETE,
    contents,
    ttlMs: cache.ttlMs,
    cacheScope: cache.cacheScope,
    _meta: serverInfoMeta({ serverInfo, },),
  };
}

//endregion

//region Tool call result: wraps whatever a handler returned

/**
 Stamps the protocol envelope onto the payload a tool handler produced.
 Handlers stay unaware of envelope fields, so a handler cannot omit or misstate them.
 
 @param result - Payload returned by a tool handler.
 
 @param serverInfo - Identity stamped into result metadata.
 
 @returns Handler payload carrying envelope fields.
 
 @example
 ```ts
 buildToolCallResult({
   result: { content: [{ type: 'text', text: 'ok' }] },
   serverInfo: { name: 'mvm', version: '0.1.0' },
 });
 ```
 */
export function buildToolCallResult(
  {
    result,
    serverInfo,
  }: {
    readonly result: ToolCallResult;
    readonly serverInfo: Implementation;
  },
): ToolCallResult & McpResult {
  return {
    resultType: RESULT_TYPE_COMPLETE,
    ...result,
    _meta: serverInfoMeta({ serverInfo, },),
  };
}

//endregion
