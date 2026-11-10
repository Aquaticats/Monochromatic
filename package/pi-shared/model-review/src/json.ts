/**
 Balanced JSON extraction for direct reviewer retries.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { packageLogger, } from './logger.ts';

/**
 Extraction logger.
 */
const l = tagged({
  tag: 'json',
  l: packageLogger,
},);

//region Constants -- bounded diagnostic text

/**
 Maximum reviewer text copied into malformed-output diagnostics.
 */
const REVIEW_TEXT_DIAGNOSTIC_LIMIT = 200;

//endregion Constants -- bounded diagnostic text

//region Balanced object scan -- string aware slice of reviewer text

/**
 Locate first balanced JSON object while ignoring braces inside strings.
 
 @param text - reviewer output to scan
 
 @returns complete object text or empty string
 
 @example
 ```ts
 findBalancedJsonObject('prefix {"approved":true} suffix');
 ```
 */
function findBalancedJsonObject(text: string,): string {
  /**
   Opening object delimiter.
   */
  const start = text.indexOf('{',);
  if (start === (-1))
    return '';

  /* oxlint-disable no-restricted-syntax/no-function-root-let -- Linear JSON boundary scanner updates constant-space cursor state. */
  /**
   Object nesting depth.
   */
  let depth = 0;
  /**
   Whether cursor is inside JSON string.
   */
  let inString = false;
  /**
   Whether current string character is escaped.
   */
  let escaped = false;
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  for (let cursor = start; cursor < text.length; cursor++) {
    /**
     Character at current scanner cursor.
     */
    const character = text[cursor];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (inString) {
      if (character === '\\')
        escaped = true;
      else if (character === '"')
        inString = false;
      continue;
    }
    if (character === '"') {
      inString = true;
      continue;
    }
    if (character === '{') {
      depth++;
      continue;
    }
    if (character !== '}')
      continue;
    depth--;
    if (depth === 0)
      return text.slice(
        start,
        cursor + 1,
      );
  }
  return '';
}

//endregion Balanced object scan -- string aware slice of reviewer text

//region Public API -- whole-text parse with balanced fallback

/**
 Parse whole reviewer output or first balanced JSON object.
 
 @param text - direct-JSON reviewer output
 
 @returns parsed unknown JSON value
 
 @throws when output contains no parseable object
 
 @example
 ```ts
 extractStructuredJson('{"approved":false,"feedback":"missing test"}');
 ```
 */
function extractStructuredJson(text: string,): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (wholeTextError) {
    // Prose around the object is the expected shape, so the fallback is normal
    // and the failure is recorded at debug level rather than as a warning.
    l.debug(
      `whole-text JSON parse failed (${caughtValueText(wholeTextError,)}); trying first balanced object`,
    );
    /**
     First balanced object candidate.
     */
    const block = findBalancedJsonObject(text,);
    if (block === '') {
      throw new Error(
        `Structured reviewer returned text without JSON object: ${text.slice(
          0,
          REVIEW_TEXT_DIAGNOSTIC_LIMIT,
        )}`,
        { cause: wholeTextError, },
      );
    }
    return JSON.parse(block,);
  }
}

//endregion Public API -- whole-text parse with balanced fallback

export {
  extractStructuredJson,
  findBalancedJsonObject,
};
