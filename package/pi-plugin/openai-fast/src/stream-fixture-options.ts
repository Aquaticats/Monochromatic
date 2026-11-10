/**
 Test-only native options exercise forwarding without invoking opaque capabilities. @module
 */
import type {
  OpenAICodexResponsesOptions,
  SimpleStreamOptions,
  StreamOptions,
} from '@earendil-works/pi-ai';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  createStreamFixtureResponse,
  StreamFixtureStopError,
} from './stream-fixture.ts';

//region Capability fixtures: forwarding tests compare identities, never call telemetry or transport.

/**
 Resolve instrumentation without altering request data.
 */
async function fixtureObserver(): Promise<void> {
  await Promise.resolve();
}

/**
 Reject a span start so accidental native telemetry fails locally.

 @throws {@link StreamFixtureStopError} when invoked unexpectedly
 */
async function stopTelemetrySpan(): Promise<never> {
  return await Promise.reject(new StreamFixtureStopError(),);
}

/**
 Reject a span write so accidental native telemetry fails locally.

 @throws {@link StreamFixtureStopError} when invoked unexpectedly
 */
function stopTelemetryWrite(): never {
  throw new StreamFixtureStopError();
}

/**
 Resolve independent local response without network access.

 @returns synthetic completed SSE response
 */
async function fixtureFetch(): Promise<Response> {
  return await Promise.resolve(createStreamFixtureResponse(),);
}

/**
 Create complete neutral options whose object-valued fields have observable identities.

 @param onPayload - caller customization retained through native option conversion

 @returns native simple options with all neutral capabilities exercised

 @mutates onPayload - callback may retain caller-owned captured state when invoked

 @internal
 */
export function createStreamFixtureOptions({
  onPayload,
}: {
  readonly onPayload: ForeignHostCapability<NonNullable<StreamOptions['onPayload']>>;
},): SimpleStreamOptions {
  return {
    apiKey: 'injected-stream-fixture-key',
    temperature: 0.5,
    maxTokens: 8_192,
    samplingParams: { top_p: 0.25, },
    signal: new AbortController().signal,
    /**
     Caller-owned parent span exercises the current native telemetry field by identity.
     */
    telemetry: {
      startSpan: stopTelemetrySpan,
      addLink: stopTelemetryWrite,
      addEvent: stopTelemetryWrite,
      setAttributes: stopTelemetryWrite,
      setStatus: stopTelemetryWrite,
    },
    fetch: fixtureFetch,
    env: { STREAM_FIXTURE_ENV: 'preserved', },
    headers: { 'x-stream-fixture': 'preserved', },
    transport: 'sse',
    timeoutMs: 1_234,
    websocketConnectTimeoutMs: 2_345,
    maxRetries: 3,
    maxRetryDelayMs: 4_567,
    cacheRetention: 'long',
    sessionId: 'stream-fixture-session',
    metadata: { fixture: 'metadata', },
    onPayload,
    onResponse: fixtureObserver,
    onProviderStreamEvent: fixtureObserver,
    toolChoice: 'none',
    reasoning: 'high',
  };
}

/**
 Create full Codex options including fields unavailable to neutral simple options.

 @param onPayload - caller customization snapshotted during priority composition

 @returns native full options with provider-specific request intent

 @mutates onPayload - callback may retain caller-owned captured state when invoked

 @internal
 */
export function createStreamFixtureFullOptions({
  onPayload,
}: {
  readonly onPayload: ForeignHostCapability<NonNullable<StreamOptions['onPayload']>>;
},): OpenAICodexResponsesOptions {
  return {
    ...createStreamFixtureOptions({ onPayload, },),
    transport: 'websocket-cached',
    headers: {
      'x-stream-fixture-runtime': 'preserved',
      'x-remove': null,
    },
    reasoningEffort: 'none',
    reasoningSummary: 'detailed',
    textVerbosity: 'high',
    toolChoice: 'required',
    serviceTier: 'flex',
  };
}

//endregion Capability fixtures
