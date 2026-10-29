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
  CacheDisabledError,
  NonMethodDecorationError,
  NotMemoizedError,
  pMemoize,
  pMemoizeClear,
  pMemoizeDecorator,
  PropertyDescriptorMissingError,
  PrivateMethodDecorationError,
  UnclearableCacheError,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    it({
      name: 'exports the memoizer, the decorator, the clearer, and every error class',
      fn: async () => {
        expect(typeof pMemoize,).toBe('function',);
        expect(typeof pMemoizeDecorator,).toBe('function',);
        expect(typeof pMemoizeClear,).toBe('function',);
        expect(typeof NotMemoizedError,).toBe('function',);
        expect(typeof CacheDisabledError,).toBe('function',);
        expect(typeof UnclearableCacheError,).toBe('function',);
        expect(typeof NonMethodDecorationError,).toBe('function',);
        expect(typeof PrivateMethodDecorationError,).toBe('function',);
        expect(typeof PropertyDescriptorMissingError,).toBe('function',);
      },
    },),

    it({
      name: 'creates a working memoized function through the package entry point',
      fn: async () => {
        /**
         Monotonic counter answered on every wrapped invocation.
         */
        const state = {
          index: 0,
        };
        /**
         Wrapped function counting invocations.
         */
        async function fixture(_key: string,): Promise<string> {
          return `entry-${state.index++}`;
        };
        const memoized = pMemoize({
          fn: fixture,
        },);

        expect(await memoized({
          args: ['k'],
        },),).toBe('entry-0',);
        expect(await memoized({
          args: ['k'],
        },),).toBe('entry-0',);
        pMemoizeClear(memoized,);
        expect(await memoized({
          args: ['k'],
        },),).toBe('entry-1',);
      },
    },),

    it({
      name: 'creates a working decorator through the package entry point',
      fn: async () => {
        /**
         Decorator built from the entry point.
         */
        const decorator = pMemoizeDecorator();
        expect(typeof decorator,).toBe('function',);
      },
    },),
  ],
},);
