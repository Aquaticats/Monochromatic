import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { $ as Int, } from '@_/types/t number/t finite/t int/t/index.ts';
import type { $ as Positive, } from '@_/types/t number/t positive/t/index.ts';

/** Shorthand for the branded index type yielded by withIndex. */
type Index = Int & (Positive | 0);

const { $, } = types.function.generator.from.iterable.withIndex.sync.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'yields elements with their indices for arrays',
      fn: async () => {
        const result = [...$({ myIterable: ['a', 'b', 'c',], },),];

        expect(result,).toEqual([
          { element: 'a', index: 0 as Index, },
          { element: 'b', index: 1 as Index, },
          { element: 'c', index: 2 as Index, },
        ],);
      },
    },),
    it({
      name: 'yields elements with their indices for strings',
      fn: async () => {
        const result = [...$({ myIterable: 'hello', },),];

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
      name: 'yields elements with their indices for numbers',
      fn: async () => {
        const result = [...$({ myIterable: [10, 20, 30,], },),];

        expect(result,).toEqual([
          { element: 10, index: 0 as Index, },
          { element: 20, index: 1 as Index, },
          { element: 30, index: 2 as Index, },
        ],);
      },
    },),
    it({
      name: 'yields nothing for empty iterable',
      fn: async () => {
        const result = [...$({ myIterable: [], },),];

        expect(result,).toEqual([],);
      },
    },),
    it({
      name: 'works with Set',
      fn: async () => {
        const result = [...$({ myIterable: new Set(['x', 'y', 'z',],), },),];

        expect(result,).toEqual([
          { element: 'x', index: 0 as Index, },
          { element: 'y', index: 1 as Index, },
          { element: 'z', index: 2 as Index, },
        ],);
      },
    },),
    it({
      name: 'works with Map',
      fn: async () => {
        const map = new Map([['key1', 'value1',], ['key2', 'value2',],],);
        const result = [...$({ myIterable: map, },),];

        expect(result,).toEqual([
          { element: ['key1', 'value1',], index: 0 as Index, },
          { element: ['key2', 'value2',], index: 1 as Index, },
        ],);
      },
    },),
    it({
      name: 'lazy evaluation - generator only iterates when consumed',
      fn: async () => {
        let callCount = 0;
        function* lazyIterable() {
          callCount++;
          yield 'first';
          callCount++;
          yield 'second';
        }

        const gen = $({ myIterable: lazyIterable(), },);

        // Generator hasn't been consumed yet
        expect(callCount,).toBe(0,);

        // Consume first element
        const first = gen.next();
        expect(first.value,).toEqual({ element: 'first', index: 0 as Index, },);
        expect(callCount,).toBe(1,);

        // Consume second element
        const second = gen.next();
        expect(second.value,).toEqual({ element: 'second', index: 1 as Index, },);
        expect(callCount,).toBe(2,);

        // Generator is done
        const done = gen.next();
        expect(done.done,).toBe(true,);
      },
    },),
  ],
},);
