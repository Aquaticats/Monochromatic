/**
 Record-level paths that upstream's suite and every earlier sidecar test left
 open, found by mutation testing
 (`doc/audit/deepmerge-ts-mutation-2026-09-24.md`).

 Each test pins 8.0.2 behaviour and names the Stryker mutant ids of that run
 it kills: `__proto__` key descriptors on every record path, setters on the
 three-value `deepmergeInto` path, the Module-tag and constructor-prototype
 branches of the plain-record check, the exported `objectHasProperty`, and
 the in-place Set merge of `deepmergeInto`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ownProto, } from './mutation-helper.ts';
import { target, } from './target.ts';

/**
 Descriptor every merged own `__proto__` key must have.

 @param value - Value stored under the key.

 @returns Fully enabled data descriptor.

 @example
 ```ts
 expect(Object.getOwnPropertyDescriptor(merged, '__proto__',),).toEqual(openDescriptor(inner,),);
 ```
 */
function openDescriptor(value: unknown,): PropertyDescriptor {
  return {
    configurable: true,
    enumerable: true,
    value,
    writable: true,
  };
}

/**
 Object inheriting from `prototype`, holding one own field.

 @param prototype - Prototype of the new object.

 @returns Object with own `a: 1`.

 @example
 ```ts
 const child = inheriting({ tag: 1, });
 ```
 */
function inheriting(prototype: object,): object {
  return Object.assign(Object.create(prototype,) as object, { a: 1, },);
}

/**
 Own `isPrototypeOf` for hand-built prototypes; the plain-record check only
 asks whether the key is own, never calls it.

 @returns Always false.

 @example
 ```ts
 Object.assign(prototype, { isPrototypeOf: isPrototypeOfStub, });
 ```
 */
function isPrototypeOfStub(): boolean {
  return false;
}

await describe({
  name: 'deepmerge-ts record paths found by mutation testing',
  children: [
    it({
      name: 'every record path stores __proto__ as an own enumerable writable configurable key',
      // Kills 881, 915, 916, and 917 (defaults/vanilla.ts) and 671, 673, 705, 706, and 707 (defaults/into.ts).
      fn: async () => {
        /**
         Values under `__proto__` in three inputs; the last wins as a leaf.
         */
        const inners = [{ p: 1, }, { p: 2, }, { p: 3, },] as const;
        /**
         Records merged by the two-record and the general path.
         */
        const merged = [
          target.deepmerge(ownProto(1,), ownProto(inners[1],),),
          target.deepmerge(ownProto(1,), ownProto(2,), ownProto(inners[2],),),
        ];
        expect(Object.getOwnPropertyDescriptor(merged[0], '__proto__',),).toEqual(openDescriptor(inners[1],),);
        expect(Object.getOwnPropertyDescriptor(merged[1], '__proto__',),).toEqual(openDescriptor(inners[2],),);
        /**
         Into targets for one source (two-record path) and two (general path).
         */
        const intos = [{}, {},];
        target.deepmergeInto(intos[0] ?? {}, ownProto(1,),);
        target.deepmergeInto(intos[1] ?? {}, ownProto(1,), ownProto(2,),);
        expect(Object.getOwnPropertyDescriptor(intos[0], '__proto__',),).toEqual(openDescriptor(1,),);
        expect(Object.getOwnPropertyDescriptor(intos[1], '__proto__',),).toEqual(openDescriptor(2,),);
        for (const record of [...merged, ...intos,]) expect(Object.getPrototypeOf(record,),).toBe(Object.prototype,);
      },
    },),
    it({
      name: 'deepmergeInto with two sources assigns through a target setter',
      // Kills 698 (defaults/into.ts).
      fn: async () => {
        /**
         Values the setter received.
         */
        const received: unknown[] = [];
        /**
         Target whose `a` is an enumerable setter-only accessor.
         */
        const into = {
          get a(): unknown {
            return undefined;
          },
          set a(value: unknown,) {
            received.push(value,);
          },
        };
        target.deepmergeInto(into, { a: 1, }, { a: 2, },);
        expect(received,).toEqual([2,],);
        expect((typeof Object.getOwnPropertyDescriptor(into, 'a',)?.set),).toBe('function',);
      },
    },),
    it({
      name: 'a Module-tagged object with a non-plain prototype merges as a record',
      // Kills 1054 and 1056 (utils.ts, the value's own tag) and 1080, 1081, and 1082 (utils.ts, the constructor
      // prototype's tag).
      fn: async () => {
        /**
         Prototype tagged `Module`, inheriting the ordinary `constructor`.
         */
        const tagged = { [Symbol.toStringTag]: 'Module', };
        expect(target.deepmerge(inheriting(tagged,), { b: 2, },),).toEqual({ a: 1, b: 2, },);
        /**
         Null-prototype `Module`-tagged prototype whose constructor points back to it.
         */
        const moduleLike = Object.create(null,) as object;
        Object.assign(moduleLike, {
          [Symbol.toStringTag]: 'Module',
          constructor: { prototype: moduleLike, },
          isPrototypeOf: isPrototypeOfStub,
        },);
        expect(target.deepmerge(inheriting(moduleLike,), { b: 2, },),).toEqual({ a: 1, b: 2, },);
      },
    },),
    it({
      name: 'an object whose constructor prototype is not Object-tagged is a leaf, even with isPrototypeOf',
      // Kills 1075, 1083, and 1084 (utils.ts).
      fn: async () => {
        /**
         Array standing in as the constructor's prototype, with an own isPrototypeOf.
         */
        const arrayPrototype = Object.assign([], { isPrototypeOf: isPrototypeOfStub, },);
        /**
         Leaf-like object: the last value wins over it.
         */
        const leaf = inheriting({ constructor: { prototype: arrayPrototype, }, },);
        /**
         Record merged after the leaf.
         */
        const later = { b: 2, };
        expect(target.deepmerge(leaf, later,),).toBe(later,);
      },
    },),
    it({
      name: 'objectHasProperty is false for functions',
      // Kills 1035 (utils.ts).
      fn: async () => {
        /**
         Function with an own enumerable property.
         */
        const withField = Object.assign(function field() {
          return 0;
        }, { x: 1, },);
        expect(target.objectHasProperty(withField, 'x',),).toBe(false,);
        expect(target.objectHasProperty({ x: 1, }, 'x',),).toBe(true,);
      },
    },),
    it({
      name: 'deepmergeInto replaces a target Set\'s contents with the merged values when filterValues drops it',
      // Kills 590 (defaults/general.ts).
      fn: async () => {
        /**
         Target Set the custom filter removes from the values.
         */
        const own = new Set(['old',],);
        /**
         Target holding that Set.
         */
        const into = { s: own, };
        target.deepmergeIntoCustom({
          filterValues: function dropTargetSet(values: readonly unknown[],) {
            return values.filter(function isNotOwn(value,) {
              return value !== own;
            },);
          },
        },)(into, { s: new Set(['a',],), }, { s: new Set(['b',],), },);
        expect(into.s,).toBe(own,);
        expect([...own,],).toEqual(['a', 'b',],);
      },
    },),
  ],
},);
