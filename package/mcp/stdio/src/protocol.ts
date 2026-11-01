// MCP protocol constants and result types for spec revision 2026-07-28.
// Justification for >100 lines: pure constant and type definitions with required TSDoc on each;
// further splitting would fragment a single cohesive set of protocol declarations.

import type { ResultMeta, } from './protocol-meta.ts';

import type {
  ResourceContents,
  ToolDefinition,
} from './protocol-tool.ts';

//region Protocol revision: every version this server accepts on the wire

/**
 MCP protocol revision this package implements.
 Revision 2026-07-28 replaced the `initialize` handshake with per-request version
 metadata plus a mandatory `server/discover` RPC.
 */
export const PROTOCOL_VERSION = '2026-07-28' as const;

/**
 Every protocol revision accepted on an inbound request, advertised through
 `server/discover` and echoed in the `data.supported` list of an unsupported-version error.
 
 Single-entry by design: handshake-era revisions (2025-11-25 and earlier) negotiate
 through `initialize`, which this package no longer serves.
 */
export const SUPPORTED_PROTOCOL_VERSIONS: readonly string[] = [PROTOCOL_VERSION,];

/**
 Reports whether an inbound request may be served under a claimed protocol revision.
 
 @param version - Revision string taken from request metadata.
 
 @returns `true` when the revision appears in {@link SUPPORTED_PROTOCOL_VERSIONS}.
 
 @example
 ```ts
 isSupportedProtocolVersion({ version: '2026-07-28' });
 // true
 ```
 */
export function isSupportedProtocolVersion(
  { version, }: { readonly version: string; },
): boolean {
  return SUPPORTED_PROTOCOL_VERSIONS.some(function matchesVersion(supported,) {
    return supported === version;
  },);
}

//endregion

//region Result envelope: fields every 2026-07-28 result carries

/**
 Discriminator every result carries so clients know how to parse it.
 `complete` marks a finished result; `input_required` marks one that asks the client
 for more input before retrying, which this package never emits.
 */
export type ResultType = 'complete' | 'input_required';

/**
 Value of {@link ResultType} for results whose content is final.
 Revision 2026-07-28 makes this field mandatory on every result a server sends.
 */
export const RESULT_TYPE_COMPLETE: ResultType = 'complete';

/**
 Fields shared by every result this server sends.
 
 @example
 ```ts
 const result: McpResult = { resultType: 'complete' };
 ```
 */
export type McpResult = {
  readonly resultType: ResultType;
  readonly _meta?: ResultMeta;
};

//endregion

//region Cache hints: freshness metadata required on discovery and listing results

/**
 Reuse boundary for a cached result.
 `public` permits any intermediary to serve it across authorization contexts;
 `private` confines reuse to the context that fetched it.
 */
export type CacheScope = 'public' | 'private';

/**
 Freshness hint carried by `server/discover` and `tools/list` results.
 `ttlMs` of zero marks a result immediately stale, so a client refetches whenever it needs one.
 
 @example
 ```ts
 const hint: CacheHint = { ttlMs: 3_600_000, cacheScope: 'public' };
 ```
 */
export type CacheHint = {
  readonly ttlMs: number;
  readonly cacheScope: CacheScope;
};

/**
 Cache hint applied when a server declares none: stale on arrival, never shared across contexts.
 Conservative because a stdio server's registry is cheap to refetch and may differ between processes.
 */
export const DEFAULT_CACHE_HINT: CacheHint = {
  ttlMs: 0,
  cacheScope: 'private',
};

//endregion

//region Capabilities: optional features this server declares through discovery

/**
 Tools capability sub-object declared in {@link ServerCapabilities}.
 `listChanged` signals that the server emits `notifications/tools/list_changed`
 when its tool set mutates; a stdio server with a fixed registry omits it.
 
 @example
 ```ts
 const tools: ToolsCapability = {};
 ```
 */
export type ToolsCapability = {
  readonly listChanged?: boolean;
};

/**
 Resources capability sub-object declared in {@link ServerCapabilities}.
 Present only when the server exposes a resource provider answering `resources/list`
 and `resources/read`.
 
 @example
 ```ts
 const resources: ResourcesCapability = {};
 ```
 */
export type ResourcesCapability = {
  readonly listChanged?: boolean;
};

/**
 Server capabilities reported by `server/discover`.
 Only `tools` is relevant for a stdio tool server.
 
 @example
 ```ts
 const capabilities: ServerCapabilities = { tools: {} };
 ```
 */
export type ServerCapabilities = {
  readonly tools?: ToolsCapability;
  readonly resources?: ResourcesCapability;
};

//endregion

//region Resource listing and reading: dynamic payloads hosts fetch on demand

/**
 Descriptor advertising one readable resource.
 
 @example
 ```ts
 const descriptor: ResourceDescriptor = {
   uri: 'search-fetch://response/abc',
   name: 'full response',
 };
 ```
 */
export type ResourceDescriptor = {
  readonly uri: string;
  readonly name: string;
  readonly title?: string;
  readonly description?: string;
  readonly mimeType?: string;
};

/**
 Payload returned from `resources/list`.
 `nextCursor` stays absent because a stdio server lists every resource in one page.
 
 @example
 ```ts
 const result: ListResourcesResult = {
   resultType: 'complete',
   resources: [],
   ttlMs: 0,
   cacheScope: 'private',
 };
 ```
 */
export type ListResourcesResult = McpResult & CacheHint & {
  readonly resources: readonly ResourceDescriptor[];
  readonly nextCursor?: string;
};

/**
 Payload returned from `resources/read`.
 
 @example
 ```ts
 const result: ReadResourceResult = {
   resultType: 'complete',
   contents: [{ uri: 'search-fetch://response/abc', text: 'full response text' }],
   ttlMs: 0,
   cacheScope: 'private',
 };
 ```
 */
export type ReadResourceResult = McpResult & CacheHint & {
  readonly contents: readonly ResourceContents[];
};

//endregion

//region Discovery and listing results: the two cacheable payloads this server returns

/**
 Payload returned from `server/discover`, the RPC that replaced the initialization handshake.
 
 @example
 ```ts
 const result: DiscoverResult = {
   resultType: 'complete',
   supportedVersions: ['2026-07-28'],
   capabilities: { tools: {} },
   ttlMs: 0,
   cacheScope: 'private',
 };
 ```
 */
export type DiscoverResult = McpResult & CacheHint & {
  readonly supportedVersions: readonly string[];
  readonly capabilities: ServerCapabilities;
  readonly instructions?: string;
};

/**
 Payload returned from `tools/list`.
 `nextCursor` stays absent because a stdio server registers every tool at construction
 and returns them in one page.
 
 @example
 ```ts
 const result: ListToolsResult = {
   resultType: 'complete',
   tools: [],
   ttlMs: 0,
   cacheScope: 'private',
 };
 ```
 */
export type ListToolsResult = McpResult & CacheHint & {
  readonly tools: readonly ToolDefinition[];
  readonly nextCursor?: string;
};

//endregion
