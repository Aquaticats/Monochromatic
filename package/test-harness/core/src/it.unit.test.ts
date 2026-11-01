/**
 Tests for the `it` test runner.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

await describe({
  // oxlint-disable-next-line no-restricted-syntax/prefer-describe-function-ref-name -- harness self-test; the function under test IS the local binding, so `it.name` is circular
  name: 'it',
  children: [
    it({
      name: 'returns name on success',
      fn: async () => {
        const result = await it({
          name: 'inner-test',
          fn: async () => {},
        },);
        expect(result.name,).toBe('inner-test',);
      },
    },),

    it({
      name: 'wraps failure with name and cause',
      fn: async () => {
        const original = new Error('root cause',);
        const rejection = it({
          name: 'failing-test',
          fn: async () => {
            throw original;
          },
        },);
        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        await expect(rejection,).rejects.toHaveProperty('message', 'failing-test',);
        await expect(rejection,).rejects.toHaveProperty('cause', original,);
      },
    },),

    it({
      name: 'respects timeout',
      fn: async () => {
        const DELAY = 200;
        const TIMEOUT = 50;
        const rejection = it({
          name: 'slow-test',
          fn: async () =>
            new Promise(resolve => {
              setTimeout(resolve, DELAY,);
            },),
          timeout: TIMEOUT,
        },);
        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        await expect(rejection,).rejects.toHaveProperty('message', 'slow-test',);
      },
    },),

    //region skip

    it({
      name: 'skip returns name without running fn',
      fn: async () => {
        let ran = false;
        const result = await it({
          name: 'skipped-test',
          skip: true,
          fn: async () => {
            ran = true;
          },
        },);
        expect(result.name,).toBe('skipped-test',);
        expect(ran,).toBe(false,);
      },
    },),

    //endregion skip

    //region repeats

    it({
      name: 'repeats runs fn the expected number of times',
      fn: async () => {
        let count = 0;
        const REPEATS = 2;
        const EXPECTED_RUNS = 3;
        await it({
          name: 'repeated-test',
          repeats: REPEATS,
          fn: async () => {
            count += 1;
          },
        },);
        expect(count,).toBe(EXPECTED_RUNS,);
      },
    },),

    it({
      name: 'repeats stops on first failure',
      fn: async () => {
        let count = 0;
        const REPEATS = 5;
        const rejection = it({
          name: 'fail-on-second',
          repeats: REPEATS,
          fn: async () => {
            count += 1;
            if (count === 2)
              throw new Error('boom',);
          },
        },);
        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        expect(count,).toBe(2,);
      },
    },),

    //endregion repeats

    //region fails

    it({
      name: 'fails treats throwing test as pass',
      fn: async () => {
        const result = await it({
          name: 'expected-failure',
          fails: true,
          fn: async () => {
            throw new Error('intentional',);
          },
        },);
        expect(result.name,).toBe('expected-failure',);
      },
    },),

    it({
      name: 'fails treats passing test as failure',
      fn: async () => {
        const rejection = it({
          name: 'unexpected-pass',
          fails: true,
          fn: async () => {},
        },);
        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        await expect(rejection,).rejects.toHaveProperty('message', 'unexpected-pass',);
      },
    },),

    it({
      name: 'fails with repeats requires every run to throw',
      fn: async () => {
        let count = 0;
        const REPEATS = 2;
        const EXPECTED_RUNS = 3;
        await it({
          name: 'always-throws',
          fails: true,
          repeats: REPEATS,
          fn: async () => {
            count += 1;
            throw new Error('intentional',);
          },
        },);
        expect(count,).toBe(EXPECTED_RUNS,);
      },
    },),
    //endregion fails
  ],
},);
