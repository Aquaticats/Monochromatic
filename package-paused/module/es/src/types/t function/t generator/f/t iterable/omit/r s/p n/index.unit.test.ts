import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

const { $, } = types.function.generator.from.iterable.omit.sync.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'omits items in the exclusion set',
      fn: async () => {
        const numbers = [1, 2, 3, 4, 5,];
        const toOmit = new Set([2, 4,],);
        const result = [...$({ iterable: numbers, toOmit, },),];

        expect(result,).toEqual([1, 3, 5,],);
      },
    },),
    it({
      name: 'returns all items when exclusion set is empty',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const result = [...$({ iterable: numbers, toOmit: new Set(), },),];

        expect(result,).toEqual([1, 2, 3,],);
      },
    },),
    it({
      name: 'returns empty when all items are in exclusion set',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const toOmit = new Set([1, 2, 3,],);
        const result = [...$({ iterable: numbers, toOmit, },),];

        expect(result,).toEqual([],);
      },
    },),
    it({
      name: 'handles string items',
      fn: async () => {
        const words = ['apple', 'banana', 'cherry',];
        const toOmit = new Set(['banana',],);
        const result = [...$({ iterable: words, toOmit, },),];

        expect(result,).toEqual(['apple', 'cherry',],);
      },
    },),
    it({
      name: 'throws when toOmit contains items not in the iterable',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const toOmit = new Set([4, 5,],);

        expect(() => [...$({ iterable: numbers, toOmit, },),]).toThrow(
          'Key not found in iterable: 4',
        );
      },
    },),
    it({
      name: 'does not throw when strict is false and toOmit contains missing items',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const toOmit = new Set([2, 4, 5,],);
        const result = [...$({ iterable: numbers, toOmit, strict: false, },),];

        expect(result,).toEqual([1, 3,],);
      },
    },),
    it({
      name: 'strict: false omits existing items and ignores missing',
      fn: async () => {
        const words = ['apple', 'banana', 'cherry',];
        const toOmit = new Set(['banana', 'mango', 'grape',],);
        const result = [...$({ iterable: words, toOmit, strict: false, },),];

        expect(result,).toEqual(['apple', 'cherry',],);
      },
    },),
    it({
      name: 'strict: false with all missing keys returns all items',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const toOmit = new Set([99, 100,],);
        const result = [...$({ iterable: numbers, toOmit, strict: false, },),];

        expect(result,).toEqual([1, 2, 3,],);
      },
    },),
    it({
      name: 'preserves order of items',
      fn: async () => {
        const numbers = [5, 3, 1, 4, 2,];
        const toOmit = new Set([3, 4,],);
        const result = [...$({ iterable: numbers, toOmit, },),];

        expect(result,).toEqual([5, 1, 2,],);
      },
    },),
    it({
      name: 'works with generator iterables',
      fn: async () => {
        function* generateNumbers(): Generator<number> {
          yield 1;
          yield 2;
          yield 3;
        }
        const toOmit = new Set([2,],);
        const result = [...$({ iterable: generateNumbers(), toOmit, },),];

        expect(result,).toEqual([1, 3,],);
      },
    },),
  ],
},);
