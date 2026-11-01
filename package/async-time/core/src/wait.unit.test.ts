/**
 Tests for `wait`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { wait, } from '../dist/final/neutral/index.mjs';

await describe({
  name: wait.name,
  children: [
    it({
      name: 'resolves after the specified delay',
      fn: async () => {
        const DELAY = 20;
        const TOLERANCE = 5;

        const start = performance.now();
        await wait(DELAY,);
        const elapsed = performance.now() - start;

        expect(elapsed,).toBeGreaterThanOrEqual(DELAY - TOLERANCE,);
      },
    },),

    it({
      name: 'resolves to undefined',
      fn: async () => {
        const result = await wait(1,);
        expect(result,).toBeUndefined();
      },
    },),
  ],
},);
