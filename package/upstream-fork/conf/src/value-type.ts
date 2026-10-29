/**
 JSON value-type gate for stored values.
 
 JSON cannot round-trip `undefined`,
 `symbol`,
 or `function`,
 so writes carrying them are rejected before anything touches disk.
 
 @module
 */

import { UnsupportedValueTypeError, } from './errors.ts';

//region Types

/**
 The `typeof` results JSON cannot represent.
 
 @example
 ```ts
 const type: UnsupportedValueType = 'function';
 ```
 */
export type UnsupportedValueType = 'undefined' | 'symbol' | 'function';

//endregion Types

//region Constants

/**
 Typeof results that can never round-trip through the config file.
 
 @example
 ```ts
 UNSUPPORTED_VALUE_TYPES.has('function'); // => true
 ```
 */
export const UNSUPPORTED_VALUE_TYPES: ReadonlySet<string> = new Set<string>([
  'undefined',
  'symbol',
  'function',
],);

//endregion Constants

//region Validation

/**
 Rejects values JSON cannot store,
 naming the key and the offending type.
 
 @param key - Store key the value is destined for,
 used verbatim in the diagnostic.
 
 @param value - Candidate value produced by the caller.
 
 @throws UnsupportedValueTypeError when `typeof value` is `undefined`,
 `symbol`, or `function`.
 
 @example
 ```ts
 checkValueType({ key: 'theme', value: 'dark', }); // passes
 ```
 */
export function checkValueType({
  key,
  value,
}: {
  readonly key: string;
  readonly value: unknown;
},): void {
  /**
   `typeof` result of the candidate value.
   */
  const type = typeof value;
  if (UNSUPPORTED_VALUE_TYPES.has(type))
    throw new UnsupportedValueTypeError({
      type,
      key,
    },);
}

//endregion Validation
