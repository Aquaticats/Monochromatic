/**
 Runtime JSON object narrowing.
 
 @module
 */

/**
 Narrows non-null objects for property validation.
 
 @param value - Untrusted JSON value at current validation boundary.
 
 @returns Whether named properties can be inspected safely.
 
 @example
 ```ts
 isRecord({ status: 'complete' }); // true
 ```
 */
export function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}
