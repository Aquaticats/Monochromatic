/**
 Store mutation and read behavior of `createCrud` over a stub context,
 plus end-to-end coverage through `createConf` from the built artifact.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  createConf,
  createCrud,
  type CrudApi,
  type CrudContext,
  InvalidKeyError,
  MissingValueError,
  NonArrayValueError,
  prepareOptions,
  ReservedKeyError,
  type StoreFile,
  UnsupportedValueTypeError,
} from '../dist/final/neutral/index.mjs';

import { createTempDirectory, } from './test-support.ts';

/**
 Runs a call expected to throw and returns the caught error,
 so one invocation can carry several assertions.
 
 @param call - Call expected to throw.
 
 @returns The caught error value.
 
 @example
 ```ts
 const error = captureThrown(function boom(): void { throw new Error('x'); });
 ```
 */
function captureThrown(call: () => unknown,): unknown {
  try {
    call();
  }
  catch (error) {
    return error;
  }
  throw new Error('Expected the call to throw, but it returned.',);
}

/**
 Extracts a printable message from a caught value,
 so message assertions read from one helper.
 
 @param error - Caught value from a call under test.
 
 @returns The error's message,
 or the value's text form when it carries none.
 
 @example
 ```ts
 caughtMessage(new Error('boom')); // => 'boom'
 ```
 */
function caughtMessage(error: unknown,): string {
  return caughtValueText(error,);
}

/**
 One stub CRUD fixture: methods under test,
 recorded writes,
 and mutable store state.
 
 @example
 ```ts
 const fixture = createCrudStub({ store: { theme: 'light', }, });
 fixture.crud.set({ key: 'theme', value: 'dark', });
 ```
 */
type CrudStub = {
  /**
   CRUD methods under test.
   */
  readonly crud: CrudApi;
  /**
   Stores passed to `assignStore`,
   in call order.
   */
  readonly assignedStores: Record<string, unknown>[];
  /**
   Mutable store state the stub file pipeline reads.
   */
  readonly state: {
    store: Record<string, unknown>;
  };
};

/**
 Builds one stub CRUD context whose file pipeline reads mutable state,
 so each case controls the store and dot-notation mode.
 
 @param store - Initial store contents the stub reads.
 
 @param defaultValues - Defaults backing `reset` and `clear`.
 
 @param accessPropertiesByDotNotation - Whether dots address nested
 properties.
 
 @returns Stub fixture with recorded writes.
 
 @example
 ```ts
 const fixture = createCrudStub({ store: {}, accessPropertiesByDotNotation: false, });
 ```
 */
function createCrudStub({
  store,
  defaultValues = {},
  accessPropertiesByDotNotation = true,
}: {
  readonly store: Record<string, unknown>;
  readonly defaultValues?: Readonly<Record<string, unknown>>;
  readonly accessPropertiesByDotNotation?: boolean;
},): CrudStub {
  /**
   Mutable store state the stub pipeline reads like a fresh disk read.
   */
  const state = {
    store: structuredClone(store,),
  };
  /**
   Stores passed to `assignStore`,
   in call order.
   */
  const assignedStores: Record<string, unknown>[] = [];
  /**
   Prepared options selecting the dot-notation mode under test.
   */
  const options = prepareOptions<Record<string, unknown>>({
    cwd: createTempDirectory(),
    accessPropertiesByDotNotation,
  },);
  /**
   Stub file pipeline whose reads always return a fresh store copy.
   */
  const storeFile: StoreFile = {
    path: '/stub/config.json',
    fileExists: function fileExists(): boolean {
      return true;
    },
    ensureDirectory: function ensureDirectory(): void {},
    readParsedFile: function readParsedFile(): Record<string, unknown> {
      return structuredClone(state.store,);
    },
    readStore: function readStore(): Record<string, unknown> {
      return structuredClone(state.store,);
    },
    writeStore: function writeStore(): void {},
  };
  /**
   CRUD context under test over the stub pipeline.
   */
  const context: CrudContext<Record<string, unknown>> = {
    options,
    defaultValues,
    storeFile,
    getStore: function getStore(): Record<string, unknown> {
      return structuredClone(state.store,);
    },
    assignStore: function assignStore(nextStore: Record<string, unknown>,): void {
      assignedStores.push(nextStore,);
      state.store = structuredClone(nextStore,);
    },
  };
  return {
    crud: createCrud(context,),
    assignedStores,
    state,
  };
}

await describe({
  name: createCrud.name,
  children: [
    it({
      name: 'set places one keyed value and assigns the merged store exactly once',
      fn: async () => {
        /**
         Stub fixture starting from two stored keys.
         */
        const fixture = createCrudStub({
          store: {
            theme: 'light',
            locale: 'en',
          },
        },);
        fixture.crud
          .set({
            key: 'theme',
            value: 'dark',
          },);
        expect(fixture.assignedStores,).toEqual([
          {
            theme: 'dark',
            locale: 'en',
          },
        ],);
      },
    },),

    it({
      name: 'set merges a multi-item values object and assigns the merged store exactly once',
      fn: async () => {
        /**
         Stub fixture starting from one stored key.
         */
        const fixture = createCrudStub({
          store: {
            theme: 'light',
          },
        },);
        fixture.crud
          .set({
            values: {
              locale: 'en',
              nested: {
                mode: 'dark',
              },
            },
          },);
        expect(fixture.assignedStores,).toEqual([
          {
            theme: 'light',
            locale: 'en',
            nested: {
              mode: 'dark',
            },
          },
        ],);
      },
    },),

    it({
      name: 'set throws MissingValueError when the single form omits its value',
      fn: async () => {
        /**
         Stub fixture under test.
         */
        const fixture = createCrudStub({
          store: {},
        },);
        /**
         Error thrown for a key without any value.
         */
        const omittedError = captureThrown(function setWithoutValue(): void {
          fixture.crud
            .set({
              key: 'theme',
            },);
        },);
        expect(omittedError,).toBeInstanceOf(MissingValueError,);
        expect(caughtMessage(omittedError,),).toBe('Use `delete()` to clear values',);
        /**
         Error thrown for an explicit `undefined` value.
         */
        const undefinedError = captureThrown(function setUndefinedValue(): void {
          fixture.crud
            .set({
              key: 'theme',
              value: undefined,
            },);
        },);
        expect(undefinedError,).toBeInstanceOf(MissingValueError,);
        expect(fixture.assignedStores,).toHaveLength(0,);
      },
    },),

    it({
      name: 'set throws ReservedKeyError when the key addresses the reserved bookkeeping subtree',
      fn: async () => {
        /**
         Stub fixture under test.
         */
        const fixture = createCrudStub({
          store: {},
        },);
        for (const key of [
          '__internal__',
          '__internal__.migrations.version',
        ]) {
          /**
           Error thrown for the reserved key path.
           */
          const error = captureThrown(function setReservedKey(): void {
            fixture.crud
              .set({
                key,
                value: 'x',
              },);
          },);
          expect(error,).toBeInstanceOf(ReservedKeyError,);
          expect(caughtMessage(error,),).toBe(
            'Please don\'t use the __internal__ key, as it\'s used to manage this module internal operations.',
          );
        }
        expect(fixture.assignedStores,).toHaveLength(0,);
      },
    },),

    it({
      name: 'set throws ReservedKeyError when nested values contain the reserved key',
      fn: async () => {
        /**
         Stub fixture under test.
         */
        const fixture = createCrudStub({
          store: {},
        },);
        /**
         Error thrown for the nested reserved key.
         */
        const error = captureThrown(function setNestedReservedKey(): void {
          fixture.crud
            .set({
              values: {
                nested: {
                  __internal__: {},
                },
              },
            },);
        },);
        expect(error,).toBeInstanceOf(ReservedKeyError,);
        expect(fixture.assignedStores,).toHaveLength(0,);
      },
    },),

    it({
      name: 'set silently refuses prototype-polluting keys in non-dot mode',
      fn: async () => {
        /**
         Stub fixture with dots disabled.
         */
        const fixture = createCrudStub({
          store: {
            theme: 'light',
          },
          accessPropertiesByDotNotation: false,
        },);
        for (const key of [
          '__proto__',
          'constructor',
          'prototype',
        ])
          fixture.crud
            .set({
              key,
              value: 'polluted',
            },);
        expect(fixture.assignedStores,).toEqual([
          {
            theme: 'light',
          },
          {
            theme: 'light',
          },
          {
            theme: 'light',
          },
        ],);
      },
    },),

    it({
      name: 'set throws InvalidKeyError when the input or key shape is wrong',
      fn: async () => {
        /**
         Stub fixture under test.
         */
        const fixture = createCrudStub({
          store: {},
        },);
        /**
         Error thrown for a null input.
         */
        const nullError = captureThrown(function setNullInput(): void {
          fixture.crud
            .set(null as never);
        },);
        expect(nullError,).toBeInstanceOf(InvalidKeyError,);
        expect(caughtMessage(nullError,),).toBe('Expected `key` to be of type `string` or `object`, got object',);
        /**
         Error thrown for a string input.
         */
        const stringError = captureThrown(function setStringInput(): void {
          fixture.crud
            .set('theme' as never);
        },);
        expect(stringError,).toBeInstanceOf(InvalidKeyError,);
        expect(caughtMessage(stringError,),).toBe('Expected `key` to be of type `string` or `object`, got string',);
        /**
         Error thrown for a non-object `values` payload.
         */
        const valuesError = captureThrown(function setNonObjectValues(): void {
          fixture.crud
            .set({
              values: 1,
            } as never);
        },);
        expect(valuesError,).toBeInstanceOf(InvalidKeyError,);
        expect(caughtMessage(valuesError,),).toBe('Expected `key` to be of type `string` or `object`, got number',);
        /**
         Error thrown for a non-string single key.
         */
        const keyError = captureThrown(function setNonStringKey(): void {
          fixture.crud
            .set({
              key: 1,
              value: 'x',
            } as never);
        },);
        expect(keyError,).toBeInstanceOf(InvalidKeyError,);
        expect(caughtMessage(keyError,),).toBe('Expected `key` to be of type `string` or `object`, got number',);
        expect(fixture.assignedStores,).toHaveLength(0,);
      },
    },),

    it({
      name: 'get reads dotted paths and falls back to supplied default values',
      fn: async () => {
        /**
         Stub fixture with a nested value.
         */
        const fixture = createCrudStub({
          store: {
            ui: {
              mode: 'dark',
            },
          },
        },);
        expect(fixture.crud
          .get('ui.mode',),).toBe('dark',);
        expect(fixture.crud
          .get({
            key: 'ui.missing',
            defaultValue: 'light',
          },),).toBe('light',);
        expect(fixture.crud
          .get('ui.missing',),).toBeUndefined();
      },
    },),

    it({
      name: 'get treats dots as literal key characters in non-dot mode',
      fn: async () => {
        /**
         Stub fixture with dots disabled and a literal dotted key.
         */
        const fixture = createCrudStub({
          store: {
            'ui.mode': 'dark',
          },
          accessPropertiesByDotNotation: false,
        },);
        expect(fixture.crud
          .get('ui.mode',),).toBe('dark',);
        expect(fixture.crud
          .get({
            key: 'ui',
            defaultValue: 'light',
          },),).toBe('light',);
      },
    },),

    it({
      name: 'get throws InvalidKeyError when the key is not a string',
      fn: async () => {
        /**
         Stub fixture under test.
         */
        const fixture = createCrudStub({
          store: {},
        },);
        /**
         Error thrown for the non-string key.
         */
        const error = captureThrown(function getNonStringKey(): void {
          fixture.crud
            .get(1 as never);
        },);
        expect(error,).toBeInstanceOf(InvalidKeyError,);
        expect(caughtMessage(error,),).toBe('Expected `key` to be of type `string`, got number',);
      },
    },),

    it({
      name: 'has reports key presence in both dot and non-dot modes',
      fn: async () => {
        /**
         Stub fixture with dots enabled.
         */
        const dotFixture = createCrudStub({
          store: {
            ui: {
              mode: 'dark',
            },
          },
        },);
        expect(dotFixture.crud
          .has('ui.mode',),).toBe(true,);
        expect(dotFixture.crud
          .has('ui.other',),).toBe(false,);
        /**
         Stub fixture with dots disabled and a literal dotted key.
         */
        const literalFixture = createCrudStub({
          store: {
            'ui.mode': 'dark',
          },
          accessPropertiesByDotNotation: false,
        },);
        expect(literalFixture.crud
          .has('ui.mode',),).toBe(true,);
        expect(literalFixture.crud
          .has('ui',),).toBe(false,);
      },
    },),

    it({
      name: 'appendToArray creates a missing array holding the first item',
      fn: async () => {
        /**
         Stub fixture starting without the key.
         */
        const fixture = createCrudStub({
          store: {},
        },);
        fixture.crud
          .appendToArray({
            key: 'items',
            value: 'first',
          },);
        expect(fixture.assignedStores,).toEqual([
          {
            items: [
              'first',
            ],
          },
        ],);
      },
    },),

    it({
      name: 'appendToArray appends items in call order',
      fn: async () => {
        /**
         Stub fixture starting with one array item.
         */
        const fixture = createCrudStub({
          store: {
            items: [
              'first',
            ],
          },
        },);
        fixture.crud
          .appendToArray({
            key: 'items',
            value: 'second',
          },);
        fixture.crud
          .appendToArray({
            key: 'items',
            value: 'third',
          },);
        expect(fixture.assignedStores,).toEqual([
          {
            items: [
              'first',
              'second',
            ],
          },
          {
            items: [
              'first',
              'second',
              'third',
            ],
          },
        ],);
      },
    },),

    it({
      name: 'appendToArray throws NonArrayValueError when the key holds a non-array value',
      fn: async () => {
        /**
         Stub fixture whose target key holds a string.
         */
        const fixture = createCrudStub({
          store: {
            items: 'not-an-array',
          },
        },);
        /**
         Error thrown for the non-array target value.
         */
        const error = captureThrown(function appendToNonArray(): void {
          fixture.crud
            .appendToArray({
              key: 'items',
              value: 'entry',
            },);
        },);
        expect(error,).toBeInstanceOf(NonArrayValueError,);
        expect(caughtMessage(error,),).toBe('The key `items` is already set to a non-array value',);
        expect(fixture.assignedStores,).toHaveLength(0,);
      },
    },),

    it({
      name: 'appendToArray rejects values JSON cannot store',
      fn: async () => {
        /**
         Stub fixture under test.
         */
        const fixture = createCrudStub({
          store: {},
        },);
        /**
         Unsupported append values with the `typeof` text each diagnostic
         names.
         */
        const unsupportedCases: readonly {
          readonly value: unknown;
          readonly type: string;
        }[] = [
          {
            value: function entry(): void {},
            type: 'function',
          },
          {
            value: Symbol('unsupported symbol entry value'),
            type: 'symbol',
          },
          {
            value: undefined,
            type: 'undefined',
          },
        ];
        for (const { value, type, } of unsupportedCases) {
          /**
           Error thrown while appending the unsupported value.
           */
          const error = captureThrown(function appendUnsupported(): void {
            fixture.crud
              .appendToArray({
                key: 'items',
                value,
              },);
          },);
          expect(error,).toBeInstanceOf(UnsupportedValueTypeError,);
          expect(caughtMessage(error,),).toBe(
            `Setting a value of type \`${type}\` for key \`items\` is not allowed as it's not supported by JSON`,
          );
        }
        expect(fixture.assignedStores,).toHaveLength(0,);
      },
    },),

    it({
      name: 'reset restores only keys that have defaults',
      fn: async () => {
        /**
         Stub fixture with one defaulted key and one without a default.
         */
        const fixture = createCrudStub({
          store: {
            theme: 'dark',
            extra: 'kept',
          },
          defaultValues: {
            theme: 'light',
          },
        },);
        fixture.crud
          .reset({
            keys: [
              'theme',
              'extra',
            ],
          },);
        expect(fixture.assignedStores,).toEqual([
          {
            theme: 'light',
            extra: 'kept',
          },
        ],);
      },
    },),

    it({
      name: 'delete removes dotted paths in dot mode',
      fn: async () => {
        /**
         Stub fixture with a nested value.
         */
        const fixture = createCrudStub({
          store: {
            ui: {
              mode: 'dark',
            },
          },
        },);
        fixture.crud
          .delete('ui.mode',);
        expect(fixture.assignedStores,).toEqual([
          {
            ui: {},
          },
        ],);
      },
    },),

    it({
      name: 'delete removes literal keys in non-dot mode',
      fn: async () => {
        /**
         Stub fixture with dots disabled and a literal dotted key.
         */
        const fixture = createCrudStub({
          store: {
            'ui.mode': 'dark',
            theme: 'light',
          },
          accessPropertiesByDotNotation: false,
        },);
        fixture.crud
          .delete('ui.mode',);
        expect(fixture.assignedStores,).toEqual([
          {
            theme: 'light',
          },
        ],);
      },
    },),

    it({
      name: 'clear restores defaults and drops every other key',
      fn: async () => {
        /**
         Stub fixture with one defaulted key and one without a default.
         */
        const fixture = createCrudStub({
          store: {
            theme: 'dark',
            extra: 1,
          },
          defaultValues: {
            theme: 'light',
          },
        },);
        fixture.crud
          .clear();
        expect(fixture.assignedStores,).toEqual([
          {
            theme: 'light',
          },
        ],);
      },
    },),

    it({
      name: 'clear drops everything when no defaults survive the merge',
      fn: async () => {
        /**
         Stub fixture whose default entry is explicitly undefined.
         */
        const fixture = createCrudStub({
          store: {
            theme: 'dark',
          },
          defaultValues: {
            theme: undefined,
          },
        },);
        fixture.crud
          .clear();
        expect(fixture.assignedStores,).toEqual([
          {},
        ],);
      },
    },),

    it({
      name: 'clear rejects defaults holding values JSON cannot store',
      fn: async () => {
        /**
         Stub fixture whose default value cannot round-trip through JSON.
         */
        const fixture = createCrudStub({
          store: {},
          defaultValues: {
            bad: function badDefault(): void {},
          },
        },);
        /**
         Error thrown while clearing with the unsupported default.
         */
        const error = captureThrown(function clearWithUnsupportedDefault(): void {
          fixture.crud
            .clear();
        },);
        expect(error,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(fixture.assignedStores,).toHaveLength(0,);
      },
    },),

    it({
      name: 'set with one unsupported value in a multi-item payload throws before assigning anything',
      fn: async () => {
        /**
         Stub fixture under test.
         */
        const fixture = createCrudStub({
          store: {},
        },);
        /**
         Error thrown while placing the payload with one function value.
         */
        const error = captureThrown(function setMixedPayload(): void {
          fixture.crud
            .set({
              values: {
                ok: 1,
                bad: function badValue(): void {},
              },
            },);
        },);
        expect(error,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(fixture.assignedStores,).toHaveLength(0,);
      },
    },),

    it({
      name: 'round-trips single and multi sets and delete through a real store',
      fn: async () => {
        /**
         Real store under test in a disposable directory.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        conf.set({
          key: 'theme',
          value: 'dark',
        },);
        conf.set({
          values: {
            locale: 'en',
            nested: {
              mode: 'dark',
            },
          },
        },);
        expect(conf.get('theme',),).toBe('dark',);
        expect(conf.store,).toEqual({
          theme: 'dark',
          locale: 'en',
          nested: {
            mode: 'dark',
          },
        },);
        expect(conf.has('locale',),).toBe(true,);
        conf.delete('locale',);
        expect(conf.has('locale',),).toBe(false,);
      },
    },),

    it({
      name: 'round-trips appendToArray and reset through a real store',
      fn: async () => {
        /**
         Defaults backing the store's reset behavior.
         */
        const defaults: Record<string, unknown> = {
          theme: 'light',
        };
        /**
         Real store with one defaulted key.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          defaults,
        },);
        conf.appendToArray({
          key: 'items',
          value: 'first',
        },);
        conf.appendToArray({
          key: 'items',
          value: 'second',
        },);
        expect(conf.get('items',),).toEqual([
          'first',
          'second',
        ],);
        conf.set({
          key: 'theme',
          value: 'dark',
        },);
        conf.set({
          key: 'extra',
          value: 'kept',
        },);
        conf.reset({
          keys: [
            'theme',
            'extra',
          ],
        },);
        expect(conf.store,).toEqual({
          theme: 'light',
          extra: 'kept',
          items: [
            'first',
            'second',
          ],
        },);
      },
    },),

    it({
      name: 'clear restores defaults and drops other keys through a real store',
      fn: async () => {
        /**
         Defaults backing the store's clear behavior.
         */
        const defaults: Record<string, unknown> = {
          theme: 'light',
        };
        /**
         Real store with one defaulted key.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          defaults,
        },);
        conf.set({
          key: 'theme',
          value: 'dark',
        },);
        conf.set({
          key: 'extra',
          value: 1,
        },);
        conf.clear();
        expect(conf.store,).toEqual({
          theme: 'light',
        },);
      },
    },),
  ],
},);
