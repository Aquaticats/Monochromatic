/**
 Runtime and narrowing tests for safe-integer proof helpers.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ArrayAtError,
  asSafeInteger,
  assertSafeInteger,
  isSafeInteger,
  type SafeInteger,
} from '../dist/final/neutral/index.mjs';

/**
 Doubles value after assertion narrows it to `SafeInteger`.
 
 @param value - Number requiring proof before arithmetic
 
 @returns Doubled safe integer
 
 @throws {@link ArrayAtError} when value is not safe integer
 
 @example
 ```ts
 const answer = doubleSafeInteger(21);
 ```
 */
function doubleSafeInteger(value: number,): number {
  assertSafeInteger(value,);
  expectTypeOf(value,).toEqualTypeOf<SafeInteger>();
  return value * 2;
}

/**
 Captures expected proof-helper error.
 
 @param operation - Failing proof operation
 
 @returns Captured `ArrayAtError`
 
 @throws Unexpected value or absent error
 
 @example
 ```ts
 const error = captureProofError(() => asSafeInteger(1.5));
 ```
 */
function captureProofError(operation: () => unknown,): ArrayAtError {
  try {
    operation();
  }
  catch (error) {
    if (error instanceof ArrayAtError)
      return error;
    throw error;
  }
  throw new Error('Expected safe-integer proof to throw ArrayAtError.',);
}

await describe({
  name: 'safe-integer proofs',
  children: [
    describe({
      name: isSafeInteger.name,
      children: [
        it({
          name: 'accepts safe integers',
          fn: async () => {
            expect(isSafeInteger(0,),).toBe(true,);
            expect(isSafeInteger(-1,),).toBe(true,);
            expect(isSafeInteger(Number.MAX_SAFE_INTEGER,),).toBe(true,);
            expect(isSafeInteger(Number.MIN_SAFE_INTEGER,),).toBe(true,);
          },
        },),

        it({
          name: 'rejects non-safe numbers',
          fn: async () => {
            expect(isSafeInteger(1.5,),).toBe(false,);
            expect(isSafeInteger(Number.NaN,),).toBe(false,);
            expect(isSafeInteger(Number.POSITIVE_INFINITY,),).toBe(false,);
            expect(isSafeInteger(Number.MAX_SAFE_INTEGER + 1,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: asSafeInteger.name,
      children: [
        it({
          name: 'returns identical value with branded type',
          fn: async () => {
            const result = asSafeInteger(3,);
            expect(result,).toBe(3,);
            expectTypeOf(result,).toEqualTypeOf<SafeInteger>();
          },
        },),

        it({
          name: 'throws diagnostic without array length',
          fn: async () => {
            const error = captureProofError(() => asSafeInteger(1.5,),);
            expect(error.index,).toBe(1.5,);
            expect(error.length,).toBeUndefined();
            expect(error.diagnostics,).toHaveLength(1,);
            expect(error.diagnostics[0]?.code,).toBe('non-safe-integer',);
          },
        },),
      ],
    },),

    describe({
      name: assertSafeInteger.name,
      children: [
        it({
          name: 'narrows accepted binding',
          fn: async () => {
            expect(doubleSafeInteger(21,),).toBe(42,);
          },
        },),

        it({
          name: 'throws for rejected binding',
          fn: async () => {
            const error = captureProofError(() => assertSafeInteger(1.5,),);
            expect(error.diagnostics[0]?.code,).toBe('non-safe-integer',);
          },
        },),
      ],
    },),
  ],
},);
