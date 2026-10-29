/**
 Exa HTTP client for Pi Search Fetch tools.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  FetchOptions,
  SearchOptions,
} from './client.ts';
import { postExaJson, } from './exa-http.ts';
import type {
  ExaClient,
  ExaClientOptions,
  ExaClientRuntime,
  ExaContentsRequestBody,
  ExaSearchRequestBody,
} from './search-fetch-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: exaLogger, },);
 ```
 */
const exaLogger = tagged({ tag: 'pi-search-fetch', },);

//region Constants

/**
 Default Exa API base URL.
 */
const DEFAULT_EXA_BASE_URL = 'https://api.exa.ai' as const;

/**
 Exa search endpoint path.
 */
const EXA_SEARCH_ENDPOINT = '/search' as const;

/**
 Exa contents endpoint path.
 */
const EXA_CONTENTS_ENDPOINT = '/contents' as const;

/**
 Fixed Exa search mode chosen for web_search.
 */
const EXA_SEARCH_TYPE = 'fast' as const;

/**
 Fixed Exa result count chosen for web_search.
 */
const EXA_SEARCH_NUM_RESULTS = 10 as const;

/**
 Separator indicating a host-like blocklist entry.
 */
const HOST_LABEL_SEPARATOR = '.' as const;

//endregion Constants

/**
 Module logger.
 */
const l = tagged({
  tag: 'exa-client',
  l: exaLogger,
},);

//region Client factory

/**
 Create Exa HTTP client that preserves this extension's fixed request policy.
 
 @param clientOptions - client options
 
 @returns frozen Exa client
 
 @example
 ```ts
 const client = createExaClient({ apiKey: 'key', blocklist: [] });
 ```
 */
function createExaClient(clientOptions: ForeignBorrowed<ExaClientOptions>,): ExaClient {
  /**
   Runtime dependencies captured by client methods.
   */
  const runtime: ExaClientRuntime = {
    ...(clientOptions.apiKey === undefined ? {} : { apiKey: clientOptions.apiKey, }),
    ...(clientOptions.credentialHint === undefined ? {} : { credentialHint: clientOptions.credentialHint, }),
    blocklist: clientOptions.blocklist,
    baseUrl: clientOptions.baseUrl ?? DEFAULT_EXA_BASE_URL,
    fetchImpl: clientOptions.fetchImpl ?? fetch,
  };

  return Object.freeze({
    /**
     Runs one caller-owned Exa search request.
     
     @param searchOptions - Search input and optional signal.
     
     @returns Parsed Exa response.
     
     @mutates searchOptions - Provider request may retain signal and invoke configured fetch capability.
     */
    search(searchOptions: ForeignBorrowed<SearchOptions>,): Promise<unknown> {
      return searchExa({
        runtime,
        options: searchOptions,
      },);
    },
    /**
     Runs one caller-owned Exa contents request.
     
     @param fetchOptions - Fetch input and optional signal.
     
     @returns Parsed Exa response.
     
     @mutates fetchOptions - Provider request may retain signal and invoke configured fetch capability.
     */
    fetch(fetchOptions: ForeignBorrowed<FetchOptions>,): Promise<unknown> {
      return fetchExa({
        runtime,
        options: fetchOptions,
      },);
    },
  },);
}

/**
 Execute a fixed-policy Exa search.
 
 @param runtime - client runtime dependencies
 
 @param options - search input and cancellation signal
 
 @returns parsed Exa response object
 
 @throws when API key is absent, Exa rejects, JSON parsing fails, or request aborts
 
 @mutates runtime - Provider request invokes `runtime.fetchImpl` and reads retained blocklist data.
 
 @mutates options - Provider request may retain `options.signal` and register abort listeners.
 */
function searchExa(
  {
    runtime,
    options,
  }: {
    readonly runtime: ExaClientRuntime;
    readonly options: SearchOptions;
  },
): Promise<unknown> {
  /**
   Logger tagged for this search call.
   */
  const innerL = tagged({
    tag: searchExa.name,
    l,
  },);
  /**
   Search input snapshot.
   */
  const { input, } = options;
  /**
   Exa search body with fixed mode, result count, and compatible blocklist.
   */
  const body: ExaSearchRequestBody = {
    query: input.query,
    type: EXA_SEARCH_TYPE,
    numResults: EXA_SEARCH_NUM_RESULTS,
    excludeDomains: exaForwardableBlocklist(runtime.blocklist,),
    ...(input.fromDate === undefined ? {} : { startPublishedDate: input.fromDate, }),
    ...(input.includeDomains === undefined ? {} : { includeDomains: input.includeDomains, }),
    ...(input.toDate === undefined ? {} : { endPublishedDate: input.toDate, }),
  };

  innerL.debug(`calling Exa search for query: ${input.query}`,);
  return postExaJson({
    runtime,
    endpoint: EXA_SEARCH_ENDPOINT,
    body,
    ...(options.signal === undefined ? {} : { signal: options.signal, }),
  },);
}

/**
 Execute a fixed-policy Exa contents fetch.
 
 @param runtime - client runtime dependencies
 
 @param options - fetch input and cancellation signal
 
 @returns parsed Exa response object
 
 @throws when API key is absent, Exa rejects, JSON parsing fails, or request aborts
 
 @mutates runtime - Provider request invokes `runtime.fetchImpl`.
 
 @mutates options - Provider request may retain `options.signal` and register abort listeners.
 */
function fetchExa(
  {
    runtime,
    options,
  }: {
    readonly runtime: ExaClientRuntime;
    readonly options: FetchOptions;
  },
): Promise<unknown> {
  /**
   Logger tagged for this fetch call.
   */
  const innerL = tagged({
    tag: fetchExa.name,
    l,
  },);
  /**
   Fetch input snapshot.
   */
  const { input, } = options;
  /**
   Exa contents body requesting text extraction for one URL.
   */
  const body: ExaContentsRequestBody = {
    urls: [input.url,],
    text: true,
  };

  innerL.debug(`calling Exa contents for URL: ${input.url}`,);
  return postExaJson({
    runtime,
    endpoint: EXA_CONTENTS_ENDPOINT,
    body,
    ...(options.signal === undefined ? {} : { signal: options.signal, }),
  },);
}

//endregion Client factory

//region Policy helpers

/**
 Return blocklist entries Exa accepts in excludeDomains.
 
 @param blocklist - normalized local blocklist
 
 @returns Exa-compatible domain entries
 
 @example
 ```ts
 exaForwardableBlocklist(['gov', 'example.com']);
 ```
 */
function exaForwardableBlocklist(blocklist: readonly string[],): readonly string[] {
  return blocklist.filter(function hasHostLabelSeparator(entry,) {
    return entry.includes(HOST_LABEL_SEPARATOR,);
  },);
}

//endregion Policy helpers

export {
  DEFAULT_EXA_BASE_URL,
  EXA_CONTENTS_ENDPOINT,
  EXA_SEARCH_ENDPOINT,
  createExaClient,
  exaForwardableBlocklist,
};
export type {
  ExaClient,
  ExaClientOptions,
  ExaContentsRequestBody,
  ExaSearchRequestBody,
} from './search-fetch-types.ts';
