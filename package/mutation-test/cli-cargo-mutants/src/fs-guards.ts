/**
 Check that a value is an object with string keys.

 @param value - parsed JSON or a caught value

 @returns true for a non-null object that is not an array

 @example
 ```ts
 isRecord({});
 ```
 */
export function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value));
}

/**
 Check that an error is a missing-file error.

 @param error - caught value

 @returns true when the error carries the code `ENOENT`

 @example
 ```ts
 isMissing(new Error('x'));
 ```
 */
export function isMissing(error: unknown): boolean {
  return (Error.isError(error,)) && ('code' in error)
    && (error.code === 'ENOENT');
}
