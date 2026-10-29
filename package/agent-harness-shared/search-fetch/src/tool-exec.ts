/**
 Shared tool execution for every host surface.

 Both adapters call these functions so search and fetch behavior stays one implementation:
 provider fallback, blocklist enforcement, ignored-key warnings, and output formatting.

 @module
 */

import * as v from 'valibot';
import type { ReadonlyDeep, } from 'type-fest';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type {
  LinkupConfig,
} from './config.ts';
import type {
  LinkupWebFetchInput,
  LinkupWebSearchInput,
} from './client.ts';
import {
  filterBlockedSearchResults,
  findBlockedUrlMatch,
} from './domain-policy.ts';
import { filterFetchResponseDataImages, } from './markdown-data-image-filter.ts';
import type {
  SearchFetchClient,
} from './search-fetch-client.ts';
import {
  FETCH_SUPPORTED_KEYS,
  LinkupWebFetchParametersSchema,
  LinkupWebSearchParametersSchema,
  SEARCH_SUPPORTED_KEYS,
  webFetchToolSpec,
  webSearchToolSpec,
  type LinkupWebFetchParams,
  type LinkupWebSearchParams,
} from './tool-spec.ts';
import {
  createLinkupToolOutput,
  type LinkupToolDetails,
  type SearchFetchToolResult,
  type TextContentItem,
} from './tool-output.ts';

//region Logger

/**
 Logger root for search-fetch shared modules.
 */
const searchFetchLogger = tagged({ tag: 'search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'tool-exec',
  l: searchFetchLogger,
},);

//endregion Logger

//region Types

/**
 Minimal client surface used by tool execution.
 */
export type SearchFetchToolClient = Pick<SearchFetchClient, 'search' | 'fetch'>;

/**
 Progress update pushed to hosts that render intermediate tool state.
 */
export type SearchFetchToolUpdate = {
  /**
   Model-visible progress text.
   */
  readonly content: TextContentItem[];
  /**
   Details published with the progress update.
   */
  readonly details: LinkupToolDetails;
};

/**
 Callback receiving progress updates from tool execution.
 */
export type SearchFetchToolUpdateCallback = (update: SearchFetchToolUpdate) => void;

/**
 Options shared by the search and fetch executors.
 */
type ExecuteToolOptions = {
  /**
   Provider-routing client used by the executor.
   */
  readonly client: SearchFetchToolClient;
  /**
   Loaded config carrying the blocklist.
   */
  readonly config: LinkupConfig;
  /**
   Raw runtime params as the host supplied them, including unsupported keys.
   */
  readonly rawParams: unknown;
  /**
   Cancellation signal forwarded to the client.
   */
  readonly signal?: AbortSignal;
  /**
   Progress callback for hosts that render it.
   */
  readonly onUpdate?: SearchFetchToolUpdateCallback;
};

/**
 Error thrown when a fetch URL matches the configured blocklist.
 */
export class BlockedUrlError extends Error {
  /**
   Build a blocklist failure naming the matching entry.

   @param entry - blocklist entry that matched the requested URL
   */
  constructor(entry: string,) {
    super(`Blocked by pi-search-fetch blocklist: ${entry}`,);
    this.name = 'BlockedUrlError';
  }
}

//endregion Types

//region Public API

/**
 Execute one web search with provider fallback and post-filter blocklist enforcement.

 @param options - client, config, raw params, and host callbacks

 @returns tool result with response content and structured details

 @throws ValiError when raw params fail the shared search schema

 @example
 ```ts
 await executeWebSearchTool({ client, config, rawParams: { query: 'docs' } });
 ```
 */
export async function executeWebSearchTool(options: ExecuteToolOptions,): Promise<SearchFetchToolResult> {
  /**
   Logger tagged for this tool execution.
   */
  const innerL = tagged({
    tag: webSearchToolSpec.name,
    l,
  },);
  /**
   Raw params as an object record, rejected loudly when the host sent another shape.
   */
  const rawParams = toRawParamRecord(options.rawParams,);
  /**
   Ignored compatibility keys supplied by the host.
   */
  const ignoredKeys = collectIgnoredKeys({
    input: rawParams,
    supportedKeys: SEARCH_SUPPORTED_KEYS,
  },);
  /**
   Validated search params with unsupported keys stripped.
   */
  const params = v.parse(
    LinkupWebSearchParametersSchema,
    rawParams,
  );
  /**
   Sanitized search input that cannot carry unsupported keys to the client.
   */
  const searchInput = supportedSearchInput(params,);

  options.onUpdate?.({
    content: [{
      type: 'text',
      text: `Searching web for: ${searchInput.query}`,
    },],
    details: {
      linkupResponse: undefined,
      rawLinkupResponse: undefined,
    },
  },);

  innerL.debug(`executing search for query: ${searchInput.query}`,);
  if (ignoredKeys.length > 0)
    innerL.warn(`ignoring search parameters: ${ignoredKeys.join(', ',)}`,);

  /**
   Provider-tagged upstream response.
   */
  const providerResponse = await options.client
    .search({
      input: searchInput,
      ...(options.signal === undefined ? {} : { signal: options.signal, }),
    },);
  /**
   Local policy-filtered response.
   */
  const filtered = filterBlockedSearchResults({
    response: providerResponse.response,
    blocklist: options.config
      .blocklist,
  },);

  return await createLinkupToolOutput({
    toolName: webSearchToolSpec.name,
    linkupResponse: filtered.linkupResponse,
    rawLinkupResponse: filtered.rawLinkupResponse,
    ignoredKeys,
    fixedBehavior: webSearchToolSpec.fixedBehavior,
    renderResultsArrayAsJsonl: webSearchToolSpec.renderResultsArrayAsJsonl,
    removedBlockedUrls: filtered.removedBlockedUrls,
    provider: providerResponse.provider,
    ...(providerResponse.fallbackChain === undefined ? {} : { fallbackChain: providerResponse.fallbackChain, }),
  },);
}

/**
 Execute one web fetch with gh routing, provider fallback, and preflight blocklist enforcement.

 @param options - client, config, raw params, and host callbacks

 @returns tool result with response content and structured details

 @throws BlockedUrlError when the requested URL matches the configured blocklist

 @throws ValiError when raw params fail the shared fetch schema

 @example
 ```ts
 await executeWebFetchTool({ client, config, rawParams: { url: 'https://example.com' } });
 ```
 */
export async function executeWebFetchTool(options: ExecuteToolOptions,): Promise<SearchFetchToolResult> {
  /**
   Logger tagged for this tool execution.
   */
  const innerL = tagged({
    tag: webFetchToolSpec.name,
    l,
  },);
  /**
   Raw params as an object record, rejected loudly when the host sent another shape.
   */
  const rawParams = toRawParamRecord(options.rawParams,);
  /**
   Ignored compatibility keys supplied by the host.
   */
  const ignoredKeys = collectIgnoredKeys({
    input: rawParams,
    supportedKeys: FETCH_SUPPORTED_KEYS,
  },);
  /**
   Validated fetch params with unsupported keys stripped.
   */
  const params = v.parse(
    LinkupWebFetchParametersSchema,
    rawParams,
  );
  /**
   Sanitized fetch input that cannot carry unsupported keys to the client.
   */
  const fetchInput = supportedFetchInput(params,);
  /**
   Matching blocklist entry for this fetch URL, when blocked.
   */
  const blockedEntry = findBlockedUrlMatch({
    url: fetchInput.url,
    blocklist: options.config
      .blocklist,
  },);

  if (blockedEntry.blocked)
    throw new BlockedUrlError(blockedEntry.entry,);

  options.onUpdate?.({
    content: [{
      type: 'text',
      text: `Fetching URL: ${fetchInput.url}`,
    },],
    details: {
      linkupResponse: undefined,
      rawLinkupResponse: undefined,
    },
  },);

  innerL.debug(`executing fetch for URL: ${fetchInput.url}`,);
  if (ignoredKeys.length > 0)
    innerL.warn(`ignoring fetch parameters: ${ignoredKeys.join(', ',)}`,);

  /**
   Provider-tagged upstream response.
   */
  const providerResponse = await options.client
    .fetch({
      input: fetchInput,
      ...(options.signal === undefined ? {} : { signal: options.signal, }),
    },);
  /**
   Model-visible response after removing base64-backed Markdown images.
   */
  const filteredResponse = filterFetchResponseDataImages(providerResponse.response,);
  if (filteredResponse.removedImageCount > 0)
    innerL.warn(`removed ${String(filteredResponse.removedImageCount,)} inline base64 Markdown image(s)`,);

  return await createLinkupToolOutput({
    toolName: webFetchToolSpec.name,
    linkupResponse: filteredResponse.linkupResponse,
    rawLinkupResponse: providerResponse.response,
    ignoredKeys,
    fixedBehavior: webFetchToolSpec.fixedBehavior,
    provider: providerResponse.provider,
    ...(providerResponse.fallbackChain === undefined ? {} : { fallbackChain: providerResponse.fallbackChain, }),
  },);
}

//endregion Public API

//region Input helpers

/**
 Coerce raw host params into an object record so ignored-key detection sees what arrived.

 @param rawParams - raw params as the host supplied them

 @returns params as a key-indexed record

 @throws ValiError when the host sent a non-object parameter payload

 @example
 ```ts
 toRawParamRecord({ query: 'docs' });
 ```
 */
function toRawParamRecord(rawParams: unknown,): Readonly<Record<string, unknown>> {
  return v.parse(
    v.record(
      v.string(),
      v.unknown(),
    ),
    rawParams,
  );
}

/**
 Collect input keys unsupported by this version.

 @param input - input record with actual runtime params and supported key names

 @returns ignored key names in caller-provided order

 @example
 ```ts
 collectIgnoredKeys({ input: { query: 'docs', limit: 3 }, supportedKeys: ['query'] });
 ```
 */
export function collectIgnoredKeys(
  {
    input,
    supportedKeys,
  }: {
    /**
     Actual runtime params as supplied by the host.
     */
    readonly input: Readonly<Record<string, unknown>>;
    /**
     Supported key names for this tool version.
     */
    readonly supportedKeys: readonly string[];
  },
): readonly string[] {
  /**
   Supported key lookup set.
   */
  const supported = new Set(supportedKeys,);
  return Object.keys(input,)
    .filter(function isIgnoredKey(key,) {
      return !supported.has(key,);
    },);
}

/**
 Build supported search input from validated params.

 @param params - validated search params

 @returns sanitized search input

 @example
 ```ts
 supportedSearchInput({ query: 'docs' });
 ```
 */
function supportedSearchInput(
  params: ReadonlyDeep<LinkupWebSearchParams>,
): LinkupWebSearchInput {
  return {
    query: params.query,
    ...(params.fromDate === undefined ? {} : { fromDate: params.fromDate, }),
    ...(params.includeDomains === undefined ? {} : { includeDomains: params.includeDomains, }),
    ...(params.toDate === undefined ? {} : { toDate: params.toDate, }),
  };
}

/**
 Build supported fetch input from validated params.

 @param params - validated fetch params

 @returns sanitized fetch input

 @example
 ```ts
 supportedFetchInput({ url: 'https://example.com' });
 ```
 */
function supportedFetchInput(
  params: LinkupWebFetchParams,
): LinkupWebFetchInput {
  return {
    url: params.url,
  };
}

//endregion Input helpers
