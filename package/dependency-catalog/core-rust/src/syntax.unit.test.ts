/**
 Unit tests for the built crate-name and version-requirement validators.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  floorVersion,
  isBoundedRequirement,
  isCrateName,
  isFloorRequirement,
  isVersion,
} from '../dist/final/node/index.mjs';

/**
 Applies a one-argument validator to every input.

 @param validate - validator under test

 @param inputs - strings to validate

 @returns one verdict per input, in order

 @example
 ```ts
 verdicts({ validate: isCrateName, inputs: ['anyhow',], }); // [true]
 ```
 */
function verdicts(
  {
    validate,
    inputs,
  }: {
    readonly validate: (input: string) => boolean;
    readonly inputs: readonly string[];
  },
): readonly boolean[] {
  return inputs.map(function verdict(input,): boolean {
    return validate(input,);
  },);
}

await describe({
  name: 'syntax',
  children: [
    it({
      name: isCrateName.name,
      fn: async () => {
        /**
         Names Cargo and crates.io accept, including the 64-character limit.
         */
        const accepted = ['anyhow', 'tracing-subscriber', 'ra_ap_hir', 'a', 'a'.repeat(64,),];
        /**
         Names to reject: empty, digit or symbol start, too long, separators, traversal, newline, non-ASCII.
         */
        const rejected = ['', '9lives', '_x', '-x', 'a'.repeat(65,), 'a b', 'a.b', '../x', 'a\nb', 'é', '\u{1F980}',];
        expect(verdicts({ validate: isCrateName, inputs: accepted, },),).toEqual(accepted.map(() => true),);
        expect(verdicts({ validate: isCrateName, inputs: rejected, },),).toEqual(rejected.map(() => false),);
      },
    },),

    it({
      name: isVersion.name,
      fn: async () => {
        /**
         Partial, full, prerelease, and build versions.
         */
        const accepted = ['1', '0.4', '1.0.104', '1.0.0-alpha.5', '1.0.0+build.7', '1.0.0-rc.1+b',];
        /**
         Malformed versions.
         */
        const rejected = ['', '1.x', '01', '1.02', '1.2.3.4', '1.0.0-', '1.0.0+', 'v1', '1..2', '1.0.0-a b',];
        expect(verdicts({ validate: isVersion, inputs: accepted, },),).toEqual(accepted.map(() => true),);
        expect(verdicts({ validate: isVersion, inputs: rejected, },),).toEqual(rejected.map(() => false),);
      },
    },),

    it({
      name: isFloorRequirement.name,
      fn: async () => {
        /**
         Single `>=` comparators.
         */
        const accepted = ['>=1', '>=1.0.104', '>= 0.4',];
        /**
         Bare, strict, compound, empty, upper-bound, and wildcard requirements.
         */
        const rejected = ['1', '>1', '>=1, <2', '>=', '<=1', '*',];
        expect(verdicts({ validate: isFloorRequirement, inputs: accepted, },),).toEqual(accepted.map(() => true),);
        expect(verdicts({ validate: isFloorRequirement, inputs: rejected, },),).toEqual(rejected.map(() => false),);
      },
    },),

    it({
      name: isBoundedRequirement.name,
      fn: async () => {
        /**
         Caret, explicit caret, tilde, and exact requirements.
         */
        const accepted = ['0.58', '^0.58', '~0.58.1', '=1.18.1',];
        /**
         Unbounded, wildcard, upper-only, compound, and empty requirements.
         */
        const rejected = ['>=0.58', '*', '<1', '0.58, <1', '',];
        expect(verdicts({ validate: isBoundedRequirement, inputs: accepted, },),).toEqual(accepted.map(() => true),);
        expect(verdicts({ validate: isBoundedRequirement, inputs: rejected, },),).toEqual(rejected.map(() => false),);
      },
    },),

    it({
      name: floorVersion.name,
      fn: async () => {
        expect(floorVersion('>=1.0.104',),).toBe('1.0.104',);
        try {
          floorVersion('1.0.104',);
        } catch (error) {
          expect(error,).toBeInstanceOf(TypeError,);
          return;
        }
        throw new Error('expected a TypeError',);
      },
    },),
  ],
},);
