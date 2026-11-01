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
  createQuickLru,
  InvalidMaxAgeError,
  InvalidMaxSizeError,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    it({
      name: 'exports the cache factory and both error classes',
      fn: async () => {
        expect(typeof createQuickLru,).toBe('function',);
        expect(typeof InvalidMaxSizeError,).toBe('function',);
        expect(typeof InvalidMaxAgeError,).toBe('function',);
      },
    },),

    it({
      name: 'creates a working cache through the package entry point',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 2,
        },);
        lru.set({
          key: 'entry',
          value: 'value',
        },);
        expect(lru.get('entry',),).toBe('value',);
        expect(lru.has('entry',),).toBe(true,);
        expect(lru.size,).toBe(1,);
        expect(lru.toString(),).toBe('QuickLRU(1/2)',);
      },
    },),

    it({
      name: 'reports upstream quick-lru toStringTag through the package entry point',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 1,
        },);
        expect(Object.prototype.toString.call(lru,),).toBe('[object QuickLRU]',);
      },
    },),
  ],
},);
