import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { types, } from '@monochromatic-dev/module-es';

import type { $ as Int, } from '@_/types/t number/t finite/t int/t/index.ts';

const { $, } = types.object.array.numberFiniteInt.from.number.exclusiveRange.sync.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'returns empty array when start equals end',
      fn: async () => {
        expect($({ startExclusive: 0, endExclusive: 0, },),).toEqual([],);
      },
    },),

    it({
      name: 'returns empty array when start is greater than end',
      fn: async () => {
        expect($({ startExclusive: 5, endExclusive: 3, },),).toEqual([],);
      },
    },),

    it({
      name: 'returns correct range for positive integers',
      fn: async () => {
        expect($({ startExclusive: 0, endExclusive: 5, },),).toEqual([1 as Int, 2 as Int,
          3 as Int, 4 as Int,],);
      },
    },),

    it({
      name: 'returns correct range for negative integers',
      fn: async () => {
        expect($({ startExclusive: -5, endExclusive: 0, },),).toEqual([-4 as Int,
          -3 as Int, -2 as Int, -1 as Int,],);
      },
    },),

    it({
      name: 'returns correct range crossing zero',
      fn: async () => {
        expect($({ startExclusive: -2, endExclusive: 3, },),).toEqual([-1 as Int,
          0 as Int, 1 as Int, 2 as Int,],);
      },
    },),

    it({
      name: 'handles single element ranges',
      fn: async () => {
        expect($({ startExclusive: 0, endExclusive: 2, },),).toEqual([1 as Int,],);
        expect($({ startExclusive: -1, endExclusive: 1, },),).toEqual([0 as Int,],);
      },
    },),

    it({
      name: 'handles decimal boundaries',
      fn: async () => {
        expect($({ startExclusive: 0.1, endExclusive: 4.9, },),).toEqual([1 as Int,
          2 as Int, 3 as Int, 4 as Int,],);
        expect($({ startExclusive: -0.9, endExclusive: 0.9, },),).toEqual([0 as Int,],);
        expect($({ startExclusive: 1.5, endExclusive: 2.3, },),).toEqual([2 as Int,],);
      },
    },),

    it({
      name: 'handles edge cases with large numbers',
      fn: async () => {
        expect($({ startExclusive: 99, endExclusive: 103, },),).toEqual([100 as Int,
          101 as Int, 102 as Int,],);
      },
    },),

    it({
      name: 'returns empty array when no integers exist in range',
      fn: async () => {
        expect($({ startExclusive: 1.1, endExclusive: 1.9, },),).toEqual([],);
      },
    },),

    it({
      name: 'handles decimal boundaries that resolve to a single integer',
      fn: async () => {
        expect($({ startExclusive: 0.5, endExclusive: 1.5, },),).toEqual([1 as Int,],);
      },
    },),
  ],
},);
