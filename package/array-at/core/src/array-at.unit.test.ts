/**
 Runtime tests for built `arrayAt` artifact.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  arrayAt,
  ArrayAtError,
  type ArrayAtDiagnostic,
} from '../dist/final/neutral/index.mjs';

/**
 Runtime diagnostic code exposed by `ArrayAtError`.
 
 @example
 ```ts
 const code: RuntimeDiagnosticCode = 'empty-array';
 ```
 */
type RuntimeDiagnosticCode = Exclude<
  ArrayAtDiagnostic['code'],
  'unproven-safe-integer'
  >;

/**
 Runtime-only view bypassing static validation for negative tests.
 
 @example
 ```ts
 uncheckedArrayAt({ array: [10], index: 2, });
 ```
 */
const uncheckedArrayAt = arrayAt as unknown as ({
  array,
  index,
}: {
  readonly array: readonly unknown[];
  readonly index: number;
}) => unknown;

/**
 Runs operation and returns expected aggregated error.
 
 @param operation - Failing operation to invoke
 
 @returns Captured `ArrayAtError`
 
 @throws Unexpected caught value or missing throw
 
 @example
 ```ts
 const error = captureArrayAtError(() => uncheckedArrayAt({
   array: [],
   index: 0,
 }));
 ```
 */
function captureArrayAtError(operation: () => unknown,): ArrayAtError {
  try {
    operation();
  }
  catch (error) {
    if (error instanceof ArrayAtError)
      return error;
    throw error;
  }

  throw new Error('Expected operation to throw ArrayAtError.',);
}

/**
 Finds runtime diagnostic by discriminating code.
 
 @param options - Captured error and requested diagnostic code
 
 @returns Matching diagnostic narrowed by code
 
 @throws Error when diagnostic is absent
 
 @example
 ```ts
 const diagnostic = diagnosticByCode({ error, code: 'empty-array', });
 ```
 */
function diagnosticByCode<const Code extends RuntimeDiagnosticCode>({
  error,
  code,
}: {
  readonly error: ArrayAtError;
  readonly code: Code;
}): Extract<ArrayAtDiagnostic, { readonly code: Code; }> {
  const diagnostic = error.diagnostics.find(candidate => candidate.code === code,);
  if (diagnostic === undefined)
    throw new Error(`Expected diagnostic code ${code}.`,);
  return diagnostic as Extract<ArrayAtDiagnostic, { readonly code: Code; }>;
}

await describe({
  name: arrayAt.name,
  children: [
    //region Successful access

    it({
      name: 'returns elements at positive and negative indices',
      fn: async () => {
        expect(arrayAt({ array: [10, 20, 30], index: 0, }),).toBe(10,);
        expect(arrayAt({ array: [10, 20, 30], index: -1, }),).toBe(30,);
        expect(arrayAt({ array: [10, 20, 30], index: -3, }),).toBe(10,);
        expect(arrayAt({ array: [10, 20], index: -0, }),).toBe(10,);
      },
    },),

    it({
      name: 'returns explicitly stored undefined',
      fn: async () => {
        const values: readonly unknown[] = [10, undefined];
        expect(arrayAt({ array: values, index: 1, }),).toBeUndefined();
      },
    },),

    //endregion Successful access

    //region Independent diagnostics

    it({
      name: 'aggregates non-safe-integer and empty-array diagnostics',
      fn: async () => {
        const error = captureArrayAtError(() => uncheckedArrayAt({
          array: [],
          index: 1.5,
        }),);
        const codes = error.diagnostics.map(({ code, }) => code).toSorted();
        const messages = error.message.split('\n').toSorted();

        expect(codes,).toEqual(['empty-array', 'non-safe-integer'],);
        expect(messages,).toEqual([
          'Cannot read from an empty array.',
          'Index 1.5 is not a safe integer.',
        ],);
        expect(error.index,).toBe(1.5,);
        expect(error.length,).toBe(0,);
        expect(error.name,).toBe('ArrayAtError',);
      },
    },),

    it({
      name: 'reports each non-safe numeric category',
      fn: async () => {
        const indices = [
          1.5,
          Number.NaN,
          Number.POSITIVE_INFINITY,
          Number.MAX_SAFE_INTEGER + 1,
        ];

        indices.forEach(index => {
          const error = captureArrayAtError(() => uncheckedArrayAt({
            array: [10],
            index,
          }),);
          expect(error.diagnostics,).toHaveLength(1,);
          expect(error.diagnostics[0]?.code,).toBe('non-safe-integer',);
        },);
      },
    },),

    it({
      name: 'reports empty-array alone for safe index',
      fn: async () => {
        const error = captureArrayAtError(() => uncheckedArrayAt({
          array: [],
          index: 0,
        }),);
        expect(error.diagnostics,).toHaveLength(1,);
        expect(error.diagnostics[0]?.code,).toBe('empty-array',);
      },
    },),

    //endregion Independent diagnostics

    //region Dependent diagnostics

    it({
      name: 'reports exact past-end direction distance and bounds',
      fn: async () => {
        const error = captureArrayAtError(() => uncheckedArrayAt({
          array: [10],
          index: 1,
        }),);
        const diagnostic = diagnosticByCode({ error, code: 'out-of-range', });

        expect(diagnostic.direction,).toBe('past-end',);
        expect(diagnostic.distance,).toBe(1,);
        expect(diagnostic.minimumPositiveIndex,).toBe(0,);
        expect(diagnostic.maximumPositiveIndex,).toBe(0,);
        expect(diagnostic.minimumNegativeIndex,).toBe(-1,);
        expect(diagnostic.maximumNegativeIndex,).toBe(-1,);
        expect(diagnostic.message,).toBe('Index 1 is past the end by 1.',);
      },
    },),

    it({
      name: 'keeps past-end distance exact at maximum safe integer',
      fn: async () => {
        const error = captureArrayAtError(() => uncheckedArrayAt({
          array: [10],
          index: Number.MAX_SAFE_INTEGER,
        }),);
        const diagnostic = diagnosticByCode({ error, code: 'out-of-range', });

        expect(diagnostic.direction,).toBe('past-end',);
        expect(diagnostic.distance,).toBe(Number.MAX_SAFE_INTEGER,);
      },
    },),

    it({
      name: 'reports exact before-start direction distance and bounds',
      fn: async () => {
        const error = captureArrayAtError(() => uncheckedArrayAt({
          array: [10, 20, 30],
          index: -4,
        }),);
        const diagnostic = diagnosticByCode({ error, code: 'out-of-range', });

        expect(diagnostic.direction,).toBe('before-start',);
        expect(diagnostic.distance,).toBe(1,);
        expect(diagnostic.minimumPositiveIndex,).toBe(0,);
        expect(diagnostic.maximumPositiveIndex,).toBe(2,);
        expect(diagnostic.minimumNegativeIndex,).toBe(-3,);
        expect(diagnostic.maximumNegativeIndex,).toBe(-1,);
        expect(diagnostic.message,).toBe('Index -4 is before the start by 1.',);
      },
    },),

    it({
      name: 'reports unassigned slots through positive and negative indices',
      fn: async () => {
        const sparse: number[] = [10];
        sparse.length = 2;
        const positiveError = captureArrayAtError(() => uncheckedArrayAt({
          array: sparse,
          index: 1,
        }),);
        const negativeError = captureArrayAtError(() => uncheckedArrayAt({
          array: sparse,
          index: -1,
        }),);
        const positiveDiagnostic = diagnosticByCode({
          error: positiveError,
          code: 'unassigned-slot',
        },);
        const negativeDiagnostic = diagnosticByCode({
          error: negativeError,
          code: 'unassigned-slot',
        },);

        expect(positiveDiagnostic.resolvedIndex,).toBe(1,);
        expect(negativeDiagnostic.resolvedIndex,).toBe(1,);
        expect(positiveDiagnostic.length,).toBe(2,);
        expect(negativeDiagnostic.index,).toBe(-1,);
      },
    },),

    //endregion Dependent diagnostics

    //region Runtime authority

    it({
      name: 'rechecks mutable tuple after it is shortened',
      fn: async () => {
        const shortened: [10] = [10];
        shortened.pop();
        const error = captureArrayAtError(() => uncheckedArrayAt({
          array: shortened,
          index: 0,
        }),);
        expect(error.diagnostics[0]?.code,).toBe('empty-array',);
      },
    },),

    it({
      name: 'freezes captured diagnostics',
      fn: async () => {
        const error = captureArrayAtError(() => uncheckedArrayAt({
          array: [],
          index: 0,
        }),);
        const mutableView = error.diagnostics as ArrayAtDiagnostic[];

        expect(Object.isFrozen(error.diagnostics,),).toBe(true,);
        expect(() => mutableView.push(error.diagnostics[0] as ArrayAtDiagnostic,)).toThrow();
      },
    },),

    //endregion Runtime authority
  ],
},);
