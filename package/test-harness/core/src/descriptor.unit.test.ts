/**
 Tests for the lazy {@link TestDescriptor} primitive.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'TestDescriptor',
  children: [
    it({
      name: 'does not invoke fn until awaited',
      fn: async () => {
        let started = false;
        const d = it({
          name: 'lazy-inner',
          fn: async () => {
            started = true;
          },
        },);

        // Several microtask turns; descriptor still must not have run.
        await Promise.resolve();
        await Promise.resolve();
        expect(started,).toBe(false,);

        await d;
        expect(started,).toBe(true,);
      },
    },),

    it({
      name: 'awaiting the same descriptor twice runs fn twice',
      fn: async () => {
        let count = 0;
        const d = it({
          name: 'twice-inner',
          fn: async () => {
            count += 1;
          },
        },);

        await d;
        await d;
        expect(count,).toBe(2,);
      },
    },),

    it({
      name: 'top-level await runs without a parent describe',
      fn: async () => {
        const result = await it({
          name: 'standalone',
          fn: async () => {},
        },);
        expect(result.name,).toBe('standalone',);
      },
    },),

    it({
      name: 'describe descriptor is also lazy',
      fn: async () => {
        let started = false;
        const d = describe({
          name: 'lazy-suite-inner',
          children: [
            it({
              name: 'lazy-suite-child',
              fn: async () => {
                started = true;
              },
            },),
          ],
        },);

        await Promise.resolve();
        await Promise.resolve();
        expect(started,).toBe(false,);

        await d;
        expect(started,).toBe(true,);
      },
    },),
  ],
},);
