/**
 OCR session JSONL text parsing.
 
 @module
 */

import { InputValidationError, } from './input-validation-error.ts';
import { isRecord, } from './json-record.ts';
import {
  replayJsonlRecords,
  type PositionedJsonlRecord,
} from './jsonl-replay.ts';
import type { NormalizedInput, } from './model.ts';

/**
 Parses one line as untrusted JSON.
 
 @param text - Non-blank JSONL line text.
 
 @param line - One-based physical line number.
 
 @returns Parsed JSON value.
 
 @throws {@link InputValidationError} when text is invalid JSON.
 
 @example
 ```ts
 parseJsonlValue({ text: '{"type":"session_start"}', line: 1 });
 ```
 */
function parseJsonlValue({
  text,
  line,
}: {
  readonly text: string;
  readonly line: number;
},): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error: unknown) {
    throw new InputValidationError(`line ${String(line,)} must be valid JSON: ${String(error,)}`,);
  }
}

/**
 Parses one JSONL line as event record.
 
 @param text - One physical JSONL line without newline delimiter.
 
 @param line - One-based line number.
 
 @returns Validated positioned event record.
 
 @throws {@link InputValidationError} when line is blank, invalid JSON, or non-object.
 
 @example
 ```ts
 parseJsonlRecord({ text: '{"type":"session_start"}', line: 1 });
 ```
 */
function parseJsonlRecord({
  text,
  line,
}: {
  readonly text: string;
  readonly line: number;
},): PositionedJsonlRecord {
  if (text.trim() === '') {
    throw new InputValidationError(`line ${String(line,)} must not be blank`,);
  }
  /**
   Parsed line before event-object narrowing.
   */
  const value = parseJsonlValue({
    text,
    line,
  });
  if ((!isRecord(value,)) || ((typeof value.type) !== 'string')) {
    throw new InputValidationError(`line ${String(line,)} must be an event object with string type`,);
  }
  return {
    record: value,
    line,
  };
}

/**
 Removes one CR suffix from a CRLF-delimited physical line.
 
 @param text - Physical line after LF splitting.
 
 @returns Line text without CR delimiter portion.
 
 @example
 ```ts
 normalizePhysicalLine('x\r'); // 'x'
 ```
 */
function normalizePhysicalLine(text: string,): string {
  return text.endsWith('\r',) ? text.slice(
    0,
    -1,
  ) : text;
}

/**
 Parses and replays OCR JSONL checkpoint semantics.
 
 @param text - Complete JSONL transcript text.
 
 @returns Replayed findings and resolved head provenance.
 
 @throws {@link InputValidationError} when any physical record is malformed.
 
 @example
 ```ts
 parseJsonlInput({ text: '{"type":"session_start"}' });
 ```
 */
export function parseJsonlInput({ text, }: { readonly text: string; },): NormalizedInput {
  /**
   Physical lines including possible terminal empty segment.
   */
  const physicalLines = text.split('\n',);
  /**
   Event lines after permitting one terminal newline.
   */
  const lines = physicalLines.at(-1,) === ''
    ? physicalLines.slice(
      0,
      -1,
    )
    : physicalLines;
  /**
   Parsed event records with source line positions.
   */
  const records = lines.map(function parsePhysicalLine(
    lineText,
    zeroBasedLine,
  ): PositionedJsonlRecord {
    return parseJsonlRecord({
      text: normalizePhysicalLine(lineText,),
      line: zeroBasedLine + 1,
    });
  },);
  return replayJsonlRecords({ records, },);
}
