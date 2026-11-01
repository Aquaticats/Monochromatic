/**
 Characterization tests for exotic inputs whose behaviour follows from the
 documented semantics (records merge, everything else resolves to the last
 value, deepmergeInto mutates the target in place) or from JavaScript itself.

 They pin every branch of upstream's `isRecord` heuristic in `src/utils.ts`,
 foreign-realm records, `arguments` objects, Map and Set key equality, and
 how deepmergeInto meets non-writable, accessor, sealed, frozen, and Proxy
 targets, so an upstream change in any of them is noticed.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { foreign, } from './exotic-arbitraries.ts';
import { target, } from './target.ts';

/**
 Build a record from a prototype and own enumerable data.

 @param prototype - Prototype of the new object.

 @param data - Own properties to copy on.

 @returns Object with the given prototype and data.

 @example
 ```ts
 withPrototype({ prototype: null, data: { a: 1, }, });
 ```
 */
function withPrototype({
  prototype,
  data,
}: {
  readonly prototype: object;
  readonly data: Readonly<Record<string, unknown>>;
},): object {
  return Object.assign(Object.create(prototype,) as object, data,);
}

/**
 Return this call's `arguments` object, the input under test; called through
 `Reflect.apply` so it declares no parameters.

 @returns Strict-mode `arguments` object of the call.

 @example
 ```ts
 Reflect.apply(collectArguments, undefined, [1, 2,]);
 ```
 */
function collectArguments(): unknown {
  // oxlint-disable-next-line prefer-rest-params -- the arguments object itself is the input under test.
  return arguments;
}

/**
 Merge `input` with `{ b: 2 }` and report whether it merged as a record.

 @param input - Candidate record.

 @returns Whether the result carries both the input's keys and `b`.

 @example
 ```ts
 mergedAsRecord({ a: 1, }); // true
 ```
 */
function mergedAsRecord(input: object,): boolean {
  /**
   Merge result; a leaf input makes it `{ b: 2 }` itself.
   */
  const merged: unknown = target.deepmerge(input, { b: 2, },);
  return ((typeof merged) === 'object') && (merged !== null) && Object.hasOwn(merged, 'a',) && Object.hasOwn(merged, 'b',);
}

await describe({
  name: 'deepmerge-ts exotic accepted behaviour',
  children: [
    it({
      name: 'isRecord: prototype chains and constructor shapes decide record versus leaf',
      fn: async () => {
        expect(
          mergedAsRecord(withPrototype({ data: { a: 1, }, prototype: Object.create(null,) as object, },),),
        ).toBe(true,);
        expect(
          mergedAsRecord(withPrototype({ data: { a: 1, }, prototype: { inherited: 1, }, },),),
        ).toBe(true,);
        expect(target.deepmerge(withPrototype({ data: { a: 1, }, prototype: { inherited: 1, }, },), {},),).toEqual({ a: 1, },);
        expect(
          mergedAsRecord(withPrototype({ data: { a: 1, }, prototype: { constructor: 5, }, },),),
        ).toBe(false,);
        expect(
          mergedAsRecord(withPrototype({ data: { a: 1, }, prototype: { constructor: {}, }, },),),
        ).toBe(false,);
        expect(
          mergedAsRecord(withPrototype({ data: { a: 1, }, prototype: { constructor: { prototype: null, }, }, },),),
        ).toBe(false,);
        expect(
          mergedAsRecord(withPrototype({ data: { a: 1, }, prototype: { isPrototypeOf: Reflect.get(Object.prototype, 'isPrototypeOf',), }, },),),
        )
          .toBe(true,);
        expect(mergedAsRecord({ a: 1, [Symbol.toStringTag]: 'Custom', },),).toBe(true,);
        expect(
          mergedAsRecord(withPrototype({ data: { a: 1, }, prototype: { [Symbol.toStringTag]: 'Custom', }, },),),
        ).toBe(false,);
      },
    },),
    it({
      name: 'foreign-realm records and arguments objects merge as records into plain objects',
      fn: async () => {
        /**
         Record built by another realm's Object.
         */
        const merged = target.deepmerge(foreign('({a:{x:1}})',), foreign('({a:{y:2}})',),);
        expect(Object.getPrototypeOf(merged,),).toBe(Object.prototype,);
        expect(JSON.stringify(merged,),).toBe('{"a":{"x":1,"y":2}}',);
        /**
         Strict-mode arguments object with two indexed entries.
         */
        const args: unknown = Reflect.apply(collectArguments, undefined, [1, 2,],);
        if (((typeof args) !== 'object') || (args === null))
          throw new TypeError('arguments object expected',);
        expect(target.deepmerge(args, { 2: 3, },),).toEqual({ 0: 1, 1: 2, 2: 3, },);
      },
    },),
    it({
      name: 'Map keys and Set elements compare by SameValueZero',
      fn: async () => {
        expect([...target.deepmerge(new Map([[-0, { a: 1, },],],), new Map([[0, { b: 2, },],],),),],).toEqual([[0, { a: 1, b: 2, },],],);
        expect([...target.deepmerge(new Map([[Number.NaN, { a: 1, },],],), new Map([[Number.NaN, { b: 2, },],],),),],)
          .toEqual([[Number.NaN, { a: 1, b: 2, },],],);
        /**
         Object used as a key in both Maps.
         */
        const key = {};
        expect(target.deepmerge(new Map([[key, { a: 1, },],],), new Map([[key, { b: 2, },],],),).get(key,),).toEqual({ a: 1, b: 2, },);
        expect([...target.deepmerge(new Set([-0, Number.NaN,],), new Set([0, Number.NaN,],),),],).toEqual([0, Number.NaN,],);
      },
    },),
    it({
      name: 'deepmergeInto meets target property attributes with ordinary assignment semantics',
      fn: async () => {
        /**
         Target whose key is non-writable.
         */
        const readOnly = Object.defineProperty({}, 'a', { configurable: true, enumerable: true, value: { x: 1, }, writable: false, },);
        expect(() => target.deepmergeInto(readOnly, { a: { y: 2, }, },),).toThrow(TypeError,);
        expect(() => target.deepmergeInto({ get a(): object {
          return { x: 1, };
        }, }, { a: { y: 2, }, },),).toThrow(TypeError,);
        expect(() => target.deepmergeInto(Object.seal({ a: 1, },), { b: 2, },),).toThrow(TypeError,);
        expect(() => target.deepmergeInto(Object.preventExtensions({ a: 1, },), { b: 2, },),).toThrow(TypeError,);
        expect(() => target.deepmergeInto({ a: Object.freeze({ x: 1, },), }, { a: { y: 2, }, },),).toThrow(TypeError,);
        expect(() => target.deepmergeInto({ a: Object.freeze([1,],), }, { a: [2,], },),).toThrow(TypeError,);
        /**
         Sealed target whose existing key still merges in place.
         */
        const sealed = Object.seal({ a: { x: 1, }, },);
        target.deepmergeInto(sealed, { a: { y: 2, }, },);
        expect(sealed,).toEqual({ a: { x: 1, y: 2, }, },);
        /**
         Frozen Set: freezing does not cover its contents, so merging still adds to it.
         */
        const frozenSet = Object.freeze(new Set([1,],),);
        target.deepmergeInto({ s: frozenSet, }, { s: new Set([2,],), },);
        expect([...frozenSet,],).toEqual([1, 2,],);
      },
    },),
    it({
      name: 'deepmergeInto writes setter-only and hidden target keys without merging their old value',
      fn: async () => {
        /**
         Value received by the setter.
         */
        const received: unknown[] = [];
        /**
         Target whose key has only an enumerable setter.
         */
        const setterOnly = Object.defineProperty({}, 'a', {
          configurable: true,
          enumerable: true,
          // oxlint-disable-next-line eslint/accessor-pairs -- a setter-only target key is the case under test.
          set: function receive(value: unknown,) {
            received.push(value,);
          },
        },);
        target.deepmergeInto(setterOnly, { a: { y: 2, }, },);
        expect(received,).toEqual([{ y: 2, },],);
        /**
         Target whose key is non-enumerable, so it does not take part in the merge.
         */
        const hidden = Object.defineProperty({}, 'a', { configurable: true, enumerable: false, value: { x: 1, }, writable: true, },);
        target.deepmergeInto(hidden, { a: { y: 2, }, },);
        expect(Object.getOwnPropertyDescriptor(hidden, 'a',),).toEqual({ configurable: true, enumerable: false, value: { y: 2, }, writable: true, },);
      },
    },),
    it({
      name: 'Proxy traps are honoured and their errors propagate',
      fn: async () => {
        expect(() => target.deepmerge(new Proxy({}, { ownKeys: function refuse(): never {
          throw new RangeError('ownKeys trap',);
        }, },), { b: 2, },),).toThrow(RangeError,);
        /**
         Revoked Proxy, which throws on every operation.
         */
        const revocable = Proxy.revocable({ a: 1, }, {},);
        revocable.revoke();
        expect(() => target.deepmerge(revocable.proxy, { b: 2, },),).toThrow(TypeError,);
        /**
         Record reached through a transparent Proxy as the deepmergeInto target.
         */
        const backing = { a: { x: 1, }, };
        target.deepmergeInto(new Proxy(backing, {},), { a: { y: 2, }, },);
        expect(backing,).toEqual({ a: { x: 1, y: 2, }, },);
      },
    },),
  ],
},);
