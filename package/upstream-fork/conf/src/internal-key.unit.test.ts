/**
 Reserved `__internal__` key detection tests for store keys and nested
 `set` payloads.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  containsReservedKey,
  INTERNAL_KEY,
  isReservedKeyPath,
  MIGRATION_KEY,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'reserved key detection',
  children: [
    it({
      name: 'names the reserved bookkeeping key and its migration path',
      fn: async () => {
        expect(INTERNAL_KEY,).toBe('__internal__',);
        expect(MIGRATION_KEY,).toBe('__internal__.migrations.version',);
      },
    },),

    it({
      name: 'treats the reserved key and its dotted paths as reserved',
      fn: async () => {
        expect(isReservedKeyPath(INTERNAL_KEY,),).toBe(true,);
        expect(isReservedKeyPath('__internal__.x',),).toBe(true,);
        expect(isReservedKeyPath(MIGRATION_KEY,),).toBe(true,);
      },
    },),

    it({
      name: 'leaves ordinary and near-miss keys unreserved',
      fn: async () => {
        for (const candidate of [
          'theme',
          '__internal_',
          '__internal_.x',
          'prefix.__internal__',
        ]) {
          expect(isReservedKeyPath(candidate,),).toBe(false,);
        }
      },
    },),

    it({
      name: 'finds reserved string key paths',
      fn: async () => {
        expect(containsReservedKey(INTERNAL_KEY,),).toBe(true,);
        expect(containsReservedKey('__internal__.x',),).toBe(true,);
        expect(containsReservedKey(MIGRATION_KEY,),).toBe(true,);
        expect(containsReservedKey('theme',),).toBe(false,);
      },
    },),

    it({
      name: 'finds a reserved key nested inside plain objects and skips clean ones',
      fn: async () => {
        expect(containsReservedKey({
          nested: {
            __internal__: {},
          },
        },),).toBe(true,);
        expect(containsReservedKey({
          nested: {
            theme: 'dark',
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'finds a reserved key nested two levels deep',
      fn: async () => {
        expect(containsReservedKey({
          one: {
            two: {
              __internal__: {},
            },
          },
        },),).toBe(true,);
      },
    },),

    it({
      name: 'finds reserved keys inside array payloads',
      fn: async () => {
        expect(containsReservedKey({
          items: [
            {
              __internal__: {},
            },
          ],
        },),).toBe(true,);
        expect(containsReservedKey([
          {
            __internal__: {},
          },
        ],),).toBe(true,);
        expect(containsReservedKey([
          {
            theme: 'dark',
          },
        ],),).toBe(false,);
      },
    },),

    it({
      name: 'reports no reserved key for non-object values',
      fn: async () => {
        expect(containsReservedKey(42,),).toBe(false,);
        expect(containsReservedKey(true,),).toBe(false,);
        expect(containsReservedKey(null,),).toBe(false,);
        expect(containsReservedKey(undefined,),).toBe(false,);
      },
    },),

    it({
      name: 'terminates on cyclic payloads and reports false for non-reserved keys',
      fn: async () => {
        /**
         Self-referential payload that would loop forever under unbounded
         recursion.
         */
        const cyclic: Record<string, unknown> = {
          name: 'cyclic',
        };
        cyclic.self = cyclic;

        expect(containsReservedKey(cyclic,),).toBe(false,);
      },
    },),

    it({
      name: 'finds a reserved key in a cyclic payload after walking past the cycle',
      fn: async () => {
        /**
         Self-referential payload carrying the reserved key away from the
         cycle.
         */
        const cyclic: Record<string, unknown> = {
          name: 'cyclic',
        };
        cyclic.self = cyclic;
        cyclic.__internal__ = {};

        expect(containsReservedKey(cyclic,),).toBe(true,);
      },
    },),
  ],
},);
