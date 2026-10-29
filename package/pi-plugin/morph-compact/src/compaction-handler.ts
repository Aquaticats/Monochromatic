/**
 Handler for the `session_before_compact` event.
 
 Checks preconditions, attempts Morph compaction, and either
 returns the compaction result, cancels the compaction, or
 returns `undefined` to fall through to pi's default summarization.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import type {
  ExtensionContext,
  SessionBeforeCompactEvent,
} from '@earendil-works/pi-coding-agent';
import {
  NO_MORPH_KEY,
  resolveMorphApiKey,
} from './api-key.ts';
import { attemptMorphCompaction, } from './compaction.ts';
import type { MorphBeforeCompactOutcome, } from './types.ts';

/**
 Latches for one-time warnings during a session.
 Set membership replaces a module-root boolean so the module stays free of
 top-level `let` (workspace lint rule).
 */
const warnedFlags = new Set<'missingKey'>();

/**
 Reset the missing-key warning flag for a new session.
 
 @example
 ```typescript
 resetMissingKeyWarning();
 ```
 */
export function resetMissingKeyWarning(): void {
  warnedFlags.delete('missingKey',);
}

/**
 Handle the `session_before_compact` event.
 Checks preconditions, attempts Morph compaction via {@link attemptMorphCompaction}, and either
 returns the compaction result, cancels the compaction, or
 returns `undefined` to fall through to pi's default summarization
 (including when the resolved key is {@link NO_MORPH_KEY}).
 
 Compaction is cancelled (not fallen through) when the session is
 too small to compact: all messages fit within pi's keepRecentTokens
 budget, leaving messagesToSummarize empty. Pi's default summarizer
 produces useless empty "(none)" summaries in this case, so we cancel
 to avoid polluting the session with blank compaction entries.
 
 @param event - the session_before_compact event
 
 @param ctx - extension context with UI and context usage access
 
 @returns compaction result, cancellation, or undefined to fall through
 
 @mutates event - Compaction can retain `event.signal` through a dependent `AbortSignal.any` relation.
 
 @example
 ```typescript
 pi.on("session_before_compact", (event, ctx) => handleBeforeCompact({ event, ctx }));
 ```
 */
export async function handleBeforeCompact({
  event,
  ctx,
}: {
  readonly event: SessionBeforeCompactEvent;
  readonly ctx: ExtensionContext;
},): Promise<MorphBeforeCompactOutcome> {
  /**
   Resolved Morph key; sentinel disables the integration for this event.
   */
  const apiKey = await resolveMorphApiKey();
  if (apiKey === NO_MORPH_KEY) {
    if (!warnedFlags.has('missingKey',)) {
      warnedFlags.add('missingKey',);
      ctx.ui
        .notify(
        'MORPH_API_KEY not set (env or ~/.pi/agent/mcp.json): Morph Compact disabled, using pi default compaction',
        'warning',
        );
    }
    return { kind: 'fallthrough', };
  }

  if (event.preparation
    .isSplitTurn)
    return { kind: 'fallthrough', };

  if (event.signal
    .aborted)
    return { kind: 'fallthrough', };

  /**
   Preparation slices read for the pre-flight emptiness check.
   */
  const {
    messagesToSummarize,
    turnPrefixMessages,
    previousSummary,
  } = event.preparation;

  /**
   True when either pending list has at least one message.
   */
  const hasMessages = (messagesToSummarize.length
    > 0)
    || (turnPrefixMessages.length
      > 0);
  if ((!hasMessages) && (previousSummary === undefined)) {
    ctx.ui
      .notify(
      'Morph Compact: nothing to compact (session too small)',
      'warning',
      );
    return { kind: 'cancel', };
  }

  /**
   Total messages slated for compaction; surfaced in the status notify.
   */
  const msgCount = messagesToSummarize.length
    + turnPrefixMessages
      .length;
  ctx.ui
    .notify(
    `Morph Compact: compressing ${msgCount} messages (${event.preparation
      .tokensBefore
      .toLocaleString()} tokens)...`,
    'info',
    );

  /**
   Current context pressure snapshot; absent when pi cannot report usage.
   */
  const contextUsage = ctx.getContextUsage();

  try {
    /**
     Outcome of the Morph attempt; success surfaces a CompactionResult.
     */
    const attempt = await attemptMorphCompaction({
      event,
      ...((contextUsage !== undefined) ? { contextUsage, } : {}),
      apiKey,
    },);

    if (attempt.kind
      === 'fallback')
      return { kind: 'fallthrough', };

    /**
     Extracted CompactionResult forwarded back to pi after the notify.
     */
    const { result, } = attempt;

    if (!event.signal
      .aborted) {
      /**
       Telemetry block stored on result.details; absent when the API omitted usage.
       */
      const morphUsage = result.details
        ?.morphUsage;
      /**
       Whole-percent reduction reported to the UI; zero when ratio unavailable.
       */
      const reductionPct = ((morphUsage?.compressionRatio
        !== undefined)
        && (morphUsage.compressionRatio
          !== 0))
        ? Math.round(
          (1 - morphUsage
            .compressionRatio) * 100,
        )
        : 0;
      /**
       Input token count rendered with locale separators for the notify.
       */
      const inTokens = morphUsage?.inputTokens
        ?.toLocaleString()
        ?? '?';
      /**
       Output token count rendered with locale separators for the notify.
       */
      const outTokens = morphUsage?.outputTokens
        ?.toLocaleString()
        ?? '?';
      /**
       Processing time string rendered for the notify line.
       */
      const ms = morphUsage?.processingTimeMs
        ?.toLocaleString()
        ?? '?';
      ctx.ui
        .notify(
        `Morph Compact: ${reductionPct}% reduction (${inTokens} → ${outTokens} tokens) in ${ms}ms`,
        'info',
        );
    }

    return {
      kind: 'compaction',
      result,
    };
  }
  catch (error) {
    if (event.signal
      .aborted)
      return { kind: 'fallthrough', };

    /**
     Best-effort diagnostic forwarded into the UI notify body.
     */
    const message = caughtValueText(error,);
    ctx.ui
      .notify(
      `Morph Compact failed: ${message}; falling back to pi default`,
      'error',
      );
    return { kind: 'fallthrough', };
  }
}
