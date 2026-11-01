/**
 OCR JSONL event field validation.
 
 @module
 */

import { normalizeComment, } from './comment-normalize.ts';
import { InputValidationError, } from './input-validation-error.ts';
import { isRecord, } from './json-record.ts';
import type { NormalizedFinding, } from './model.ts';

/**
 Reads optional string field from JSONL event.
 
 @param record - Event carrying candidate field.
 
 @param key - Event property to inspect.
 
 @param line - One-based JSONL line number.
 
 @returns Empty string when absent or supplied string.
 
 @throws {@link InputValidationError} when supplied field is not string.
 
 @example
 ```ts
 eventString({ record: {}, key: 'filePath', line: 1 }); // ''
 ```
 */
export function eventString({
  record,
  key,
  line,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly line: number;
},): string {
  /**
   Untrusted optional event property.
   */
  const value = record[key];
  if (value === undefined) {
    return '';
  }
  if ((typeof value) !== 'string') {
    throw new InputValidationError(`line ${String(line,)} property ${key} must be a string`,);
  }
  return value;
}

/**
 Extracts resolved head metadata from session-end manifest when present.
 
 @param record - Session event carrying optional run manifest.
 
 @param line - One-based JSONL line number.
 
 @returns Optional resolved-head property.
 
 @throws {@link InputValidationError} when manifest shape is malformed.
 
 @example
 ```ts
 jsonlResolvedHeadMetadata({ record: {}, line: 1 }); // {}
 ```
 */
export function jsonlResolvedHeadMetadata({
  record,
  line,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly line: number;
},): { readonly resolvedHead?: string; } {
  if (record.run_manifest === undefined) {
    return {};
  }
  if ((!isRecord(record.run_manifest,))
    || (!isRecord(record.run_manifest
      .input,)))
    {
    throw new InputValidationError(`line ${String(line,)} run_manifest.input must be an object`,);
  }
  /**
   Candidate resolved head from frozen run manifest.
   */
  const value = record.run_manifest
    .input
    .resolved_head;
  if ((value === undefined) || (value === '')) {
    return {};
  }
  if ((typeof value) !== 'string') {
    throw new InputValidationError(
      `line ${String(line,)} run_manifest.input.resolved_head must be a string`,
    );
  }
  return { resolvedHead: value, };
}

/**
 Normalizes comments carried by completed or reused checkpoint.
 
 @param record - Checkpoint event.
 
 @param line - One-based JSONL line number.
 
 @returns Findings inheriting event path when comment path is empty.
 
 @throws {@link InputValidationError} when comments or path fields are malformed.
 
 @example
 ```ts
 checkpointFindings({ record: { comments: [] }, line: 2 }); // []
 ```
 */
export function checkpointFindings({
  record,
  line,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly line: number;
},): readonly NormalizedFinding[] {
  if (record.comments === undefined) {
    return [];
  }
  if (!Array.isArray(record.comments,)) {
    throw new InputValidationError(`line ${String(line,)} property comments must be an array`,);
  }
  /**
   Primary item path used by modern session records.
   */
  const filePath = eventString({
    record,
    key: 'filePath',
    line,
  });
  /**
   New path fallback used by renamed-file session records.
   */
  const newPath = eventString({
    record,
    key: 'newPath',
    line,
  });
  /**
   Effective inherited path for pathless comments.
   */
  const fallbackPath = filePath === '' ? newPath : filePath;
  return record.comments
    .map(function normalizeCheckpointComment(value,): NormalizedFinding {
      return normalizeComment({
        value,
        position: {
          kind: 'line',
          value: line,
        },
        fallbackPath,
      },);
    },);
}
