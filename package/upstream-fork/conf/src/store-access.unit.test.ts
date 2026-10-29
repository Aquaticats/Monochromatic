/**
 Store key access tests covering dot-notation and literal-key modes over
 the in-memory store primitives.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createPlainObject,
  getStoreValue,
  hasStoreValue,
  withStoreValue,
  withoutStoreValue,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'store access',
  children: [
    it({
      name: 'creates store objects with a null prototype',
      fn: async () => {
        /**
         Store minted by the primitive under test.
         */
        const store = createPlainObject();

        expect(Object.getPrototypeOf(store,),).toBeNull();
        expect(Object.keys(store,),).toEqual([],);
      },
    },),

    it({
      name: 'reads top-level and nested keys in dot mode',
      fn: async () => {
        /**
         Store carrying one flat and one nested key.
         */
        const store = {
          theme: 'dark',
          nested: {
            count: 1,
          },
        };

        expect(getStoreValue({
          store,
          key: 'theme',
          accessPropertiesByDotNotation: true,
        },),).toBe('dark',);
        expect(getStoreValue({
          store,
          key: 'nested.count',
          accessPropertiesByDotNotation: true,
        },),).toBe(1,);
        expect(hasStoreValue({
          store,
          key: 'nested.count',
          accessPropertiesByDotNotation: true,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'falls back to the default for missing keys in dot mode',
      fn: async () => {
        expect(getStoreValue({
          store: {},
          key: 'missing.path',
          defaultValue: 'fallback',
          accessPropertiesByDotNotation: true,
        },),).toBe('fallback',);
        expect(getStoreValue({
          store: {},
          key: 'missing.path',
          accessPropertiesByDotNotation: true,
        },),).toBeUndefined();
        expect(hasStoreValue({
          store: {},
          key: 'missing.path',
          accessPropertiesByDotNotation: true,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'reads literal dotted keys in non-dot mode only',
      fn: async () => {
        /**
         Store whose only key name itself contains a dot.
         */
        const store = {
          'a.b': 1,
        };

        expect(getStoreValue({
          store,
          key: 'a.b',
          accessPropertiesByDotNotation: false,
        },),).toBe(1,);
        expect(getStoreValue({
          store,
          key: 'a.b',
          defaultValue: 'fallback',
          accessPropertiesByDotNotation: true,
        },),).toBe('fallback',);
      },
    },),

    it({
      name: 'falls back to the default for missing keys in non-dot mode',
      fn: async () => {
        expect(getStoreValue({
          store: {},
          key: 'missing',
          defaultValue: 'fallback',
          accessPropertiesByDotNotation: false,
        },),).toBe('fallback',);
        expect(getStoreValue({
          store: {},
          key: 'missing',
          accessPropertiesByDotNotation: false,
        },),).toBeUndefined();
        expect(hasStoreValue({
          store: {},
          key: 'missing',
          accessPropertiesByDotNotation: false,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'matches `key in store` membership in non-dot mode for present and inherited names',
      fn: async () => {
        /**
         Empty literal store whose prototype chain still carries
         `Object.prototype`.
         */
        const store: Record<string, unknown> = {};

        /**
         Value the inherited-name probe resolves to,
         which must be the prototype's function rather than the default.
         */
        const inheritedNameValue = getStoreValue({
          store,
          key: 'toString',
          defaultValue: 'fallback',
          accessPropertiesByDotNotation: false,
        },);
        expect(typeof inheritedNameValue,).toBe('function',);
        expect(inheritedNameValue === 'fallback',).toBe(false,);
        expect(hasStoreValue({
          store,
          key: 'toString',
          accessPropertiesByDotNotation: false,
        },),).toBe(true,);

        /**
         Store whose `present` key exists with an `undefined` value,
         which `in` membership still counts as present.
         */
        const storeWithUndefined: Record<string, unknown> = {
          present: undefined,
        };
        expect(getStoreValue({
          store: storeWithUndefined,
          key: 'present',
          defaultValue: 'fallback',
          accessPropertiesByDotNotation: false,
        },),).toBeUndefined();
      },
    },),

    it({
      name: 'places nested values on the returned copy in dot mode',
      fn: async () => {
        /**
         Store handed in,
         whose nested object the dotted write reaches in place per the
         documented `@mutates` contract.
         */
        const store = {
          nested: {
            theme: 'light',
          },
        };
        /**
         Returned copy carrying the placed value.
         */
        const next = withStoreValue({
          store,
          key: 'nested.theme',
          value: 'dark',
          accessPropertiesByDotNotation: true,
        },);

        expect(getStoreValue({
          store: next,
          key: 'nested.theme',
          accessPropertiesByDotNotation: true,
        },),).toBe('dark',);
        expect(next === store,).toBe(false,);
        expect(Object.keys(store,),).toEqual([
          'nested',
        ],);
        // The shared nested object records the write (documented @mutates).
        expect(getStoreValue({
          store,
          key: 'nested.theme',
          accessPropertiesByDotNotation: true,
        },),).toBe('dark',);
      },
    },),

    it({
      name: 'places literal keys at the top level in non-dot mode',
      fn: async () => {
        /**
         Store whose only key name itself contains a dot.
         */
        const store = {
          'a.b': 1,
        };
        /**
         Returned copy with the literal key overwritten top-level.
         */
        const next = withStoreValue({
          store,
          key: 'a.b',
          value: 2,
          accessPropertiesByDotNotation: false,
        },);

        expect(getStoreValue({
          store: next,
          key: 'a.b',
          accessPropertiesByDotNotation: false,
        },),).toBe(2,);
        expect(Object.keys(next,),).toEqual([
          'a.b',
        ],);
      },
    },),

    it({
      name: 'refuses __proto__, constructor, and prototype keys in non-dot mode',
      fn: async () => {
        /**
         Key names the non-dot write path must refuse.
         */
        const forbiddenKeys = [
          '__proto__',
          'constructor',
          'prototype',
        ];
        for (const forbidden of forbiddenKeys) {
          /**
           Returned copy built with the refused key,
           which must stay exactly the clean input.
           */
          const next = withStoreValue({
            store: {
              safe: 1,
            },
            key: forbidden,
            value: 'polluted',
            accessPropertiesByDotNotation: false,
          },);

          expect(Object.hasOwn(next, forbidden,),).toBe(false,);
          expect(Object.getPrototypeOf(next,),).toBeNull();
          expect(Object.keys(next,),).toEqual([
            'safe',
          ],);
        }
      },
    },),

    it({
      name: 'returns a fresh copy and leaves the input own keys unchanged',
      fn: async () => {
        /**
         Store whose top-level keys are asserted unchanged below.
         */
        const store = {
          theme: 'dark',
        };
        /**
         Returned copy carrying the extra key.
         */
        const next = withStoreValue({
          store,
          key: 'extra',
          value: 1,
          accessPropertiesByDotNotation: false,
        },);

        expect(next === store,).toBe(false,);
        expect(Object.keys(store,),).toEqual([
          'theme',
        ],);
        expect(store.theme,).toBe('dark',);
      },
    },),

    it({
      name: 'removes nested keys on the returned copy in dot mode',
      fn: async () => {
        /**
         Store handed in,
         whose nested object the dotted delete reaches in place per the
         documented `@mutates` contract.
         */
        const store = {
          nested: {
            theme: 'dark',
          },
        };
        /**
         Returned copy missing the nested key.
         */
        const next = withoutStoreValue({
          store,
          key: 'nested.theme',
          accessPropertiesByDotNotation: true,
        },);

        expect(getStoreValue({
          store: next,
          key: 'nested.theme',
          defaultValue: 'missing',
          accessPropertiesByDotNotation: true,
        },),).toBe('missing',);
        expect(next === store,).toBe(false,);
        expect(Object.keys(store,),).toEqual([
          'nested',
        ],);
      },
    },),

    it({
      name: 'removes literal keys in non-dot mode and keeps the input intact',
      fn: async () => {
        /**
         Store whose own keys are asserted unchanged below.
         */
        const store = {
          'a.b': 1,
          safe: 2,
        };
        /**
         Returned copy missing the literal key.
         */
        const next = withoutStoreValue({
          store,
          key: 'a.b',
          accessPropertiesByDotNotation: false,
        },);

        expect(Object.keys(next,),).toEqual([
          'safe',
        ],);
        expect(Object.keys(store,),).toEqual([
          'a.b',
          'safe',
        ],);
        expect(next === store,).toBe(false,);
      },
    },),
  ],
},);
