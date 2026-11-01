/**
 Tests for the package entry point's export surface.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidConcurrencyError,
  InvalidRejectOnClearError,
  limitFunction,
  pLimit,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    it({
      name: 'exports the limiter factory, the wrapper factory, and both error classes',
      fn: async () => {
        expect(typeof pLimit,).toBe('function',);
        expect(typeof limitFunction,).toBe('function',);
        expect(typeof InvalidConcurrencyError,).toBe('function',);
        expect(typeof InvalidRejectOnClearError,).toBe('function',);
      },
    },),

    it({
      name: 'creates a working limiter through the package entry point',
      fn: async () => {
        const limit = pLimit({
          concurrency: 1,
          rejectOnClear: false,
        },);
        const result = await limit({
          fn: function greet(name: string,): string {
            return `hello ${name}`;
          },
          args: ['entry'],
        },);
        expect(result,).toBe('hello entry',);
        expect(limit.activeCount,).toBe(0,);
        expect(limit.pendingCount,).toBe(0,);
      },
    },),

    it({
      name: 'creates a working wrapped function through the package entry point',
      fn: async () => {
        const limited = limitFunction({
          fn: function identity(value: number,): number {
            return value;
          },
          options: {
            concurrency: 1,
          },
        },);
        expect(await limited({
          args: [7],
        },),).toBe(7,);
      },
    },),
  ],
},);
