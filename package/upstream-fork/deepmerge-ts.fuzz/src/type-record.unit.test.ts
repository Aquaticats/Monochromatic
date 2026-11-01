/**
 Type-level tests for record merging through `deepmerge`.

 Each case pins the static result type with `expectTypeOf` (checked by
 `lint:types`) and the runtime value with `expect`, so the pair shows the
 type describes what the call returns. Divergences where the type claims
 something the runtime contradicts live in `./type-known-defect.unit.test.ts`.

 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { target, } from './target.ts';

await describe({
  name: 'deepmerge record result types',
  children: [
    it({
      name: 'arity: zero inputs is undefined, one input is itself',
      fn: async () => {
        const none = target.deepmerge();
        expectTypeOf(none,).toEqualTypeOf<undefined>();
        expect(none,).toBeUndefined();

        const one = target.deepmerge({ a: 1, },);
        expectTypeOf(one,).toEqualTypeOf<{ a: number; }>();
        expect(one,).toEqual({ a: 1, },);
      },
    },),
    it({
      name: 'disjoint keys union and overlapping scalars take the last type',
      fn: async () => {
        const merged = target.deepmerge({ a: 1, shared: 'x', }, { b: true, shared: 2, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: number; shared: number; b: boolean; }>();
        expect(merged,).toEqual({ a: 1, shared: 2, b: true, },);
      },
    },),
    it({
      name: 'literal types survive with as const and the last literal wins',
      fn: async () => {
        const merged = target.deepmerge({ mode: 'a', n: 1, } as const, { mode: 'b', } as const,);
        expectTypeOf(merged,).toEqualTypeOf<{ mode: 'b'; n: 1; }>();
        expect(merged,).toEqual({ mode: 'b', n: 1, },);
      },
    },),
    it({
      name: 'nested records merge recursively in the type',
      fn: async () => {
        const merged = target.deepmerge({ r: { x: 1, deep: { p: 'p', }, }, }, { r: { y: 'y', deep: { q: true, }, }, },);
        expectTypeOf(merged,).toEqualTypeOf<{ r: { x: number; deep: { p: string; q: boolean; }; y: string; }; }>();
        expect(merged,).toEqual({ r: { x: 1, deep: { p: 'p', q: true, }, y: 'y', }, },);
      },
    },),
    it({
      name: 'three and four inputs fold left to right',
      fn: async () => {
        const three = target.deepmerge({ a: 1, }, { a: 'two', b: 2, }, { b: false, c: null, },);
        expectTypeOf(three,).toEqualTypeOf<{ a: string; b: boolean; c: null; }>();
        expect(three,).toEqual({ a: 'two', b: false, c: null, },);

        const four = target.deepmerge({ w: 1, }, { x: 2, }, { y: 3, }, { z: 4, },);
        expectTypeOf(four,).toEqualTypeOf<{ w: number; x: number; y: number; z: number; }>();
      },
    },),
    it({
      name: 'an undefined-typed property is filtered and keeps the earlier type',
      fn: async () => {
        const merged = target.deepmerge({ a: 'kept', }, { a: undefined, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: string; }>();
        expect(merged,).toEqual({ a: 'kept', },);
      },
    },),
    it({
      name: 'null is a value, not a filter',
      fn: async () => {
        const merged = target.deepmerge({ a: { x: 1, }, }, { a: null, },);
        expectTypeOf(merged,).toEqualTypeOf<{ a: null; }>();
        expect(merged,).toEqual({ a: null, },);
      },
    },),
    it({
      name: 'optional keys: required anywhere stays required, optional everywhere stays optional',
      fn: async () => {
        const required: { a: number; } = { a: 1, };
        const optional: { a?: string; } = {};
        const requiredThenOptional = target.deepmerge(required, optional,);
        expectTypeOf(requiredThenOptional,).toEqualTypeOf<{ a: number | string; }>();
        expect(requiredThenOptional,).toEqual({ a: 1, },);

        const optionalNumber: { a?: number; } = {};
        const bothOptional = target.deepmerge(optionalNumber, optional,);
        expectTypeOf(bothOptional,).toEqualTypeOf<{ a?: number | string; }>();
        expect(bothOptional,).toEqual({},);
      },
    },),
    it({
      name: 'readonly properties merge into mutable result properties',
      fn: async () => {
        const first: Readonly<{ a: number; }> = { a: 1, };
        const second: Readonly<{ b: string; }> = { b: 'b', };
        const merged = target.deepmerge(first, second,);
        expectTypeOf(merged,).toEqualTypeOf<{ a: number; b: string; }>();
      },
    },),
    it({
      name: 'identical input types short-circuit to that type',
      fn: async () => {
        type Settings = { name: string; tags: string[]; nested: { on: boolean; }; };
        const left: Settings = { name: 'l', tags: ['a',], nested: { on: true, }, };
        const right: Settings = { name: 'r', tags: ['b',], nested: { on: false, }, };
        const merged = target.deepmerge(left, right,);
        expectTypeOf(merged,).toEqualTypeOf<Settings>();
        expect(merged,).toEqual({ name: 'r', tags: ['a', 'b',], nested: { on: false, }, },);
      },
    },),
    it({
      name: 'symbol and numeric keys are tracked like string keys',
      fn: async () => {
        const tag: unique symbol = Symbol('type-level symbol key probe',);
        const merged = target.deepmerge({ [tag]: 1, 0: 'zero', }, { [tag]: 'one', 1: true, },);
        expectTypeOf(merged,).toEqualTypeOf<{ [tag]: string; 0: string; 1: boolean; }>();
        expect(merged[tag],).toBe('one',);
      },
    },),
    it({
      name: 'function-valued properties are leaves and the last one wins',
      fn: async () => {
        const merged = target.deepmerge({ run: (x: number,): number => x, }, { run: (x: string,): string => x, },);
        expectTypeOf(merged,).toEqualTypeOf<{ run: (x: string,) => string; }>();
        expect(merged.run('k',),).toBe('k',);
      },
    },),
    it({
      name: 'a kind mismatch at a key takes the last type',
      fn: async () => {
        const merged = target.deepmerge({ v: [1,], }, { v: { a: 1, }, },);
        expectTypeOf(merged,).toEqualTypeOf<{ v: { a: number; }; }>();
        expect(merged,).toEqual({ v: { a: 1, }, },);

        const toScalar = target.deepmerge({ v: { a: 1, }, }, { v: 'flat', },);
        expectTypeOf(toScalar,).toEqualTypeOf<{ v: string; }>();
      },
    },),
    it({
      name: 'unknown-length argument lists degrade to unknown',
      fn: async () => {
        const list: readonly { a: number; }[] = [{ a: 1, }, { a: 2, },];
        const merged = target.deepmerge(...list,);
        expectTypeOf(merged,).toEqualTypeOf<unknown>();
        expect(merged,).toEqual({ a: 2, },);
      },
    },),
  ],
},);
