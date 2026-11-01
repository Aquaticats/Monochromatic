/**
 Core store behavior of `createConf` ported from upstream `conf`'s test suite:
 reads,
 writes,
 array appends,
 resets,
 deletes,
 clears,
 store surfaces,
 persistence-on-change,
 and reserved-key protection.
 
 @module
 */

import {
  existsSync,
  rmSync,
  statSync,
} from 'node:fs';

import {
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type Conf,
  createConf,
  InvalidKeyError,
  MissingValueError,
  NonArrayValueError,
  ReservedKeyError,
  type StoreChange,
  UnsupportedValueTypeError,
  type ValueChange,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
  readMigrationVersion,
  writeConfigFile,
} from './test-support.ts';

/**
 Fixture value shared by the read and write cases,
 matching upstream `conf`'s test fixture.
 */
const FIXTURE = '🦄';

/**
 Creates a store rooted in a fresh disposable directory,
 so every case starts from an empty config file.
 
 @returns Store persisting to a fresh temp directory.
 */
function createFreshStore(): Conf {
  return createConf({ cwd: createTempDirectory(), },);
}

/**
 Runs a call expected to throw and returns the captured error so class and message text can be asserted.
 
 @param call - Call that must throw.
 
 @returns Captured thrown error.
 */
function captureThrown(call: () => unknown,): Error {
  try {
    call();
  }
  catch (error) {
    return error as Error;
  }
  throw new Error('expected the call to throw, but it returned',);
}

await describe({
  name: createConf.name,
  children: [
    //region Reads

    it({
      name: '.get()',
      fn: async () => {
        /**
         Store probed before and after a write.
         */
        const config = createFreshStore();
        expect(config.get('foo',),).toBeUndefined();
        expect(config.get({
          key: 'foo',
          defaultValue: FIXTURE,
        },),).toBe(FIXTURE,);
        config.set({
          key: 'foo',
          value: FIXTURE,
        },);
        expect(config.get('foo',),).toBe(FIXTURE,);
      },
    },),
    it({
      name: '.get() - `defaults` option',
      fn: async () => {
        /**
         Store whose `defaults` option seeds nested values.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 42,
            nested: {
              bar: 55,
            },
          },
        },);
        expect(store.get('foo',),).toBe(42,);
        expect(store.get('nested.bar',),).toBe(55,);
      },
    },),
    it({
      name: '.get() - `schema` option - default',
      fn: async () => {
        /**
         Store whose schema declares a top-level default and a nested default,
         mirroring upstream `conf`'s `failingTest('.get() - `schema` option - default')` case
         ( whose body must throw because the nested default never reaches the store ).
         Only top-level schema defaults surface;
         `nested.bar` stays undefined while `foo` returns its default.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              type: 'boolean',
              default: true,
            },
            nested: {
              type: 'object',
              properties: {
                bar: {
                  type: 'number',
                  default: 55,
                },
              },
            },
          },
        },);
        expect(store.get('foo',),).toBe(true,);
        expect(store.get('nested.bar',),).toBeUndefined();
      },
    },),
    it({
      name: '.get() - invalid key',
      fn: async () => {
        /**
         Store carrying a value proving invalid probes leave state untouched.
         */
        const config = createFreshStore();
        config.set({
          key: 'foo',
          value: FIXTURE,
        },);
        /**
         Error thrown when get receives `undefined`:
         upstream `conf` names the received type in InvalidKeyError.
         */
        const undefinedError = captureThrown((): unknown => config.get(undefined as never,),);
        expect(undefinedError,).toBeInstanceOf(InvalidKeyError,);
        expect(undefinedError.name,).toBe('InvalidKeyError',);
        expect(undefinedError.message,).toBe('Expected `key` to be of type `string`, got undefined',);
        /**
         Error thrown when get receives `null`:
         upstream names `object` because `typeof null` is 'object'.
         */
        const nullError = captureThrown((): unknown => config.get(null as never,),);
        expect(nullError,).toBeInstanceOf(InvalidKeyError,);
        expect(nullError.message,).toBe('Expected `key` to be of type `string`, got object',);
        /**
         Error thrown when get receives a number:
         upstream names the received type.
         */
        const numberError = captureThrown((): unknown => config.get(1 as never,),);
        expect(numberError,).toBeInstanceOf(InvalidKeyError,);
        expect(numberError.message,).toBe('Expected `key` to be of type `string`, got number',);
        // The store is unchanged.
        expect(config.get('foo',),).toBe(FIXTURE,);
      },
    },),
    it({
      name: '.has()',
      fn: async () => {
        /**
         Store probed at existing and missing keys.
         */
        const config = createFreshStore();
        config.set({
          key: 'foo',
          value: FIXTURE,
        },);
        config.set({
          key: 'baz.boo',
          value: FIXTURE,
        },);
        expect(config.has('foo',),).toBe(true,);
        expect(config.has('baz.boo',),).toBe(true,);
        expect(config.has('missing',),).toBe(false,);
      },
    },),

    //endregion Reads

    //region Writes

    it({
      name: '.set()',
      fn: async () => {
        /**
         Store carrying single keyed writes at top level and through dot notation.
         */
        const config = createFreshStore();
        config.set({
          key: 'foo',
          value: FIXTURE,
        },);
        config.set({
          key: 'baz.boo',
          value: FIXTURE,
        },);
        expect(config.get('foo',),).toBe(FIXTURE,);
        expect(config.get('baz.boo',),).toBe(FIXTURE,);
      },
    },),
    it({
      name: '.set() - with object',
      fn: async () => {
        /**
         Store carrying one multi-item values write.
         */
        const config = createFreshStore();
        config.set({
          values: {
            foo1: 'bar1',
            foo2: 'bar2',
            baz: {
              boo: 'foo',
              foo: {
                bar: 'baz',
              },
            },
          },
        },);
        expect(config.get('foo1',),).toBe('bar1',);
        expect(config.get('foo2',),).toBe('bar2',);
        expect(config.get('baz',),).toEqual({
          boo: 'foo',
          foo: {
            bar: 'baz',
          },
        },);
        expect(config.get('baz.boo',),).toBe('foo',);
        expect(config.get('baz.foo',),).toEqual({ bar: 'baz', },);
        expect(config.get('baz.foo.bar',),).toBe('baz',);
      },
    },),
    it({
      name: '.set() - with undefined',
      fn: async () => {
        /**
         Store probed by both value-omitting set forms.
         */
        const config = createFreshStore();
        /**
         Error thrown when the single-key form names `undefined` as its value.
         */
        const namedUndefinedError = captureThrown((): unknown => config.set({
          key: 'foo',
          value: undefined,
        },),);
        expect(namedUndefinedError,).toBeInstanceOf(MissingValueError,);
        expect(namedUndefinedError.name,).toBe('MissingValueError',);
        expect(namedUndefinedError.message,).toBe('Use `delete()` to clear values',);
        /**
         Error thrown when the single-key form omits its value entirely.
         */
        const omittedValueError = captureThrown((): unknown => config.set({ key: 'foo', },),);
        expect(omittedValueError,).toBeInstanceOf(MissingValueError,);
        expect(omittedValueError.message,).toBe('Use `delete()` to clear values',);
      },
    },),
    it({
      name: '.set() - with unsupported values',
      fn: async () => {
        /**
         Store probed with each JSON-unsupported value type.
         */
        const config = createFreshStore();
        /**
         Error thrown for a function value under a single key.
         */
        const functionError = captureThrown((): unknown => config.set({
          key: 'a',
          value: function unsupported(): void {},
        },),);
        expect(functionError,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(functionError.message,).toBe(
          "Setting a value of type `function` for key `a` is not allowed as it's not supported by JSON",
        );
        /**
         Error thrown for a symbol value under a single key.
         */
        const symbolError = captureThrown((): unknown => config.set({
          key: 'a',
          value: Symbol('probe symbol created to fail JSON serialization',),
        },),);
        expect(symbolError,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(symbolError.message,).toBe(
          "Setting a value of type `symbol` for key `a` is not allowed as it's not supported by JSON",
        );
        /**
         Error thrown for an `undefined` value inside the multi-item form.
         */
        const valuesUndefinedError = captureThrown((): unknown => config.set({
          values: {
            a: undefined,
          },
        },),);
        expect(valuesUndefinedError,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(valuesUndefinedError.message,).toBe(
          "Setting a value of type `undefined` for key `a` is not allowed as it's not supported by JSON",
        );
        /**
         Error thrown for a method value inside the multi-item form.
         */
        const valuesFunctionError = captureThrown((): unknown => config.set({
          values: {
            a: function unsupported(): void {},
          },
        },),);
        expect(valuesFunctionError,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(valuesFunctionError.message,).toBe(
          "Setting a value of type `function` for key `a` is not allowed as it's not supported by JSON",
        );
        /**
         Error thrown for a symbol value inside the multi-item form.
         */
        const valuesSymbolError = captureThrown((): unknown => config.set({
          values: {
            a: Symbol('probe symbol created to fail JSON serialization',),
          },
        },),);
        expect(valuesSymbolError,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(valuesSymbolError.message,).toBe(
          "Setting a value of type `symbol` for key `a` is not allowed as it's not supported by JSON",
        );
      },
    },),
    it({
      name: '.set() - invalid key',
      fn: async () => {
        /**
         Store probed by non-object and non-string-key set calls.
         */
        const config = createFreshStore();
        /**
         Error thrown when the whole set input is a number,
         matching upstream `conf`'s `set(1, 'unicorn')` probe.
         */
        const numberInputError = captureThrown((): unknown => config.set(1 as never,),);
        expect(numberInputError,).toBeInstanceOf(InvalidKeyError,);
        expect(numberInputError.name,).toBe('InvalidKeyError',);
        expect(numberInputError.message,).toBe('Expected `key` to be of type `string` or `object`, got number',);
        /**
         Error thrown when the single-key form carries a numeric key.
         */
        const numericKeyError = captureThrown((): unknown => config.set({
          key: 1 as never,
          value: 'unicorn',
        },),);
        expect(numericKeyError,).toBeInstanceOf(InvalidKeyError,);
        expect(numericKeyError.message,).toBe('Expected `key` to be of type `string` or `object`, got number',);
      },
    },),
    it({
      name: '.appendToArray()',
      fn: async () => {
        /**
         Store exercising array creation and growth at every addressing depth.
         */
        const config = createFreshStore();
        // Appending to a non-existent key creates the array.
        config.appendToArray({
          key: 'newArray',
          value: 'first',
        },);
        expect(config.get('newArray',),).toEqual([
          'first',
        ],);
        // Appending to an existing array extends it.
        config.set({
          key: 'items',
          value: [
            'a',
            'b',
          ],
        },);
        config.appendToArray({
          key: 'items',
          value: 'c',
        },);
        expect(config.get('items',),).toEqual([
          'a',
          'b',
          'c',
        ],);
        // Appending keeps object items intact.
        config.set({
          key: 'objects',
          value: [{ id: 1, }, { id: 2, },],
        },);
        config.appendToArray({
          key: 'objects',
          value: { id: 3, },
        },);
        expect(config.get('objects',),).toEqual([
          { id: 1, },
          { id: 2, },
          { id: 3, },
        ],);
        // Nested arrays use dot notation.
        config.set({
          key: 'nested.items',
          value: [1, 2,],
        },);
        config.appendToArray({
          key: 'nested.items',
          value: 3,
        },);
        expect(config.get('nested.items',),).toEqual([1, 2, 3,],);
        // Appending creates a nested array that does not exist yet.
        config.appendToArray({
          key: 'deeply.nested.array',
          value: 'value',
        },);
        expect(config.get('deeply.nested.array',),).toEqual(['value',],);
      },
    },),
    it({
      name: '.appendToArray() - error when key is not array',
      fn: async () => {
        /**
         Store holding one non-array value per JSON value kind.
         */
        const config = createFreshStore();
        config.set({
          key: 'notArray',
          value: 'string value',
        },);
        /**
         Error thrown for a string value under the appended key.
         */
        const stringError = captureThrown((): unknown => config.appendToArray({
          key: 'notArray',
          value: 'item',
        },),);
        expect(stringError,).toBeInstanceOf(NonArrayValueError,);
        expect(stringError.name,).toBe('NonArrayValueError',);
        expect(stringError.message,).toBe('The key `notArray` is already set to a non-array value',);
        config.set({
          key: 'numberValue',
          value: 42,
        },);
        /**
         Error thrown for a number value under the appended key.
         */
        const numberError = captureThrown((): unknown => config.appendToArray({
          key: 'numberValue',
          value: 'item',
        },),);
        expect(numberError,).toBeInstanceOf(NonArrayValueError,);
        expect(numberError.message,).toBe('The key `numberValue` is already set to a non-array value',);
        config.set({
          key: 'objectValue',
          value: { foo: 'bar', },
        },);
        /**
         Error thrown for an object value under the appended key.
         */
        const objectError = captureThrown((): unknown => config.appendToArray({
          key: 'objectValue',
          value: 'item',
        },),);
        expect(objectError,).toBeInstanceOf(NonArrayValueError,);
        expect(objectError.message,).toBe('The key `objectValue` is already set to a non-array value',);
        config.set({
          key: 'nested.notArray',
          value: false,
        },);
        /**
         Error thrown for a nested non-array value under the appended path.
         */
        const nestedError = captureThrown((): unknown => config.appendToArray({
          key: 'nested.notArray',
          value: 'item',
        },),);
        expect(nestedError,).toBeInstanceOf(NonArrayValueError,);
        expect(nestedError.message,).toBe('The key `nested.notArray` is already set to a non-array value',);
      },
    },),
    it({
      name: '.appendToArray() - without dot notation',
      fn: async () => {
        /**
         Store whose keys stay literal instead of addressing nested properties.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          accessPropertiesByDotNotation: false,
        },);
        // Basic functionality without dot notation.
        store.appendToArray({
          key: 'items',
          value: 'first',
        },);
        expect(store.get('items',),).toEqual([
          'first',
        ],);
        // A dot notation key is treated as one literal key.
        store.appendToArray({
          key: 'nested.items',
          value: 'value',
        },);
        expect(store.get('nested.items',),).toEqual(['value',],);
      },
    },),
    it({
      name: '.appendToArray() - value validation',
      fn: async () => {
        /**
         Store holding a valid array to append onto.
         */
        const config = createFreshStore();
        config.set({
          key: 'items',
          value: ['valid',],
        },);
        /**
         Error thrown for a function item.
         */
        const functionError = captureThrown((): unknown => config.appendToArray({
          key: 'items',
          value: function unsupported(): void {},
        },),);
        expect(functionError,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(functionError.message,).toBe(
          "Setting a value of type `function` for key `items` is not allowed as it's not supported by JSON",
        );
        /**
         Error thrown for a symbol item.
         */
        const symbolError = captureThrown((): unknown => config.appendToArray({
          key: 'items',
          value: Symbol('probe symbol created to fail JSON serialization',),
        },),);
        expect(symbolError,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(symbolError.message,).toBe(
          "Setting a value of type `symbol` for key `items` is not allowed as it's not supported by JSON",
        );
        /**
         Error thrown for an `undefined` item.
         */
        const undefinedError = captureThrown((): unknown => config.appendToArray({
          key: 'items',
          value: undefined,
        },),);
        expect(undefinedError,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(undefinedError.message,).toBe(
          "Setting a value of type `undefined` for key `items` is not allowed as it's not supported by JSON",
        );
      },
    },),
    it({
      name: '.appendToArray() - change events',
      fn: async () => {
        /**
         Store watched at the appended key.
         */
        const config = createFreshStore();
        /**
         Change records collected across both appends.
         */
        const changes: ValueChange<unknown>[] = [];
        /**
         Subscription recording each change of the watched key.
         */
        const unsubscribe = config.onDidChange({
          key: 'items',
          callback: function recordValueChange(change,): void {
            changes.push(change,);
          },
        },);
        // The first append creates the array and fires one change event.
        config.appendToArray({
          key: 'items',
          value: 'first',
        },);
        expect(changes,).toHaveLength(1,);
        expect(changes[0]?.newValue,).toEqual(['first',],);
        expect(changes[0]?.oldValue,).toBeUndefined();
        // The second append fires one change event with the updated array.
        config.appendToArray({
          key: 'items',
          value: 'second',
        },);
        expect(changes,).toHaveLength(2,);
        expect(changes[1]?.newValue,).toEqual([
          'first',
          'second',
        ],);
        expect(changes[1]?.oldValue,).toEqual(['first',],);
        unsubscribe();
      },
    },),
    it({
      name: '.appendToArray() - empty arrays',
      fn: async () => {
        /**
         Store whose explicitly empty arrays receive appends.
         */
        const config = createFreshStore();
        config.set({
          key: 'empty',
          value: [],
        },);
        config.appendToArray({
          key: 'empty',
          value: 'item',
        },);
        expect(config.get('empty',),).toEqual(['item',],);
        // Multiple appends onto an explicit empty array accumulate in order.
        config.set({
          key: 'multi',
          value: [],
        },);
        config.appendToArray({
          key: 'multi',
          value: 'a',
        },);
        config.appendToArray({
          key: 'multi',
          value: 'b',
        },);
        config.appendToArray({
          key: 'multi',
          value: 'c',
        },);
        expect(config.get('multi',),).toEqual([
          'a',
          'b',
          'c',
        ],);
      },
    },),
    it({
      name: '.reset() - `defaults` option',
      fn: async () => {
        /**
         Store whose changed keys reset to their `defaults` values.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 42,
            bar: 99,
          },
        },);
        store.set({
          key: 'foo',
          value: 77,
        },);
        store.set({
          key: 'bar',
          value: 0,
        },);
        store.reset({
          keys: [
            'foo',
            'bar',
          ],
        },);
        expect(store.get('foo',),).toBe(42,);
        expect(store.get('bar',),).toBe(99,);
      },
    },),
    it({
      name: '.reset() - dot notation',
      fn: async () => {
        /**
         Store whose nested default paths reset through dot notation.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          defaults: {
            options: {
              items: [1, 2,],
              nested: {
                value: 'default',
              },
            },
          },
        },);
        store.set({
          key: 'options.items',
          value: [9, 9,],
        },);
        store.set({
          key: 'options.nested.value',
          value: 'changed',
        },);
        store.reset({
          keys: [
            'options.items',
            'options.nested.value',
          ],
        },);
        expect(store.get('options.items',),).toEqual([1, 2,],);
        expect(store.get('options.nested.value',),).toBe('default',);
        // A nested key with no default is left alone.
        store.set({
          key: 'options.other',
          value: 'kept',
        },);
        store.reset({
          // @ts-expect-error - `options.other` is added at runtime, so it is not part of the inferred type.
          keys: ['options.other',],
        },);
        expect(store.get('options.other',),).toBe('kept',);
      },
    },),
    it({
      name: '.reset() - falsy `defaults` option',
      fn: async () => {
        /**
         Store whose defaults are all falsy but present.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 0,
            bar: '',
            fox: false,
            bax: true,
          },
        },);
        store.set({
          key: 'foo',
          value: 5,
        },);
        store.set({
          key: 'bar',
          value: 'exist',
        },);
        store.set({
          key: 'fox',
          value: true,
        },);
        store.reset({
          keys: [
            'foo',
            'bar',
            'fox',
            'bax',
          ],
        },);
        expect(store.get('foo',),).toBe(0,);
        expect(store.get('bar',),).toBe('',);
        expect(store.get('fox',),).toBe(false,);
        expect(store.get('bax',),).toBe(true,);
      },
    },),
    it({
      name: '.reset() - `schema` option',
      fn: async () => {
        /**
         Store whose schema defaults back each reset key.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              default: 42,
            },
            bar: {
              default: 99,
            },
          },
        },);
        store.set({
          key: 'foo',
          value: 77,
        },);
        store.set({
          key: 'bar',
          value: 0,
        },);
        store.reset({
          keys: [
            'foo',
            'bar',
          ],
        },);
        expect(store.get('foo',),).toBe(42,);
        expect(store.get('bar',),).toBe(99,);
      },
    },),
    it({
      name: '.delete()',
      fn: async () => {
        /**
         Store holding nested values deleted one path at a time.
         */
        const config = createFreshStore();
        config.set({
          key: 'foo',
          value: 'bar',
        },);
        config.set({
          key: 'baz.boo',
          value: true,
        },);
        config.set({
          key: 'baz.foo.bar',
          value: 'baz',
        },);
        config.delete('foo',);
        expect(config.get('foo',),).toBeUndefined();
        config.delete('baz.boo',);
        expect(config.get('baz.boo',),).toBeUndefined();
        config.delete('baz.foo',);
        expect(config.get('baz.foo',),).toBeUndefined();
        // A delete at one dotted path leaves sibling paths intact.
        config.set({
          key: 'foo.bar.baz',
          value: { awesome: 'icecream', },
        },);
        config.set({
          key: 'foo.bar.zoo',
          value: { awesome: 'redpanda', },
        },);
        config.delete('foo.bar.baz',);
        expect(config.get('foo.bar.zoo.awesome',),).toBe('redpanda',);
      },
    },),
    it({
      name: '.clear()',
      fn: async () => {
        /**
         Store emptied by one clear call.
         */
        const config = createFreshStore();
        config.set({
          key: 'foo',
          value: 'bar',
        },);
        config.set({
          key: 'foo1',
          value: 'bar1',
        },);
        config.set({
          key: 'baz.boo',
          value: true,
        },);
        config.clear();
        expect(config.size,).toBe(0,);
      },
    },),
    it({
      name: '.clear() - `defaults` option',
      fn: async () => {
        /**
         Store whose clear restores every `defaults` value.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 42,
            bar: 99,
          },
        },);
        store.set({
          key: 'foo',
          value: 2,
        },);
        store.clear();
        expect(store.get('foo',),).toBe(42,);
        expect(store.get('bar',),).toBe(99,);
      },
    },),
    it({
      name: '.clear() - `schema` option',
      fn: async () => {
        /**
         Store whose clear restores every schema default.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          schema: {
            foo: {
              default: 42,
            },
            bar: {
              default: 99,
            },
          },
        },);
        store.set({
          key: 'foo',
          value: 2,
        },);
        store.clear();
        expect(store.get('foo',),).toBe(42,);
        expect(store.get('bar',),).toBe(99,);
      },
    },),
    it({
      name: '.clear() - change events',
      fn: async () => {
        /**
         Store carrying changed and extra keys before the clear.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 42,
            bar: 'hello',
          },
        },);
        store.set({
          key: 'foo',
          value: 100,
        },);
        store.set({
          key: 'baz',
          value: 'extra',
        },);
        /**
         Whole-store change records collected across the clear.
         */
        const events: StoreChange<Record<string, unknown>>[] = [];
        /**
         Subscription recording each whole-store change.
         */
        const unsubscribe = store.onDidAnyChange(function recordStoreChange(change,): void {
          events.push(structuredClone(change,),);
        },);
        // Clear emits exactly one change event carrying the final state.
        store.clear();
        unsubscribe();
        expect(events,).toHaveLength(1,);
        expect(events[0]?.newValue,).toEqual({
          foo: 42,
          bar: 'hello',
        },);
        expect(events[0]?.oldValue,).toEqual({
          foo: 100,
          bar: 'hello',
          baz: 'extra',
        },);
      },
    },),
    it({
      name: '.clear() - without dot notation',
      fn: async () => {
        /**
         Store whose literal dotted default key survives the clear as one key.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          accessPropertiesByDotNotation: false,
          defaults: {
            foo: 42,
            'nested.key': 'value',
          },
        },);
        store.set({
          key: 'foo',
          value: 100,
        },);
        store.set({
          key: 'other',
          value: 'test',
        },);
        store.clear();
        expect(store.get('foo',),).toBe(42,);
        // The dotted default stays one literal key instead of nesting.
        expect(store.get('nested.key',),).toBe('value',);
        expect(store.get('other',),).toBeUndefined();
        expect(store.size,).toBe(2,);
      },
    },),
    it({
      name: '.clear() - validation error',
      fn: async () => {
        /**
         Store whose defaults contain a JSON-unsupported value.
         */
        const store = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 42,
            bad: function invalidDefault(): void {},
          },
        },);
        // Construction succeeds, but the invalid default never reaches the store.
        expect(store.get('foo',),).toBe(42,);
        expect(store.get('bad',),).toBeUndefined();
        /**
         Error thrown when clear validates the defaults and meets the invalid value.
         */
        const error = captureThrown((): unknown => store.clear(),);
        expect(error,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(error.message,).toBe(
          "Setting a value of type `function` for key `bad` is not allowed as it's not supported by JSON",
        );
      },
    },),

    //endregion Writes

    //region Store surfaces

    it({
      name: '.size',
      fn: async () => {
        /**
         Store holding exactly one item.
         */
        const config = createFreshStore();
        config.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(config.size,).toBe(1,);
      },
    },),
    it({
      name: '.store',
      fn: async () => {
        /**
         Store whose whole contents are read and then replaced.
         */
        const config = createFreshStore();
        config.set({
          key: 'foo',
          value: 'bar',
        },);
        config.set({
          key: 'baz.boo',
          value: true,
        },);
        expect(config.store,).toEqual({
          foo: 'bar',
          baz: {
            boo: true,
          },
        },);
        // Assigning the store replaces everything.
        config.store = { replaced: 'yes', };
        expect(config.store,).toEqual({ replaced: 'yes', },);
        expect(config.get('foo',),).toBeUndefined();
        expect(config.size,).toBe(1,);
      },
    },),
    it({
      name: 'ensure `.store` is always an object',
      fn: async () => {
        /**
         Directory backing the store,
         removed underneath it mid-test.
         */
        const directory = createTempDirectory();
        /**
         Store whose backing directory is removed underneath it.
         */
        const config = createConf({ cwd: directory, },);
        rmSync(
          directory,
          {
            force: true,
            recursive: true,
          },
        );
        // Reads keep reporting an object store even with the file tree gone.
        expect(config.store,).toEqual({},);
        expect(config.get('foo',),).toBeUndefined();
      },
    },),
    it({
      name: 'instance is iterable',
      fn: async () => {
        /**
         Store whose entries iterate in insertion order.
         */
        const config = createFreshStore();
        config.set({
          values: {
            foo: FIXTURE,
            bar: FIXTURE,
          },
        },);
        expect([...config,],).toEqual([
          [
            'foo',
            FIXTURE,
          ],
          [
            'bar',
            FIXTURE,
          ],
        ],);
      },
    },),

    //endregion Store surfaces

    //region Persistence

    it({
      name: "doesn't write to disk upon instantiation if and only if the store didn't change",
      fn: async () => {
        /**
         Store created without defaults before anything is written.
         */
        const untouched = createConf({ cwd: createTempDirectory(), },);
        expect(existsSync(untouched.path,),).toBe(false,);
        /**
         Store whose defaults change the merge,
         forcing the instantiation write.
         */
        const withDefaults = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 'bar',
          },
        },);
        expect(existsSync(withDefaults.path,),).toBe(true,);
        /**
         Directory holding a config file already at the defaults merge.
         */
        const stableDirectory = createTempDirectory();
        writeConfigFile({
          directory: stableDirectory,
          data: {
            foo: 'bar',
          },
        },);
        /**
         Config file path whose modification time must not move.
         */
        const stablePath = join(
          stableDirectory,
          'config.json',
        );
        /**
         Modification time recorded before the unchanged instantiation.
         */
        const before = statSync(stablePath,).mtimeMs;
        createConf({
          cwd: stableDirectory,
          defaults: {
            foo: 'bar',
          },
        },);
        expect(statSync(stablePath,).mtimeMs,).toBe(before,);
        // The file only appears after the first real write.
        untouched.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(existsSync(untouched.path,),).toBe(true,);
      },
    },),

    //endregion Persistence

    //region Migrations

    it({
      name: 'migrations - reserved key protection',
      fn: async () => {
        /**
         Store whose reserved bookkeeping key must stay write-proof.
         */
        const config = createFreshStore();
        /**
         Error thrown when the single-key form targets `__internal__`.
         */
        const directError = captureThrown((): unknown => config.set({
          key: '__internal__',
          value: {},
        },),);
        expect(directError,).toBeInstanceOf(ReservedKeyError,);
        expect(directError.name,).toBe('ReservedKeyError',);
        expect(directError.message,).toBe(
          "Please don't use the __internal__ key, as it's used to manage this module internal operations.",
        );
        /**
         Error thrown when the multi-item form carries `__internal__`.
         */
        const valuesError = captureThrown((): unknown => config.set({
          values: {
            __internal__: {},
          },
        },),);
        expect(valuesError,).toBeInstanceOf(ReservedKeyError,);
        expect(valuesError.message,).toBe(
          "Please don't use the __internal__ key, as it's used to manage this module internal operations.",
        );
        /**
         Error thrown when a dotted write reaches inside `__internal__`.
         */
        const dottedError = captureThrown((): unknown => config.set({
          key: '__internal__.migrations.version',
          value: '0.0.1',
        },),);
        expect(dottedError,).toBeInstanceOf(ReservedKeyError,);
        expect(dottedError.message,).toBe(
          "Please don't use the __internal__ key, as it's used to manage this module internal operations.",
        );
      },
    },),
    it({
      name: 'migrations - should not expose the internal key',
      fn: async () => {
        /**
         Directory holding a pre-existing config so migrations run.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Store whose migration records a version under the reserved key.
         */
        const store = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function toV1(migrationStore,): void {
              migrationStore.set({
                key: 'foo',
                value: 'bar',
              },);
            },
          },
        },);
        expect(store.store,).toEqual({ foo: 'bar', },);
        expect(store.get('__internal__',),).toBeUndefined();
        expect(store.has('__internal__',),).toBe(false,);
        expect(store.size,).toBe(1,);
        expect([...store,],).toEqual([
          [
            'foo',
            'bar',
          ],
        ],);
        // The version is still persisted.
        expect(readMigrationVersion({ configPath: store.path, },),).toBe('1.0.0',);
      },
    },),

    //endregion Migrations
  ],
},);
