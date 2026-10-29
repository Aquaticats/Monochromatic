/**
 Type-level literal index resolution and dependent diagnostics.
 
 @module
 */

import type { ArrayAtDiagnostic, } from './diagnostic-types.ts';
import type {
  createBeforeStartDiagnostic,
  createNonSafeIntegerDiagnostic,
  createPastEndDiagnostic,
  createUnassignedSlotDiagnostic,
} from './diagnostic-factories.ts';
import type {
  DecimalNumber,
  Decrement,
  IsExactUndefined,
  IsUsableIndexText,
  NegativeNumber,
  SubtractNumbers,
} from './type-arithmetic-number.ts';

/**
 Resolves signed literal index or returns precise diagnostic.
 
 @example
 ```ts
 type Resolution = ResolveLiteralIndex<readonly [10, 20], -1>;
 ```
 */
export type ResolveLiteralIndex<
  ArrayValue extends readonly unknown[],
  Index extends number,
  > = `${Index}` extends `-${infer MagnitudeText}`
    ? CheckLiteralIndex<
      ArrayValue,
      Index,
      MagnitudeText,
      true
      >
    : CheckLiteralIndex<
      ArrayValue,
      Index,
      `${Index}`,
      false
      >;

/**
 Applies safe-integer and range checks after sign parsing.
 
 @example
 ```ts
 type Resolution = CheckLiteralIndex<readonly [10, 20], -1, '1', true>;
 ```
 */
type CheckLiteralIndex<
  ArrayValue extends readonly unknown[],
  Index extends number,
  MagnitudeText extends string,
  IsNegative extends boolean,
  > = IsUsableIndexText<MagnitudeText> extends false
    ? ReturnType<typeof createNonSafeIntegerDiagnostic<Index>>
    : [
      ArrayValue['length'],
      DecimalNumber<MagnitudeText>,
    ] extends [
      infer Length extends number,
      infer Magnitude extends number,
    ]
      ? IsNegative extends true
        ? ResolveNegativeLiteralIndex<
          Index,
          Length,
          Magnitude
          >
        : ResolvePositiveLiteralIndex<
          Index,
          Length,
          Magnitude
          >
      : never;

/**
 Resolves negative literal index or reports before-start distance.
 
 @example
 ```ts
 type Resolution = ResolveNegativeLiteralIndex<-3, 2, 3>;
 ```
 */
type ResolveNegativeLiteralIndex<
  Index extends number,
  Length extends number,
  Magnitude extends number,
  > = [SubtractNumbers<Length, Magnitude>] extends [never]
    ? ReturnType<typeof createBeforeStartDiagnostic<
      Index,
      SubtractNumbers<Magnitude, Length> & number,
      Decrement<Length>,
      Length,
      NegativeNumber<Length>
      >>
    : SubtractNumbers<Length, Magnitude>;

/**
 Resolves positive literal index or reports past-end distance.
 
 Distance is measured from last valid index, so first invalid positive index
 is past end by one rather than zero.
 
 @example
 ```ts
 type Resolution = ResolvePositiveLiteralIndex<2, 2, 2>;
 ```
 */
type ResolvePositiveLiteralIndex<
  Index extends number,
  Length extends number,
  Magnitude extends number,
  > = [SubtractNumbers<Length, Magnitude>] extends [never]
    ? ReturnType<typeof createPastEndDiagnostic<
      Index,
      SubtractNumbers<Magnitude, Decrement<Length>> & number,
      Decrement<Length>,
      Length,
      NegativeNumber<Length>
      >>
    : SubtractNumbers<Length, Magnitude> extends 0
      ? ReturnType<typeof createPastEndDiagnostic<
        Index,
        SubtractNumbers<Magnitude, Decrement<Length>> & number,
        Decrement<Length>,
        Length,
        NegativeNumber<Length>
        >>
      : Magnitude;

/**
 Returns dependent range or slot diagnostic for one literal index.
 
 `never` means access is statically valid.
 
 @example
 ```ts
 type Diagnostic = LiteralIndexDiagnostic<readonly [10], 2>;
 ```
 */
export type LiteralIndexDiagnostic<
  ArrayValue extends readonly unknown[],
  Index extends number,
  > = ResolveLiteralIndex<ArrayValue, Index> extends infer Resolution
    ? Resolution extends number
      ? IsExactUndefined<ArrayValue[Resolution]> extends true
        ? ReturnType<typeof createUnassignedSlotDiagnostic<
          Index,
          Resolution,
          ArrayValue['length']
          >>
        : never
      : Resolution extends ArrayAtDiagnostic
        ? Resolution
        : never
    : never;
