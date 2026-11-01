/**
 Tests for `withTimeout`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { withTimeout, } from '../dist/final/neutral/index.mjs';

await describe({
  name: withTimeout.name,
  children: [
    it({
      name: 'resolves when promise settles in time',
      fn: async () => {
        const DELAY = 10;
        const TIMEOUT = 200;
        const EXPECTED = 'done';

        const result = await withTimeout({
          promise: new Promise(function resolveAfterDelay(resolve,) {
            setTimeout(function onDelay() {
              resolve(EXPECTED,);
            }, DELAY,);
          },),
          ms: TIMEOUT,
          label: 'fast',
        },);

        expect(result,).toBe(EXPECTED,);
      },
    },),

    it({
      name: 'rejects with descriptive message on timeout',
      fn: async () => {
        const DELAY = 300;
        const TIMEOUT = 10;

        const rejection = withTimeout({
          promise: new Promise(function resolveAfterDelay(resolve,) {
            setTimeout(resolve, DELAY,);
          },),
          ms: TIMEOUT,
          label: 'timeout-label',
        },);

        await expect(rejection,).rejects.toBeInstanceOf(Error,);
        await expect(rejection,).rejects.toHaveProperty(
          'message',
          `Timed out after ${String(TIMEOUT,)}ms: timeout-label`,
        );
      },
    },),
  ],
},);
