/**
 Tests for function arity wrappers.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  binary,
  unary,
} from '@monochromatic-dev/function-arity-core';

await describe({
  name: 'function-arity',
  children: [
    describe({
      name: unary.name,
      children: [
        it({
          name: 'forwards only first positional argument',
          fn: async () => {
            /** Radix presence values observed by wrapped parser. */
            const observedRadixPresence: string[] = [];
            /** Parser that records whether iterator index reaches radix. */
            const parseWithObservedRadix = (
              value: string,
              radix?: number,
            ): number => {
              observedRadixPresence.push((radix === undefined) ? 'missing' : 'present',);
              return Number.parseInt(
                value,
                radix,
              );
            };

            /** Parsed values through unary wrapper. */
            const parsed = ['10', '10', '10',].map(unary(parseWithObservedRadix,),);

            expect(parsed,).toEqual([10, 10, 10,],);
            expect(observedRadixPresence,).toEqual(['missing', 'missing', 'missing',],);
          },
        },),
        it({
          name: 'preserves unary type shape',
          fn: async () => {
            /** Parser used for type inference. */
            const parseWithOptionalRadix = (
              value: string,
              radix?: number,
            ): number => Number.parseInt(
              value,
              radix,
            );

            expectTypeOf(unary(parseWithOptionalRadix,),).toEqualTypeOf<(
              argument: string,
            ) => number>();
            expectTypeOf(unary(Number.parseInt,),).toEqualTypeOf<(
              argument: string,
            ) => number>();
          },
        },),
      ],
    },),
    describe({
      name: binary.name,
      children: [
        it({
          name: 'forwards only first two positional arguments',
          fn: async () => {
            /** Source collection presence values observed by wrapped renderer. */
            const observedCollectionPresence: string[] = [];
            /** Renderer that records whether iterator source collection reaches callback. */
            const renderWithObservedCollection = (
              value: string,
              index: number,
              collection?: readonly string[],
            ): string => {
              observedCollectionPresence.push((collection === undefined) ? 'missing' : 'present',);
              return `${index}:${value}`;
            };

            /** Rendered values through binary wrapper. */
            const rendered = ['a', 'b',].map(binary(renderWithObservedCollection,),);

            expect(rendered,).toEqual(['0:a', '1:b',],);
            expect(observedCollectionPresence,).toEqual(['missing', 'missing',],);
          },
        },),
        it({
          name: 'preserves binary type shape',
          fn: async () => {
            /** Renderer used for type inference. */
            const renderWithOptionalCollection = (
              value: string,
              index: number,
              _collection?: readonly string[],
            ): string => `${index}:${value}`;

            expectTypeOf(binary(renderWithOptionalCollection,),).toEqualTypeOf<(
              firstArgument: string,
              secondArgument: number,
            ) => string>();
          },
        },),
      ],
    },),
  ],
},);
