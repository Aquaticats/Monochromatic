/**
 Linkup HTTP client for Pi Search Fetch tools.
 
 @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  DEFAULT_LINKUP_BASE_URL,
  LINKUP_FETCH_ENDPOINT,
  LINKUP_FETCH_EXTRACT_IMAGES,
  LINKUP_FETCH_INCLUDE_RAW_HTML,
  LINKUP_FETCH_RENDER_JS,
  LINKUP_SEARCH_DEPTH,
  LINKUP_SEARCH_ENDPOINT,
  LINKUP_SEARCH_OUTPUT_TYPE,
} from './client-constants.ts';
import type {
  ClientRuntime,
  FetchOptions,
  LinkupClient,
  LinkupClientOptions,
  LinkupFetchRequestBody,
  LinkupSearchRequestBody,
  SearchOptions,
} from './client-types.ts';
import { postJson, } from './client-http.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: linkupLogger, },);
 ```
 */
const linkupLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'client',
  l: linkupLogger,
},);

//region Client factory

/**
 Create Linkup HTTP client that preserves this extension's fixed request policy.
 
 @param clientOptions - client options
 
 @returns frozen Linkup client
 
 @example
 ```ts
 const client = createLinkupClient({ apiKey: 'key', blocklist: [] });
 ```
 */
function createLinkupClient(clientOptions: ForeignBorrowed<LinkupClientOptions>,): LinkupClient {
  /**
   Runtime dependencies captured by client methods.
   */
  const runtime: ClientRuntime = {
    ...(clientOptions.apiKey === undefined ? {} : { apiKey: clientOptions.apiKey, }),
    ...(clientOptions.credentialHint === undefined ? {} : { credentialHint: clientOptions.credentialHint, }),
    blocklist: clientOptions.blocklist,
    baseUrl: clientOptions.baseUrl ?? DEFAULT_LINKUP_BASE_URL,
    fetchImpl: clientOptions.fetchImpl ?? fetch,
  };

  return Object.freeze({
    /**
     Runs one caller-owned Linkup search request.
     
     @param searchOptions - Search input and optional signal.
     
     @returns Parsed Linkup response.
     
     @mutates searchOptions - Provider request may retain signal and invoke configured fetch capability.
     */
    search(searchOptions: ForeignBorrowed<SearchOptions>,): Promise<unknown> {
      return searchLinkup({
        runtime,
        options: searchOptions,
      },);
    },
    /**
     Runs one caller-owned Linkup fetch request.
     
     @param fetchOptions - Fetch input and optional signal.
     
     @returns Parsed Linkup response.
     
     @mutates fetchOptions - Provider request may retain signal and invoke configured fetch capability.
     */
    fetch(fetchOptions: ForeignBorrowed<FetchOptions>,): Promise<unknown> {
      return fetchLinkup({
        runtime,
        options: fetchOptions,
      },);
    },
  },);
}

/**
 Execute a fixed-policy Linkup search.
 
 @param runtime - client runtime dependencies
 
 @param options - search input and cancellation signal
 
 @returns parsed Linkup response object
 
 @throws when API key is absent, Linkup rejects, JSON parsing fails, or request aborts
 
 @mutates runtime - Provider request invokes `runtime.fetchImpl` and reads retained blocklist data.
 
 @mutates options - Provider request may retain `options.signal` and register abort listeners.
 */
function searchLinkup(
  {
    runtime,
    options,
  }: {
    readonly runtime: ClientRuntime;
    readonly options: SearchOptions;
  },
): Promise<unknown> {
  /**
   Logger tagged for this search call.
   */
  const innerL = tagged({
    tag: searchLinkup.name,
    l,
  },);
  /**
   Search input snapshot.
   */
  const {input} = options;
  /**
   Linkup search body with fixed depth, output type, and global blocklist.
   */
  const body: LinkupSearchRequestBody = {
    q: input.query,
    depth: LINKUP_SEARCH_DEPTH,
    outputType: LINKUP_SEARCH_OUTPUT_TYPE,
    excludeDomains: runtime.blocklist,
    ...(input.fromDate === undefined ? {} : { fromDate: input.fromDate, }),
    ...(input.includeDomains === undefined ? {} : { includeDomains: input.includeDomains, }),
    ...(input.toDate === undefined ? {} : { toDate: input.toDate, }),
  };

  innerL.debug(`calling Linkup search for query: ${input.query}`,);
  return postJson({
    runtime,
    endpoint: LINKUP_SEARCH_ENDPOINT,
    body,
    ...(options.signal === undefined ? {} : { signal: options.signal, }),
  },);
}

/**
 Execute a fixed-policy Linkup fetch.
 
 @param runtime - client runtime dependencies
 
 @param options - fetch input and cancellation signal
 
 @returns parsed Linkup response object
 
 @throws when API key is absent, Linkup rejects, JSON parsing fails, or request aborts
 
 @mutates runtime - Provider request invokes `runtime.fetchImpl`.
 
 @mutates options - Provider request may retain `options.signal` and register abort listeners.
 */
function fetchLinkup(
  {
    runtime,
    options,
  }: {
    readonly runtime: ClientRuntime;
    readonly options: FetchOptions;
  },
): Promise<unknown> {
  /**
   Logger tagged for this fetch call.
   */
  const innerL = tagged({
    tag: fetchLinkup.name,
    l,
  },);
  /**
   Fetch input snapshot.
   */
  const {input} = options;
  /**
   Linkup fetch body with fixed rendering and extraction flags.
   */
  const body: LinkupFetchRequestBody = {
    url: input.url,
    renderJs: LINKUP_FETCH_RENDER_JS,
    extractImages: LINKUP_FETCH_EXTRACT_IMAGES,
    includeRawHtml: LINKUP_FETCH_INCLUDE_RAW_HTML,
  };

  innerL.debug(`calling Linkup fetch for URL: ${input.url}`,);
  return postJson({
    runtime,
    endpoint: LINKUP_FETCH_ENDPOINT,
    body,
    ...(options.signal === undefined ? {} : { signal: options.signal, }),
  },);
}

//endregion Client factory



export {
  DEFAULT_LINKUP_BASE_URL,
  LINKUP_FETCH_ENDPOINT,
  LINKUP_SEARCH_ENDPOINT,
} from './client-constants.ts';
export { createLinkupClient, };
export type {
  FetchLike,
  FetchOptions,
  LinkupClient,
  LinkupClientOptions,
  LinkupFetchRequestBody,
  LinkupSearchRequestBody,
  LinkupWebFetchInput,
  LinkupWebSearchInput,
  SearchOptions,
} from './client-types.ts';
