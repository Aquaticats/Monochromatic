/**
 Search provider routing for Search Fetch tools.
 
 @module
 */

import { caughtValueText as errorMessage, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type {
  LinkupClient,
  SearchOptions,
} from './client.ts';
import {
  hasConfiguredCredential,
  rethrowIfCancelled,
  searchLinkupFallback,
} from './provider-fallback.ts';
import type {
  DeepseekClient,
  ExaClient,
  ProviderFallback,
  ProviderResponse,
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
  tag: 'search-fetch-search',
  l: searchFetchLogger,
},);

/**
 Search through DeepSeek first, then Exa, then Linkup.
 
 @param runtime - client runtime dependencies
 
 @param deepseekClient - DeepSeek client
 
 @param exaClient - Exa client
 
 @param linkupClient - Linkup client
 
 @param options - search options
 
 @returns provider-tagged search response
 
 @example
 ```ts
 await searchWithFallback({ runtime, deepseekClient, exaClient, linkupClient, options: { input: { query: 'docs' } } });
 ```
 */
async function searchWithFallback(
  {
    runtime,
    deepseekClient,
    exaClient,
    linkupClient,
    options,
  }: {
    readonly runtime: SearchFetchClientRuntime;
    readonly deepseekClient: DeepseekClient;
    readonly exaClient: ExaClient;
    readonly linkupClient: LinkupClient;
    readonly options: SearchOptions;
  },
): Promise<ProviderResponse> {
  /**
   Fallback steps taken before the next provider attempt.
   */
  const fallbackChain: ProviderFallback[] = [];
  if (hasConfiguredCredential(runtime.deepseekApiKey,)) {
    try {
      return {
        provider: 'deepseek',
        response: await deepseekClient.search(options,),
      };
    }
    catch (error: unknown) {
      rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
      /**
       Safe DeepSeek failure text for logs and details.
       */
      const reason = errorMessage(error,);
      l.warn(`DeepSeek search unavailable; falling back to Exa: ${reason}`,);
      fallbackChain.push({
        from: 'deepseek',
        to: 'exa',
        reason,
      },);
    }
  }
  else {
    l.warn('DeepSeek search unavailable; falling back to Exa: missing DeepSeek API key',);
    fallbackChain.push({
      from: 'deepseek',
      to: 'exa',
      reason: 'missing DeepSeek API key',
    },);
  }

  if (hasConfiguredCredential(runtime.exaApiKey,)) {
    try {
      return {
        provider: 'exa',
        response: await exaClient.search(options,),
        ...(fallbackChain.length === 0 ? {} : { fallbackChain, }),
      };
    }
    catch (error: unknown) {
      rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
      /**
       Safe Exa failure text for logs and details.
       */
      const reason = errorMessage(error,);
      l.warn(`Exa search unavailable; falling back to Linkup: ${reason}`,);
      fallbackChain.push({
        from: 'exa',
        to: 'linkup',
        reason,
      },);
    }
  }
  else {
    l.warn('Exa search unavailable; falling back to Linkup: missing Exa API key',);
    fallbackChain.push({
      from: 'exa',
      to: 'linkup',
      reason: 'missing Exa API key',
    },);
  }

  return searchLinkupFallback({
    linkupClient,
    options,
    fallbackChain,
  },);
}

export { searchWithFallback, };
