/**
 Fetch provider routing for Search Fetch tools.
 
 @module
 */

import { caughtValueText as errorMessage, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type {
  FetchOptions,
  LinkupClient,
} from './client.ts';
import type { GhClient, } from './github-fetch-types.ts';
import { planGitHubFetch, } from './github-url-plan.ts';
import {
  fetchExaFallback,
  hasConfiguredCredential,
  rethrowIfCancelled,
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
  tag: 'search-fetch-fetch',
  l: searchFetchLogger,
},);

/**
 Fetch through gh for planned GitHub URLs, then DeepSeek, then Linkup, then Exa contents.
 
 @param runtime - client runtime dependencies
 
 @param ghClient - gh-backed fetch client
 
 @param deepseekClient - DeepSeek client
 
 @param exaClient - Exa client
 
 @param linkupClient - Linkup client
 
 @param options - fetch options
 
 @returns provider-tagged fetch response
 
 @throws when the caller-owned signal cancels a gh invocation
 
 @mutates options - gh children register abort listeners on `options.signal`.
 
 @example
 ```ts
 await fetchWithFallback({ runtime, ghClient, deepseekClient, exaClient, linkupClient, options: { input: { url: 'https://example.com' } } });
 ```
 */
async function fetchWithFallback(
  {
    runtime,
    ghClient,
    deepseekClient,
    exaClient,
    linkupClient,
    options,
  }: {
    readonly runtime: SearchFetchClientRuntime;
    readonly ghClient: GhClient;
    readonly deepseekClient: DeepseekClient;
    readonly exaClient: ExaClient;
    readonly linkupClient: LinkupClient;
    readonly options: FetchOptions;
  },
): Promise<ProviderResponse> {
  /**
   gh plan for this fetch URL.
   */
  const plan = planGitHubFetch({ url: options.input
    .url, },);
  if (!plan.planned) {
    l.debug(`routing ${options.input
      .url} past gh: ${plan.reason}`,);
    return fetchDeepseekThenLinkupThenExa({
      runtime,
      deepseekClient,
      exaClient,
      linkupClient,
      options,
    },);
  }

  try {
    return {
      provider: 'gh',
      response: await ghClient.fetch({
        url: options.input
          .url,
        kind: plan.kind,
        attempts: plan.attempts,
        ...(options.signal === undefined ? {} : { signal: options.signal, }),
      },),
    };
  }
  catch (error: unknown) {
    rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
    /**
     Safe gh failure text for logs and details.
     */
    const reason = errorMessage(error,);
    l.warn(`gh fetch unavailable; falling back to DeepSeek: ${reason}`,);
    return fetchDeepseekThenLinkupThenExa({
      runtime,
      deepseekClient,
      exaClient,
      linkupClient,
      options,
      fallbackChain: [
        {
          from: 'gh',
          to: 'deepseek',
          reason,
        },
      ],
    },);
  }
}

/**
 Fetch through DeepSeek first, then Linkup, then Exa contents.
 
 @param runtime - client runtime dependencies
 
 @param deepseekClient - DeepSeek client
 
 @param exaClient - Exa client
 
 @param linkupClient - Linkup client
 
 @param options - fetch options
 
 @param fallbackChain - fallback steps already taken before DeepSeek
 
 @returns provider-tagged fetch response
 */
async function fetchDeepseekThenLinkupThenExa(
  {
    runtime,
    deepseekClient,
    exaClient,
    linkupClient,
    options,
    fallbackChain = [],
  }: {
    readonly runtime: SearchFetchClientRuntime;
    readonly deepseekClient: DeepseekClient;
    readonly exaClient: ExaClient;
    readonly linkupClient: LinkupClient;
    readonly options: FetchOptions;
    readonly fallbackChain?: readonly ProviderFallback[];
  },
): Promise<ProviderResponse> {
  rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
  /**
   Fallback steps accumulated through this helper.
   */
  const chain: ProviderFallback[] = [...fallbackChain,];
  if (hasConfiguredCredential(runtime.deepseekApiKey,)) {
    try {
      return {
        provider: 'deepseek',
        response: await deepseekClient.fetch(options,),
        ...(chain.length === 0 ? {} : { fallbackChain: chain, }),
      };
    }
    catch (error: unknown) {
      rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
      /**
       Safe DeepSeek failure text for logs and details.
       */
      const reason = errorMessage(error,);
      l.warn(`DeepSeek fetch unavailable; falling back to Linkup: ${reason}`,);
      chain.push({
        from: 'deepseek',
        to: 'linkup',
        reason,
      },);
    }
  }
  else {
    l.warn('DeepSeek fetch unavailable; falling back to Linkup: missing DeepSeek API key',);
    chain.push({
      from: 'deepseek',
      to: 'linkup',
      reason: 'missing DeepSeek API key',
    },);
  }

  return fetchLinkupThenExa({
    runtime,
    exaClient,
    linkupClient,
    options,
    fallbackChain: chain,
  },);
}

/**
 Fetch through Linkup first and fall back to Exa contents.
 
 @param runtime - client runtime dependencies
 
 @param exaClient - Exa client
 
 @param linkupClient - Linkup client
 
 @param options - fetch options
 
 @param fallbackChain - fallback steps already taken before Linkup
 
 @returns provider-tagged fetch response
 */
async function fetchLinkupThenExa(
  {
    runtime,
    exaClient,
    linkupClient,
    options,
    fallbackChain = [],
  }: {
    readonly runtime: SearchFetchClientRuntime;
    readonly exaClient: ExaClient;
    readonly linkupClient: LinkupClient;
    readonly options: FetchOptions;
    readonly fallbackChain?: readonly ProviderFallback[];
  },
): Promise<ProviderResponse> {
  rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
  if (hasConfiguredCredential(runtime.linkupApiKey,)) {
    try {
      return {
        provider: 'linkup',
        response: await linkupClient.fetch(options,),
        ...(fallbackChain.length === 0 ? {} : { fallbackChain, }),
      };
    }
    catch (error: unknown) {
      rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
      /**
       Safe Linkup failure text for logs and details.
       */
      const reason = errorMessage(error,);
      l.warn(`Linkup fetch unavailable; falling back to Exa: ${reason}`,);
      return fetchExaFallback({
        exaClient,
        options,
        fallbackChain: [
          ...fallbackChain,
          {
            from: 'linkup',
            to: 'exa',
            reason,
          },
        ],
      },);
    }
  }

  l.warn('Linkup fetch unavailable; falling back to Exa: missing Linkup API key',);
  return fetchExaFallback({
    exaClient,
    options,
    fallbackChain: [
      ...fallbackChain,
      {
        from: 'linkup',
        to: 'exa',
        reason: 'missing Linkup API key',
      },
    ],
  },);
}

export { fetchWithFallback, };
