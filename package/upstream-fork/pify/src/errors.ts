/**
 Input validation errors thrown by `pify`.
 
 The class extends `TypeError` to keep the rejection shapes upstream `pify`
 callers already handle (`instanceof TypeError` keeps holding).
 
 Derived from [`pify`](https://github.com/sindresorhus/pify) by Sindre Sorhus
 (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. The message text matches `pify` 6.1.0 verbatim.
 
 @module
 */

//region Errors

/**
 Thrown when the `input` argument is neither a function nor an object.
 
 @example
 ```ts
 import { InvalidInputError, pify, } from '\@monochromatic-dev/upstream-fork-pify';
 
 try {
   pify({ input: 'not a function', },);
 }
 catch (error) {
   error instanceof InvalidInputError; // => true
 }
 ```
 */
export class InvalidInputError extends TypeError {
  /**
   Builds the error with upstream `pify`'s message text so callers migrating
   from `pify` see identical diagnostics.
   
   @param input - Rejected `pify` input, named in the message the way upstream
   spells it: `null` stays `null`, everything else reports its `typeof` string.
   */
  constructor(input: unknown,) {
    super(`Expected \`input\` to be a \`Function\` or \`Object\`, got \`${input === null ? 'null' : typeof input}\``,);
    this.name = 'InvalidInputError';
  }
}

//endregion Errors
