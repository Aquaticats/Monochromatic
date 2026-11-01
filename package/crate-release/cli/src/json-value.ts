import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';

/**
 Parse text as JSON without throwing.

 @param text - text under test

 @returns the parsed value, or a miss that carries the parser's complaint when the text is not JSON

 @example
 ```ts
 parseJson('{"a":1}');
 ```
 */
export function parseJson(text: string): Lookup<unknown> {
  try {
    return foundValue(JSON.parse(text) as unknown);
  }
  catch (error) {
    return missingValue(`The text is not JSON: ${caughtValueText(error)}`);
  }
}

/**
 Check that a value is a plain object.

 @param value - value under test

 @returns true for objects other than null and arrays

 @example
 ```ts
 isRecord({});
 ```
 */
export function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value));
}
