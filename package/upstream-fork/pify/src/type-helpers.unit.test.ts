/**
 Tests for the tuple and string type helpers behind the promisified result
 computation.
 
 @module
 */

import {
  describe,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  DropLastArrayElement,
  EmptyTuple,
  LastArrayElement,
  StringEndsWith,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'type-helpers',
  children: [
    it({
      name: 'LastArrayElement reads the trailing callback slot',
      fn: async () => {
        expectTypeOf<
          LastArrayElement<[string, (error: unknown) => void]>
          >().toEqualTypeOf<(error: unknown) => void>();
        expectTypeOf<LastArrayElement<EmptyTuple>>().toEqualTypeOf<never>();
      },
    },),

    it({
      name: 'DropLastArrayElement keeps the caller argument tuple',
      fn: async () => {
        expectTypeOf<
          DropLastArrayElement<[string, (error: unknown) => void]>
          >().toEqualTypeOf<[string]>();
        expectTypeOf<DropLastArrayElement<[string]>>().toEqualTypeOf<EmptyTuple>();
      },
    },),

    it({
      name: 'StringEndsWith classifies suffix unions and non-string keys',
      fn: async () => {
        expectTypeOf<
          StringEndsWith<'readFileSync', 'Sync' | 'Stream'>
          >().toEqualTypeOf<true>();
        expectTypeOf<
          StringEndsWith<'readFile', 'Sync' | 'Stream'>
          >().toEqualTypeOf<false>();
        expectTypeOf<
          StringEndsWith<symbol, 'Sync' | 'Stream'>
          >().toEqualTypeOf<false>();
      },
    },),
  ],
},);
