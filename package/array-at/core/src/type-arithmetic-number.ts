/**
 Bounded whole-number arithmetic for array-index types.
 
 TypeScript cannot subtract numeric literal types directly. These helpers
 convert safe integers to decimal characters and perform long subtraction.
 
 @module
 */

import type {
  DecimalCharacters,
  DigitCharacter,
  SubtractDigitSequences,
} from './type-arithmetic-digit.ts';

//region Decimal conversion

/**
 Removes leading zeros while preserving one zero for an all-zero value.
 
 Recursion is bounded by safe integer decimal width.
 
 @example
 ```ts
 type Normalized = StripLeadingZeros<'00417'>;
 ```
 */
export type StripLeadingZeros<Text extends string> =
  Text extends `0${infer Rest}`
    ? Rest extends ''
      ? '0'
      : StripLeadingZeros<Rest>
    : Text;

/**
 Converts decimal text into numeric literal type.
 
 @example
 ```ts
 type Value = DecimalNumber<'417'>;
 ```
 */
export type DecimalNumber<Text extends string> =
  Text extends `${infer Value extends number}`
    ? Value
    : never;

/**
 Subtracts non-negative integer literals using bounded decimal arithmetic.
 
 Returns `never` when subtrahend exceeds minuend.
 
 @example
 ```ts
 type Difference = SubtractNumbers<100_000, 3>;
 ```
 */
export type SubtractNumbers<
  Minuend extends number,
  Subtrahend extends number,
  > = SubtractDigitSequences<
    DecimalCharacters<`${Minuend}`>,
    DecimalCharacters<`${Subtrahend}`>
    > extends infer Difference extends string
    ? DecimalNumber<StripLeadingZeros<Difference>>
    : never;

/**
 Decrements positive integer literal.
 
 @example
 ```ts
 type LastIndex = Decrement<3>;
 ```
 */
export type Decrement<Value extends number> =
  SubtractNumbers<Value, 1> & number;

/**
 Extracts absolute numeric literal from signed numeric literal.
 
 @example
 ```ts
 type Positive = AbsoluteNumber<-2>;
 ```
 */
export type AbsoluteNumber<Value extends number> =
  `${Value}` extends `-${infer Magnitude extends number}`
    ? Magnitude
    : Value;

/**
 Negates non-negative numeric literal.
 
 @example
 ```ts
 type NegativeThree = NegativeNumber<3>;
 ```
 */
export type NegativeNumber<Value extends number> =
  `-${Value}` extends `${infer Negative extends number}`
    ? Negative
    : never;

//endregion Decimal conversion

//region Index usability

/**
 Maximum safe integer written as decimal text for type comparison.
 
 @example
 ```ts
 type MaximumText = MaximumSafeIntegerText;
 ```
 */
type MaximumSafeIntegerText = '9007199254740991';

/**
 Reports whether every character in non-empty text is decimal digit.
 
 Recursion is bounded by safe integer decimal width at call sites.
 
 @example
 ```ts
 type IsWhole = IsDigitText<'123'>;
 ```
 */
export type IsDigitText<Text extends string> =
  Text extends `${DigitCharacter}${infer Rest}`
    ? Rest extends ''
      ? true
      : IsDigitText<Rest>
    : false;

/**
 Reports whether decimal text is within JavaScript safe-integer magnitude.
 
 @example
 ```ts
 type Fits = FitsSafeInteger<'42'>;
 ```
 */
export type FitsSafeInteger<Text extends string> = [
  SubtractDigitSequences<
    DecimalCharacters<MaximumSafeIntegerText>,
    DecimalCharacters<Text>
    >,
] extends [never]
  ? false
  : true;

/**
 Returns unsigned decimal spelling for numeric literal.
 
 @example
 ```ts
 type Magnitude = NumberMagnitude<-42>;
 ```
 */
export type NumberMagnitude<Value extends number> =
  `${Value}` extends `-${infer Magnitude}`
    ? Magnitude
    : `${Value}`;

/**
 Reports whether unsigned decimal text can participate in exact arithmetic.
 
 Fractions, infinities, exponential spellings, and values beyond safe-integer
 magnitude return `false`.
 
 @example
 ```ts
 type Usable = IsUsableIndexText<'42'>;
 ```
 */
export type IsUsableIndexText<Text extends string> =
  IsDigitText<Text> extends true
    ? FitsSafeInteger<Text>
    : false;

//endregion Index usability

//region Type predicates

/**
 Reports whether type is exactly `any`.
 
 @example
 ```ts
 type Result = IsAny<any>;
 ```
 */
export type IsAny<Value> = 0 extends 1 & Value
  ? true
  : false;

/**
 Reports whether type is exactly `undefined` rather than merely containing it.
 
 TypeScript cannot distinguish an omitted tuple slot from an explicitly stored
 `undefined`, so static hole detection deliberately rejects both.
 
 @example
 ```ts
 type Result = IsExactUndefined<undefined>;
 ```
 */
export type IsExactUndefined<Value> = IsAny<Value> extends true
  ? false
  : [Value] extends [undefined]
    ? [undefined] extends [Value]
      ? true
      : false
    : false;

//endregion Type predicates
