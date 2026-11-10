/**
 Tests for `createValueStore`, the mutable single-value store both observable variants wrap.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createValueStore, } from '../dist/final/node/index.mjs';

await describe({
  name: createValueStore.name,
  children: [
    it({
      name: 'getValue returns the initial value',
      fn: async () => {
        const store = createValueStore(1,);
        expect(store.getValue(),).toBe(1,);
      },
    },),

    it({
      name: 'replaceValue returns the replaced value and leaves the new one readable',
      fn: async () => {
        const store = createValueStore('a',);
        expect(store.replaceValue('b',),).toBe('a',);
        expect(store.getValue(),).toBe('b',);
      },
    },),

    it({
      name: 'two stores keep separate values',
      fn: async () => {
        const first = createValueStore(1,);
        const second = createValueStore(2,);
        first.replaceValue(3,);
        expect(first.getValue(),).toBe(3,);
        expect(second.getValue(),).toBe(2,);
      },
    },),
  ],
},);
