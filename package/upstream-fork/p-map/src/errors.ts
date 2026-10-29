/**
 Configuration errors thrown by the concurrent mappers.
 
 Every class extends `TypeError` and carries upstream `p-map`'s message text
 verbatim (including its value and `typeof` interpolations), so callers
 migrating from `p-map` see identical diagnostics and the fuzz sidecar's
 differential oracle can compare both implementations directly. Message
 construction uses the same template literals upstream uses, so a
 symbol-typed value still fails with `TypeError: Cannot convert a Symbol
 value to a string` instead of producing a message.
 
 @module
 */

//region Coercion

/**
 Renders one interpolated value the way a template literal coerces it.
 
 Upstream `p-map` interpolates raw option values into its validation
 messages, so a symbol-typed value throws
 `TypeError: Cannot convert a Symbol value to a string` while the message is
 built instead of producing text. `String()` would silently render symbols,
 so the fork keeps the template-literal coercion exactly.
 
 @param value - Value interpolated into a validation message.
 
 @returns The value rendered as a template literal renders it.
 
 @throws TypeError when the value is a symbol, like upstream `p-map`.
 
 @example
 ```ts
 templateCoerce(0,); // => '0'
 ```
 */
function templateCoerce(value: unknown,): string {
  // oxlint-disable-next-line typescript/restrict-template-expressions -- the coercion must be a template literal: upstream `p-map`'s message construction throws for symbol-typed values, and `String()` would silently render them instead
  return `${value}`;
}

//endregion Coercion

//region Errors

/**
 Thrown when the input is neither an `Iterable` nor an `AsyncIterable`.
 
 @example
 ```ts
 import { InvalidInputError, pMap, } from '\@monochromatic-dev/upstream-fork-p-map';
 
 try {
   await pMap({
     iterable: 5 as never,
     mapper: function identity(value: unknown,): unknown {
       return value;
     },
   });
 }
 catch (error) {
   error instanceof InvalidInputError; // => true
 }
 ```
 */
export class InvalidInputError extends TypeError {
  /**
   Builds the error with upstream `p-map`'s message text, interpolating the
   rejected input's runtime type the same way upstream does.
   
   @param input - Value that was neither iterable nor async iterable.
   */
  constructor(input: unknown,) {
    super(`Expected \`input\` to be either an \`Iterable\` or \`AsyncIterable\`, got (${typeof input})`,);
    this.name = 'InvalidInputError';
  }
}

/**
 Thrown when the mapper is missing or not a function.
 
 @example
 ```ts
 import { MapperRequiredError, pMap, } from '\@monochromatic-dev/upstream-fork-p-map';
 
 try {
   await pMap({
     iterable: [1],
     mapper: undefined as never,
   });
 }
 catch (error) {
   error instanceof MapperRequiredError; // => true
 }
 ```
 */
export class MapperRequiredError extends TypeError {
  /**
   Builds the error with upstream `p-map`'s message text so callers
   migrating from `p-map` see identical diagnostics.
   */
  constructor() {
    super('Mapper function is required',);
    this.name = 'MapperRequiredError';
  }
}

/**
 Thrown when `concurrency` is not a safe integer from 1 and up and not
 `Number.POSITIVE_INFINITY`.
 
 @example
 ```ts
 import { InvalidConcurrencyError, pMap, } from '\@monochromatic-dev/upstream-fork-p-map';
 
 try {
   await pMap({
     iterable: [1],
     mapper: function identity(value: number,): number {
       return value;
     },
     options: { concurrency: 0, },
   });
 }
 catch (error) {
   error instanceof InvalidConcurrencyError; // => true
 }
 ```
 */
export class InvalidConcurrencyError extends TypeError {
  /**
   Builds the error with upstream `p-map`'s message text, interpolating the
   rejected bound's value and runtime type the same way upstream does.
   
   @param concurrency - Rejected concurrency bound.
   */
  constructor(concurrency: unknown,) {
    super(`Expected \`concurrency\` to be an integer from 1 and up or \`Infinity\`, got \`${templateCoerce(concurrency,)}\` (${typeof concurrency})`,);
    this.name = 'InvalidConcurrencyError';
  }
}

/**
 Thrown when `backpressure` is not a safe integer from `concurrency` and up
 and not `Number.POSITIVE_INFINITY`.
 
 @example
 ```ts
 import { InvalidBackpressureError, pMapIterable, } from '\@monochromatic-dev/upstream-fork-p-map';
 
 try {
   pMapIterable({
     iterable: [1],
     mapper: function identity(value: number,): number {
       return value;
     },
     options: {
       concurrency: 4,
       backpressure: 2,
     },
   });
 }
 catch (error) {
   error instanceof InvalidBackpressureError; // => true
 }
 ```
 */
export class InvalidBackpressureError extends TypeError {
  /**
   Builds the error with upstream `p-map`'s message text, interpolating the
   resolved bound and the rejected backpressure's value and runtime type the
   same way upstream does.
   
   @param backpressure - Rejected backpressure bound, interpolated into the
   message.
   
   @param concurrency - Resolved concurrency bound named in the message.
   */
  constructor(
    {
      backpressure,
      concurrency,
    }: {
      /**
       Rejected backpressure bound, interpolated into the message.
       */
      readonly backpressure: unknown;
      /**
       Resolved concurrency bound, interpolated into the message.
       */
      readonly concurrency: number;
    },
  ) {
    super(`Expected \`backpressure\` to be an integer from \`concurrency\` (${concurrency}) and up or \`Infinity\`, got \`${templateCoerce(backpressure,)}\` (${typeof backpressure})`,);
    this.name = 'InvalidBackpressureError';
  }
}

/**
 Thrown when the input of a keyed map is not an object or a `Map`.

 @example
 ```ts
 import { InvalidKeyedInputError, pMapKeyed, } from '\@monochromatic-dev/upstream-fork-p-map';

 try {
   await pMapKeyed({
     input: 5 as never,
     mapper: function identity(value: unknown,): unknown {
       return value;
     },
   });
 }
 catch (error) {
   error instanceof InvalidKeyedInputError; // => true
 }
 ```
 */
export class InvalidKeyedInputError extends TypeError {
  /**
   Builds the error with upstream `p-map`'s message text, naming `null`
   explicitly because `typeof null` is `'object'`.

   @param input - Value that was not an object or a `Map`.
   */
  constructor(input: unknown,) {
    super(`Expected \`input\` to be an object, got (${input === null ? 'null' : typeof input})`,);
    this.name = 'InvalidKeyedInputError';
  }
}

//endregion Errors
