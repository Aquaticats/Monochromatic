/**
 Tests for the core cache surface: storage, presence, deletion, and the
 string renderings.
 
 Every case here pins observable behavior of upstream `quick-lru` 7.3.0,
 including its duplicate-key accounting, so the fuzz sidecar's
 differential oracle can compare both implementations directly.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createQuickLru,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'cache core',
  children: [
    //region Storage

    it({
      name: 'stores one value and reads it back',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 2,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        expect(lru.get('a',),).toBe('alpha',);
        expect(lru.has('a',),).toBe(true,);
        expect(lru.peek('a',),).toBe('alpha',);
      },
    },),

    it({
      name: 'returns the cache itself from set so calls chain',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 2,
        },);
        /**
         Chained result of two set calls.
         */
        const chained = lru
          .set({
            key: 'a',
            value: 1,
          },)
          .set({
            key: 'b',
            value: 2,
          },);
        expect(chained,).toBe(lru,);
        expect(lru.get('b',),).toBe(2,);
      },
    },),

    it({
      name: 'reports absent keys as undefined through get and peek',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 2,
        },);
        expect(lru.get('missing',),).toBe(undefined,);
        expect(lru.peek('missing',),).toBe(undefined,);
        expect(lru.has('missing',),).toBe(false,);
      },
    },),

    it({
      name: 'overwrites an existing recent item in place without growing',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 3,
        },);
        lru.set({
          key: 'a',
          value: 1,
        },);
        lru.set({
          key: 'a',
          value: 2,
        },);
        expect(lru.get('a',),).toBe(2,);
        expect(lru.size,).toBe(1,);
      },
    },),

    it({
      name: 'keeps a duplicate key across both maps consistent on delete',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 3,
        },);
        lru.set({
          key: 'a',
          value: 1,
        },);
        lru.set({
          key: 'b',
          value: 2,
        },);
        lru.set({
          key: 'c',
          value: 3,
        },);
        lru.set({
          key: 'a',
          value: 4,
        },);
        expect(lru.__oldCache.has('a',),).toBe(true,);
        expect(lru.delete('a',),).toBe(true,);
        expect(lru.has('a',),).toBe(false,);
        expect(lru.delete('a',),).toBe(false,);
      },
    },),

    it({
      name: 'stores function values like any other value',
      fn: async () => {
        /**
         Function stored as a cache value.
         */
        function greet(): string {
          return 'hi';
        }
        const lru = createQuickLru<string, () => string>({
          maxSize: 2,
        },);
        lru.set({
          key: 'fn',
          value: greet,
        },);
        expect(lru.get('fn',),).toBe(greet,);
      },
    },),

    it({
      name: 'distinguishes non-primitive keys by reference',
      fn: async () => {
        /**
         First object key.
         */
        const firstKey = {
          id: 1,
        };
        /**
         Second object key with the same shape but its own identity.
         */
        const secondKey = {
          id: 1,
        };
        const lru = createQuickLru<object, string>({
          maxSize: 3,
        },);
        lru.set({
          key: firstKey,
          value: 'first',
        },);
        lru.set({
          key: secondKey,
          value: 'second',
        },);
        expect(lru.get(firstKey,),).toBe('first',);
        expect(lru.get(secondKey,),).toBe('second',);
        expect(lru.size,).toBe(2,);
      },
    },),

    it({
      name: 'keeps stored undefined values visible through has, matching upstream',
      fn: async () => {
        const lru = createQuickLru<string, undefined>({
          maxSize: 2,
        },);
        lru.set({
          key: 'a',
          value: undefined,
        },);
        expect(lru.has('a',),).toBe(true,);
        expect(lru.get('a',),).toBe(undefined,);
      },
    },),

    it({
      name: 'deletes and clears without touching unrelated entries',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 4,
        },);
        lru.set({
          key: 'a',
          value: 1,
        },);
        lru.set({
          key: 'b',
          value: 2,
        },);
        expect(lru.delete('a',),).toBe(true,);
        expect(lru.get('b',),).toBe(2,);
        lru.clear();
        expect(lru.size,).toBe(0,);
        expect(lru.get('b',),).toBe(undefined,);
        expect(lru.delete('b',),).toBe(false,);
      },
    },),

    //endregion Storage

    //region Counters and bounds

    it({
      name: 'reports size capped at maxSize even while both maps hold items',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 2,
        },);
        lru.set({
          key: 'a',
          value: 1,
        },);
        lru.set({
          key: 'b',
          value: 2,
        },);
        lru.set({
          key: 'c',
          value: 3,
        },);
        expect(lru.size,).toBe(2,);
        expect([...lru].length,).toBe(3,);
      },
    },),

    it({
      name: 'counts duplicate keys once in size, matching upstream accounting',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 5,
        },);
        for (const key of [
          'a',
          'b',
          'c',
          'd',
          'e',
        ]) {
          lru.set({
            key,
            value: 1,
          },);
        }
        lru.evict(3,);
        lru.set({
          key: 'd',
          value: 9,
        },);
        expect(lru.size,).toBe(2,);
        expect([...lru].length,).toBe(2,);
      },
    },),

    it({
      name: 'reports maxSize and maxAge from construction',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 7,
          maxAge: 1_000,
        },);
        expect(lru.maxSize,).toBe(7,);
        expect(lru.maxAge,).toBe(1_000,);
      },
    },),

    //endregion Counters and bounds

    //region Renderings and hooks

    it({
      name: 'renders QuickLRU(size/maxSize) through toString',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 5,
        },);
        expect(lru.toString(),).toBe('QuickLRU(0/5)',);
        lru.set({
          key: 'a',
          value: 1,
        },);
        expect(lru.toString(),).toBe('QuickLRU(1/5)',);
      },
    },),

    it({
      name: 'reports upstream quick-lru toStringTag',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 1,
        },);
        expect(lru[Symbol.toStringTag],).toBe('QuickLRU',);
        expect(Object.prototype.toString.call(lru,),).toBe('[object QuickLRU]',);
      },
    },),

    it({
      name: 'renders through the node custom inspection symbol like toString',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 3,
        },);
        lru.set({
          key: 'a',
          value: 1,
        },);
        /**
         Cache carrying the runtime-only inspection member, which the
         `QuickLru` type cannot name because `Symbol.for` keys are not
         expressible in type literals.
         */
        const inspectable = lru as unknown as Record<PropertyKey, () => string>;
        /**
         Inspection hook as installed at runtime.
         */
        const inspectCustom = inspectable[Symbol.for('nodejs.util.inspect.custom',)];
        expect(typeof inspectCustom,).toBe('function',);
        expect(inspectCustom?.(),).toBe(lru.toString(),);
      },
    },),

    it({
      name: 'peeks a live old-map item without promoting it',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 2,
        },);
        lru.set({
          key: 'a',
          value: 'alpha',
        },);
        lru.set({
          key: 'b',
          value: 'beta',
        },);
        lru.set({
          key: 'c',
          value: 'gamma',
        },);
        expect(lru.peek('a',),).toBe('alpha',);
        expect(lru.__oldCache.has('a',),).toBe(true,);
        expect([...lru.entriesAscending()].map(function readKey(entry: [string, string],): string {
          return entry[0];
        },),).toEqual([
          'a',
          'b',
          'c',
        ],);
      },
    },),

    it({
      name: 'reports size from the old map alone while the recent map is empty',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 1.5,
        },);
        lru.set({
          key: 'a',
          value: 1,
        },);
        lru.set({
          key: 'b',
          value: 2,
        },);
        expect(lru.size,).toBe(2,);
      },
    },),

    it({
      name: 'exposes the old map through the __oldCache test hook',
      fn: async () => {
        const lru = createQuickLru<string, number>({
          maxSize: 2,
        },);
        lru.set({
          key: 'a',
          value: 1,
        },);
        expect(lru.__oldCache.size,).toBe(0,);
        lru.set({
          key: 'b',
          value: 2,
        },);
        lru.set({
          key: 'c',
          value: 3,
        },);
        expect([...lru.__oldCache.keys()],).toEqual([
          'a',
          'b',
        ],);
      },
    },),

    //endregion Renderings and hooks
  ],
},);
