/**
 fast-check arbitraries over log records whose messages attack the sink
 boundaries: JSON delimiters and escapes for the JSONL sinks, terminal
 control sequences for the console sink, and every code-unit class the
 default string arbitrary rarely reaches (DEL, C1 controls, lone
 surrogates).

 The corpus itself lives in `boundary-corpus.ts`, which imports no
 fast-check, so the coverage driver replays it deterministically.

 @module
 */

import type { LogRecord, } from '@monochromatic-dev/logger-core';
import {
  type Arbitrary,
  array,
  constant,
  constantFrom,
  integer,
  nat,
  oneof,
  record,
  string,
} from 'fast-check';

import {
  BOUNDARY_TOKENS,
  LEVELS,
} from './boundary-corpus.ts';

//region Code-unit ranges

/**
 Highest C0 control code unit.
 */
const C0_CONTROL_LAST = 0x1F;

/**
 DEL, the lone control above printable ASCII.
 */
const DELETE_CODE_UNIT = 0x7F;

/**
 First C1 control code unit.
 */
const C1_CONTROL_FIRST = 0x80;

/**
 Last C1 control code unit.
 */
const C1_CONTROL_LAST = 0x9F;

/**
 Largest timestamp `Date` can render as ISO text.
 */
const MAX_TIMESTAMP = 8_640_000_000_000_000;

/**
 Longest run of arbitrary binary text between boundary tokens.
 */
const MAX_CHUNK_LENGTH = 16;

/**
 Most pieces one message is assembled from.
 */
const MAX_PIECES = 8;

/**
 Most records one boundary run writes.
 */
const MAX_RECORDS = 12;

//endregion Code-unit ranges

//region Arbitraries

/**
 One control character drawn uniformly across the three classes the
 console sink must neutralize, so DEL and the C1 range appear as often as
 the C0 range instead of once per million binary code units.

 @returns Single-character string.

 @example
 ```ts
 sample(controlCharacter(), 1); // e.g. ['\u001B']
 ```
 */
function controlCharacter(): Arbitrary<string> {
  return oneof(
    integer({
      min: 0,
      max: C0_CONTROL_LAST,
    },),
    constant(DELETE_CODE_UNIT,),
    integer({
      min: C1_CONTROL_FIRST,
      max: C1_CONTROL_LAST,
    },),
  )
    .map(function toCharacter(codeUnit,): string {
      return String.fromCodePoint(codeUnit,);
    },);
}

/**
 Message assembled from binary text chunks, control characters, and
 boundary tokens in random order, so every adversarial fragment can land
 at the start, the end, or between other fragments.

 @returns Message text.

 @example
 ```ts
 sample(adversarialMessage(), 1); // e.g. ['ab"\u001B[2J\uD800']
 ```
 */
export function adversarialMessage(): Arbitrary<string> {
  return array(
    oneof(
      string({
        unit: 'binary',
        maxLength: MAX_CHUNK_LENGTH,
      },),
      controlCharacter(),
      constantFrom(...BOUNDARY_TOKENS,),
    ),
    { maxLength: MAX_PIECES, },
  )
    .map(function join(pieces: readonly string[],): string {
      return pieces.join('',);
    },);
}

/**
 One log record at any level with an adversarial message and a timestamp
 `Date` can render.

 @returns Record arbitrary.

 @example
 ```ts
 sample(logRecord(), 1);
 ```
 */
export function logRecord(): Arbitrary<LogRecord> {
  return record({
    level: constantFrom(...LEVELS,),
    message: adversarialMessage(),
    timestamp: nat({ max: MAX_TIMESTAMP, },),
  },);
}

/**
 Non-empty record sequences, short enough that a shrunk counterexample
 stays readable and long enough to cross a severity-triggered flush.

 @returns Record list arbitrary.

 @example
 ```ts
 sample(logRecords(), 1);
 ```
 */
export function logRecords(): Arbitrary<readonly LogRecord[]> {
  return array(
    logRecord(),
    {
      minLength: 1,
      maxLength: MAX_RECORDS,
    },
  );
}

//endregion Arbitraries
