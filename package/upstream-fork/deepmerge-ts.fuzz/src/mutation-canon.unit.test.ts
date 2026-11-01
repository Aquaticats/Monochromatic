/**
 Example tests for `./mutation-canon.ts`: each observable difference the
 differential check relies on changes the serialization, and equal outcomes
 serialize equally.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  canonicalize,
  labelInputs,
  outcomeOf,
} from './mutation-canon.ts';

/**
 Serialize `roots` against `inputs` labelled first.

 @param inputs - Inputs before the call.

 @param roots - Values after the call.

 @returns Canonical text.

 @example
 ```ts
 serialize({ inputs: [], roots: [1,], });
 ```
 */
function serialize(
  {
    inputs,
    roots,
  }: {
    readonly inputs: readonly unknown[];
    readonly roots: readonly unknown[];
  },
): string {
  return canonicalize({
    labels: labelInputs(inputs,),
    roots,
  },);
}

await describe({
  name: 'mutation canonical outcome',
  children: [
    it({
      name: 'equal fresh graphs serialize equally, and a changed leaf or key order does not',
      fn: async () => {
        expect(serialize({ inputs: [], roots: [{ a: [1,], },], },),).toBe(serialize({ inputs: [], roots: [{ a: [1,], },], },),);
        expect(serialize({ inputs: [], roots: [{ a: [1,], },], },),).not.toBe(serialize({ inputs: [], roots: [{ a: [2,], },], },),);
        expect(serialize({ inputs: [], roots: [{ a: 1, b: 2, },], },),).not.toBe(serialize({ inputs: [], roots: [{ b: 2, a: 1, },], },),);
      },
    },),
    it({
      name: 'an input node differs from an equal fresh copy, and a back reference from a copy',
      fn: async () => {
        /**
         Input node.
         */
        const shared = { v: 1, };
        expect(serialize({ inputs: [shared,], roots: [{ k: shared, },], },),)
          .not.toBe(serialize({ inputs: [shared,], roots: [{ k: { v: 1, }, },], },),);
        /**
         Self-referencing result.
         */
        const looped: Record<string, unknown> = {};
        looped.self = looped;
        expect(serialize({ inputs: [], roots: [looped,], },),).not.toBe(serialize({ inputs: [], roots: [{ self: {}, },], },),);
      },
    },),
    it({
      name: 'prototype, descriptor flags, collections, dates, and -0 are all observed',
      fn: async () => {
        /**
         Pairs that must serialize differently.
         */
        const pairs: readonly (readonly [unknown, unknown])[] = [
          [{}, Object.create(null,),],
          [Object.defineProperty({}, 'a', { enumerable: true, value: 1, },), { a: 1, },],
          [new Map([[1, 2,],],), new Map([[1, 3,],],),],
          [new Set([1,],), new Set([2,],),],
          [new Date(0,), new Date(1,),],
          [-0, 0,],
        ];
        expect(pairs.filter(function same([left, right,],) {
          return serialize({ inputs: [], roots: [left,], },) === serialize({ inputs: [], roots: [right,], },);
        },),).toEqual([],);
      },
    },),
    it({
      name: 'a getter is recorded without being invoked, and a throw serializes as its class',
      fn: async () => {
        /**
         Getter call count.
         */
        const calls = { count: 0, };
        /**
         Object with an accessor.
         */
        const accessor = Object.defineProperty({}, 'g', {
          enumerable: true,
          get: function read() {
            calls.count += 1;
            return 1;
          },
        },);
        serialize({ inputs: [], roots: [accessor,], },);
        expect(calls.count,).toBe(0,);
        expect(outcomeOf({
          inputs: [],
          run: function fail() {
            throw new RangeError('x',);
          },
        },),).toBe('throw:RangeError',);
      },
    },),
    it({
      name: 'input labels come from before the call, so a mutated input still reads as that input',
      fn: async () => {
        /**
         Input the call mutates.
         */
        const input: Record<string, unknown> = { a: 1, };
        /**
         Outcome where the call adds a key to the input.
         */
        const mutated = outcomeOf({
          inputs: [input,],
          run: function mutate() {
            input.b = 2;
            return [input,];
          },
        },);
        expect(mutated,).toContain('"origin":"input:0"',);
        expect(mutated,).toContain('string:b',);
      },
    },),
    it({
      name: 'a revoked Proxy is labelled and described by its error instead of throwing',
      fn: async () => {
        /**
         Revocable proxy, revoked before serialization.
         */
        const { proxy, revoke, } = Proxy.revocable({}, {},);
        revoke();
        expect(labelInputs([proxy,],).size,).toBe(1,);
        expect(serialize({ inputs: [proxy,], roots: [proxy,], },),).toContain('TypeError',);
      },
    },),
  ],
},);
