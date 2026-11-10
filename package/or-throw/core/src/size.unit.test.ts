/**
 Tests for `getSize`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { getSize, } from './size.ts';

/**
 Caller-owned class with own enumerable fields, standing in for every
 domain type that `Object.keys` would otherwise size.
 */
class SampleRecord {
  readonly first = 1;

  readonly second = 2;
}

/**
 Object shapes `getSize` documents as intentionally rejected because they
 carry no element count of their own.
 */
const REJECTED_OBJECT_SHAPES: readonly (readonly [string, unknown])[] = [
  ['WeakSet', new WeakSet(),],
  ['WeakMap', new WeakMap(),],
  ['Date', new Date(),],
  ['RegExp', /a/,],
  // `Object(5,)` builds the boxed wrapper without the banned `new Number(...)` form.
  ['boxed number', Object(5,),],
  ['boxed empty string', Object('',),],
  ['boxed nonempty string', Object('ab',),],
  ['Error', new Error('rejected shape control',),],
  ['class instance', new SampleRecord(),],
  ['object with a custom prototype', Object.create({ inherited: 1, },),],
];

await describe({
  name: getSize.name,
  children: [
    //region Recognized containers

    it({
      name: 'returns the character count of strings',
      fn: async () => {
        expect(getSize('',),).toBe(0,);
        expect(getSize('hello',),).toBe('hello'.length,);
      },
    },),

    it({
      name: 'returns the element count of arrays',
      fn: async () => {
        const elements = [1, 2, 3,] as const;
        expect(getSize([],),).toBe(0,);
        expect(getSize(elements,),).toBe(elements.length,);
      },
    },),

    it({
      name: 'returns the entry count of Sets and Maps',
      fn: async () => {
        const members = new Set([1, 2,],);
        const entries = new Map([['a', 1,],],);
        expect(getSize(new Set(),),).toBe(0,);
        expect(getSize(members,),).toBe(members.size,);
        expect(getSize(new Map(),),).toBe(0,);
        expect(getSize(entries,),).toBe(entries.size,);
      },
    },),

    it({
      name: 'counts own enumerable string keys of plain objects',
      fn: async () => {
        expect(getSize({},),).toBe(0,);
        expect(getSize({ a: 1, b: 2, },),).toBe(2,);
      },
    },),

    it({
      name: 'counts own enumerable string keys of null-prototype objects',
      fn: async () => {
        const record: Record<string, number> = Object.create(null,);
        record.a = 1;
        expect(getSize(Object.create(null,),),).toBe(0,);
        expect(getSize(record,),).toBe(1,);
      },
    },),

    //endregion Recognized containers

    //region Rejected shapes

    it({
      name: 'throws for null and undefined',
      fn: async () => {
        expect(() => getSize(null,)).toThrow('Expected sized container, got null',);
        expect(() => getSize(undefined,)).toThrow('Expected sized container, got undefined',);
      },
    },),

    it({
      name: 'throws for primitives without a length',
      fn: async () => {
        expect(() => getSize(42,)).toThrow('Expected sized container, got number',);
        expect(() => getSize(true,)).toThrow('Expected sized container, got boolean',);
        expect(() => getSize(1n,)).toThrow('Expected sized container, got bigint',);
        expect(() => getSize(Symbol('non container symbol value',),)).toThrow('sized container',);
        expect(() => getSize(() => 1,)).toThrow('Expected sized container, got function',);
      },
    },),

    ...REJECTED_OBJECT_SHAPES.map(function makeRejectionTest([shapeName, value,],) {
      return it({
        name: `throws for ${shapeName}`,
        fn: async () => {
          expect(() => getSize(value,)).toThrow('Expected sized container, got [object]',);
        },
      },);
    },),

    //endregion Rejected shapes
  ],
},);
