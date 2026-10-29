import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fieldOf,
  firstItem,
  isJsonRecord,
  isStringArray,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: isJsonRecord.name,
      children: [
        it({
          name: 'accepts plain objects only',
          fn: async () => {
            expect(isJsonRecord({ a: 1, },),).toBe(true,);
            expect(isJsonRecord(null,),).toBe(false,);
            expect(isJsonRecord([],),).toBe(false,);
            expect(isJsonRecord('text',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: fieldOf.name,
      children: [
        it({
          name: 'reads a property from an object and nothing from other values',
          fn: async () => {
            expect(fieldOf({ value: { a: 1, }, key: 'a', },),).toBe(1,);
            expect(fieldOf({ value: ['a',], key: '0', },),).toBeUndefined();
            expect(fieldOf({ value: 'text', key: 'length', },),).toBeUndefined();
          },
        },),
      ],
    },),
    describe({
      name: firstItem.name,
      children: [
        it({
          name: 'returns the first element of an array and nothing otherwise',
          fn: async () => {
            expect(firstItem(['a', 'b',],),).toBe('a',);
            expect(firstItem([],),).toBeUndefined();
            expect(firstItem({ 0: 'a', },),).toBeUndefined();
          },
        },),
      ],
    },),
    describe({
      name: isStringArray.name,
      children: [
        it({
          name: 'accepts arrays whose elements are all strings',
          fn: async () => {
            expect(isStringArray(['a', 'b',],),).toBe(true,);
            expect(isStringArray([],),).toBe(true,);
            expect(isStringArray(['a', 1,],),).toBe(false,);
            expect(isStringArray('a',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
