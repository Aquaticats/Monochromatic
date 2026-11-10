/**
 Finds the first error in a failed check's output and the cause that two reports of the same break share.

 Reads the output line by line with string operations, one pass, so a very long output costs time in proportion to its
 length only.

 @module
 */

import { createHash, } from 'node:crypto';

import { isDigit, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

import { stripAnsi, } from './oxlint-augment.ts';
import type { FirstError, } from './daily-update-package-check-types.ts';

/**
 Answer of {@link firstErrorOf} for output with no line that starts an error.
 */
export const NO_ERROR_IN_OUTPUT: unique symbol = Symbol('the output has no line that starts an error',);

/**
 Answer of {@link errorStartOf} for a line that does not start an error.
 */
export const NOT_AN_ERROR_START: unique symbol = Symbol('the line does not start an error',);

/**
 Lines of an error shown in an issue: the error line and what continues it.
 Chosen so the location and the first note of a compiler error fit; an issue body is limited to 65,536 characters
 and one error is far below that.
 */
export const EXCERPT_LINES = 14;

/**
 Hexadecimal digits of the digest that stands for an error without a code.
 */
const CAUSE_DIGEST_LENGTH = 12;

/**
 What `mise` puts at the start of every line of a task: an opening bracket and two slashes.
 */
const TASK_PREFIX_START = '[//';

/**
 What ends the task prefix.
 */
const TASK_PREFIX_END = ']';

/**
 The line `mise` prints when a task fails starts with this; it ends an error's excerpt.
 */
const TASK_FAILURE_START = 'ERROR ';

/**
 A Rust error with a code starts with this, followed by the code and a closing bracket.
 */
const RUST_CODED_START = 'error[';

/**
 What follows the code of a Rust error.
 */
const RUST_CODED_END = ']: ';

/**
 A Rust error without a code starts with this.
 */
const RUST_PLAIN_START = 'error: ';

/**
 A TypeScript error carries this before its code.
 */
const TYPESCRIPT_MARKER = 'error TS';

/**
 Letters that start a TypeScript error code.
 */
const TYPESCRIPT_CODE_START = 'TS';

/**
 What follows the code of a TypeScript error.
 */
const TYPESCRIPT_CODE_END = ': ';

/**
 Whether a text is made of ASCII digits only and is not empty.

 @param text - the text

 @returns true when every character is a digit

 @example
 ```ts
 isDigits('0433',); // true
 ```
 */
export function isDigits(text: string,): boolean {
  return (text !== '')
    && text.split('',)
      .every(function checkDigit(character,) {
        return isDigit(character,);
      },);
}

/**
 A line read as the start of an error.
 */
export type ErrorStart = {
  /**
   Compiler error code, empty when the error has none.
   */
  readonly code: string;
  /**
   Message without the code.
   */
  readonly message: string;
};

/**
 Removes the prefix `mise` puts before each line of a task, such as `[//package/a/b:lint:clippy] `.

 @param line - one output line

 @returns the line without the prefix

 @example
 ```ts
 withoutTaskPrefix('[//package/a/b:lint:clippy] error: x',); // 'error: x'
 ```
 */
export function withoutTaskPrefix(line: string,): string {
  if (!line.startsWith(TASK_PREFIX_START,))
    return line;
  /**
   Position of the bracket that ends the prefix.
   */
  const end = line.indexOf(TASK_PREFIX_END,);
  return end === (-1) ? line : line.slice(end + TASK_PREFIX_END.length,)
    .trimStart();
}

/**
 Reads one output line as the start of an error.

 @param line - one line without its task prefix

 @returns the code and message, or {@link NOT_AN_ERROR_START} when the line starts no error

 @example
 ```ts
 errorStartOf('error[E0425]: cannot find value `x`',); // { code: 'E0425', message: 'cannot find value `x`', }
 ```
 */
export function errorStartOf(line: string,): ErrorStart | typeof NOT_AN_ERROR_START {
  if (line.startsWith(RUST_CODED_START,)) {
    /**
     Position of the bracket that closes the code.
     */
    const end = line.indexOf(RUST_CODED_END,);
    if (end !== (-1)) {
      return {
        code: line.slice(
          RUST_CODED_START.length,
          end,
        ),
        message: line.slice(end + RUST_CODED_END.length,),
      };
    }
  }
  /**
   Position of the TypeScript marker, which stands at the start of the line or after a space.
   */
  const marker = line.indexOf(TYPESCRIPT_MARKER,);
  if ((marker === 0) || ((marker > 0) && (line.charAt(marker - 1,) === ' '))) {
    /**
     The code and the message after the marker, split at the first separator.
     */
    const after = line.slice((marker + TYPESCRIPT_MARKER.length) - TYPESCRIPT_CODE_START.length,);
    /**
     Position of the separator after the code.
     */
    const separator = after.indexOf(TYPESCRIPT_CODE_END,);
    if ((separator !== (-1)) && isDigits(after.slice(
      TYPESCRIPT_CODE_START.length,
      separator,
    ),)) {
      return {
        code: after.slice(
          0,
          separator,
        ),
        message: after.slice(separator + TYPESCRIPT_CODE_END.length,),
      };
    }
  }
  if (line.startsWith(RUST_PLAIN_START,))
    return {
      code: '',
      message: line.slice(RUST_PLAIN_START.length,),
    };
  return NOT_AN_ERROR_START;
}

/**
 Finds the first error in a check's output.

 @param output - the first part of the output, where the first error is

 @returns the first error, or {@link NO_ERROR_IN_OUTPUT} when no line starts one

 @example
 ```ts
 firstErrorOf('warning: x\n\nerror[E0433]: failed to resolve\n --> src/a.rs:1:1\n',); // code 'E0433'
 ```
 */
export function firstErrorOf(output: string,): FirstError | typeof NO_ERROR_IN_OUTPUT {
  /**
   Lines without colors and task prefixes.
   */
  const lines = stripAnsi(output,)
    .split('\n',)
    .map(withoutTaskPrefix,);
  /**
   Position of the first line that starts an error.
   */
  const start = lines.findIndex(function startsError(line,) {
    return errorStartOf(line,) !== NOT_AN_ERROR_START;
  },);
  if (start === (-1))
    return NO_ERROR_IN_OUTPUT;
  /**
   Code and message of that line.
   */
  const found = errorStartOf(lines[start] ?? '',);
  if (found === NOT_AN_ERROR_START)
    return NO_ERROR_IN_OUTPUT;
  /**
   Lines after the first that continue it: up to a blank line or the next error.
   */
  const continuation: string[] = [];
  for (const line of lines.slice(start + 1,)) {
    if (
      (line.trim() === '') || line.startsWith(TASK_FAILURE_START,)
        || (errorStartOf(line,) !== NOT_AN_ERROR_START)
        || (continuation.length >= (EXCERPT_LINES - 1))
    ) {
      break;
    }
    continuation.push(line,);
  }
  return {
    code: found.code,
    message: found.message,
    excerpt: [
      lines[start] ?? '',
      ...continuation,
    ].join('\n',),
  };
}

/**
 The cause two reports of the same break share: the error code, or for an error without one a digest of its message
 with the digits left out, so a moved line number does not make a new cause.

 @param error - the first error

 @returns the cause

 @example
 ```ts
 causeOf({ code: 'E0433', message: 'x', excerpt: '', },); // 'E0433'
 ```
 */
export function causeOf(error: FirstError,): string {
  if (error.code !== '')
    return error.code;
  return `message-${
    createHash('sha256',)
      .update(error.message
        .split('',)
        .filter(function notDigit(character,) {
        return !isDigits(character,);
      },)
        .join('',),)
      .digest('hex',)
      .slice(
        0,
        CAUSE_DIGEST_LENGTH,
      )
  }`;
}
