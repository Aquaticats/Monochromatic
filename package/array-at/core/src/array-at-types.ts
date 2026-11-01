/**
 Public diagnostic computation, call validation, and return-type resolution.
 
 @module
 */

import type {
  ArrayAtDiagnostic,
  EmptyArrayDiagnostic,
  NonSafeIntegerDiagnostic,
  UnprovenSafeIntegerDiagnostic,
} from './diagnostic-types.ts';
import type { SafeInteger, } from './safe-integer.ts';
import type {
  IsUsableIndexText,
  NumberMagnitude,
} from './type-arithmetic-number.ts';
import type {
  LiteralIndexDiagnostic,
  ResolveLiteralIndex,
} from './type-index-resolution.ts';

//region Diagnostic computation

/**
 Index diagnostic that can be determined without array shape.
 
 @example
 ```ts
 type Diagnostic = IndexEvidence<1.5>;
 ```
 */
type IndexEvidence<Index extends number> = Index extends number
  ? [Index] extends [SafeInteger]
    ? never
    : number extends Index
      ? UnprovenSafeIntegerDiagnostic
      : IsUsableIndexText<NumberMagnitude<Index>> extends false
        ? NonSafeIntegerDiagnostic<Index>
        : never
  : never;

/**
 Empty-array diagnostic statically proven by fixed length.
 
 @example
 ```ts
 type Diagnostic = EmptyArrayEvidence<readonly []>;
 ```
 */
type EmptyArrayEvidence<ArrayValue extends readonly unknown[]> =
  number extends ArrayValue['length']
    ? never
    : ArrayValue['length'] extends 0
      ? EmptyArrayDiagnostic
      : never;

/**
 Range or slot diagnostic whose prerequisites are statically satisfied.
 
 Branded and plain-number indices defer dependent checks to runtime. Dynamic
 arrays likewise defer checks because their length is not statically known.
 
 @example
 ```ts
 type Diagnostic = DependentIndexEvidence<readonly [10], 2>;
 ```
 */
type DependentIndexEvidence<
  ArrayValue extends readonly unknown[],
  Index extends number,
  > = Index extends number
    ? [Index] extends [SafeInteger]
      ? never
      : number extends Index
        ? never
        : IsUsableIndexText<NumberMagnitude<Index>> extends false
          ? never
          : number extends ArrayValue['length']
            ? never
            : ArrayValue['length'] extends 0
              ? never
              : LiteralIndexDiagnostic<ArrayValue, Index>
    : never;

/**
 Union of unordered diagnostics detectable for one array member.
 
 @example
 ```ts
 type Diagnostics = ArrayMemberDiagnosticUnion<readonly [], 1.5>;
 ```
 */
type ArrayMemberDiagnosticUnion<
  ArrayValue extends readonly unknown[],
  Index extends number,
  > =
  | IndexEvidence<Index>
    | EmptyArrayEvidence<ArrayValue>
    | DependentIndexEvidence<ArrayValue, Index>;

/**
 Converts diagnostic union into readonly unordered collection type.
 
 Empty tuple represents statically valid access. Invalid access exposes every
 detectable diagnostic as collection element union without promising order.
 
 @example
 ```ts
 type Diagnostics = DiagnosticCollection<EmptyArrayDiagnostic>;
 ```
 */
type DiagnosticCollection<Diagnostic extends ArrayAtDiagnostic> =
  [Diagnostic] extends [never]
    ? readonly never[]
    : readonly Diagnostic[];

/**
 Computes unordered diagnostics detectable for array and index types.
 
 Union arrays distribute so each possible member retains its own length and
 element types. Runtime validation still handles dynamic lengths and mutation.
 
 @example
 ```ts
 type Diagnostics = ArrayAtDiagnostics<readonly [10], 2>;
 ```
 */
export type ArrayAtDiagnostics<
  ArrayValue extends readonly unknown[],
  Index extends number,
  > = ArrayValue extends readonly unknown[]
    ? DiagnosticCollection<ArrayMemberDiagnosticUnion<ArrayValue, Index>>
    : never;

//endregion Diagnostic computation

//region Correlated call validation

/**
 Correlated argument shape accepted by `arrayAt`.
 
 @example
 ```ts
 const argument: ArrayAtArgument = { array: [10], index: 0, };
 ```
 */
export type ArrayAtArgument = {
  readonly array: readonly unknown[];
  readonly index: number;
};

/**
 Private impossible key used to place diagnostics on invalid calls.
 
 @example
 ```ts
 type Validation = ValidateArrayAtArgument<ArrayAtArgument>;
 ```
 */
declare const arrayAtDiagnosticTag: unique symbol;

/**
 Diagnostic union across every correlated argument member.
 
 @example
 ```ts
 type Diagnostic = ArgumentDiagnosticUnion<{
   readonly array: readonly [];
   readonly index: 0;
 }>;
 ```
 */
type ArgumentDiagnosticUnion<Argument extends ArrayAtArgument> =
  Argument extends {
    readonly array: infer ArrayValue extends readonly unknown[];
    readonly index: infer Index extends number;
  }
    ? ArrayMemberDiagnosticUnion<ArrayValue, Index>
    : never;

/**
 Impossible property carrying unordered diagnostics for invalid call.
 
 @example
 ```ts
 type Failure = InvalidArrayAtArgument<EmptyArrayDiagnostic>;
 ```
 */
type InvalidArrayAtArgument<Diagnostic extends ArrayAtDiagnostic> = {
  readonly [arrayAtDiagnosticTag]: readonly Diagnostic[];
};

/**
 Adds no requirements to valid argument and impossible property to invalid one.
 
 @example
 ```ts
 type Validation = ValidateArrayAtArgument<{
   readonly array: readonly [10];
   readonly index: 0;
 }>;
 ```
 */
export type ValidateArrayAtArgument<Argument extends ArrayAtArgument> = [
  ArgumentDiagnosticUnion<Argument>,
] extends [never]
  ? unknown
  : InvalidArrayAtArgument<ArgumentDiagnosticUnion<Argument>>;

//endregion Correlated call validation

//region Return type

/**
 Resolves one array and index pair to returned element type.
 
 @example
 ```ts
 type Last = ArrayElementAt<readonly [10, 20], -1>;
 ```
 */
type ArrayElementAt<
  ArrayValue extends readonly unknown[],
  Index extends number,
  > = Index extends number
    ? [Index] extends [SafeInteger]
      ? ArrayValue[number]
      : number extends Index
        ? ArrayValue[number]
        : number extends ArrayValue['length']
          ? ArrayValue[number]
          : ResolveLiteralIndex<ArrayValue, Index> extends infer Resolved extends number
            ? ArrayValue[Resolved]
            : ArrayValue[number]
    : never;

/**
 Computes result type while preserving correlated union argument members.
 
 @example
 ```ts
 type Result = ArrayAtResult<{
   readonly array: readonly [10, 20];
   readonly index: -1;
 }>;
 ```
 */
export type ArrayAtResult<Argument extends ArrayAtArgument> =
  Argument extends {
    readonly array: infer ArrayValue extends readonly unknown[];
    readonly index: infer Index extends number;
  }
    ? ArrayElementAt<ArrayValue, Index>
    : never;

//endregion Return type
