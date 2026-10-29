import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { $ as Int, } from '@_/types/t number/t finite/t int/t/index.ts';
import type { $ as Positive, } from '@_/types/t number/t positive/t/index.ts';

/** Shorthand for the branded index type yielded by withIndex. */
type Index = Int & (Positive | 0);

const { $, } = types.function.generator.from.iterable.withIndex.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'yields elements with their indices for arrays',
      fn: async () => {
        const result = await Array.fromAsync($({ myIterable: ['a', 'b', 'c',], },),);

        expect(result,).toEqual([
          { element: 'a', index: 0 as Index, },
          { element: 'b', index: 1 as Index, },
          { element: 'c', index: 2 as Index, },
        ],);
      },
    },),
    it({
      name: 'handles empty iterables',
      fn: async () => {
        const emptyArray = await Array.fromAsync($({ myIterable: [], },),);
        expect(emptyArray,).toEqual([],);

        const emptyString = await Array.fromAsync($({ myIterable: '', },),);
        expect(emptyString,).toEqual([],);

        const emptySet = await Array.fromAsync($({ myIterable: new Set(), },),);
        expect(emptySet,).toEqual([],);
      },
    },),
    it({
      name: 'yields characters with indices for strings',
      fn: async () => {
        const result = await Array.fromAsync($({ myIterable: 'hello', },),);

        expect(result,).toEqual([
          { element: 'h', index: 0 as Index, },
          { element: 'e', index: 1 as Index, },
          { element: 'l', index: 2 as Index, },
          { element: 'l', index: 3 as Index, },
          { element: 'o', index: 4 as Index, },
        ],);
      },
    },),
    it({
      name: 'yields Set values with indices',
      fn: async () => {
        const mySet = new Set([10, 20, 30,],);
        const result = await Array.fromAsync($({ myIterable: mySet, },),);

        expect(result,).toEqual([
          { element: 10, index: 0 as Index, },
          { element: 20, index: 1 as Index, },
          { element: 30, index: 2 as Index, },
        ],);
      },
    },),
    it({
      name: 'yields Map entries with indices',
      fn: async () => {
        const myMap = new Map([
          ['key1', 'value1',],
          ['key2', 'value2',],
          ['key3', 'value3',],
        ],);
        const result = await Array.fromAsync($({ myIterable: myMap, },),);

        expect(result,).toEqual([
          { element: ['key1', 'value1',], index: 0 as Index, },
          { element: ['key2', 'value2',], index: 1 as Index, },
          { element: ['key3', 'value3',], index: 2 as Index, },
        ],);
      },
    },),
    it({
      name: 'handles async generators',
      fn: async () => {
        async function* asyncGen(): AsyncGenerator<string> {
          yield 'first';
          yield 'second';
          yield 'third';
        }

        const result = await Array.fromAsync($({ myIterable: asyncGen(), },),);

        expect(result,).toEqual([
          { element: 'first', index: 0 as Index, },
          { element: 'second', index: 1 as Index, },
          { element: 'third', index: 2 as Index, },
        ],);
      },
    },),
    it({
      name: 'handles single element iterable',
      fn: async () => {
        const result = await Array.fromAsync($({ myIterable: ['only',], },),);

        expect(result,).toEqual([
          { element: 'only', index: 0 as Index, },
        ],);
      },
    },),
    it({
      name: 'correctly increments indices for large arrays',
      fn: async () => {
        const LARGE_ARRAY_SIZE = 100;
        const largeArray = Array.from({ length: LARGE_ARRAY_SIZE, },
          (_, arrayIndex,) => arrayIndex,);
        const result = await Array.fromAsync($({ myIterable: largeArray, },),);

        expect(result,).toHaveLength(LARGE_ARRAY_SIZE,);
        expect(result[0],).toEqual({ element: 0, index: 0 as Index, },);
        expect(result[LARGE_ARRAY_SIZE - 1],).toEqual({ element: 99,
          index: 99 as Index, },);

        result.forEach((item, arrayIndex,) => {
          expect(item.index,).toBe(arrayIndex as Index,);
          expect(item.element,).toBe(arrayIndex,);
        },);
      },
    },),
    it({
      name: 'handles generator functions as input',
      fn: async () => {
        function* numberGen(): Generator<number> {
          yield 1;
          yield 2;
          yield 3;
        }

        const result = await Array.fromAsync($({ myIterable: numberGen(), },),);

        expect(result,).toEqual([
          { element: 1, index: 0 as Index, },
          { element: 2, index: 1 as Index, },
          { element: 3, index: 2 as Index, },
        ],);
      },
    },),
    it({
      name: 'type checking for index and element',
      fn: async () => {
        const gen = $({ myIterable: [1, 2, 3,], },);
        const firstItem = await gen.next();

        expect(firstItem.done,).toBe(false,);
        // oxlint-disable-next-line typescript/strict-boolean-expressions -- IteratorResult.done is boolean|undefined
        if (firstItem.done)
          throw new Error('Generator unexpectedly done',);

        type IndexType = typeof firstItem.value.index;
        type ElementType = typeof firstItem.value.element;

        expectTypeOf<IndexType>().toExtend<number>();
        expectTypeOf<ElementType>().toEqualTypeOf<1 | 2 | 3>();
      },
    },),
  ],
},);
