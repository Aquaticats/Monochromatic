/**
 Unit tests for safe catalog alias candidate resolution.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  resolveNpmNames,
} from './version-resolve.ts';

await describe({
  name: resolveNpmNames.name,
  children: [
    it({
      name: 'returns the catalog key before a valid scoped alias target',
      fn: async () => {
        expect(resolveNpmNames({
          catalogKey: 'zod',
          catalogValue: 'npm:@jsr/zod__zod@>=4.1.8',
        },),).toEqual([
          'zod',
          '@jsr/zod__zod',
        ],);
      },
    },),

    it({
      name: 'returns only the key when an alias target is invalid',
      fn: async () => {
        expect(resolveNpmNames({
          catalogKey: 'zod',
          catalogValue: 'npm:../../outside@>=1.0.0',
        },),).toEqual(['zod',],);
      },
    },),

    it({
      name: 'returns no candidates when the catalog key is invalid',
      fn: async () => {
        expect(resolveNpmNames({
          catalogKey: '../../outside',
          catalogValue: '>=1.0.0',
        },),).toEqual([],);
      },
    },),
  ],
},);
