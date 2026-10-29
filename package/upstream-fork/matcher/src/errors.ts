/**
 Configuration errors thrown by the wildcard matcher.
 
 Both classes extend `TypeError` to keep the rejection shapes upstream
 `matcher` callers already handle (`instanceof TypeError` keeps holding).
 
 @module
 */

//region Errors

/**
 Thrown when an `inputs` value is not a string, a string list, or `undefined`,
 or when an `inputs` list holds a non-string entry.
 
 Carries upstream `matcher`'s message text verbatim, so migrating callers see
 identical diagnostics.
 
 @example
 ```ts
 import { InvalidInputsError, matcher, } from '\@monochromatic-dev/upstream-fork-matcher';
 
 try {
   matcher(1, 'a',);
 }
 catch (error) {
   error instanceof InvalidInputsError; // => true
 }
 ```
 */
export class InvalidInputsError extends TypeError {
  /**
   Builds the error with upstream `matcher`'s message text so callers
   migrating from `matcher` see identical diagnostics.
   
   @param value - Offending `inputs` value, rendered into the message.
   
   @param element - Offending list entry, rendered into the message when the
   list itself is an array.
   */
  constructor(
    {
      value,
      element,
    }: {
      readonly value?: unknown;
      readonly element?: unknown;
    } = {},
  ) {
    super(
      (element === undefined)
        ? `Expected 'inputs' to be a string or an array, but got a type of '${typeof value}'`
        : `Expected 'inputs' to be an array of strings, but found a type of '${typeof element}' in the array`,
    );
    this.name = 'InvalidInputsError';
  }
}

/**
 Thrown when a `patterns` value is not a string, a string list, or `undefined`,
 or when a `patterns` list holds a non-string entry.
 
 Carries upstream `matcher`'s message text verbatim, so migrating callers see
 identical diagnostics.
 
 @example
 ```ts
 import { InvalidPatternsError, matcher, } from '\@monochromatic-dev/upstream-fork-matcher';
 
 try {
   matcher('a', 1,);
 }
 catch (error) {
   error instanceof InvalidPatternsError; // => true
 }
 ```
 */
export class InvalidPatternsError extends TypeError {
  /**
   Builds the error with upstream `matcher`'s message text so callers
   migrating from `matcher` see identical diagnostics.
   
   @param value - Offending `patterns` value, rendered into the message.
   
   @param element - Offending list entry, rendered into the message when the
   list itself is an array.
   */
  constructor(
    {
      value,
      element,
    }: {
      readonly value?: unknown;
      readonly element?: unknown;
    } = {},
  ) {
    super(
      (element === undefined)
        ? `Expected 'patterns' to be a string or an array, but got a type of '${typeof value}'`
        : `Expected 'patterns' to be an array of strings, but found a type of '${typeof element}' in the array`,
    );
    this.name = 'InvalidPatternsError';
  }
}

//endregion Errors
