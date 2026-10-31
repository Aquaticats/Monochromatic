/**
 Provider-routing client for Pi Search Fetch tools.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  DEFAULT_LINKUP_BASE_URL,
  createLinkupClient,
  type FetchOptions,
  type SearchOptions,
} from './client.ts';
import {
  DEFAULT_DEEPSEEK_BASE_URL,
  createDeepseekClient,
} from './deepseek-client.ts';
import {
  DEFAULT_EXA_BASE_URL,
  createExaClient,
} from './exa-client.ts';
import { createGhClient, } from './gh-client.ts';
import { fetchWithFallback, } from './search-fetch-fetch.ts';
import { searchWithFallback, } from './search-fetch-search.ts';
import type {
  ProviderResponse,
  SearchFetchClient,
  SearchFetchClientOptions,
  SearchFetchClientRuntime,
} from './search-fetch-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 */
const searchFetchLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'search-fetch-client',
  l: searchFetchLogger,
},);

/**
 Create provider-routing Search Fetch client.
 
 @param clientOptions - client options
 
 @returns frozen provider-routing client
 
 @example
 ```ts
 const client = createSearchFetchClient({ deepseekApiKey: 'key', blocklist: [] });
 ```
 */
function createSearchFetchClient(
  clientOptions: ForeignBorrowed<SearchFetchClientOptions>,
): SearchFetchClient {
  /**
   Runtime dependencies captured by client methods.
   */
  const runtime: SearchFetchClientRuntime = {
    ...(clientOptions.deepseekApiKey === undefined ? {} : { deepseekApiKey: clientOptions.deepseekApiKey, }),
    ...(clientOptions.exaApiKey === undefined ? {} : { exaApiKey: clientOptions.exaApiKey, }),
    ...(clientOptions.linkupApiKey === undefined ? {} : { linkupApiKey: clientOptions.linkupApiKey, }),
    ...(clientOptions.credentialHint === undefined ? {} : { credentialHint: clientOptions.credentialHint, }),
    blocklist: clientOptions.blocklist,
    deepseekBaseUrl: clientOptions.deepseekBaseUrl ?? DEFAULT_DEEPSEEK_BASE_URL,
    exaBaseUrl: clientOptions.exaBaseUrl ?? DEFAULT_EXA_BASE_URL,
    linkupBaseUrl: clientOptions.linkupBaseUrl ?? DEFAULT_LINKUP_BASE_URL,
    fetchImpl: clientOptions.fetchImpl ?? fetch,
  };
  /**
   DeepSeek client shared by routed operations.
   */
  const deepseekClient = createDeepseekClient({
    ...(runtime.deepseekApiKey === undefined ? {} : { apiKey: runtime.deepseekApiKey, }),
    ...(runtime.credentialHint === undefined ? {} : { credentialHint: runtime.credentialHint, }),
    baseUrl: runtime.deepseekBaseUrl,
    fetchImpl: runtime.fetchImpl,
  },);
  /**
   Exa client shared by routed operations.
   */
  const exaClient = createExaClient({
    ...(runtime.exaApiKey === undefined ? {} : { apiKey: runtime.exaApiKey, }),
    ...(runtime.credentialHint === undefined ? {} : { credentialHint: runtime.credentialHint, }),
    blocklist: runtime.blocklist,
    baseUrl: runtime.exaBaseUrl,
    fetchImpl: runtime.fetchImpl,
  },);
  /**
   Linkup client shared by routed operations.
   */
  const linkupClient = createLinkupClient({
    ...(runtime.linkupApiKey === undefined ? {} : { apiKey: runtime.linkupApiKey, }),
    ...(runtime.credentialHint === undefined ? {} : { credentialHint: runtime.credentialHint, }),
    blocklist: runtime.blocklist,
    baseUrl: runtime.linkupBaseUrl,
    fetchImpl: runtime.fetchImpl,
  },);
  /**
   gh-backed client serving planned GitHub URLs before paid providers.
   */
  const ghClient = clientOptions.ghClient ?? createGhClient({},);

  return Object.freeze({
    search(searchOptions: ForeignBorrowed<SearchOptions>,): Promise<ProviderResponse> {
      return searchWithFallback({
        runtime,
        deepseekClient,
        exaClient,
        linkupClient,
        options: searchOptions,
      },);
    },
    fetch(fetchOptions: ForeignBorrowed<FetchOptions>,): Promise<ProviderResponse> {
      return fetchWithFallback({
        runtime,
        ghClient,
        deepseekClient,
        exaClient,
        linkupClient,
        options: fetchOptions,
      },);
    },
  },);
}

export { createSearchFetchClient, };
export type {
  ProviderFallback,
  ProviderResponse,
  SearchFetchClient,
  SearchFetchClientOptions,
  SearchFetchProvider,
} from './search-fetch-types.ts';
