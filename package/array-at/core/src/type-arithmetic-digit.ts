/**
 Bounded digit arithmetic used by array-index diagnostics.
 
 Every recursive walk is bounded by a safe integer's decimal representation.
 Tuple construction is used only for one decimal digit at a time.
 
 @module
 */

//region Decimal primitives

/**
 Decimal digit character.
 
 @example
 ```ts
 const digit: DigitCharacter = '7';
 ```
 */
export type DigitCharacter =
  | '0'
    | '1'
    | '2'
    | '3'
    | '4'
    | '5'
    | '6'
    | '7'
    | '8'
    | '9';

/**
 Borrow state carried between subtraction columns.
 
 @example
 ```ts
 const borrow: Borrow = 1;
 ```
 */
export type Borrow = 0 | 1;

/**
 Removes non-empty accumulator marker from bounded tuple counter.
 
 Marker prevents genuine zero-length sequence state from resembling fake
 optionality to repository lint rules.
 
 @example
 ```ts
 type EmptySequence = DropAccumulatorMarker<readonly [unknown]>;
 ```
 */
type DropAccumulatorMarker<Accumulator extends readonly unknown[]> =
  Accumulator extends readonly [
    unknown,
    ...infer Values,
  ]
    ? Values
    : never;

/**
 Constructs tuple whose length represents one bounded decimal value.
 
 Callers use values from zero through ten, so recursion cannot approach
 TypeScript's instantiation limit.
 
 @example
 ```ts
 type ThreeSlots = TupleOfLength<3>;
 ```
 */
export type TupleOfLength<
  Length extends number,
  Accumulator extends readonly unknown[] = readonly [unknown],
  > = DropAccumulatorMarker<Accumulator>['length'] extends Length
    ? DropAccumulatorMarker<Accumulator>
    : TupleOfLength<Length, readonly [
      ...Accumulator,
      unknown,
    ]>;

/**
 Converts decimal character into numeric literal.
 
 @example
 ```ts
 type Seven = DigitValue<'7'>;
 ```
 */
export type DigitValue<Character extends string> =
  Character extends `${infer Value extends number}`
    ? Value
    : never;

/**
 Splits bounded decimal text into character tuple.
 
 Safe integers have at most sixteen decimal digits, which bounds recursion.
 
 @example
 ```ts
 type Digits = DecimalCharacters<'407'>;
 ```
 */
export type DecimalCharacters<
  Text extends string,
  Accumulator extends readonly [
    unknown,
    ...string[]
  ] = readonly [unknown],
  > = Text extends `${infer Character}${infer Rest}`
    ? DecimalCharacters<Rest, readonly [
      ...Accumulator,
      Character,
    ]>
    : Accumulator extends readonly [
      unknown,
      ...infer Characters extends readonly string[],
    ]
      ? Characters
      : never;

//endregion Decimal primitives

//region Subtraction

/**
 Subtracts one decimal column and reports outgoing borrow.
 
 @example
 ```ts
 type Difference = SubtractDigit<2, 7, 0>;
 ```
 */
export type SubtractDigit<
  Top extends number,
  Bottom extends number,
  Borrowed extends Borrow,
  > = TupleOfLength<Top> extends readonly [
    ...TupleOfLength<Bottom>,
    ...TupleOfLength<Borrowed>,
    ...infer Remainder,
  ]
    ? readonly [
      Remainder['length'],
      0,
    ]
    : readonly [
      ...TupleOfLength<Top>,
      ...TupleOfLength<10>,
    ] extends readonly [
      ...TupleOfLength<Bottom>,
      ...TupleOfLength<Borrowed>,
      ...infer Remainder,
    ]
      ? readonly [
        Remainder['length'],
        1,
      ]
      : never;

/**
 Performs right-to-left long subtraction over bounded decimal digits.
 
 `never` means bottom value exceeds top value. Result retains leading zeros
 for a later normalization step.
 
 @example
 ```ts
 type Difference = SubtractDigitSequences<
   readonly ['4', '0', '7'],
   readonly ['9'],
 >;
 ```
 */
export type SubtractDigitSequences<
  Top extends readonly string[],
  Bottom extends readonly string[],
  Borrowed extends Borrow = 0,
  Accumulator extends string = '',
  > = Top extends readonly [
    ...infer TopInitial extends readonly string[],
    infer TopLast extends string,
  ]
    ? Bottom extends readonly [
      ...infer BottomInitial extends readonly string[],
      infer BottomLast extends string,
    ]
      ? SubtractDigit<
        DigitValue<TopLast>,
        DigitValue<BottomLast>,
        Borrowed
        > extends readonly [
          infer Difference extends number,
          infer NextBorrow extends Borrow,
        ]
        ? SubtractDigitSequences<
          TopInitial,
          BottomInitial,
          NextBorrow,
          `${Difference}${Accumulator}`
          >
        : never
      : SubtractDigit<DigitValue<TopLast>, 0, Borrowed> extends readonly [
        infer Difference extends number,
        infer NextBorrow extends Borrow,
      ]
        ? SubtractDigitSequences<
          TopInitial,
          Bottom,
          NextBorrow,
          `${Difference}${Accumulator}`
          >
        : never
    : Bottom extends readonly [
      string,
      ...string[],
    ]
      ? never
      : Borrowed extends 0
        ? Accumulator
        : never;

//endregion Subtraction
