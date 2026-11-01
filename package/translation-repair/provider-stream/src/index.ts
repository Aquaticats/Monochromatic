//region Public index
// The names `@monochromatic-dev/translation-repair-provider-stream` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export { describeAbandon, } from './abandon-kind.ts';
export { deliveredNoContent, } from './abandon-silence.ts';
export { scanAnthropicDeltas, } from './anthropic-delta-scan.ts';
export {
  type ExtractedCompletion,
  MalformedCompletionError,
  readFinishReason,
  readUsage,
  ProviderHttpError,
} from './completion-shape.ts';
export { decodedFormsOf, } from './credential-decoded-forms.ts';
export {
  CREDENTIAL_MARKER,
  credentialsOfHeaders,
  maskCredentials,
  maskCredentialsInCutText,
  MINIMUM_CREDENTIAL_UNITS,
} from './credential-mask.ts';
export { needlesOf, } from './credential-needles.ts';
export { findNeedles, } from './credential-search.ts';
export { decodedViewsOf, } from './credential-views.ts';
export {
  exchangeFailureLogText,
  exchangeFailureText,
} from './exchange-failure-text.ts';
export {
  FALSE_START_WINDOW,
  parseAnswerJson,
  readJsonPastFalseStart,
} from './json-false-start.ts';
export { readMaskedJsonBody, } from './masked-json-body.ts';
export {
  formatUsageNote,
  parseModelJson,
  stripCodeFence,
  stripThinkBlock,
} from './model-content.ts';
export {
  clampQuoteDepth,
  quoteDepth,
  quoteDepthBound,
} from './quote-depth-clamp.ts';
export {
  carriesContent,
  pastQuoteMarkers,
} from './quote-line.ts';
export {
  countQuotedPassages,
  dropsQuotedPassage,
  isQuotedPassages,
  type QuotedPassageCount,
  type QuotedPassages,
  quoteLossRefusalFinding,
} from './quote-preservation.ts';
export { sseCommentOf, } from './sse-comment-line.ts';
export { ssePayloadOf, } from './sse-data-line.ts';
export {
  reportStreamProgress,
  StreamCutShortError,
  type StreamOutcome,
} from './stream-cut.ts';
export {
  type DegenerationDetector,
  type DegenerationVerdict,
  watchForDegeneration,
} from './stream-degeneration.ts';
export { deliveredCharsOf, } from './stream-delivered-chars.ts';
export {
  type ChannelDelta,
  type DeltaScanner,
  scanStreamDeltas,
  type StreamChannel,
} from './stream-delta-scan.ts';
export { drainBody, } from './stream-drain.ts';
export {
  requireStreamBoundsInsideCallTimeout,
  type StreamBoundSetting,
  StreamBoundsOutsideCallTimeoutError,
} from './stream-bound-config.ts';
export {
  armIdleGuard,
  type GeneratedChars,
  type IdleGuard,
  STREAM_FIRST_CONTENT_MS,
  STREAM_IDLE_MS,
  type StreamProgress,
  StreamStalledError,
} from './stream-idle-guard.ts';
export {
  isSelfEndedStream,
  StreamOverrunError,
} from './stream-overrun.ts';
export {
  type RecurrenceDetector,
  type RecurrenceVerdict,
  watchForRecurrence,
} from './stream-recurrence-watch.ts';
export {
  type RunawayVerdict,
  type RunawayWatch,
  StreamDegenerateError,
  watchRunaway,
} from './stream-runaway-watch.ts';
export { capturedHeadersOf, } from './response-header-capture.ts';
export {
  DEFAULT_WIRE_FORMAT,
  scannerFor,
  type StreamWireFormat,
} from './stream-wire-format.ts';
export {
  parseQuotaSnapshot,
  QuotaShapeError,
  type QuotaSnapshot,
} from './synthetic-quota.ts';
export {
  fetchTransport,
  type ModelTransport,
  type TransportExchange,
  type TransportReply,
} from './synthetic-transport.ts';
export { TransportRequestFailedError, } from './transport-request-error.ts';

//endregion Public index
