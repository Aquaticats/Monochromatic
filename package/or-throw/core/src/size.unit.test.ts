/**
 Tests for `getSize`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { getSize, } from '@monochromatic-dev/or-throw-core';

import { createCallerOwnedRecord, } from './caller-owned-record.test-fixture.ts';

/* oxlint-disable no-restricted-syntax/no-regex -- a RegExp instance is one of the rejected container shapes, so the fixture cannot be built from string APIs; the pattern is a fixed literal with no caller input and no backtracking risk */
// oxlint-disable-next-line eslint/prefer-regex-literals -- the constructor form keeps the fixture an instance rather than a matching literal
const REGEXP_SHAPE = new RegExp('a', 'u',);
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Object shapes `getSize` documents as intentionally rejected because they carry
 no element count of their own.
 */
const REJECTED_OBJECT_SHAPES: readonly (readonly [string, unknown])[] = [
  ['WeakSet', new WeakSet(),],
  ['WeakMap', new WeakMap(),],
  ['Date', new Date(),],
  ['RegExp', REGEXP_SHAPE,],
  // `Object(...)` builds the boxed wrapper; `new Number(...)`/`new String(...)` are banned.
  ['boxed number', new Object(5,),],
  ['boxed empty string', new Object('',),],
  ['boxed nonempty string', new Object('ab',),],
  ['Error', new Error('rejected shape control',),],
  ['caller-owned instance', createCallerOwnedRecord(),],
];

await describe({
  name: getSize.name,
  children: [
    //region Recognized containers

    it({
      name: 'returns the character count of strings',
      fn: async () => {
        const text = 'hello';
        expect(getSize('',),).toBe(0,);
        expect(getSize(text,),).toBe(text.length,);
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
        const emptySet = new Set();
        const members = new Set([1, 2,],);
        const emptyMap = new Map();
        const entries = new Map([['a', 1,],],);
        expect(getSize(emptySet,),).toBe(emptySet.size,);
        expect(getSize(members,),).toBe(members.size,);
        expect(getSize(emptyMap,),).toBe(emptyMap.size,);
        expect(getSize(entries,),).toBe(entries.size,);
      },
    },),

    it({
      name: 'counts own enumerable string keys of plain objects',
      fn: async () => {
        const record = { a: 1, b: 2, };
        expect(getSize({},),).toBe(0,);
        expect(getSize(record,),).toBe(Object.keys(record,).length,);
      },
    },),

    it({
      name: 'counts own enumerable string keys of null-prototype objects',
      fn: async () => {
        const emptyRecord: object = {};
        Object.setPrototypeOf(emptyRecord, null,);
        const record = { a: 1, };
        Object.setPrototypeOf(record, null,);
        expect(getSize(emptyRecord,),).toBe(0,);
        expect(getSize(record,),).toBe(Object.keys(record,).length,);
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
