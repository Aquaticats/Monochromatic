/**
 Provider fallback execution and cancellation guards for Search Fetch routing.
 
 @module
 */

import { caughtValueText as errorMessage, } from '@monochromatic-dev/caught-value-core/ts';

import type {
  FetchOptions,
  LinkupClient,
  SearchOptions,
} from './client.ts';
import type { ExaClient, } from './exa-client.ts';
import type {
  ProviderFallback,
  ProviderResponse,
} from './search-fetch-types.ts';

//region Fallback helpers

/**
 Execute Linkup search fallback and wrap failures with earlier fallback context.
 
 @param linkupClient - Linkup client
 
 @param options - search options
 
 @param fallbackChain - fallback steps already taken before Linkup
 
 @returns provider-tagged Linkup response
 
 @throws combined provider failure when Linkup also fails and the caller did not cancel
 
 @example
 ```ts
 await searchLinkupFallback({ linkupClient, options: { input: { query: 'docs' } }, fallbackChain: [] });
 ```
 */
async function searchLinkupFallback(
  {
    linkupClient,
    options,
    fallbackChain,
  }: {
    readonly linkupClient: LinkupClient;
    readonly options: SearchOptions;
    readonly fallbackChain: readonly ProviderFallback[];
  },
): Promise<ProviderResponse> {
  try {
    return {
      provider: 'linkup',
      response: await linkupClient.search(options,),
      fallbackChain,
    };
  }
  catch (error: unknown) {
    rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
    throw combinedFallbackError({
      operation: 'search',
      fallbackChain,
      finalProvider: 'Linkup',
      finalError: error,
    },);
  }
}

/**
 Execute Exa fetch fallback and wrap failures with earlier fallback context.
 
 @param exaClient - Exa client
 
 @param options - fetch options
 
 @param fallbackChain - fallback steps already taken before Exa
 
 @returns provider-tagged Exa response
 
 @throws combined provider failure when Exa also fails and the caller did not cancel
 
 @example
 ```ts
 await fetchExaFallback({ exaClient, options: { input: { url: 'https://example.com' } }, fallbackChain: [] });
 ```
 */
async function fetchExaFallback(
  {
    exaClient,
    options,
    fallbackChain,
  }: {
    readonly exaClient: ExaClient;
    readonly options: FetchOptions;
    readonly fallbackChain: readonly ProviderFallback[];
  },
): Promise<ProviderResponse> {
  try {
    return {
      provider: 'exa',
      response: await exaClient.fetch(options,),
      fallbackChain,
    };
  }
  catch (error: unknown) {
    rethrowIfCancelled(options.signal === undefined ? {} : { signal: options.signal, },);
    throw combinedFallbackError({
      operation: 'fetch',
      fallbackChain,
      finalProvider: 'Exa',
      finalError: error,
    },);
  }
}

/**
 Build an error naming every earlier fallback reason and the final provider failure.
 
 @param operation - operation name
 
 @param fallbackChain - fallback steps already taken
 
 @param finalProvider - final provider display name
 
 @param finalError - final provider error
 
 @returns combined provider failure
 
 @mutates finalError - `errorMessage` may invoke string-conversion hooks.
 
 @example
 ```ts
 combinedFallbackError({ operation: 'fetch', fallbackChain: [], finalProvider: 'Exa', finalError: new Error('boom') });
 ```
 */
function combinedFallbackError(
  {
    operation,
    fallbackChain,
    finalProvider,
    finalError,
  }: {
    readonly operation: string;
    readonly fallbackChain: readonly ProviderFallback[];
    readonly finalProvider: string;
    readonly finalError: unknown;
  },
): Error {
  /**
   Failure details in routing order, ending with the final provider failure.
   */
  const details = [
    ...fallbackChain.map(function describeStep(step: ProviderFallback,): string {
      return `${step.from} unavailable: ${step.reason}`;
    },),
    `${finalProvider} failed: ${errorMessage(finalError,)}`,
  ];
  return new Error(
    `Search Fetch ${operation} failed. ${details.join('. ',)}`,
    { cause: finalError, },
  );
}

//endregion Fallback helpers

//region Utility helpers

/**
 Rethrow a caller cancellation instead of continuing to the next provider.
 
 @param signal - caller-owned cancellation signal, when supplied
 
 @throws the signal's abort reason when the caller already cancelled
 
 @example
 ```ts
 rethrowIfCancelled({ signal: new AbortController().signal });
 ```
 */
function rethrowIfCancelled({ signal, }: { readonly signal?: AbortSignal; }): void {
  signal?.throwIfAborted();
}

/**
 Return whether optional credential has non-blank content.
 
 @param value - optional credential
 
 @returns whether credential is configured
 
 @example
 ```ts
 hasCredential({ value: 'linkup-key' });
 ```
 */
function hasCredential({ value, }: { readonly value: string; }): boolean {
  return value.trim() !== '';
}

/**
 Return whether an optional provider credential is configured.
 
 @param value - optional provider credential
 
 @returns whether the credential exists and is not blank
 
 @example
 ```ts
 hasConfiguredCredential('linkup-key');
 ```
 */
function hasConfiguredCredential(value?: string,): boolean {
  return (value !== undefined) && hasCredential({ value, });
}

//endregion Utility helpers

export {
  combinedFallbackError,
  fetchExaFallback,
  hasConfiguredCredential,
  hasCredential,
  rethrowIfCancelled,
  searchLinkupFallback,
};
