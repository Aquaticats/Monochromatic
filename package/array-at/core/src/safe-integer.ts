/**
 Safe-integer proof type and runtime refinements.
 
 @module
 */

import { ArrayAtError, } from './array-at-error.ts';
import { createNonSafeIntegerDiagnostic, } from './diagnostic-factories.ts';

/**
 Private brand key proving safe-integer validation occurred.
 
 @example
 ```ts
 type Branded = SafeInteger;
 ```
 */
declare const safeIntegerTag: unique symbol;

/**
 Number proven to satisfy `Number.isSafeInteger`.
 
 Safe integers exclude fractions, infinities, `NaN`, and integers whose
 IEEE-754 representation cannot distinguish adjacent integer values.
 
 @example
 ```ts
 const index: SafeInteger = asSafeInteger(value);
 ```
 */
export type SafeInteger = number & {
  readonly [safeIntegerTag]: true;
};

/**
 Reports whether number is safe integer and narrows its type.
 
 @param value - Number requiring safe-integer proof
 
 @returns Whether number has exact safe-integer representation
 
 @example
 ```ts
 if (isSafeInteger(index))
   arrayAt({ array, index, });
 ```
 */
export function isSafeInteger(value: number,): value is SafeInteger {
  return Number.isSafeInteger(value);
}

/**
 Creates proof-helper error without array context.
 
 @param value - Number rejected by safe-integer predicate
 
 @returns Aggregated error containing non-safe-integer diagnostic
 
 @example
 ```ts
 const error = safeIntegerError(1.5);
 ```
 */
function safeIntegerError(value: number,): ArrayAtError {
  return new ArrayAtError({
    diagnostics: [createNonSafeIntegerDiagnostic({ index: value, })],
    index: value,
  },);
}

/**
 Asserts number is safe integer and narrows existing binding.
 
 @param value - Number requiring safe-integer proof
 
 @returns Nothing; successful assertion narrows caller binding
 
 @throws {@link ArrayAtError} when number is not safe integer
 
 @example
 ```ts
 assertSafeInteger(index);
 arrayAt({ array, index, });
 ```
 */
export function assertSafeInteger(value: number,): asserts value is SafeInteger {
  if (!isSafeInteger(value,))
    throw safeIntegerError(value,);
}

/**
 Returns number branded after safe-integer validation.
 
 @param value - Number requiring safe-integer proof
 
 @returns Same number carrying safe-integer proof
 
 @throws {@link ArrayAtError} when number is not safe integer
 
 @example
 ```ts
 const value = arrayAt({ array, index: asSafeInteger(rawIndex), });
 ```
 */
export function asSafeInteger(value: number,): SafeInteger {
  assertSafeInteger(value,);
  return value;
}
