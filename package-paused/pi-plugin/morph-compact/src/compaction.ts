/**
 Morph Compact API client and compression ratio logic.
 */

import type {
  CompactionResult,
  ContextUsage,
  SessionBeforeCompactEvent,
} from '@earendil-works/pi-coding-agent';
import {
  computeFileLists,
  formatFileOperations,
} from './file-tracking.ts';
import {
  buildMorphInput,
  extractLatestQuery,
  wrapMorphOutput,
} from './formatting.ts';
import { createMorphCompactClient, } from './morph-client.ts';
import {
  convertToLlm,
  serializeConversation,
} from './pi-utils.ts';
import type {
  MorphCompactionAttempt,
  MorphCompactionDetails,
} from './types.ts';

//region Compression ratio constants

/**
 Maximum query length sent to Morph (avoids oversized prompts).
 */
const MAX_QUERY_LENGTH = 500;

/**
 Default compaction timeout in milliseconds.
 */
const COMPACTION_TIMEOUT_MS = 120_000;

/**
 Compression ratio when context pressure is critical (\>80% window).
 
 Ratios are higher than traditional summarization because Morph Compact
 is deletion-based at 33K tok/s; re-triggering compaction is fast enough
 that aggressive pruning is unnecessary. Preserving more context per cycle
 reduces drift between compaction rounds, keeping the model's working
 memory closer to the full conversation.
 */
const RATIO_CRITICAL = 0.3;

/**
 Compression ratio when context pressure is high (\>60% window).
 
 See {@link RATIO_CRITICAL} for rationale on higher ratios.
 */
const RATIO_HIGH = 0.4;

/**
 Compression ratio when context pressure is moderate.
 
 See {@link RATIO_CRITICAL} for rationale on higher ratios.
 */
const RATIO_MODERATE = 0.5;

/**
 Context usage threshold for critical compression.
 */
const THRESHOLD_CRITICAL = 0.8;

/**
 Context usage threshold for high compression.
 */
const THRESHOLD_HIGH = 0.6;

//endregion

//region Compression ratio selection

/**
 Choose compression ratio based on context pressure.
 Higher pressure means more aggressive deletion.
 
 @param contextUsage - current context token usage and window size
 
 @returns compression ratio between 0.3 and 0.5
 
 @example
 ```typescript
 chooseCompressionRatio({ tokens: 90000, contextWindow: 100000 })
 // Returns 0.3 (critical pressure)
 ```
 */
export function chooseCompressionRatio(
  contextUsage?: Readonly<Pick<ContextUsage, 'tokens' | 'contextWindow'>>,
): number {
  if ((contextUsage === undefined) || (contextUsage.tokens
    === null))
    return RATIO_HIGH;
  /**
   Pressure proxy chosen for adaptive ratio selection.
   */
  const fraction = contextUsage.tokens
    / contextUsage
      .contextWindow;
  if (fraction > THRESHOLD_CRITICAL)
    return RATIO_CRITICAL;
  if (fraction > THRESHOLD_HIGH)
    return RATIO_HIGH;
  return RATIO_MODERATE;
}

//endregion

//region Compaction attempt

/**
 Attempt Morph Compact compaction.
 Returns `{ kind: "success", result }` on success, or `{ kind: "fallback" }`
 when pi's default compaction should be used instead.
 
 @param options - Compaction event, usage snapshot, and Morph credential.
 
 @returns compaction attempt result
 
 @throws {@link MorphApiError} when the Morph Compact API call fails
 
 @mutates options - DOM commit 5796f716 AbortSignal.any dependent-signal relations can retain `options.event.signal`.
 
 @example
 ```typescript
 const attempt = await attemptMorphCompaction({
   event,
   contextUsage: ctx.getContextUsage(),
   apiKey,
 });
 if (attempt.kind === "success") {
   return { compaction: attempt.result };
 }
 ```
 */
export async function attemptMorphCompaction(
  options: {
    readonly event: SessionBeforeCompactEvent;
    readonly contextUsage?: Readonly<ContextUsage>;
    readonly apiKey: string;
  },
): Promise<MorphCompactionAttempt> {
  /**
   Compaction inputs named after the effect-bearing options boundary.
   */
  const {
    event,
    contextUsage,
    apiKey,
  } = options;
  /**
   Destructured event surface used throughout the attempt body.
   */
  const {
    preparation,
    branchEntries,
    customInstructions,
    signal,
  } = event;
  /**
   Preparation slice carries the message ranges and prior summary.
   */
  const {
    messagesToSummarize,
    turnPrefixMessages,
    previousSummary,
    tokensBefore,
    firstKeptEntryId,
    fileOps,
  } = preparation;

  /**
   Combined message list fed to Morph; order reflects branch order.
   */
  const allMessages = [
    ...messagesToSummarize,
    ...turnPrefixMessages,
  ];

  // When there are no new messages but a previous summary exists,
  // Morph can still re-compress the previous summary to save space.
  // The index.ts handler already cancels when both are empty.
  if ((allMessages.length
    === 0) && (previousSummary === undefined))
    return { kind: 'fallback', };

  /**
   Serialized conversation used as Morph input; empty when re-compressing summary alone.
   */
  const conversationText = allMessages.length
    > 0
    ? serializeConversation(convertToLlm(allMessages,),)
    : '';
  /**
   Final prompt body sent to Morph; merges prior summary with new content.
   */
  const input = buildMorphInput({
    serializedConversation: conversationText,
    ...((previousSummary !== undefined) ? { previousSummary, } : {}),
  },);
  if (input.trim()
    === '')
    return { kind: 'fallback', };

  /**
   Latest user intent forwarded to Morph for relevance ranking.
   */
  const query = extractLatestQuery({
    branchEntries,
    ...((customInstructions !== undefined) ? { customInstructions, } : {}),
  },)
    .slice(
    0,
    MAX_QUERY_LENGTH,
    );
  /**
   Adaptive compression ratio derived from current context pressure.
   */
  const ratio = chooseCompressionRatio(contextUsage,);

  if (signal.aborted)
    return { kind: 'fallback', };

  // Combined signal: respects user cancel + hard timeout
  /**
   Cancellation signal merging user abort with hard timeout.
   */
  const combinedSignal = AbortSignal.any([
    signal,
    AbortSignal.timeout(COMPACTION_TIMEOUT_MS,),
  ],);

  /**
   Per-call client constructed with the resolved API key.
   */
  const client = createMorphCompactClient({
    morphApiKey: apiKey,
  },);
  /**
   Network response payload from Morph Compact.
   */
  const result = await client.compact({
    input,
    query,
    compressionRatio: ratio,
    preserveRecent: 0,
    includeMarkers: true,
    includeLineRanges: true,
    signal: combinedSignal,
  },);

  /**
   Trimmed compacted body; empty payload triggers fallback.
   */
  const output = result.output
    ?.trim();
  if ((output === undefined) || (output === ''))
    return { kind: 'fallback', };

  /**
   Read vs modified split appended after Morph's summary.
   */
  const {
    readFiles,
    modifiedFiles,
  } = computeFileLists(fileOps,);
  /**
   Final summary string surfaced to pi as compaction output.
   */
  const summary = `${wrapMorphOutput(output,)}${
    formatFileOperations({
      readFiles,
      modifiedFiles,
    },)
  }`;

  /**
   Optional Morph telemetry rolled into details for the UI panel.
   */
  const morphUsage = result.usage
    !== undefined
    ? {
      inputTokens: result.usage
        .input_tokens,
      outputTokens: result.usage
        .output_tokens,
      compressionRatio: result.usage
        .compression_ratio,
      processingTimeMs: result.usage
        .processing_time_ms,
    }
    : undefined;

  /**
   Backend-specific payload pi stores alongside the summary.
   */
  const details: MorphCompactionDetails = {
    backend: 'morph',
    version: 1,
    query,
    compressionRatio: ratio,
    ...(morphUsage !== undefined ? { morphUsage, } : {}),
    compactedLineRanges: result.messages?.[0]
      ?.compacted_line_ranges
      ?? [],
    readFiles,
    modifiedFiles,
  };

  /**
   Final pi-shaped compaction record returned to the caller.
   */
  const compactionResult: CompactionResult<MorphCompactionDetails> = {
    summary,
    firstKeptEntryId,
    tokensBefore,
    details,
  };

  return {
    kind: 'success',
    result: compactionResult,
  };
}

//endregion
