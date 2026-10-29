/**
 Throws helper implementation.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Throws an `Error` from expression position.
 
 Takes a prebuilt `Error` value so callers keep stack, message, cause, and
 subclass identity under their own domain policy. This helper is useful in
 expression-only slots such as nullish fallbacks, default parameter
 initializers, destructuring defaults, and class field initializers.
 Its `never` return type tells TypeScript that control cannot continue past
 the fallback branch.
 
 @param error - Error instance to throw from expression position
 
 @throws Same `Error` instance passed by caller
 
 @example
 ```ts
 const token = maybeToken ?? throws(new MissingTokenError(),);
 ```
 
 @example
 ```ts
 function readConfig(path = throws(new MissingPathError(),),): Config {
   return parseConfig(path,);
 }
 ```
 */
export function throws(error: ForeignBorrowed<Error>,): never {
  throw error;
}
