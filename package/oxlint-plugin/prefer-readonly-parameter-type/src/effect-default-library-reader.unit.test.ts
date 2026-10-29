import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { VERIFIED_READER_COUNT, } from '../dist/final/node/index.mjs';

/** Trap names a proxy records while a reader runs over it. */
type TrapHits = string[];

/**
 Builds a fully trapped object recording every operation a reader performs on it.
 
 Every trap is instrumented, not only the ones expected, because the point is to catch a
 reader reaching somewhere unlisted rather than to confirm it reaches where it should.
 
 @param hits - Accumulator receiving trap names in call order.
 
 @returns proxy standing in for a caller-owned object.
 
 @example
 ```ts
 trappedObject({ hits: [] });
 ```
 */
function trappedObject({ hits, }: { readonly hits: TrapHits; },): Record<string, unknown> {
  /**
   Ordinary target holding one data property.
   */
  const target: Record<string, unknown> = { first: 'value', };
  return new Proxy(target, {
    get(receiver, property, value,): unknown {
      hits.push('get',);
      return Reflect.get(receiver, property, value,);
    },
    set(receiver, property, value,): boolean {
      hits.push('set',);
      return Reflect.set(receiver, property, value,);
    },
    has(receiver, property,): boolean {
      hits.push('has',);
      return Reflect.has(receiver, property,);
    },
    deleteProperty(receiver, property,): boolean {
      hits.push('deleteProperty',);
      return Reflect.deleteProperty(receiver, property,);
    },
    ownKeys(receiver,): ArrayLike<string | symbol> {
      hits.push('ownKeys',);
      return Reflect.ownKeys(receiver,);
    },
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Mirrors the external ProxyHandler.getOwnPropertyDescriptor contract, which returns a descriptor or `undefined` for an absent own property; the trap cannot narrow what the specification defines.
    getOwnPropertyDescriptor(receiver, property,): PropertyDescriptor | undefined {
      hits.push('getOwnPropertyDescriptor',);
      return Reflect.getOwnPropertyDescriptor(receiver, property,);
    },
    defineProperty(receiver, property, descriptor,): boolean {
      hits.push('defineProperty',);
      return Reflect.defineProperty(receiver, property, descriptor,);
    },
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Mirrors the external ProxyHandler.getPrototypeOf contract, whose `null` is the real end of a prototype chain rather than an absent value.
    getPrototypeOf(receiver,): object | null {
      hits.push('getPrototypeOf',);
      return Reflect.getPrototypeOf(receiver,);
    },
    setPrototypeOf(receiver, prototype,): boolean {
      hits.push('setPrototypeOf',);
      return Reflect.setPrototypeOf(receiver, prototype,);
    },
  },);
}

/**
 Traps that would mean a reader is not a reader.
 */
const WRITING_TRAPS: ReadonlySet<string> = new Set([
  'set',
  'deleteProperty',
  'defineProperty',
  'setPrototypeOf',
]);

await describe({
  name: 'default-library reader authority',
  concurrency: 1,
  children: [
    it({
      name: 'reaches no writing trap on the value it reads',
      fn: async () => {
        /* Executable evidence for the claim the authority makes, rather than a restatement
         * of it. A reader that ever reached a writing trap would be mutating a
         * caller-owned value while this rule reports it as read-only, which is the exact
         * failure the registry exists to prevent going unnoticed. */
        for (const run of [
          function readEntries(value: Record<string, unknown>,): unknown {
            return Object.entries(value,);
          },
          function readValues(value: Record<string, unknown>,): unknown {
            return Object.values(value,);
          },
          function readKeys(value: Record<string, unknown>,): unknown {
            return Object.keys(value,);
          },
          function readHasOwn(value: Record<string, unknown>,): unknown {
            return Object.hasOwn(value, 'first',);
          },
          /* Reaches one trap and no other, which is the whole of its claim: it reads the
           * prototype slot and touches no property of the operand at all. */
          function readPrototype(value: Record<string, unknown>,): unknown {
            return Object.getPrototypeOf(value,);
          },
        ]) {
          /** Traps this reader reached. */
          const hits: TrapHits = [];
          run(trappedObject({ hits, },),);
          expect(hits.length > 0,).toBe(true,);
          expect(
            hits.filter(function isWriting(trap,): boolean {
              return WRITING_TRAPS.has(trap,);
            },),
          ).toEqual([],);
        }
      },
    },),
    it({
      name: 'collects its keys before reading any value',
      fn: async () => {
        /* The narrowing claim in `MEMBER_CHANNEL_OWN_PROPERTY`: an accessor firing during
         * the walk cannot add or remove entries from the result, because the key list is
         * taken first. Measured by trap order rather than asserted. */
        /** Traps reached while entries runs. */
        const hits: TrapHits = [];
        Object.entries(trappedObject({ hits, },),);
        /** Position of the first key enumeration. */
        const firstOwnKeys = hits.indexOf('ownKeys',);
        /** Position of the first value read. */
        const firstGet = hits.indexOf('get',);
        expect(firstOwnKeys,).toBe(0,);
        expect(firstGet > firstOwnKeys,).toBe(true,);
      },
    },),
    it({
      name: 'separates results that carry the operand from results that cannot',
      fn: async () => {
        /* The distinction the authority turns on, checked against the runtime rather than
         * against the table restating itself. `entries` hands back the very object the
         * operand held, so a write through it reaches the operand; `keys` hands back
         * freshly built strings, which cannot carry anything. */
        /** Object whose value is an identifiable reference. */
        const held = { label: 'held', };
        /** Operand holding that reference. */
        const operand = { first: held, };
        expect(Object.entries(operand,)[0]?.[1] === held,).toBe(true,);
        expect(Object.values(operand,)[0] === held,).toBe(true,);
        expect(Object.keys(operand,)
          .every(function isString(key,): boolean {
            return (typeof key) === 'string';
          },),).toBe(true,);
        /* The table's size, pinned so a reader added without evidence fails here as well
         * as in the architecture registry. The sentinels themselves need no assertion:
         * they are distinct symbols and the compiler rejects comparing them. */
        /* The prototype a reader hands back is reachable from the operand rather than
         * freshly built, which is why its entry carries the operand: writing through it
         * changes what the operand does. */
        expect(Object.getPrototypeOf(operand,) === Object.prototype,).toBe(true,);
        expect(VERIFIED_READER_COUNT,).toBe(5,);
      },
    },),
  ],
},);
