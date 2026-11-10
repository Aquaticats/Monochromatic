/**
 Narrows an untrusted value to a record whose fields are readable.
 
 @module
 */

import {
  isArray,
  isFunction,
  isObject,
} from '@sindresorhus/is';

/**
 Returns whether a value is a record: an object that is neither `null` nor an
 array nor a function.
 
 The accepted set is the one this repository's hand-written copies accepted
 (`value !== null && typeof value === 'object' && !Array.isArray(value)`), so a
 class instance, `Map`, `Date` or error still passes. `@sindresorhus/is` states
 those three halves as `object`, `array` and `function`, and the function
 exclusion is spelled out because the library's `object` predicate accepts
 functions while the copies refused them.
 
 The narrowed type is readonly: a value that arrived as `unknown` has not been
 validated past "it is a record", so a caller that wants to change a field
 builds a new record instead.
 
 @param value - Candidate value, usually straight from `JSON.parse`, a response
 body or a caught throw.
 
 @returns Whether value is a record the caller may read fields from.
 
 @example
 ```ts
 isRecord({ field: 'value' }); // true
 isRecord([]); // false
 isRecord(new Error('failure')); // true
 ```
 */
export function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return isObject(value,)
    && (!isArray(value,))
    && (!isFunction(value,));
}
