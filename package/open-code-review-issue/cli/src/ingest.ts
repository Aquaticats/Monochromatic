/**
 Structured OpenCodeReview input parsing and normalization.
 
 @module
 */

import { normalizeComment, } from './comment-normalize.ts';
import { InputValidationError, } from './input-validation-error.ts';
import { isRecord, } from './json-record.ts';
import { parseJsonlInput, } from './jsonl.ts';
import type {
  NormalizedFinding,
  NormalizedInput,
} from './model.ts';

/**
 Reads optional resolved head from complete OCR result manifest.
 
 @param result - Validated top-level result record.
 
 @returns Resolved head string when available.
 
 @example
 ```ts
 readResolvedHead({ manifest: { input: { resolved_head: 'abc' } } }); // 'abc'
 ```
 */
function readResolvedHeadMetadata(
  result: Readonly<Record<string, unknown>>,
): { readonly resolvedHead?: string; } {
  if ((!isRecord(result.manifest,))
    || (!isRecord(result.manifest
      .input,))
    || ((typeof result.manifest
      .input
      .resolved_head) !== 'string')
    || (result.manifest
      .input
      .resolved_head
      === ''))
    {
    return {};
  }
  return {
    resolvedHead: result.manifest
      .input
      .resolved_head,
  };
}

/**
 Normalizes one top-level comment array.
 
 @param comments - Untrusted OCR comment values.
 
 @returns Findings with one-based record positions.
 
 @example
 ```ts
 normalizeComments([]); // []
 ```
 */
function normalizeComments(comments: readonly unknown[],): readonly NormalizedFinding[] {
  return comments.map(function normalizeEnvelopeComment(
    value,
    index,
  ): NormalizedFinding {
    return normalizeComment({
      value,
      position: {
        kind: 'record',
        value: index + 1,
      },
    },);
  },);
}

/**
 Converts a parsed JSON document into supported result or comment envelope.
 
 @param parsed - Complete parsed JSON document.
 
 @returns Normalized adapter input.
 
 @throws {@link InputValidationError} when document is unsupported.
 
 @example
 ```ts
 parseJsonDocument([]); // comments envelope
 ```
 */
function parseJsonDocument(parsed: unknown,): NormalizedInput {
  if (Array.isArray(parsed,)) {
    return {
      inputKind: 'comments',
      findings: normalizeComments(parsed,),
    };
  }
  if ((!isRecord(parsed,))
    || ((typeof parsed.status) !== 'string')
    || (!Array.isArray(parsed.comments,)))
    {
    throw new InputValidationError('input is not a complete OCR result or comment array',);
  }
  /**
   Optional head provenance copied only from recognized manifest path.
   */
  const resolvedHeadMetadata = readResolvedHeadMetadata(parsed,);
  return {
    inputKind: 'result',
    ...resolvedHeadMetadata,
    findings: normalizeComments(parsed.comments,),
  };
}

/**
 Parses one supported OCR structured input envelope atomically.
 
 @param text - Complete JSON or JSONL text supplied by trusted transport boundary.
 
 @returns Normalized findings and available head provenance.
 
 @throws {@link InputValidationError} when JSON or recognized schema is invalid.
 
 @example
 ```ts
 parseStructuredInput({ text: '{"status":"complete","comments":[]}' });
 ```
 */
export function parseStructuredInput({ text, }: { readonly text: string; },): NormalizedInput {
  try {
    /**
     Complete JSON document before envelope narrowing.
     */
    const parsed: unknown = JSON.parse(text,);
    if (isRecord(parsed,) && ((typeof parsed.type) === 'string')) {
      return parseJsonlInput({ text, },);
    }
    return parseJsonDocument(parsed,);
  }
  catch (error: unknown) {
    if (error instanceof InputValidationError) {
      throw error;
    }
    return parseJsonlInput({ text, },);
  }
}
