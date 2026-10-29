import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  median,
  percentile,
  PERCENTILE_99,
} from './metrics-stats.ts';

await describe({
  name: '',
  children: [
    describe({
      name: median.name,
      children: [
        it({
          name: 'returns 0 for empty input',
          fn: async () => {
            expect(median([],),).toBe(0,);
          },
        },),

        it({
          name: 'returns the middle value for an odd-length array',
          fn: async () => {
            expect(median([1, 2, 3, 4, 5,],),).toBe(3,);
          },
        },),

        it({
          name: 'averages the two middle values for an even-length array',
          fn: async () => {
            expect(median([1, 2, 3, 4,],),).toBe(2.5,);
          },
        },),

        it({
          name: 'works for a single-element array',
          fn: async () => {
            expect(median([42,],),).toBe(42,);
          },
        },),
      ],
    },),

    describe({
      name: percentile.name,
      children: [
        it({
          name: 'returns 0 for empty input',
          fn: async () => {
            expect(percentile({
              sortedAsc: [],
              p: PERCENTILE_99,
            },),)
              .toBe(0,);
          },
        },),

        it({
          name: 'returns the last element for p99 on a long array',
          fn: async () => {
            const samples = Array.from(
              {
                length: 100,
              },
              function gen(_, index,) {
                return index + 1;
              },
            );
            expect(percentile({
              sortedAsc: samples,
              p: PERCENTILE_99,
            },),)
              .toBe(99,);
          },
        },),

        it({
          name: 'returns the median when p=0.5',
          fn: async () => {
            expect(percentile({
              sortedAsc: [1, 2, 3, 4,],
              p: 0.5,
            },),)
              .toBe(2,);
          },
        },),

        it({
          name: 'returns the first element for very small p',
          fn: async () => {
            expect(percentile({
              sortedAsc: [10, 20, 30,],
              p: 0.01,
            },),)
              .toBe(10,);
          },
        },),
      ],
    },),
  ],
},);
