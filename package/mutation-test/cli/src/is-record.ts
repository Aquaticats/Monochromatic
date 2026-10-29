/**
 JSON record type guard shared by manifest and report validation.
 
 @example
 ```ts
 isRecord({});
 // true
 ```
 */

/**
 Returns whether a value is a non-null, non-array object record.
 
 @param value - Candidate value.
 
 @returns Whether value is a JSON-like record.
 
 @example
 ```ts
 isRecord([]);
 // false
 ```
 */
export function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}
