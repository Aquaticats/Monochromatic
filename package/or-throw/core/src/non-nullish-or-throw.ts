/**
 `nonNullishOrThrow`: assert non-nullish, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

/* oxlint-disable no-restricted-syntax/no-nullish-union -- function's contract is to accept a possibly-nullish input (`null` or `undefined`) and throw; the `undefined` member is the parameter's reason to exist */
/**
 Asserts that a value is not `null` or `undefined`, returning it with a narrowed type.
 
 Replaces the non-null assertion operator (`!`) with a runtime check that throws
 instead of silently producing incorrect behavior when the assumption fails.
 
 @param value - Value to assert as non-nullish
 
 @returns Same value with `null | undefined` removed from the type
 
 @throws Error when value is `null` or `undefined`
 
 @example
 DOM element lookup:
 ```ts
 const el = nonNullishOrThrow(document.querySelector('.my-element',),);
 // el is now Element, not Element | null
 ```
 
 @example
 Optional chaining replacement:
 ```ts
 const home: string = nonNullishOrThrow(process.env['HOME'],);
 // home is string, not string | undefined
 ```
 
 @example
 Regex match groups:
 ```ts
 const match = text.match(/pattern/,);
 const group = nonNullishOrThrow(match?.[1],);
 ```
 */
export function nonNullishOrThrow<T,>(value: T | null | undefined,): T {
  if ((value === null) || (value === undefined))
    throw new Error(`Expected non-nullish value, got ${formatUnknownValue(value,)}`,);
  return value;
}
/* oxlint-enable no-restricted-syntax/no-nullish-union */
