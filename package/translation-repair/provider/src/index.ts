//region Public index
// The names `@monochromatic-dev/translation-repair-provider` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export * from './index-direct.ts';
export * from './index-hold.ts';
export * from './index-neuralwatt.ts';
export * from './index-reading.ts';
export { extractAnthropicCompletion, } from './anthropic-completion.ts';
export {
  type AnthropicContentBlock,
  type AnthropicImageSource,
  contentBlocksFor,
  MalformedImageUriError,
  readImageSource,
} from './anthropic-content.ts';
export {
  type AnthropicMessage,
  type AnthropicRequestBody,
  type AnthropicToolChoice,
  buildAnthropicBody,
  EmptyConversationError,
  speakingTurns,
  systemTextOf,
} from './anthropic-request.ts';
export {
  answerToolDefinition,
  answerToolName,
  type AnthropicToolDefinition,
  type ReadableResponseFormat,
  renderToolSystemPrompt,
  UnnameableToolError,
} from './anthropic-tool.ts';
export {
  requireWholeAnthropicMessage,
  StreamErrorEventError,
} from './anthropic-whole-message.ts';
export { ledgerAtBound, } from './bedrock-bound-ledger.ts';
export { appendLedgerLine, } from './ledger-line-append.ts';
export {
  BEDROCK_AUTH_HEADER,
  BEDROCK_MANTLE_BASE_URL,
  BEDROCK_MODELS,
  BEDROCK_ROUTE_PREFIX,
  bedrockChatUrlFor,
  type BedrockModelInfo,
} from './bedrock-catalog.ts';
export {
  BEDROCK_PER_MODEL_CONCURRENCY,
  type BedrockClient,
  BedrockModelNotServedError,
  createBedrockClient,
} from './bedrock-client.ts';
export {
  BEDROCK_COST_UNREPORTED,
  bedrockAttemptBound,
  bedrockCostOf,
} from './bedrock-cost.ts';
export {
  BEDROCK_CREDIT_USD,
  BEDROCK_CREDIT_USD_VAR,
  BEDROCK_LEDGER_PATH_VAR,
  BedrockCreditOverrideError,
  type BedrockCredits,
  bedrockCreditUsdFrom,
  type BedrockLedger,
  type BedrockLedgerEntry,
  bedrockLedgerFromEnv,
  bedrockLedgerPathFrom,
  BedrockLedgerShapeError,
  createBedrockLedger,
  defaultBedrockLedgerPath,
} from './bedrock-ledger.ts';
export {
  requireBedrockStreamEnd,
  withDoneSentinel,
} from './bedrock-stream-end.ts';
export {
  HOLD_POLL_MS,
  NOBODY_REFUSED,
  readBudgetsPastHolds,
  shortestHold,
  waitOutHold,
} from './budget-hold-wait.ts';
export {
  BEDROCK_DRY_MARGIN_USD,
  bedrockIsDry,
  bedrockMeterLevel,
  EveryProviderDryError,
  hyperIsDry,
  hyperMeterLevel,
  type ModelReach,
  NO_PROVIDER,
  openRouterIsDry,
  openRouterMeterLevel,
  type ProviderChoice,
  providerServing,
  routeProviderFor,
  syntheticIsDry,
  syntheticMeterLevel,
} from './budget-routing.ts';
export {
  armCallDeadline,
  type CallDeadline,
} from './call-deadline.ts';
export {
  type ChannelMarkerStrip,
  stripChannelMarker,
} from './channel-marker.ts';
export {
  carriesPicture,
  type ChatJsonOutcome,
  type ChatJsonRequest,
  type ChatTextReply,
  type ChatTextRequest,
  type JsonSchemaResponseFormat,
  messageText,
  type ModelCaller,
  type SchemaMismatchReason,
  type ModelClient,
  type VisionMessage,
} from './chat-contract.ts';
export { isDecisionStateOverContext, } from './decision-context-refusal.ts';
export {
  type Decider,
  type DecisionAnswer,
  type DecisionChoiceAnswer,
  type DecisionChoiceQuestion,
  type DecisionNoulAnswer,
  type DecisionNoulQuestion,
  type DecisionQuestion,
  type DecisionReply,
  DecisionReplyShapeError,
  type DecisionRequest,
  type DecisionScoreAnswer,
  type DecisionScoreQuestion,
  type DecisionState,
  isDecisionAnswer,
  readDecisionReplyBody,
} from './decision-contract.ts';
export {
  createHyperClient,
  HYPER_PER_MODEL_CONCURRENCY,
  type HyperClient,
  ModelNotServedError,
} from './hyper-client.ts';
export { reaskElsewhereNudged, } from './nudged-reask.ts';
export {
  type AbandonedSpendEstimate,
  estimateAbandonedSpend,
  rawCharsPerCompletionTokenOf,
  reportAbandonedSpend,
} from './openrouter-abandoned-spend.ts';
export {
  CACHED_UNREPORTED,
  openRouterCachedTokensOf,
} from './openrouter-cached-tokens.ts';
export {
  OPENROUTER_AUTH_HEADER,
  OPENROUTER_CHAT_URL,
  OPENROUTER_CREDITS_URL,
  OPENROUTER_DROPPED_SEATS,
  OPENROUTER_MODELS,
  OPENROUTER_PICTURES_UNMEASURED,
  OPENROUTER_PROVIDER_PREFERENCES,
  OPENROUTER_WITHHELD,
  type OpenRouterModelInfo,
  type OpenRouterProviderPreferences,
  openRouterProviderPreferencesFor,
} from './openrouter-catalog.ts';
export { openRouterChunksOf, } from './openrouter-chunk-scan.ts';
export {
  createOpenRouterClient,
  OPENROUTER_PER_MODEL_CONCURRENCY,
  type OpenRouterClient,
  OpenRouterModelNotServedError,
  reportedSpendFieldsOf,
} from './openrouter-client.ts';
export {
  COST_UNREPORTED,
  openRouterCostOf,
} from './openrouter-cost.ts';
export {
  type OpenRouterCredits,
  OpenRouterCreditsShapeError,
  parseOpenRouterCredits,
} from './openrouter-credits.ts';
export {
  ENDPOINT_UNREPORTED,
  type EndpointReading,
  openRouterEndpointOf,
} from './openrouter-endpoint.ts';
export {
  ERROR_FINISH_ABSENT,
  type ErrorFinishReading,
  openRouterErrorFinishOf,
} from './openrouter-error-finish.ts';
export {
  InStreamProviderError,
  InStreamRefusalError,
  openRouterStreamErrorOf,
  requireNoStreamError,
  STREAM_ERROR_ABSENT,
  type StreamErrorReading,
} from './openrouter-stream-error.ts';
export {
  isBudgetRefusal,
  isCallSizedRefusal,
  isPaymentRefusal,
  isUpstreamModelRefusal,
  statedWaitMsOf,
} from './provider-budget-refusal.ts';
export {
  type BudgetView,
  createProviderBudgets,
  type ProviderBudgets,
  type ProviderBudgetsWithStates,
} from './provider-budget.ts';
export {
  type MeterLevel,
  type MeterRecord,
  meterRecordOf,
  type MeterState,
  readEveryMeter,
  routesAsDry,
  UNCONFIGURED_METER,
} from './provider-meters.ts';
export {
  type RoutedCore,
  routedJson,
  secondOpinionsFrom,
} from './provider-router-reask.ts';
export { type ProviderCallers, } from './provider-callers.ts';
export { createRoutingClient, } from './provider-router.ts';
export {
  createRequestPace,
  HYPER_PACE_WINDOW_MS,
  HYPER_REQUESTS_PER_HOUR,
  HYPER_REQUESTS_PER_HOUR_VAR,
  hyperRequestsPerHour,
  type RequestPace,
} from './request-pace.ts';
export {
  failureForReply,
  SyntheticRequestTooLargeError,
} from './request-size-refusal.ts';
export { retryAfterMsOf, } from './retry-stated-wait.ts';
export {
  bedrockIdFor,
  type BedrockSpelling,
  hyperIdFor,
  type HyperSpelling,
  openRouterIdFor,
  type OpenRouterSpelling,
  reachOf,
  readsImages,
  type SyntheticEntry,
  syntheticEntryFor,
  visionReachOf,
} from './roster-reach.ts';
export {
  renderSchemaForPrompt,
  SCHEMA_BLOCK_HEADING,
  withSchemaInSystemPrompt,
} from './schema-prompt.ts';
export {
  isSpendReckoning,
  reportSpend,
  SPEND_MARKER,
  SPEND_RECKONINGS,
  type SpendReckoning,
} from './spend-line.ts';
export {
  armStreamBound,
  isStreamBoundCut,
  type StreamBound,
  StreamBoundError,
} from './stream-bound.ts';
export { extractStreamedCompletion, } from './stream-completion.ts';
export {
  SYNTHETIC_CHAT_BASE_URL,
  SYNTHETIC_MODELS,
  SYNTHETIC_QUOTAS_URL,
  SYNTHETIC_WITHHELD,
  type SyntheticModelInfo,
  syntheticServes,
} from './synthetic-catalog.ts';
export {
  createSyntheticClient,
  SYNTHETIC_PER_MODEL_CONCURRENCY,
  SyntheticModelNotServedError,
} from './synthetic-client.ts';
export {
  type AbandonedAttempt,
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';
export {
  createUpstreamModelHolds,
  UPSTREAM_MODEL_HOLD_MS,
  type UpstreamModelHolds,
} from './upstream-model-hold.ts';

//endregion Public index
