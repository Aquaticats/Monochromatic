/**
 Compile-time behavior tests for built `arrayAt` declarations.
 
 Invalid-call helpers are exported only so TypeScript checks their bodies;
 unit runner never invokes them.
 
 @module
 */

import {
  describe,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  arrayAt,
  type ArrayAtDiagnostic,
  type ArrayAtError,
  type ArrayAtDiagnostics,
  asSafeInteger,
  assertSafeInteger,
  isSafeInteger,
} from '../dist/final/neutral/index.mjs';

/**
 Sentinel returned by guard example when integer proof fails.
 
 @example
 ```ts
 const rejected = NOT_SAFE_INTEGER;
 ```
 */
const NOT_SAFE_INTEGER = Symbol('index does not have safe-integer proof',);

/**
 Empty tuple value used to test empty-array diagnostics without fake optionality.
 
 @example
 ```ts
 type EmptyArray = typeof EMPTY_ARRAY;
 ```
 */
const EMPTY_ARRAY = [] as const;

/**
 Diagnostic code union carried by unordered computed collection.
 
 @example
 ```ts
 type Codes = DiagnosticCodes<ArrayAtDiagnostics<readonly [], 1.5>>;
 ```
 */
type DiagnosticCodes<Diagnostics extends readonly ArrayAtDiagnostic[]> =
  Diagnostics[number]['code'];

/**
 Narrows plain index with assertion before array access.
 
 @param index - Unproven numeric index
 
 @returns Selected tuple element union
 
 @throws Array access or proof error
 
 @example
 ```ts
 const value = accessAfterAssertion(0);
 ```
 */
function accessAfterAssertion(index: number,): 10 | 20 | 30 {
  assertSafeInteger(index,);
  return arrayAt({ array: [10, 20, 30], index, });
}

/**
 Narrows plain index with predicate before array access.
 
 @param index - Unproven numeric index
 
 @returns Selected tuple element union or `null` for rejected integer proof
 
 @throws Array access error after successful proof
 
 @example
 ```ts
 const value = accessAfterGuard(0);
 ```
 */
function accessAfterGuard(
  index: number,
): 10 | 20 | 30 | typeof NOT_SAFE_INTEGER {
  if (!isSafeInteger(index,))
    return NOT_SAFE_INTEGER;
  return arrayAt({ array: [10, 20, 30], index, });
}

await describe({
  name: 'arrayAt static interface',
  children: [
    it({
      name: 'returns exact tuple elements',
      fn: async () => {
        const first = arrayAt({ array: [10, 20, 30], index: 0, });
        const middle = arrayAt({ array: [10, 20, 30], index: -2, });
        const object = arrayAt({
          array: ['a', { value: 1, }, null],
          index: 1,
        },);
        const nullable = arrayAt({
          array: ['a', { value: 1, }, null],
          index: 2,
        },);
        const anyElement = undefined as any;
        const anyResult = arrayAt({ array: [anyElement], index: 0, });

        expectTypeOf(first,).toEqualTypeOf<10>();
        expectTypeOf(middle,).toEqualTypeOf<20>();
        expectTypeOf(object,).toEqualTypeOf<{ readonly value: 1; }>();
        expectTypeOf(nullable,).toEqualTypeOf<null>();
        expectTypeOf(anyResult,).toBeAny();
      },
    },),

    it({
      name: 'returns element union for proven dynamic index',
      fn: async () => {
        const result = arrayAt({
          array: [10, 20, 30],
          index: asSafeInteger(1,),
        },);
        expectTypeOf(result,).toEqualTypeOf<10 | 20 | 30>();
        expectTypeOf<ReturnType<typeof accessAfterAssertion>>()
          .toEqualTypeOf<10 | 20 | 30>();
        expectTypeOf<ReturnType<typeof accessAfterGuard>>()
          .toEqualTypeOf<10 | 20 | 30 | typeof NOT_SAFE_INTEGER>();
      },
    },),

    it({
      name: 'returns element type for unknown-length array',
      fn: async () => {
        const values: string[] = ['value'];
        const result = arrayAt({ array: values, index: 0, });
        expectTypeOf(result,).toEqualTypeOf<string>();
      },
    },),

    it({
      name: 'computes unordered aggregate diagnostic codes',
      fn: async () => {
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<typeof EMPTY_ARRAY, 1.5>>>()
          .toEqualTypeOf<'empty-array' | 'non-safe-integer'>();
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<typeof EMPTY_ARRAY, number>>>()
          .toEqualTypeOf<'empty-array' | 'unproven-safe-integer'>();
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<readonly [10], number>>>()
          .toEqualTypeOf<'unproven-safe-integer'>();
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<readonly [10], 1>>>()
          .toEqualTypeOf<'out-of-range'>();
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<readonly [10], 0>>>()
          .toEqualTypeOf<never>();
        // oxlint-disable-next-line eslint/no-loss-of-precision -- Type-level test verifies Infinity literal is diagnosed before arithmetic.
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<readonly [10], 1e999>>>()
          .toEqualTypeOf<'non-safe-integer'>();
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<readonly [10], 100_000_000_000_000_000>>>()
          .toEqualTypeOf<'non-safe-integer'>();
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<readonly [10], 9_007_199_254_740_991>>>()
          .toEqualTypeOf<'out-of-range'>();
        expectTypeOf<DiagnosticCodes<ArrayAtDiagnostics<readonly [10], 9_007_199_254_740_992>>>()
          .toEqualTypeOf<'non-safe-integer'>();
        expectTypeOf<'code' extends keyof ArrayAtError ? true : false>()
          .toEqualTypeOf<false>();
        expectTypeOf<'hint' extends keyof ArrayAtError ? true : false>()
          .toEqualTypeOf<false>();
      },
    },),

    it({
      name: 'computes exact directional range metadata',
      fn: async () => {
        type PastEnd = Extract<
          ArrayAtDiagnostics<readonly [10], 1>[number],
          { readonly direction: 'past-end'; }
          >;
        type BeforeStart = Extract<
          ArrayAtDiagnostics<readonly [10, 20, 30], -4>[number],
          { readonly direction: 'before-start'; }
          >;
        type LargePastEnd = Extract<
          ArrayAtDiagnostics<readonly [10, 20, 30], 100_000>[number],
          { readonly direction: 'past-end'; }
          >;
        type LargeBeforeStart = Extract<
          ArrayAtDiagnostics<readonly [10, 20, 30], -1_000_000_000>[number],
          { readonly direction: 'before-start'; }
          >;
        type ZeroPaddedDifference = Extract<
          ArrayAtDiagnostics<
            readonly [10, 20, 30, 40, 50, 60, 70, 80, 90, 100],
            10
            >[number],
          { readonly direction: 'past-end'; }
          >;

        expectTypeOf<PastEnd['distance']>().toEqualTypeOf<1>();
        expectTypeOf<PastEnd['maximumPositiveIndex']>().toEqualTypeOf<0>();
        expectTypeOf<BeforeStart['distance']>().toEqualTypeOf<1>();
        expectTypeOf<BeforeStart['minimumNegativeIndex']>().toEqualTypeOf<-3>();
        expectTypeOf<LargePastEnd['distance']>().toEqualTypeOf<99_998>();
        expectTypeOf<LargeBeforeStart['distance']>().toEqualTypeOf<999_999_997>();
        expectTypeOf<ZeroPaddedDifference['distance']>().toEqualTypeOf<1>();
      },
    },),
  ],
},);

//region Invalid literal calls

/**
 Verifies empty tuple calls are rejected.
 
 @example
 ```ts
 rejectsEmptyArray();
 ```
 */
export function rejectsEmptyArray(): void {
  // @ts-expect-error -- empty tuple has no valid index.
  arrayAt({ array: [], index: 0, });
}

/**
 Verifies plain numbers require safe-integer proof.
 
 @param index - Unproven numeric index
 
 @example
 ```ts
 rejectsPlainNumber(0);
 ```
 */
export function rejectsPlainNumber(index: number,): void {
  // @ts-expect-error -- plain number lacks safe-integer proof.
  arrayAt({ array: [10, 20, 30], index, });
}

/**
 Verifies positive out-of-range literal is rejected.
 
 @example
 ```ts
 rejectsPastEnd();
 ```
 */
export function rejectsPastEnd(): void {
  // @ts-expect-error -- index is one position past array end.
  arrayAt({ array: [10], index: 1, });
}

/**
 Verifies negative out-of-range literal is rejected.
 
 @example
 ```ts
 rejectsBeforeStart();
 ```
 */
export function rejectsBeforeStart(): void {
  // @ts-expect-error -- index is one position before array start.
  arrayAt({ array: [10], index: -2, });
}

/**
 Verifies fractional literal is rejected.
 
 @example
 ```ts
 rejectsFraction();
 ```
 */
export function rejectsFraction(): void {
  // @ts-expect-error -- fractional index is not safe integer.
  arrayAt({ array: [10, 20], index: 1.5, });
}

/**
 Verifies exact `undefined` tuple element is conservatively rejected.
 
 @example
 ```ts
 rejectsStaticUndefined();
 ```
 */
export function rejectsStaticUndefined(): void {
  // @ts-expect-error -- static type cannot distinguish stored undefined from hole.
  arrayAt({ array: [10, undefined], index: 1, });
}

/**
 Verifies large positive safe integer reports range without compiler overflow.
 
 @example
 ```ts
 rejectsLargePositiveIndex();
 ```
 */
export function rejectsLargePositiveIndex(): void {
  // @ts-expect-error -- large safe integer is outside tuple bounds.
  arrayAt({ array: [10, 20, 30], index: 100_000, });
}

/**
 Verifies large negative safe integer reports range without compiler overflow.
 
 @example
 ```ts
 rejectsLargeNegativeIndex();
 ```
 */
export function rejectsLargeNegativeIndex(): void {
  // @ts-expect-error -- large negative safe integer is outside tuple bounds.
  arrayAt({ array: [10, 20, 30], index: -1_000_000_000, });
}

//endregion Invalid literal calls

//region Union correlation

/**
 Verifies every member of valid index union contributes result type.
 
 @param index - Union containing only valid tuple indices
 
 @returns Elements selected by possible index members
 
 @example
 ```ts
 const result = accessValidUnionIndex(0);
 ```
 */
export function accessValidUnionIndex(index: 0 | 2,): 10 | 30 {
  return arrayAt({ array: [10, 20, 30], index, });
}

/**
 Verifies one invalid index union member rejects whole call.
 
 @param index - Union containing valid and invalid tuple indices
 
 @example
 ```ts
 rejectsInvalidUnionIndex(2);
 ```
 */
export function rejectsInvalidUnionIndex(index: 0 | 2,): void {
  // @ts-expect-error -- index union contains out-of-range member.
  arrayAt({ array: [10], index, });
}

/**
 Correlated valid input alternatives.
 
 @example
 ```ts
 declare const input: ValidCorrelatedInput;
 ```
 */
type ValidCorrelatedInput =
  | { readonly array: readonly [10]; readonly index: 0; }
    | { readonly array: readonly [10, 20]; readonly index: 1; };

/**
 Invalid input alternatives containing one impossible member.
 
 @example
 ```ts
 declare const input: InvalidCorrelatedInput;
 ```
 */
type InvalidCorrelatedInput =
  | { readonly array: readonly [10]; readonly index: 1; }
    | { readonly array: readonly [10, 20]; readonly index: 1; };

/**
 Verifies valid correlated input preserves result union.
 
 @param input - Correlated valid alternatives
 
 @returns Selected element union
 
 @example
 ```ts
 declare const input: ValidCorrelatedInput;
 const result = accessCorrelated(input);
 ```
 */
export function accessCorrelated(input: ValidCorrelatedInput,): 10 | 20 {
  return arrayAt(input,);
}

/**
 Verifies invalid correlated alternative rejects entire call.
 
 @param input - Alternatives including impossible access
 
 @example
 ```ts
 declare const input: InvalidCorrelatedInput;
 rejectsInvalidCorrelation(input);
 ```
 */
export function rejectsInvalidCorrelation(input: InvalidCorrelatedInput,): void {
  // @ts-expect-error -- one correlated member has out-of-range index.
  arrayAt(input,);
}

//endregion Union correlation
