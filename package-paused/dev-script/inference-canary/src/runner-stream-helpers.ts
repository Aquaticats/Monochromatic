/**
 * Streaming helper types and utilities for the OpenAI SDK streaming wrapper.
 *
 * Contains the {@link PartialCompletionError} class for abort handling,
 * and helper functions for timing display, usage parsing, and result assembly.
 */
import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';

import {
  l,
  tagged,
} from './log.ts';

import type {
  CompletionResult,
  StreamTiming,
  StreamUsage,
} from './runner-types.ts';

//region PartialCompletionError: thrown on stream abort, carries whatever data was collected before cancellation

/**
 * Error thrown when a stream is aborted mid-response.
 * Carries the partial {@link CompletionResult} so callers can persist
 * whatever chunks were received before cancellation.
 *
 * @example
 * ```ts
 * try {
 *   return await streamCompletion(client, messages, config, label, signal);
 * } catch (error) {
 *   if (error instanceof PartialCompletionError) {
 *     console.log('Partial text:', error.partialResult.text);
 *   }
 *   throw error;
 * }
 * ```
 */
export class PartialCompletionError extends Error {
  /**
   * Partial completion data collected before the stream was aborted
   */
  readonly partialResult: CompletionResult;

  /**
   * @param message - human-readable error description
   *
   * @param partialResult - completion data collected before abort
   */
  constructor(
    message: string,
    partialResult: CompletionResult,
  ) {
    super(message,);
    this.name = 'PartialCompletionError';
    this.partialResult = partialResult;
  }
}

//endregion PartialCompletionError

/**
 * Options for {@link logTiming}.
 *
 * @example
 * ```ts
 * const opts: LogTimingOptions = {
 *   probeName: 'sudoku-solver',
 *   timing: streamTiming,
 * };
 * ```
 */
type LogTimingOptions = {
  /**
   * Probe/call label for log prefix
   */
  readonly probeName: string;
  /**
   * Collected timing data
   */
  readonly timing: StreamTiming;
};

/**
 * Logs a timing summary for a streamed response.
 * Only ttfc and total are shown; chunk count and inter-chunk gaps are too granular
 * for routine human inspection and are preserved in the StreamTiming object for callers
 * that need them.
 *
 * @param probeName - probe/call label for log prefix
 *
 * @param timing - collected timing data
 *
 * @example
 * ```ts
 * logTiming({ probeName: 'sudoku-solver', timing });
 * // logs: [timing:sudoku-solver] ttfc=150ms total=3.2s
 * ```
 */
export function logTiming({
  probeName,
  timing,
}: LogTimingOptions,): void {
  /**
   * Timing-specific logger tagged with the call label.
   */
  const rl = tagged({
    tag: `timing:${probeName}`,
    l,
  },);
  /**
   * Total stream duration rendered as seconds with one decimal, suitable for terse human log lines.
   */
  const totalSeconds = (timing.totalMs
    / MS_PER_SECOND).toFixed(1,);
  rl.info(
    `ttfc=${String(timing.timeToFirstChunkMs,)}ms`
      + ` total=${totalSeconds}s`,
  );
}

/**
 * Options for {@link buildStreamUsage}.
 *
 * @example
 * ```ts
 * const opts: BuildStreamUsageOptions = { promptTokens: 10, completionTokens: 20, totalTokens: 30 };
 * ```
 */
type BuildStreamUsageOptions = {
  /**
   * Tokens in the prompt
   */
  readonly promptTokens: number;
  /**
   * Tokens in the generated completion
   */
  readonly completionTokens: number;
  /**
   * Sum of prompt and completion tokens
   */
  readonly totalTokens: number;
  /**
   * Tokens used for internal reasoning, omitted when the model does not report them
   */
  readonly reasoningTokens?: number;
};

/**
 * Assembles a {@link StreamUsage} from already-extracted token counts.
 * Takes primitive fields rather than the SDK's mutable `CompletionUsage` so the
 * parameter stays deeply readonly; callers narrow the SDK shape first.
 *
 * @param promptTokens - tokens in the prompt
 *
 * @param completionTokens - tokens in the generated completion
 *
 * @param totalTokens - sum of prompt and completion tokens
 *
 * @param reasoningTokens - tokens used for internal reasoning, omitted when not reported
 *
 * @returns normalized usage record
 *
 * @example
 * ```ts
 * const usage = buildStreamUsage({ promptTokens: 10, completionTokens: 20, totalTokens: 30 });
 * usage.totalTokens; // 30
 * ```
 */
export function buildStreamUsage({
  promptTokens,
  completionTokens,
  totalTokens,
  reasoningTokens,
}: BuildStreamUsageOptions,): StreamUsage {
  return {
    promptTokens,
    completionTokens,
    totalTokens,
    ...((reasoningTokens !== undefined) ? { reasoningTokens, } : {}),
  };
}

/**
 * Options for {@link buildResult}.
 *
 * @example
 * ```ts
 * const opts: BuildResultOptions = {
 *   chunks: ['hello', ' world'],
 *   reasoningChunks: ['thinking...'],
 *   timing: streamTiming,
 *   usage: streamUsage,
 *   finishReason: 'stop',
 * };
 * ```
 */
type BuildResultOptions = {
  /**
   * Collected content deltas
   */
  readonly chunks: readonly string[];
  /**
   * Collected reasoning deltas
   */
  readonly reasoningChunks: readonly string[];
  /**
   * Computed timing breakdown
   */
  readonly timing: StreamTiming;
  /**
   * Normalized token usage, omitted when the API reported none
   */
  readonly usage?: StreamUsage;
  /**
   * Stop reason from the final chunk, omitted when not reported
   */
  readonly finishReason?: string;
};

/**
 * Builds a {@link CompletionResult} from accumulated stream data.
 *
 * @param chunks - collected content deltas
 *
 * @param reasoningChunks - collected reasoning deltas
 *
 * @param timing - computed timing breakdown
 *
 * @param usage - normalized token usage, omitted when the API reported none
 *
 * @param finishReason - stop reason from the final chunk, omitted when not reported
 *
 * @returns assembled completion result
 *
 * @example
 * ```ts
 * const result = buildResult({ chunks, reasoningChunks, timing, usage, finishReason: 'stop' });
 * result.text; // joined content chunks
 * ```
 */
export function buildResult({
  chunks,
  reasoningChunks,
  timing,
  usage,
  finishReason,
}: BuildResultOptions,): CompletionResult {
  return {
    text: chunks.join('',),
    reasoning: reasoningChunks.join('',),
    timing,
    ...((usage !== undefined) ? { usage, } : {}),
    ...((finishReason !== undefined) ? { finishReason, } : {}),
  };
}
