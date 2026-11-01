/**
 Host-neutral tool contracts for search and fetch.

 One valibot declaration per tool drives MCP advertisement and validation directly,
 and feeds the pi adapter through `valibot-to-typebox.ts`.
 The parameter schemas stay open objects so ignored keys warn instead of being rejected.

 @module
 */

import { formatSize, } from '@monochromatic-dev/agent-harness-shared-truncate/ts';
import * as v from 'valibot';

import {
  LINKUP_VISIBLE_JSON_MAX_BYTES,
} from './tool-output.ts';

//region Constants

/**
 Public search tool name shared by every host surface.
 */
export const LINKUP_WEB_SEARCH_TOOL_NAME = 'web_search' as const;

/**
 Public fetch tool name shared by every host surface.
 */
export const LINKUP_WEB_FETCH_TOOL_NAME = 'web_fetch' as const;

/**
 Supported search input keys.
 */
export const SEARCH_SUPPORTED_KEYS: readonly string[] = [
  'query',
  'fromDate',
  'includeDomains',
  'toDate',
] as const;

/**
 Supported fetch input keys.
 */
export const FETCH_SUPPORTED_KEYS: readonly string[] = [
  'url',
] as const;

/**
 Search ignored-key fixed behavior text.
 */
export const SEARCH_FIXED_BEHAVIOR: string = 'This extension uses DeepSeek search first with maximum results and page content, Exa fast search next, Linkup standard search as fallback, the configured global blocklist, and no per-search result-count controls.';

/**
 Fetch ignored-key fixed behavior text.
 */
export const FETCH_FIXED_BEHAVIOR: string = 'This extension fetches mapped GitHub URLs with the gh CLI first, then uses DeepSeek fetch, Linkup renderJs fetch, and Exa contents fallback.';

//endregion Constants

//region Schema builder types

/**
 String schema carrying a model-facing description.
 */
export type DescribedString = v.SchemaWithPipe<readonly [
  v.StringSchema<undefined>,
  v.DescriptionAction<string, string>,
]>;

/**
 String array schema carrying a model-facing description.
 */
export type DescribedStringArray = v.SchemaWithPipe<readonly [
  v.ArraySchema<v.StringSchema<undefined>, undefined>,
  v.DescriptionAction<string[], string>,
]>;

/**
 Optional entry wrapper preserving the wrapped schema type.
 */
export type OptionalEntry<Wrapped extends v.GenericSchema> = v.OptionalSchema<Wrapped, undefined>;

//endregion Schema builder types

//region Schemas

/**
 Model-facing search parameter schema, open so ignored keys reach the warning path.
 */
export const LinkupWebSearchParametersSchema: v.ObjectSchema<{
  readonly query: DescribedString;
  readonly fromDate: OptionalEntry<DescribedString>;
  readonly includeDomains: OptionalEntry<DescribedStringArray>;
  readonly toDate: OptionalEntry<DescribedString>;
}, undefined> = v.object({
  query: v.pipe(
    v.string(),
    v.description('Search query sent to DeepSeek first, then Exa and Linkup fallback. DeepSeek receives up to 50 results with page content. Be specific and include names, dates, versions, or locations when relevant.'),
  ),
  fromDate: v.optional(v.pipe(
    v.string(),
    v.description('Optional ISO date forwarded to providers as a start date; DeepSeek receives an after: operator when it fits the query limit and results are filtered locally.'),
  )),
  includeDomains: v.optional(v.pipe(
    v.array(v.string()),
    v.description('Optional domain allow-list forwarded to Exa and Linkup as includeDomains, and enforced on DeepSeek results with site: operators or a local filter.'),
  )),
  toDate: v.optional(v.pipe(
    v.string(),
    v.description('Optional ISO date forwarded to providers as an end date; DeepSeek receives a before: operator when it fits the query limit and results are filtered locally.'),
  )),
});

/**
 Model-facing fetch parameter schema, open so ignored keys reach the warning path.
 */
export const LinkupWebFetchParametersSchema: v.ObjectSchema<{
  readonly url: DescribedString;
}, undefined> = v.object({
  url: v.pipe(
    v.string(),
    v.description('Absolute URL to fetch. Mapped GitHub URLs are fetched with the gh CLI; every other URL uses DeepSeek first, then Linkup and Exa fallback.'),
  ),
});

//endregion Schemas

//region Public types

/**
 Runtime search params accepted after schema validation.
 */
export type LinkupWebSearchParams = v.InferOutput<typeof LinkupWebSearchParametersSchema>;

/**
 Runtime fetch params accepted after schema validation.
 */
export type LinkupWebFetchParams = v.InferOutput<typeof LinkupWebFetchParametersSchema>;

/**
 Host-neutral tool contract every adapter renders from.
 */
export type SearchFetchToolSpec<Parameters> = {
  /**
   Tool name shared across hosts.
   */
  readonly name: 'web_search' | 'web_fetch';
  /**
   Short label hosts may render in transcripts.
   */
  readonly label: string;
  /**
   Model-facing tool description.
   */
  readonly description: string;
  /**
   One-line prompt guidance hosts may surface.
   */
  readonly promptSnippet: string;
  /**
   Model-facing usage rules attached to the tool.
   */
  readonly promptGuidelines: readonly string[];
  /**
   Valibot parameter schema, the single source for both hosts.
   */
  readonly parameters: Parameters;
  /**
   Input keys this tool version honors.
   */
  readonly supportedKeys: readonly string[];
  /**
   Fixed behavior text quoted when keys are ignored.
   */
  readonly fixedBehavior: string;
  /**
   Whether accepted search result envelopes render as JSONL.
   */
  readonly renderResultsArrayAsJsonl: boolean;
};

//endregion Public types

//region Tool specs

/**
 Host-neutral search tool contract.
 */
export const webSearchToolSpec: SearchFetchToolSpec<typeof LinkupWebSearchParametersSchema> = {
  name: LINKUP_WEB_SEARCH_TOOL_NAME,
  label: 'Web Search',
  description: `Search the web with DeepSeek search first (up to 50 results with page content), Exa fast search next, and Linkup standard search fallback. Uses the configured global blocklist. Filters are sent to DeepSeek as query operators when they fit and applied locally otherwise. Exact {"results":[...]} responses, plus exact metadata envelopes with requestId, resolvedSearchType, results, searchTime, and costDollars, return object results as JSONL; other output is JSON and may be truncated after ${formatSize(LINKUP_VISIBLE_JSON_MAX_BYTES,)} with a full response temp path.`,
  promptSnippet: 'Search the web with DeepSeek first (maximum results with page content), Exa fast next, Linkup fallback, and the global blocklist.',
  promptGuidelines: [
    'Use web_search to discover sources across the web before fetching a specific page.',
    'web_search requests up to 50 DeepSeek results with page content first, then uses Exa type="fast" and Linkup depth="standard" fallback; do not rely on deep, web-answer, limit, or maxResults controls.',
    'web_search forwards fromDate, toDate, and includeDomains to providers and enforces them locally on DeepSeek results.',
    'web_search applies the configured global blocklist locally after providers respond.',
    'web_search returns object results as JSONL for exact {"results":[...]} responses and exact metadata envelopes with requestId, resolvedSearchType, results, searchTime, and costDollars.',
  ],
  parameters: LinkupWebSearchParametersSchema,
  supportedKeys: SEARCH_SUPPORTED_KEYS,
  fixedBehavior: SEARCH_FIXED_BEHAVIOR,
  renderResultsArrayAsJsonl: true,
};

/**
 Host-neutral fetch tool contract.
 */
export const webFetchToolSpec: SearchFetchToolSpec<typeof LinkupWebFetchParametersSchema> = {
  name: LINKUP_WEB_FETCH_TOOL_NAME,
  label: 'Web Fetch',
  description: `Fetch one page, using the gh CLI for mapped GitHub URLs and DeepSeek fetch first for everything else, with Linkup renderJs and Exa contents fallback. Blocked hosts throw before providers are called. Inline Markdown images backed by base64 data URLs are removed after fetch. Output is raw markdown when the provider returns only a markdown field; otherwise JSON. Model-visible output may be truncated after ${formatSize(LINKUP_VISIBLE_JSON_MAX_BYTES,)} with a full response temp path.`,
  promptSnippet: 'Fetch a known URL with the gh CLI for mapped GitHub URLs, DeepSeek fetch next, Linkup renderJs then Exa fallback, and the global blocklist preflight.',
  promptGuidelines: [
    'Use web_fetch when the URL is already known and the goal is to read page content.',
    'web_fetch routes github.com, gist.github.com, api.github.com, and raw.githubusercontent.com URLs for repositories, files, directories, issues, pull requests, pull request diffs, commits, comparisons, releases, gists, and REST endpoints through the local gh CLI.',
    'web_fetch falls back to DeepSeek fetch, then Linkup renderJs=true, then Exa contents for every other URL, for GitHub pages with no gh mapping, and when gh is missing, unauthenticated, or fails; unsupported fetch knobs are ignored with a warning.',
    'web_fetch returns raw markdown when the provider responds with only a markdown field; otherwise it returns JSON.',
    'web_fetch refuses configured blocked hosts before any provider network request is made.',
  ],
  parameters: LinkupWebFetchParametersSchema,
  supportedKeys: FETCH_SUPPORTED_KEYS,
  fixedBehavior: FETCH_FIXED_BEHAVIOR,
  renderResultsArrayAsJsonl: false,
};

//endregion Tool specs
