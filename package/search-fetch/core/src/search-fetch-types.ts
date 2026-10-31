
/**
 Provider-neutral Search Fetch client types.
 
 @module
 */

import type {
  FetchLike,
  FetchOptions,
  SearchOptions,
} from './client.ts';
import type { GhClient, } from './github-fetch-types.ts';

/**
 Search Fetch provider identifier.
 */
type SearchFetchProvider = 'deepseek' | 'exa' | 'gh' | 'linkup';

/**
 One fallback step taken between providers.
 */
type ProviderFallback = {
  /**
   Provider that failed or was unavailable.
   */
  readonly from: SearchFetchProvider;
  /**
   Provider attempted after fallback.
   */
  readonly to: SearchFetchProvider;
  /**
   Safe fallback reason.
   */
  readonly reason: string;
};

/**
 Provider-tagged response returned by Search Fetch clients.
 */
type ProviderResponse = {
  /**
   Provider that produced response.
   */
  readonly provider: SearchFetchProvider;
  /**
   Provider response payload.
   */
  readonly response: unknown;
  /**
   Fallback steps taken before the responding provider answered.
   */
  readonly fallbackChain?: readonly ProviderFallback[];
};

/**
 Search Fetch client constructor options.
 */
type SearchFetchClientOptions = {
  /**
   Optional DeepSeek API key.
   */
  readonly deepseekApiKey?: string;
  /**
   Optional Exa API key.
   */
  readonly exaApiKey?: string;
  /**
   Optional Linkup API key.
   */
  readonly linkupApiKey?: string;
  /**
   Normalized global blocklist.
   */
  readonly blocklist: readonly string[];
  /**
   Optional DeepSeek base URL override for tests.
   */
  readonly deepseekBaseUrl?: string;
  /**
   Optional Exa base URL override for tests.
   */
  readonly exaBaseUrl?: string;
  /**
   Optional Linkup base URL override for tests.
   */
  readonly linkupBaseUrl?: string;
  /**
   Optional fetch implementation override for tests.
   */
  readonly fetchImpl?: FetchLike;
  /**
   Optional gh-backed fetch client override for tests.
   */
  readonly ghClient?: GhClient;
  /**
   Optional value-free credential-store diagnostic appended to missing-key errors.
   */
  readonly credentialHint?: string;
};

/**
 Search Fetch client runtime dependencies.
 */
type SearchFetchClientRuntime = {
  /**
   Optional DeepSeek API key.
   */
  readonly deepseekApiKey?: string;
  /**
   Optional Exa API key.
   */
  readonly exaApiKey?: string;
  /**
   Optional Linkup API key.
   */
  readonly linkupApiKey?: string;
  /**
   Normalized global blocklist.
   */
  readonly blocklist: readonly string[];
  /**
   DeepSeek base URL.
   */
  readonly deepseekBaseUrl: string;
  /**
   Exa base URL.
   */
  readonly exaBaseUrl: string;
  /**
   Linkup base URL.
   */
  readonly linkupBaseUrl: string;
  /**
   Fetch implementation.
   */
  readonly fetchImpl: FetchLike;
  /**
   Optional value-free credential-store diagnostic appended to missing-key errors.
   */
  readonly credentialHint?: string;
};

/**
 Provider-neutral client surface used by tools.
 */
type SearchFetchClient = {
  /**
   Execute provider-routed web search.
   */
  readonly search: (options: SearchOptions,) => Promise<ProviderResponse>;
  /**
   Execute provider-routed page fetch.
   */
  readonly fetch: (options: FetchOptions,) => Promise<ProviderResponse>;
};

/**
 Exa request body for `/search`.
 */
type ExaSearchRequestBody = {
  /**
   Query string.
   */
  readonly query: string;
  /**
   Fixed Exa search mode.
   */
  readonly type: 'fast';
  /**
   Fixed result count.
   */
  readonly numResults: 10;
  /**
   Provider-compatible domain blocklist.
   */
  readonly excludeDomains: readonly string[];
  /**
   Optional include-domain filter.
   */
  readonly includeDomains?: readonly string[];
  /**
   Optional earliest published date.
   */
  readonly startPublishedDate?: string;
  /**
   Optional latest published date.
   */
  readonly endPublishedDate?: string;
};

/**
 Exa request body for `/contents`.
 */
type ExaContentsRequestBody = {
  /**
   URLs fetched by Exa contents.
   */
  readonly urls: readonly string[];
  /**
   Request extracted page text.
   */
  readonly text: true;
};

/**
 Low-level Exa client constructor options.
 */
type ExaClientOptions = {
  /**
   Optional Exa API key.
   */
  readonly apiKey?: string;
  /**
   Normalized global blocklist.
   */
  readonly blocklist: readonly string[];
  /**
   Optional base URL override for tests.
   */
  readonly baseUrl?: string;
  /**
   Optional fetch implementation override for tests.
   */
  readonly fetchImpl?: FetchLike;
  /**
   Optional value-free credential-store diagnostic appended to missing-key errors.
   */
  readonly credentialHint?: string;
};

/**
 Low-level Exa client surface.
 */
type ExaClient = {
  /**
   Execute fixed-policy Exa search.
   */
  readonly search: (options: SearchOptions,) => Promise<unknown>;
  /**
   Execute fixed-policy Exa contents fetch.
   */
  readonly fetch: (options: FetchOptions,) => Promise<unknown>;
};

/**
 Exa client runtime dependencies.
 */
type ExaClientRuntime = {
  /**
   Optional Exa API key.
   */
  readonly apiKey?: string;
  /**
   Normalized global blocklist.
   */
  readonly blocklist: readonly string[];
  /**
   Exa API base URL.
   */
  readonly baseUrl: string;
  /**
   Fetch implementation.
   */
  readonly fetchImpl: FetchLike;
  /**
   Optional value-free credential-store diagnostic appended to missing-key errors.
   */
  readonly credentialHint?: string;
};

/**
 Exa POST JSON options.
 */
type ExaPostJsonOptions = {
  /**
   Endpoint path beginning with slash.
   */
  readonly endpoint: string;
  /**
   JSON request body.
   */
  readonly body: unknown;
  /**
   Abort signal from Pi tool execution.
   */
  readonly signal?: AbortSignal;
};

/**
 DeepSeek request body for `/v1/tools/search`.
 */
type DeepseekSearchRequestBody = {
  /**
   Search query, optionally carrying filter operators.
   */
  readonly query: string;
  /**
   Fixed maximum result count requested from DeepSeek.
   */
  readonly count: number;
  /**
   Fixed page-content request; summaries and bodies both return.
   */
  readonly need_content: true;
};

/**
 DeepSeek request body for `/v1/tools/fetch`.
 */
type DeepseekFetchRequestBody = {
  /**
   URL to fetch.
   */
  readonly url: string;
};

/**
 Low-level DeepSeek client constructor options.
 */
type DeepseekClientOptions = {
  /**
   Optional DeepSeek API key.
   */
  readonly apiKey?: string;
  /**
   Optional base URL override for tests.
   */
  readonly baseUrl?: string;
  /**
   Optional fetch implementation override for tests.
   */
  readonly fetchImpl?: FetchLike;
  /**
   Optional value-free credential-store diagnostic appended to missing-key errors.
   */
  readonly credentialHint?: string;
};

/**
 Low-level DeepSeek client surface.
 */
type DeepseekClient = {
  /**
   Execute fixed-policy DeepSeek search.
   */
  readonly search: (options: SearchOptions,) => Promise<unknown>;
  /**
   Execute fixed-policy DeepSeek fetch.
   */
  readonly fetch: (options: FetchOptions,) => Promise<unknown>;
};

/**
 DeepSeek client runtime dependencies.
 */
type DeepseekClientRuntime = {
  /**
   Optional DeepSeek API key.
   */
  readonly apiKey?: string;
  /**
   DeepSeek API base URL.
   */
  readonly baseUrl: string;
  /**
   Fetch implementation.
   */
  readonly fetchImpl: FetchLike;
  /**
   Optional value-free credential-store diagnostic appended to missing-key errors.
   */
  readonly credentialHint?: string;
};

/**
 DeepSeek POST JSON options.
 */
type DeepseekPostJsonOptions = {
  /**
   Endpoint path beginning with slash.
   */
  readonly endpoint: string;
  /**
   JSON request body.
   */
  readonly body: unknown;
  /**
   Abort signal from tool execution.
   */
  readonly signal?: AbortSignal;
};

export type {
  LinkupWebFetchInput,
  LinkupWebSearchInput,
} from './client.ts';
export type {
  DeepseekClient,
  DeepseekClientOptions,
  DeepseekClientRuntime,
  DeepseekFetchRequestBody,
  DeepseekPostJsonOptions,
  DeepseekSearchRequestBody,
  ExaClient,
  ExaClientOptions,
  ExaClientRuntime,
  ExaContentsRequestBody,
  ExaPostJsonOptions,
  ExaSearchRequestBody,
  ProviderFallback,
  ProviderResponse,
  SearchFetchClient,
  SearchFetchClientOptions,
  SearchFetchClientRuntime,
  SearchFetchProvider,
};
