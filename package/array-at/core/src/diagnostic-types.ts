/**
 Structured diagnostic types for `arrayAt` validation.
 
 @module
 */

import type { NegativeNumber, } from './type-arithmetic-number.ts';

//region Independent diagnostics

/**
 Diagnostic for index outside JavaScript safe-integer domain.
 
 @example
 ```ts
 type Diagnostic = NonSafeIntegerDiagnostic<1.5>;
 ```
 */
export type NonSafeIntegerDiagnostic<Index extends number = number> = {
  readonly code: 'non-safe-integer';
  readonly message: `Index ${Index} is not a safe integer.`;
  readonly hint:
    'Use an integer from -9007199254740991 through 9007199254740991.';
  readonly index: Index;
};

/**
 Diagnostic for array without elements.
 
 @example
 ```ts
 const diagnostic: EmptyArrayDiagnostic = {
   code: 'empty-array',
   hint: 'Assign at least one element before reading by index.',
   length: 0,
   message: 'Cannot read from an empty array.',
 };
 ```
 */
export type EmptyArrayDiagnostic = {
  readonly code: 'empty-array';
  readonly message: 'Cannot read from an empty array.';
  readonly hint: 'Assign at least one element before reading by index.';
  readonly length: 0;
};

/**
 Static-only diagnostic for plain `number` without safe-integer proof.
 
 @example
 ```ts
 type Diagnostic = UnprovenSafeIntegerDiagnostic;
 ```
 */
export type UnprovenSafeIntegerDiagnostic = {
  readonly code: 'unproven-safe-integer';
  readonly message: 'Index type number is not proven to be a safe integer.';
  readonly hint:
    'Use asSafeInteger(index), assertSafeInteger(index), or isSafeInteger(index).';
};

//endregion Independent diagnostics

//region Range diagnostics

/**
 Exact negative lower bound for literal length or broad number for runtime length.
 
 @example
 ```ts
 type Bound = MinimumNegativeBound<3>;
 ```
 */
type MinimumNegativeBound<Length extends number> = number extends Length
  ? number
  : NegativeNumber<Length>;

/**
 Direction in which index exceeds valid array bounds.
 
 @example
 ```ts
 const direction: ArrayAtRangeDirection = 'past-end';
 ```
 */
export type ArrayAtRangeDirection = 'before-start' | 'past-end';

/**
 Diagnostic for positive index beyond last assigned position.
 
 @example
 ```ts
 type Diagnostic = PastEndDiagnostic<3, 1, 2, 3>;
 ```
 */
export type PastEndDiagnostic<
  Index extends number = number,
  Distance extends number = number,
  LastIndex extends number = number,
  Length extends number = number,
  MinimumNegativeIndex extends number = MinimumNegativeBound<Length>,
  > = {
    readonly code: 'out-of-range';
    readonly message: `Index ${Index} is past the end by ${Distance}.`;
    readonly hint:
      `Use 0 through ${LastIndex}, or -1 through -${Length}.`;
    readonly index: Index;
    readonly length: Length;
    readonly direction: 'past-end';
    readonly distance: Distance;
    readonly minimumPositiveIndex: 0;
    readonly maximumPositiveIndex: LastIndex;
    readonly minimumNegativeIndex: MinimumNegativeIndex;
    readonly maximumNegativeIndex: -1;
  };

/**
 Diagnostic for negative index before first assigned position.
 
 @example
 ```ts
 type Diagnostic = BeforeStartDiagnostic<-4, 1, 2, 3>;
 ```
 */
export type BeforeStartDiagnostic<
  Index extends number = number,
  Distance extends number = number,
  LastIndex extends number = number,
  Length extends number = number,
  MinimumNegativeIndex extends number = MinimumNegativeBound<Length>,
  > = {
    readonly code: 'out-of-range';
    readonly message: `Index ${Index} is before the start by ${Distance}.`;
    readonly hint:
      `Use 0 through ${LastIndex}, or -1 through -${Length}.`;
    readonly index: Index;
    readonly length: Length;
    readonly direction: 'before-start';
    readonly distance: Distance;
    readonly minimumPositiveIndex: 0;
    readonly maximumPositiveIndex: LastIndex;
    readonly minimumNegativeIndex: MinimumNegativeIndex;
    readonly maximumNegativeIndex: -1;
  };

//endregion Range diagnostics

//region Slot diagnostics

/**
 Diagnostic for in-range array slot without an assigned value.
 
 @example
 ```ts
 type Diagnostic = UnassignedSlotDiagnostic<-1, 1, 2>;
 ```
 */
export type UnassignedSlotDiagnostic<
  Index extends number = number,
  ResolvedIndex extends number = number,
  Length extends number = number,
  > = {
    readonly code: 'unassigned-slot';
    readonly message:
      `Index ${Index} resolves to unassigned array slot ${ResolvedIndex}.`;
    readonly hint: 'Assign a value to that slot before reading it.';
    readonly index: Index;
    readonly length: Length;
    readonly resolvedIndex: ResolvedIndex;
  };

//endregion Slot diagnostics

//region Public unions

/**
 One static or runtime diagnostic emitted by array access validation.
 
 @example
 ```ts
 function readCode(diagnostic: ArrayAtDiagnostic): string {
   return diagnostic.code;
 }
 ```
 */
export type ArrayAtDiagnostic =
  | NonSafeIntegerDiagnostic
    | EmptyArrayDiagnostic
    | UnprovenSafeIntegerDiagnostic
    | PastEndDiagnostic
    | BeforeStartDiagnostic
    | UnassignedSlotDiagnostic;

/**
 Diagnostic that can survive TypeScript erasure and occur at runtime.
 
 @example
 ```ts
 type RuntimeCode = RuntimeArrayAtDiagnostic['code'];
 ```
 */
export type RuntimeArrayAtDiagnostic = Exclude<
  ArrayAtDiagnostic,
  UnprovenSafeIntegerDiagnostic
  >;

/**
 Non-empty runtime diagnostic collection accepted by `ArrayAtError`.
 
 @example
 ```ts
 declare const diagnostics: NonEmptyRuntimeArrayAtDiagnostics;
 const first = diagnostics[0];
 ```
 */
export type NonEmptyRuntimeArrayAtDiagnostics = readonly [
  RuntimeArrayAtDiagnostic,
  ...readonly RuntimeArrayAtDiagnostic[],
];

//endregion Public unions
