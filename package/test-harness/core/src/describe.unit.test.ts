/**
 Tests for the `describe` suite runner.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

await describe({
  // oxlint-disable-next-line no-restricted-syntax/prefer-describe-function-ref-name -- harness self-test; the function under test IS the local binding, so `describe.name` is circular
  name: 'describe',
  children: [
    it({
      name: 'returns name on success',
      fn: async () => {
        const result = await describe({
          name: 'inner-suite',
          children: [
            it({ name: 'passing', fn: async () => {}, },),
          ],
        },);
        expect(result.name,).toBe('inner-suite',);
      },
    },),

    it({
      name: 'runs children concurrently',
      fn: async () => {
        const DELAY = 100;
        const MAX_SEQUENTIAL = 180;
        const start = performance.now();

        await describe({
          name: 'concurrent-suite',
          children: [
            it({
              name: 'child-a',
              fn: async () =>
                new Promise(resolve => {
                  setTimeout(resolve, DELAY,);
                },),
            },),
            it({
              name: 'child-b',
              fn: async () =>
                new Promise(resolve => {
                  setTimeout(resolve, DELAY,);
                },),
            },),
          ],
        },);

        const elapsed = performance.now() - start;
        expect(elapsed,).toBeLessThan(MAX_SEQUENTIAL,);
      },
    },),

    it({
      name: 'wraps single child failure',
      fn: async () => {
        const rejection = describe({
          name: 'failing-suite',
          children: [
            it({
              name: 'bad',
              fn: async () => {
                throw new Error('boom',);
              },
            },),
          ],
        },);
        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        await expect(rejection,).rejects.toHaveProperty('message', 'failing-suite',);
      },
    },),

    it({
      name: 'wraps multiple child failures in AggregateError',
      fn: async () => {
        const rejection = describe({
          name: 'multi-fail-suite',
          children: [
            it({
              name: 'bad-1',
              fn: async () => {
                throw new Error('one',);
              },
            },),
            it({
              name: 'bad-2',
              fn: async () => {
                throw new Error('two',);
              },
            },),
          ],
        },);
        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        await expect(rejection,).rejects.toHaveProperty('message', 'multi-fail-suite',);
      },
    },),

    it({
      name: 'empty name re-throws cause directly',
      fn: async () => {
        const rejection = describe({
          name: '',
          children: [
            it({
              name: 'bad',
              fn: async () => {
                throw new Error('direct',);
              },
            },),
          ],
        },);
        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        await expect(rejection,).rejects.toHaveProperty('message', 'bad',);
      },
    },),

    it({
      name: 'limits concurrency to the given value',
      fn: async () => {
        const DELAY = 50;
        const CHILD_COUNT = 4;
        const EXPECTED_PEAK = 2;
        let peak = 0;
        let active = 0;

        await describe({
          name: 'concurrency-suite',
          concurrency: EXPECTED_PEAK,
          children: Array.from({ length: CHILD_COUNT, }, (_, index,) =>
            it({
              name: `limited-${String(index,)}`,
              fn: async () => {
                active += 1;
                if (active > peak)
                  peak = active;

                await new Promise(resolve => {
                  setTimeout(resolve, DELAY,);
                },);
                active -= 1;
              },
            },),),
        },);

        expect(peak,).toBe(EXPECTED_PEAK,);
      },
    },),

    //region inheritance

    it({
      name: 'nested describe inherits parent concurrency',
      fn: async () => {
        const DELAY = 30;
        const CHILD_COUNT = 4;
        const EXPECTED_PEAK = 1;
        let peak = 0;
        let active = 0;

        await describe({
          name: 'outer-sequential',
          concurrency: 1,
          children: [
            describe({
              name: 'inner-without-override',
              children: Array.from({ length: CHILD_COUNT, }, (_, index,) =>
                it({
                  name: `inherited-${String(index,)}`,
                  fn: async () => {
                    active += 1;
                    if (active > peak)
                      peak = active;

                    await new Promise(resolve => {
                      setTimeout(resolve, DELAY,);
                    },);
                    active -= 1;
                  },
                },),),
            },),
          ],
        },);

        expect(peak,).toBe(EXPECTED_PEAK,);
      },
    },),

    it({
      name: 'nested describe overrides inherited concurrency',
      fn: async () => {
        const DELAY = 30;
        const CHILD_COUNT = 4;
        const EXPECTED_PEAK = 4;
        let peak = 0;
        let active = 0;

        await describe({
          name: 'outer-sequential-override',
          concurrency: 1,
          children: [
            describe({
              name: 'inner-unbounded',
              concurrency: Number.POSITIVE_INFINITY,
              children: Array.from({ length: CHILD_COUNT, }, (_, index,) =>
                it({
                  name: `overridden-${String(index,)}`,
                  fn: async () => {
                    active += 1;
                    if (active > peak)
                      peak = active;

                    await new Promise(resolve => {
                      setTimeout(resolve, DELAY,);
                    },);
                    active -= 1;
                  },
                },),),
            },),
          ],
        },);

        expect(peak,).toBe(EXPECTED_PEAK,);
      },
    },),

    //endregion inheritance

    it({
      name: 'respects suite timeout',
      fn: async () => {
        const DELAY = 300;
        const TIMEOUT = 50;
        const rejection = describe({
          name: 'slow-suite',
          children: [
            it({
              name: 'slow',
              fn: async () =>
                new Promise(resolve => {
                  setTimeout(resolve, DELAY,);
                },),
            },),
          ],
          timeout: TIMEOUT,
        },);
        await expect(rejection,).rejects.toBeInstanceOf(Error,);
      },
    },),
  ],
},);
