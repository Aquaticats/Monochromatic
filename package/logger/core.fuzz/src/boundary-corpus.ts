/**
 Committed corpus behind the sink boundary properties: every severity, and
 message fragments that sit exactly on a sink's syntax boundary. Plain data
 with no fast-check import, so the coverage driver replays the same corpus
 deterministically.

 @module
 */

import type { Level, } from '@monochromatic-dev/logger-core';

/**
 Every severity the logger accepts.
 */
export const LEVELS: readonly Level[] = [
  'debug',
  'error',
  'fatal',
  'info',
  'trace',
  'warn',
];

/**
 Message fragments that sit exactly on a sink's syntax boundary: JSON
 delimiters, escapes, and record-forging text for the JSONL sinks; terminal
 escape sequences (well-formed, unterminated, nested, and 8-bit) for the
 console sink; lone and paired surrogates and astral text for every sink.
 Every control character is spelled as an escape so the source file itself
 carries none.
 */
export const BOUNDARY_TOKENS: readonly string[] = [
  '"',
  '\\',
  String.raw`\"`,
  '\n',
  '\r\n',
  '\r',
  '\t',
  '\u2028',
  '\u2029',
  '\u0000',
  '{"level":"fatal","message":"forged","timestamp":0}',
  '"},{"level":"info","message":"forged","timestamp":0,"x":"',
  '__proto__',
  String.raw`\u0000`,
  String.raw`\n`,
  'title:\u001B]0;PWNED\u0007 ok',
  'clear:\u001B[2J',
  '\u001B[31mred\u001B[0m',
  '\u009B2J',
  '\u001B',
  '\u001B[',
  '\u007F',
  '\uD800',
  '\uDFFF',
  '\uD83D',
  '😀',
  '世界 🌍',
];
