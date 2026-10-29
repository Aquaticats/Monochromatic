import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

const { $, } = types.function.generator.from.iterable.pick.sync.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'picks items in the inclusion set',
      fn: async () => {
        const numbers = [1, 2, 3, 4, 5,];
        const toPick = new Set([2, 4,],);
        const result = [...$({ iterable: numbers, toPick, },),];

        expect(result,).toEqual([2, 4,],);
      },
    },),
    it({
      name: 'returns empty when inclusion set is empty',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const result = [...$({ iterable: numbers, toPick: new Set(), },),];

        expect(result,).toEqual([],);
      },
    },),
    it({
      name: 'returns all items when all are in inclusion set',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const toPick = new Set([1, 2, 3,],);
        const result = [...$({ iterable: numbers, toPick, },),];

        expect(result,).toEqual([1, 2, 3,],);
      },
    },),
    it({
      name: 'handles string items',
      fn: async () => {
        const words = ['apple', 'banana', 'cherry',];
        const toPick = new Set(['banana', 'cherry',],);
        const result = [...$({ iterable: words, toPick, },),];

        expect(result,).toEqual(['banana', 'cherry',],);
      },
    },),
    it({
      name: 'throws when toPick contains items not in the iterable',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const toPick = new Set([2, 4, 5,],);

        expect(() => [...$({ iterable: numbers, toPick, },),]).toThrow(
          'Key not found in iterable: 4',
        );
      },
    },),
    it({
      name: 'does not throw when strict is false and toPick contains missing items',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const toPick = new Set([2, 4, 5,],);
        const result = [...$({ iterable: numbers, toPick, strict: false, },),];

        expect(result,).toEqual([2,],);
      },
    },),
    it({
      name: 'strict: false returns only existing items from toPick',
      fn: async () => {
        const words = ['apple', 'banana', 'cherry',];
        const toPick = new Set(['banana', 'mango', 'grape',],);
        const result = [...$({ iterable: words, toPick, strict: false, },),];

        expect(result,).toEqual(['banana',],);
      },
    },),
    it({
      name: 'strict: false with all missing keys returns empty array',
      fn: async () => {
        const numbers = [1, 2, 3,];
        const toPick = new Set([99, 100,],);
        const result = [...$({ iterable: numbers, toPick, strict: false, },),];

        expect(result,).toEqual([],);
      },
    },),
    it({
      name: 'preserves order of items from iterable',
      fn: async () => {
        const numbers = [5, 3, 1, 4, 2,];
        const toPick = new Set([1, 2,],);
        const result = [...$({ iterable: numbers, toPick, },),];

        expect(result,).toEqual([1, 2,],);
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
        const toPick = new Set([2, 3,],);
        const result = [...$({ iterable: generateNumbers(), toPick, },),];

        expect(result,).toEqual([2, 3,],);
      },
    },),
  ],
},);
