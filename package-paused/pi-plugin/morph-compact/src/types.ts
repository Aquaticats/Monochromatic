/**
 Types for Morph Compact pi extension.
 */

import type { CompactionResult, } from '@earendil-works/pi-coding-agent';

/**
 Metadata stored in the CompactionEntry details field.
 */
export type MorphCompactionDetails = {
  backend: 'morph';
  version: 1;
  query: string;
  compressionRatio: number;
  morphUsage?: {
    inputTokens?: number;
    outputTokens?: number;
    compressionRatio?: number;
    processingTimeMs?: number;
  };
  compactedLineRanges?: {
    start: number;
    end: number;
  }[];
  readFiles: string[];
  modifiedFiles: string[];
};

/**
 Shape returned when Morph compaction succeeds.
 */
export type MorphCompactionSuccess = {
  kind: 'success';
  result: CompactionResult<MorphCompactionDetails>;
};

/**
 Shape returned when Morph compaction should fall through to pi default.
 */
export type MorphCompactionFallback = {
  kind: 'fallback';
};

/**
 Result of a Morph compaction attempt.
 */
export type MorphCompactionAttempt =
  | MorphCompactionSuccess
    | MorphCompactionFallback;

/**
 Outcome of {@link handleBeforeCompact}, kept as a closed discriminated union
 (no `undefined`) so the bridge can translate it into pi's
 {@link SessionBeforeCompactResult} (`{ compaction }`, `{ cancel }`, or a bare
 `undefined` that lets other extensions keep their result).
 */
export type MorphBeforeCompactOutcome =
  | {
    /**
     Morph produced a compaction result for pi to apply.
     */
    readonly kind: 'compaction';
    /**
     Compaction record forwarded to pi.
     */
    readonly result: CompactionResult<MorphCompactionDetails>;
  }
    | {
    /**
     Session is too small to compact; cancel pi's default summarizer.
     */
      readonly kind: 'cancel';
    }
    | {
    /**
     Defer to pi's default compaction (missing key, abort, or failure).
     */
      readonly kind: 'fallthrough';
    };
