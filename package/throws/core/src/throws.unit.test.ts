/**
 Tests for `throws`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { throws, } from '../dist/final/neutral/index.mjs';

/**
 Calls `throws` and returns caught value for exact identity assertions.
 
 @param error - Error instance to pass through `throws`
 
 @returns Value caught from `throws`
 
 @throws Error if `throws` unexpectedly returns
 
 @example
 ```ts
 const caught = captureThrownError(new Error('boom',),);
 ```
 */
function captureThrownError(error: Error,): unknown {
  try {
    throws(error,);
  }
  catch (caught) {
    return caught;
  }

  throw new Error('Expected throws() to throw.',);
}

/**
 Options for token fallback expression type checks.
 
 @example
 ```ts
 const options: RequireTokenOptions = { value: 'present', };
 ```
 */
type RequireTokenOptions = {
  /** Token value when caller supplied one. */
  value?: string;
};

/**
 Requires a token through a nullish fallback expression.
 
 @param options - Candidate token options
 
 @returns Candidate token when present
 
 @throws Error when `options.value` is missing
 
 @example
 ```ts
 const token = requireToken({ value: 'present', },);
 ```
 */
function requireToken(options: RequireTokenOptions,): string {
  return options.value ?? throws(new Error('Missing token',),);
}

await describe({
  name: throws.name,
  children: [
    it({
      name: 'throws the exact same Error instance',
      fn: async () => {
        const error = new Error('missing value',);

        expect(captureThrownError(error,),).toBe(error,);
      },
    },),

    it({
      name: 'works in a nullish coalescing expression without widening the value type',
      fn: async () => {
        const token = requireToken({ value: 'present', },);

        expect(token,).toBe('present',);
        expectTypeOf(token,).toEqualTypeOf<string>();
      },
    },),

    it({
      name: 'exposes an Error-only parameter and never return type',
      fn: async () => {
        expectTypeOf<Parameters<typeof throws>[0]>()
          .toExtend<Error>();
        expectTypeOf<Error>()
          .toExtend<Parameters<typeof throws>[0]>();
        expectTypeOf<ReturnType<typeof throws>>().toEqualTypeOf<never>();
      },
    },),
  ],
},);

/**
 Compile-time assertion that string overloads are absent.
 
 @example
 ```ts
 rejectsStringInput();
 ```
 */
export function rejectsStringInput(): void {
  // @ts-expect-error - strings are rejected so thrown values keep Error metadata.
  throws('missing',);
}

/**
 Compile-time assertion that descriptor-object overloads are absent.
 
 @example
 ```ts
 rejectsDescriptorInput();
 ```
 */
export function rejectsDescriptorInput(): void {
  // @ts-expect-error - descriptor policy fields are rejected so custom Error subclasses stay faithful.
  throws({ code: 'MISSING_VALUE', message: 'missing', name: 'MissingValueError', },);
}
