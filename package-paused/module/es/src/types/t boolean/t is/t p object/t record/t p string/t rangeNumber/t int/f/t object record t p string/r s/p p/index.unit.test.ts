import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

// TODO: import compiled.
import { $ as isRangeNumberInt, } from './index.ts';

await describe({
  name: isRangeNumberInt.name,
  children: [
    it({
      name: 'should return true for valid integer ranges',
      fn: async () => {
        expect(isRangeNumberInt({
          startInclusive: 0,
          endInclusive: 10,
        },),)
          .toBe(true,);

        expect(isRangeNumberInt({
          startInclusive: -5,
          endInclusive: 5,
        },),)
          .toBe(true,);

        expect(isRangeNumberInt({
          startInclusive: 100,
          endInclusive: 100,
        },),)
          .toBe(true,);

        expect(isRangeNumberInt({
          startInclusive: -100,
          endInclusive: -50,
        },),)
          .toBe(true,);
      },
    },),
    it({
      name: 'should return false for non-integer ranges',
      fn: async () => {
        expect(isRangeNumberInt({
          startInclusive: 0.5,
          endInclusive: 10,
        },),)
          .toBe(false,);

        expect(isRangeNumberInt({
          startInclusive: 0,
          endInclusive: 10.5,
        },),)
          .toBe(false,);

        expect(isRangeNumberInt({
          startInclusive: 1.1,
          endInclusive: 2.2,
        },),)
          .toBe(false,);
      },
    },),
    it({
      name: 'should return false when start > end',
      fn: async () => {
        expect(isRangeNumberInt({
          startInclusive: 10,
          endInclusive: 5,
        },),)
          .toBe(false,);

        expect(isRangeNumberInt({
          startInclusive: 0,
          endInclusive: -1,
        },),)
          .toBe(false,);
      },
    },),
    it({
      name: 'should return false for non-number values',
      fn: async () => {
        expect(isRangeNumberInt({
          startInclusive: '0',
          endInclusive: 10,
        },),)
          .toBe(false,);

        expect(isRangeNumberInt({
          startInclusive: 0,
          endInclusive: '10',
        },),)
          .toBe(false,);

        expect(isRangeNumberInt({
          startInclusive: null,
          endInclusive: 10,
        },),)
          .toBe(false,);

        expect(isRangeNumberInt({
          startInclusive: undefined,
          endInclusive: 10,
        },),)
          .toBe(false,);
      },
    },),
    it({
      name: 'should return false for special numeric values',
      fn: async () => {
        expect(isRangeNumberInt({
          startInclusive: Number.NaN,
          endInclusive: 10,
        },),)
          .toBe(false,);

        expect(isRangeNumberInt({
          startInclusive: 0,
          endInclusive: Infinity,
        },),)
          .toBe(false,);

        expect(isRangeNumberInt({
          startInclusive: -Infinity,
          endInclusive: 0,
        },),)
          .toBe(false,);
      },
    },),
    it({
      name: 'should handle edge cases at integer boundaries',
      fn: async () => {
        expect(isRangeNumberInt({
          startInclusive: Number.MAX_SAFE_INTEGER,
          endInclusive: Number.MAX_SAFE_INTEGER,
        },),)
          .toBe(true,);

        expect(isRangeNumberInt({
          startInclusive: Number.MIN_SAFE_INTEGER,
          endInclusive: Number.MIN_SAFE_INTEGER,
        },),)
          .toBe(true,);

        expect(isRangeNumberInt({
          startInclusive: Number.MIN_SAFE_INTEGER,
          endInclusive: Number.MAX_SAFE_INTEGER,
        },),)
          .toBe(true,);
      },
    },),
  ],
},);
