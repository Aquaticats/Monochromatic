/**
 Linkup client type definitions.
 
 @module
 */

import type {
  LINKUP_FETCH_EXTRACT_IMAGES,
  LINKUP_FETCH_INCLUDE_RAW_HTML,
  LINKUP_FETCH_RENDER_JS,
  LINKUP_SEARCH_DEPTH,
  LINKUP_SEARCH_OUTPUT_TYPE,
} from './client-constants.ts';

/**
 Model-facing search input supported by Pi Search Fetch.
 */
type LinkupWebSearchInput = {
  /**
   Natural language search query.
   */
  readonly query: string;
  /**
   Optional start date forwarded to Linkup.
   */
  readonly fromDate?: string;
  /**
   Optional include-domain filter forwarded to Linkup.
   */
  readonly includeDomains?: readonly string[];
  /**
   Optional end date forwarded to Linkup.
   */
  readonly toDate?: string;
};

/**
 Model-facing fetch input supported by Pi Search Fetch.
 */
type LinkupWebFetchInput = {
  /**
   Absolute URL to fetch through Linkup.
   */
  readonly url: string;
};

/**
 Request body sent to Linkup search.
 */
type LinkupSearchRequestBody = {
  /**
   Linkup query field.
   */
  readonly q: string;
  /**
   Fixed search depth.
   */
  readonly depth: typeof LINKUP_SEARCH_DEPTH;
  /**
   Fixed search output type.
   */
  readonly outputType: typeof LINKUP_SEARCH_OUTPUT_TYPE;
  /**
   Global blocklist sent to Linkup as excluded domains.
   */
  readonly excludeDomains: readonly string[];
  /**
   Optional start date.
   */
  readonly fromDate?: string;
  /**
   Optional include-domain filter.
   */
  readonly includeDomains?: readonly string[];
  /**
   Optional end date.
   */
  readonly toDate?: string;
};

/**
 Request body sent to Linkup fetch.
 */
type LinkupFetchRequestBody = {
  /**
   URL to fetch.
   */
  readonly url: string;
  /**
   Fixed JavaScript rendering behavior.
   */
  readonly renderJs: typeof LINKUP_FETCH_RENDER_JS;
  /**
   Fixed image extraction behavior.
   */
  readonly extractImages: typeof LINKUP_FETCH_EXTRACT_IMAGES;
  /**
   Fixed raw HTML behavior.
   */
  readonly includeRawHtml: typeof LINKUP_FETCH_INCLUDE_RAW_HTML;
};

/**
 Fetch implementation injected for tests or supplied by globalThis.
 */
type FetchLike = typeof fetch;

/**
 Linkup client constructor options.
 */
type LinkupClientOptions = {
  /**
   Optional Linkup API key.
   */
  readonly apiKey?: string;
  /**
   Normalized global blocklist sent to search.
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
 Arguments for a Linkup search request.
 */
type SearchOptions = {
  /**
   Supported search input.
   */
  readonly input: LinkupWebSearchInput;
  /**
   Abort signal from Pi tool execution.
   */
  readonly signal?: AbortSignal;
};

/**
 Arguments for a Linkup fetch request.
 */
type FetchOptions = {
  /**
   Supported fetch input.
   */
  readonly input: LinkupWebFetchInput;
  /**
   Abort signal from Pi tool execution.
   */
  readonly signal?: AbortSignal;
};

/**
 Linkup client surface used by tools.
 */
type LinkupClient = {
  /**
   Execute a fixed-policy Linkup search.
   */
  readonly search: (options: SearchOptions,) => Promise<unknown>;
  /**
   Execute a fixed-policy Linkup fetch.
   */
  readonly fetch: (options: FetchOptions,) => Promise<unknown>;
};

/**
 Arguments for internal POST requests.
 */
type PostJsonOptions = {
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
 Dependencies captured by internal client functions.
 */
type ClientRuntime = {
  /**
   Optional Linkup API key.
   */
  readonly apiKey?: string;
  /**
   Normalized global blocklist.
   */
  readonly blocklist: readonly string[];
  /**
   Linkup API base URL.
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
 Result of extracting a Linkup error message.
 */
type ExtractedLinkupErrorMessage = {
  /**
   Whether a Linkup message was present.
   */
  readonly found: false;
} | {
  /**
   Whether a Linkup message was present.
   */
  readonly found: true;
  /**
   Extracted Linkup message.
   */
  readonly message: string;
};

export type {
  ClientRuntime,
  ExtractedLinkupErrorMessage,
  FetchLike,
  FetchOptions,
  LinkupClient,
  LinkupClientOptions,
  LinkupFetchRequestBody,
  LinkupSearchRequestBody,
  LinkupWebFetchInput,
  LinkupWebSearchInput,
  PostJsonOptions,
  SearchOptions,
};
