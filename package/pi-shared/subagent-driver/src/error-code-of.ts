/**
 The code a failure carries, when it carries one, so a caller can tell a missing path from a real failure without reading the
 message a program chose to print.

 @module
 */

/**
 Reads the code a failure carries.

 @param error - what was caught

 @returns the code, or empty text when the failure carries none

 @example
 ```ts
 const code = errorCodeOf({ code: 'ENOENT', },);
 ```
 */
export function errorCodeOf(error: unknown,): string {
  if (((typeof error) !== 'object') || (error === null)
    || (!('code' in error))) {
    return '';
  }
  return ((typeof error.code) === 'string') ? error.code : '';
}
