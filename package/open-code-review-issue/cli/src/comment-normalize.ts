/**
 OCR comment validation and normalization.
 
 @module
 */

import {
  normalizeCategoryMetadata,
  normalizeSeverityMetadata,
} from './finding-vocabulary.ts';
import { InputValidationError, } from './input-validation-error.ts';
import { isRecord, } from './json-record.ts';
import type {
  InputPosition,
  NormalizedFinding,
} from './model.ts';

/**
 Reads mandatory string property from one untrusted record.
 
 @param record - Record carrying candidate property.
 
 @param key - Property whose type is part of accepted OCR schema.
 
 @param positionLabel - Input position for diagnostic evidence.
 
 @returns Validated string value.
 
 @throws {@link InputValidationError} when property is absent or non-string.
 
 @example
 ```ts
 requiredString({ record: { path: 'a.ts' }, key: 'path', positionLabel: 'record 1' });
 ```
 */
function requiredString({
  record,
  key,
  positionLabel,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly positionLabel: string;
},): string {
  /**
   Untrusted property value at current validation boundary.
   */
  const value = record[key];
  if ((typeof value) !== 'string') {
    throw new InputValidationError(`${positionLabel} property ${key} must be a string`,);
  }
  return value;
}

/**
 Reads optional string property with empty-string normalization.
 
 @param record - Record carrying candidate property.
 
 @param key - Optional OCR property.
 
 @param positionLabel - Input position for diagnostic evidence.
 
 @returns Empty string when absent or validated supplied text.
 
 @throws {@link InputValidationError} when supplied value is non-string.
 
 @example
 ```ts
 optionalString({ record: {}, key: 'existing_code', positionLabel: 'record 1' });
 ```
 */
function optionalString({
  record,
  key,
  positionLabel,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly positionLabel: string;
},): string {
  /**
   Untrusted property value at current validation boundary.
   */
  const value = record[key];
  if (value === undefined) {
    return '';
  }
  if ((typeof value) !== 'string') {
    throw new InputValidationError(`${positionLabel} property ${key} must be a string`,);
  }
  return value;
}

/**
 Reads positive integer line property.
 
 @param record - Record carrying line property.
 
 @param key - Start or end line key.
 
 @param positionLabel - Input position for diagnostic evidence.
 
 @returns Validated positive integer.
 
 @throws {@link InputValidationError} when property is not positive integer.
 
 @example
 ```ts
 positiveLine({ record: { start_line: 2 }, key: 'start_line', positionLabel: 'record 1' });
 ```
 */
function positiveLine({
  record,
  key,
  positionLabel,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly positionLabel: string;
},): number {
  /**
   Untrusted property value at current validation boundary.
   */
  const value = record[key];
  if (((typeof value) !== 'number') || (!Number.isInteger(value,))
    || (value < 1)) {
    throw new InputValidationError(`${positionLabel} property ${key} must be a positive integer`,);
  }
  return value;
}

/**
 Selects explicit comment path or inherited JSONL item path.
 
 @param record - Comment carrying optional explicit path.
 
 @param fallbackPath - Item-level path inherited by pathless JSONL comments.
 
 @param positionLabel - Input position for diagnostic evidence.
 
 @returns Non-empty source path.
 
 @throws {@link InputValidationError} when no usable path exists.
 
 @example
 ```ts
 commentPath({ record: { path: '' }, fallbackPath: 'a.ts', positionLabel: 'line 2' });
 ```
 */
function commentPath({
  record,
  fallbackPath,
  positionLabel,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly fallbackPath?: string;
  readonly positionLabel: string;
},): string {
  /**
   Explicit comment path before JSONL inheritance.
   */
  const suppliedPath = requiredString({
    record,
    key: 'path',
    positionLabel,
  });
  /**
   Effective non-empty path after optional inheritance.
   */
  const path = suppliedPath === '' ? fallbackPath ?? '' : suppliedPath;
  if (path.trim() === '') {
    throw new InputValidationError(`${positionLabel} property path must not be empty`,);
  }
  return path;
}

/**
 Detects at least one line containing non-whitespace text.
 
 @param text - Finding text field used as possible title summary source.
 
 @returns Whether title generation can obtain a meaningful line.
 
 @example
 ```ts
 hasNonWhitespaceLine('  \nvalue'); // true
 ```
 */
function hasNonWhitespaceLine(text: string,): boolean {
  return text.split('\n',)
    .some(function lineHasText(line,): boolean {
      return line.trim() !== '';
    },);
}

/**
 Converts one OCR comment to adapter-owned naming and position metadata.
 
 @param value - Untrusted comment value.
 
 @param position - Input position attached to normalized finding.
 
 @param fallbackPath - Item path inherited by a pathless JSONL comment.
 
 @returns Validated normalized finding.
 
 @throws {@link InputValidationError} when comment violates OCR schema.
 
 @example
 ```ts
 normalizeComment({
   value: { path: 'a.ts', content: 'x', start_line: 1, end_line: 1 },
   position: { kind: 'record', value: 1 },
 });
 ```
 */
export function normalizeComment({
  value,
  position,
  fallbackPath,
}: {
  readonly value: unknown;
  readonly position: InputPosition;
  readonly fallbackPath?: string;
},): NormalizedFinding {
  /**
   Human-readable input position for validation diagnostics.
   */
  const positionLabel = `${position.kind} ${String(position.value,)}`;
  if (!isRecord(value,)) {
    throw new InputValidationError(`${positionLabel} must be an object`,);
  }
  /**
   Effective source path after JSONL inheritance.
   */
  const path = commentPath({
    record: value,
    ...(fallbackPath === undefined ? {} : { fallbackPath, }),
    positionLabel,
  });
  /**
   Primary finding prose retained unchanged.
   */
  const content = requiredString({
    record: value,
    key: 'content',
    positionLabel,
  });
  /**
   Existing source text retained unchanged.
   */
  const existingCode = optionalString({
    record: value,
    key: 'existing_code',
    positionLabel,
  });
  /**
   Suggested source text retained unchanged.
   */
  const suggestionCode = optionalString({
    record: value,
    key: 'suggestion_code',
    positionLabel,
  });
  if (![
    content,
    existingCode,
    suggestionCode,
  ].some(hasNonWhitespaceLine,)) {
    throw new InputValidationError(`${positionLabel} must contain a non-whitespace line`,);
  }
  /**
   Inclusive first source line.
   */
  const startLine = positiveLine({
    record: value,
    key: 'start_line',
    positionLabel,
  });
  /**
   Inclusive last source line.
   */
  const endLine = positiveLine({
    record: value,
    key: 'end_line',
    positionLabel,
  });
  if (endLine < startLine) {
    throw new InputValidationError(`${positionLabel} end_line precedes start_line`,);
  }
  /**
   Optional normalized category property.
   */
  const categoryMetadata = normalizeCategoryMetadata({
    record: value,
    positionLabel,
  });
  /**
   Optional normalized severity property.
   */
  const severityMetadata = normalizeSeverityMetadata({
    record: value,
    positionLabel,
  });
  return {
    position,
    path,
    content,
    existingCode,
    suggestionCode,
    startLine,
    endLine,
    ...categoryMetadata,
    ...severityMetadata,
  };
}
