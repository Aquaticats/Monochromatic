/**
 * Result types for the canary runner.
 *
 * Configuration types and defaults live in runner-config.ts.
 */
import type OpenAI from 'openai';
import type { Probe, } from './probes.ts';

//region OpenAI client view: narrow readonly surface this package actually calls

/**
 * Minimal readonly view of the OpenAI client exposing only the streaming
 * `chat.completions.create` method this package uses.
 *
 * The full `OpenAI` class is deeply mutable and is not in the
 * `prefer-readonly-parameter-types` allow-list (that list lives in another
 * package and cannot be edited here), so passing `OpenAI` directly trips the
 * rule. Declaring the narrow shape keeps client parameters deeply readonly
 * while a real `OpenAI` instance remains structurally assignable to it.
 */
export type ChatClient = {
  readonly chat: {
    readonly completions: {
      readonly create: OpenAI['chat']['completions']['create'];
    };
  };
};

//endregion OpenAI client view

//region Shared branded types: template-literal types used across runner, models, server-time, and artifacts

/**
 * ISO 8601 timestamp string, e.g. "2026-02-23T12:00:00.000Z".
 * Template literal narrows the type to strings starting with a year segment,
 * catching obvious non-timestamp values at compile time.
 */
export type ISOTimestamp = `${number}-${string}`;

/**
 * OpenRouter model ID in provider/name format, e.g. "anthropic/claude-sonnet-4.6".
 * Template literal enforces the slash separator, preventing bare model names.
 */
export type OpenRouterModelId = `${string}/${string}`;

//endregion Shared branded types

export type {
  RunnerConfig,
  VerbosityLevel,
} from './runner-config.ts';
export { defaultConfig, } from './runner-config.ts';

//region Message and timing types: chat message shape and streaming timing breakdown used by runner-stream.ts

export type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

/**
 * Timing breakdown for a single API call
 */
export type StreamTiming = {
  /**
   * Milliseconds from request start to first chunk arriving
   */
  readonly timeToFirstChunkMs: number;
  /**
   * Milliseconds between consecutive chunks (for diagnosing stalls)
   */
  readonly interChunkMs: readonly number[];
  /**
   * Total wall-clock milliseconds for the full response
   */
  readonly totalMs: number;
  /**
   * Number of chunks received
   */
  readonly chunkCount: number;
};

/**
 * Token usage from a streaming chat completion.
 * Populated when the API returns usage data (requires `stream_options.include_usage`).
 */
export type StreamUsage = {
  /**
   * Tokens in the prompt
   */
  readonly promptTokens: number;
  /**
   * Tokens in the generated completion (includes reasoning tokens)
   */
  readonly completionTokens: number;
  /**
   * Tokens used for internal reasoning, absent when the model does not report them
   */
  readonly reasoningTokens?: number;
  /**
   * Sum of prompt and completion tokens
   */
  readonly totalTokens: number;
};

/**
 * Full result from a streaming chat completion call.
 * Captures everything the API returns: text, reasoning trace, timing, usage, and stop reason.
 */
export type CompletionResult = {
  /**
   * Concatenated content deltas (the "visible" response)
   */
  readonly text: string;
  /**
   * Concatenated reasoning/thinking deltas, empty string when the model produced none
   */
  readonly reasoning: string;
  /**
   * Per-chunk timing breakdown
   */
  readonly timing: StreamTiming;
  /**
   * Token usage, absent when the API did not include usage data
   */
  readonly usage?: StreamUsage;
  /**
   * Why generation stopped (e.g. "stop", "length"), absent when not reported
   */
  readonly finishReason?: string;
};

/**
 * Snapshot of runner configuration persisted alongside artifacts and history entries.
 * Captures the settings that affect model output so results can be reproduced.
 */
export type ConfigSnapshot = {
  readonly verbosity: string;
  readonly reasoning: boolean;
  readonly maxTokens: number;
  readonly consistencyRuns: number;
};

//endregion Message and timing types

//region Probe and report result types: ProbeResult (per-probe) and CanaryReport (per-model) returned by runCanary

/**
 * Result of a single probe execution
 */
export type ProbeResult = {
  readonly name: string;
  readonly category: Probe['category'];
  /**
   * Scores from each consistency run (length = consistencyRuns)
   */
  readonly scores: readonly number[];
  /**
   * Mean score across all runs
   */
  readonly meanScore: number;
  /**
   * Whether all runs agreed (high consistency = reliable signal)
   */
  readonly consistent: boolean;
  /**
   * Score after a second pass where the model gets its code + diagnostics and fixes them.
   * Undefined if the probe doesn't support it or the first pass had no diagnostics.
   */
  readonly pass2Score?: number;
  /**
   * Improvement from pass 1 to pass 2 (pass2Score - meanScore).
   * Positive = model improved; zero/negative = degradation signal.
   */
  readonly fixDelta?: number;
  /**
   * Set to true when the probe timed out; score is forced to 0 rather than
   * failing the whole model so partial results can still be recorded in history.
   */
  readonly timedOut?: boolean;
  /**
   * Timing from the last consistency run
   */
  readonly timing?: StreamTiming;
  /**
   * Token usage from the last consistency run
   */
  readonly usage?: StreamUsage;
};

/**
 * Aggregate report across all probes
 */
export type CanaryReport = {
  readonly model: OpenRouterModelId;
  /**
   * Human-readable model label for display and grouping
   */
  readonly label: string;
  readonly timestamp: ISOTimestamp;
  readonly results: readonly ProbeResult[];
  /**
   * Overall health score 0-1
   */
  readonly overallScore: number;
  /**
   * Per-category scores for targeted diagnosis
   */
  readonly categoryScores: Readonly<Record<string, number>>;
  /**
   * Whether this run failed entirely (API error, timeout, etc.)
   */
  readonly failed: boolean;
  /**
   * Error message if failed
   */
  readonly error?: string;
};

//endregion Probe and report result types
