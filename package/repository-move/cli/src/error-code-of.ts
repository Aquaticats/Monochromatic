/**
 Reads the system error code out of a caught value without asserting its type: a caught value is `unknown`, and a thrown
 string or object has no code.

 @module
 */

/**
 Reads the code of a system error.

 @param error - the caught value

 @returns the code (such as `ENOENT`), or the empty text for a value without a text code

 @example
 ```ts
 import { errorCodeOf, } from './error-code-of.ts';

 errorCodeOf(new Error('no such file',),);
 // ''
 ```
 */
export function errorCodeOf(error: unknown,): string {
  if (!Error.isError(error,)) {
    return '';
  }
  if (!('code' in error)) {
    return '';
  }
  /**
   The code, which a system error holds as text.
   */
  const { code, } = error;
  return (typeof code) === 'string' ? code : '';
}
