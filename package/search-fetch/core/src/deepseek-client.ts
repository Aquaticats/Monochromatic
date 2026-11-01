/**
 DeepSeek Tools API client for Pi Search Fetch tools.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  FetchOptions,
  SearchOptions,
} from './client.ts';
import {
  filterDeepseekResults,
  searchResultsFromResponse,
} from './deepseek-filter.ts';
import { postDeepseekJson, } from './deepseek-http.ts';
import { composeDeepseekQuery, } from './deepseek-query.ts';
import type {
  DeepseekClient,
  DeepseekClientOptions,
  DeepseekClientRuntime,
  DeepseekFetchRequestBody,
  DeepseekSearchRequestBody,
} from './search-fetch-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 */
const deepseekLogger = tagged({ tag: 'pi-search-fetch', },);

//region Constants

/**
 Default DeepSeek API base URL.
 */
const DEFAULT_DEEPSEEK_BASE_URL = 'https://api.deepseek.com' as const;

/**
 DeepSeek Search endpoint path.
 */
const DEEPSEEK_SEARCH_ENDPOINT = '/v1/tools/search' as const;

/**
 DeepSeek Fetch endpoint path.
 */
const DEEPSEEK_FETCH_ENDPOINT = '/v1/tools/fetch' as const;

/**
 Fixed result count requested from DeepSeek search.
 */
// oxlint-disable-next-line eslint/no-magic-numbers -- DeepSeek documents 50 as the maximum result count; the constant name is the only readable place to record the API value.
const DEEPSEEK_SEARCH_COUNT = 50 as const;

/**
 Fixed page-content request sent to DeepSeek search.
 */
const DEEPSEEK_SEARCH_NEED_CONTENT = true as const;

//endregion Constants

/**
 Module logger.
 */
const l = tagged({
  tag: 'deepseek-client',
  l: deepseekLogger,
},);

//region Client factory

/**
 Create a DeepSeek client that preserves this extension's fixed request policy.
 
 @param clientOptions - client options
 
 @returns frozen DeepSeek client
 
 @example
 ```ts
 const client = createDeepseekClient({ apiKey: 'key' });
 ```
 */
function createDeepseekClient(clientOptions: ForeignBorrowed<DeepseekClientOptions>,): DeepseekClient {
  /**
   Runtime dependencies captured by client methods.
   */
  const runtime: DeepseekClientRuntime = {
    ...(clientOptions.apiKey === undefined ? {} : { apiKey: clientOptions.apiKey, }),
    ...(clientOptions.credentialHint === undefined ? {} : { credentialHint: clientOptions.credentialHint, }),
    baseUrl: clientOptions.baseUrl ?? DEFAULT_DEEPSEEK_BASE_URL,
    fetchImpl: clientOptions.fetchImpl ?? fetch,
  };

  return Object.freeze({
    /**
     Runs one caller-owned DeepSeek search request.
     
     @param searchOptions - Search input and optional signal.
     
     @returns Filtered DeepSeek results envelope.
     
     @mutates searchOptions - Provider request may retain signal and invoke configured fetch capability.
     */
    search(searchOptions: ForeignBorrowed<SearchOptions>,): Promise<unknown> {
      return searchDeepseek({
        runtime,
        options: searchOptions,
      },);
    },
    /**
     Runs one caller-owned DeepSeek fetch request.
     
     @param fetchOptions - Fetch input and optional signal.
     
     @returns Normalized markdown response.
     
     @mutates fetchOptions - Provider request may retain signal and invoke configured fetch capability.
     */
    fetch(fetchOptions: ForeignBorrowed<FetchOptions>,): Promise<unknown> {
      return fetchDeepseek({
        runtime,
        options: fetchOptions,
      },);
    },
  },);
}

/**
 Execute a fixed-policy DeepSeek search.
 
 @param runtime - client runtime dependencies
 
 @param options - search input and cancellation signal
 
 @returns DeepSeek results envelope after local filtering and deduplication
 
 @throws when API key is absent, the query exceeds the 128 code point limit,
 DeepSeek rejects, JSON parsing fails, or the response shape is unexpected
 
 @mutates runtime - Provider request invokes `runtime.fetchImpl`.
 
 @mutates options - Provider request may retain `options.signal` and register abort listeners.
 */
async function searchDeepseek(
  {
    runtime,
    options,
  }: {
    readonly runtime: DeepseekClientRuntime;
    readonly options: SearchOptions;
  },
): Promise<unknown> {
  /**
   Logger tagged for this search call.
   */
  const innerL = tagged({
    tag: searchDeepseek.name,
    l,
  },);
  /**
   Search input snapshot.
   */
  const { input, } = options;
  /**
   Query composed from the user query plus the filter operators that fit.
   */
  const query = composeDeepseekQuery(input,);
  /**
   DeepSeek search body with fixed count and content policy.
   */
  const body: DeepseekSearchRequestBody = {
    query,
    count: DEEPSEEK_SEARCH_COUNT,
    need_content: DEEPSEEK_SEARCH_NEED_CONTENT,
  };

  innerL.debug(`calling DeepSeek search for query: ${query}`,);
  /**
   Parsed DeepSeek response.
   */
  const response = await postDeepseekJson({
    runtime,
    endpoint: DEEPSEEK_SEARCH_ENDPOINT,
    body,
    ...(options.signal === undefined ? {} : { signal: options.signal, }),
  },);
  /**
   Raw results array from the response envelope.
   */
  const rawResults = searchResultsFromResponse({ response, },);
  /**
   Results kept after provider filters and deduplication.
   */
  const filteredResults = filterDeepseekResults({
    results: rawResults,
    input,
  },);
  if (filteredResults.length !== rawResults.length)
    innerL.debug(
      `DeepSeek search kept ${String(filteredResults.length,)} of ${String(rawResults.length,)} results after filters`,
    );
  return {
    ...(isRecord(response,)
      ? response
      : {}),
    results: filteredResults,
  };
}

/**
 Execute a fixed-policy DeepSeek fetch.
 
 @param runtime - client runtime dependencies
 
 @param options - fetch input and cancellation signal
 
 @returns markdown-only response envelope
 
 @throws when API key is absent, DeepSeek rejects, page content is empty,
 JSON parsing fails, or the response shape is unexpected
 
 @mutates runtime - Provider request invokes `runtime.fetchImpl`.
 
 @mutates options - Provider request may retain `options.signal` and register abort listeners.
 */
async function fetchDeepseek(
  {
    runtime,
    options,
  }: {
    readonly runtime: DeepseekClientRuntime;
    readonly options: FetchOptions;
  },
): Promise<unknown> {
  /**
   Logger tagged for this fetch call.
   */
  const innerL = tagged({
    tag: fetchDeepseek.name,
    l,
  },);
  /**
   Fetch input snapshot.
   */
  const { input, } = options;
  /**
   DeepSeek fetch body.
   */
  const body: DeepseekFetchRequestBody = {
    url: input.url,
  };

  innerL.debug(`calling DeepSeek fetch for URL: ${input.url}`,);
  /**
   Parsed DeepSeek response.
   */
  const response = await postDeepseekJson({
    runtime,
    endpoint: DEEPSEEK_FETCH_ENDPOINT,
    body,
    ...(options.signal === undefined ? {} : { signal: options.signal, }),
  },);
  if (!isRecord(response,))
    throw new Error(
      `DeepSeek ${DEEPSEEK_FETCH_ENDPOINT} returned an unexpected response shape`,
    );
  /**
   Fetched page body.
   */
  const {content} = response;
  if (((typeof content) !== 'string') || (content.trim() === ''))
    throw new Error(
      `DeepSeek ${DEEPSEEK_FETCH_ENDPOINT} returned no page content for ${input.url}`,
    );
  return { markdown: content, };
}

/**
 Return whether a value is a non-null object record.
 
 @param value - unknown value
 
 @returns whether the value can be read by string keys
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

//endregion Client factory

export {
  DEFAULT_DEEPSEEK_BASE_URL,
  DEEPSEEK_FETCH_ENDPOINT,
  DEEPSEEK_SEARCH_COUNT,
  DEEPSEEK_SEARCH_ENDPOINT,
  DEEPSEEK_SEARCH_NEED_CONTENT,
  createDeepseekClient,
};
export type {
  DeepseekClient,
  DeepseekClientOptions,
  DeepseekFetchRequestBody,
  DeepseekSearchRequestBody,
} from './search-fetch-types.ts';
