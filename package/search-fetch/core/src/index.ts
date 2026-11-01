/**
 Public API for \@monochromatic-dev/search-fetch-core.

 Host-neutral search and fetch core: provider fallback, global host blocklist,
 gh URL routing, shared tool specs, and guarded valibot to TypeBox conversion.
 */

//region Client and config

export {
  createLinkupClient,
} from './client.ts';
export {
  loadLinkupConfig,
} from './config.ts';
export {
  configPathForHome,
  legacyConfigPathForHome,
} from './config-paths.ts';
export {
  createExaClient,
  exaForwardableBlocklist,
} from './exa-client.ts';
export {
  DEFAULT_DEEPSEEK_BASE_URL,
  DEEPSEEK_FETCH_ENDPOINT,
  DEEPSEEK_SEARCH_COUNT,
  DEEPSEEK_SEARCH_ENDPOINT,
  DEEPSEEK_SEARCH_NEED_CONTENT,
  createDeepseekClient,
} from './deepseek-client.ts';
export {
  createSearchFetchClient,
} from './search-fetch-client.ts';
export {
  WholeCallDeadlineError,
  WHOLE_CALL_DEADLINE_MILLISECONDS,
  withWholeCallDeadline,
} from './whole-call-deadline.ts';

//endregion Client and config

//region Domain policy and routing

export {
  filterBlockedSearchResults,
  findBlockedHostMatch,
  findBlockedUrlMatch,
  isBlockedHost,
  isBlockedUrl,
  normalizeBlocklist,
  normalizeBlocklistEntry,
  normalizeHostForPolicy,
} from './domain-policy.ts';
export {
  createGhClient,
  GhFetchError,
} from './gh-client.ts';
export {
  createGhCommandRunner,
  GH_DEADLINE_MILLISECONDS,
  runGhCommand,
} from './gh-process.ts';
export { planGitHubFetch, } from './github-url-plan.ts';
export {
  isPrintableAscii,
  validateEndpointFragment,
  validatePositionalPathArgument,
  validateReferenceNumber,
  validateTokenArgument,
} from './github-url-validation.ts';
export {
  filterFetchResponseDataImages,
  filterMarkdownDataImages,
} from './markdown-data-image-filter.ts';

//endregion Domain policy and routing

//region Tool contracts and execution

export { withFileMutationQueue, } from './file-mutation-queue.ts';
export {
  pruneResponseTempDirs,
  RESPONSE_MAX_AGE_MILLISECONDS,
  RESPONSE_MAX_DIR_COUNT,
  RESPONSE_TEMP_DIR_PREFIX,
} from './response-retention.ts';
export {
  createJsonContent,
  createLinkupToolOutput,
  createWarningContent,
  LINKUP_VISIBLE_JSON_MAX_BYTES,
} from './tool-output.ts';
export {
  FETCH_FIXED_BEHAVIOR,
  FETCH_SUPPORTED_KEYS,
  LINKUP_WEB_FETCH_TOOL_NAME,
  LINKUP_WEB_SEARCH_TOOL_NAME,
  LinkupWebFetchParametersSchema,
  LinkupWebSearchParametersSchema,
  SEARCH_FIXED_BEHAVIOR,
  SEARCH_SUPPORTED_KEYS,
  webFetchToolSpec,
  webSearchToolSpec,
} from './tool-spec.ts';
export {
  BlockedUrlError,
  collectIgnoredKeys,
  executeWebFetchTool,
  executeWebSearchTool,
} from './tool-exec.ts';
export {
  valibotToTypeBox,
  ValibotToTypeBoxError,
} from './valibot-to-typebox.ts';

//endregion Tool contracts and execution

//region Types

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
} from './client.ts';
export type {
  LinkupConfig,
  LinkupConfigSource,
  LoadLinkupConfigOptions,
} from './config.ts';
export type {
  BlocklistMatch,
  SearchResultFilterResult,
} from './domain-policy.ts';
export type {
  ExaClient,
  ExaClientOptions,
  ExaContentsRequestBody,
  ExaSearchRequestBody,
} from './exa-client.ts';
export type {
  DeepseekClient,
  DeepseekClientOptions,
  DeepseekFetchRequestBody,
  DeepseekSearchRequestBody,
} from './deepseek-client.ts';
export type {
  GhClient,
  GhClientFetchOptions,
  GhClientOptions,
  GhCommandNotRan,
  GhCommandOutcome,
  GhCommandRan,
  GhCommandRequest,
  GhCommandRunner,
  GhCommandRunnerOptions,
  GhFetchAttempt,
  GhInvocation,
  GhMarkdownResponse,
  GitHubFetchRequestKind,
  GitHubFetchPlan,
  GitHubRefPathSplit,
  PlannedGitHubFetch,
  TokenValidation,
  UnplannedGitHubFetch,
} from './github-fetch-types.ts';
export type {
  FetchResponseDataImageFilterResult,
  MarkdownDataImageFilterResult,
} from './markdown-data-image-filter.ts';
export type {
  ProviderFallback,
  ProviderResponse,
  SearchFetchClient,
  SearchFetchClientOptions,
  SearchFetchProvider,
} from './search-fetch-client.ts';
export type {
  FileMutationQueueInput,
} from './file-mutation-queue.ts';
export type {
  JsonContentResult,
  LinkupToolDetails,
  LinkupToolOutputOptions,
  SearchFetchToolResult,
  TextContentItem,
  WarningContentOptions,
} from './tool-output.ts';
export type {
  LinkupWebFetchParams,
  LinkupWebSearchParams,
  SearchFetchToolSpec,
} from './tool-spec.ts';
export type {
  SearchFetchToolClient,
  SearchFetchToolUpdate,
  SearchFetchToolUpdateCallback,
} from './tool-exec.ts';
export type {
  ValibotSchemaNode,
} from './valibot-to-typebox.ts';

//endregion Types
