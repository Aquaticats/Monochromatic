/**
 Change-subscription behavior: key-change pairs,
 whole-store pairs,
 omission rules,
 unsubscribe,
 and the clear and appendToArray events.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createConf,
  InvalidCallbackError,
  InvalidKeyError,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
} from './test-support.ts';

/**
 Value stored under one key in the fixtures below.
 */
const FIXTURE_VALUE = '🦄';

/**
 One change payload as the subscriptions deliver it,
 with the omitted sides staying absent.
 */
type RecordedChange = {
  readonly newValue?: unknown;
  readonly oldValue?: unknown;
};

await describe({
  name: 'change events',
  children: [
    it({
      name: 'reports new and old values for a changed key and stops after unsubscribe',
      fn: async () => {
        /**
         Store carrying one flat and one nested key.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        conf.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);
        /**
         Changes reported for `foo`.
         */
        const fooChanges: RecordedChange[] = [];
        /**
         Subscription that is removed again below.
         */
        const unsubscribeFoo = conf.onDidChange({
          key: 'foo',
          callback: function recordFooChange(change: RecordedChange,): void {
            fooChanges.push(change,);
          },
        },);
        conf.set({
          key: 'foo',
          value: '🐴',
        },);
        expect(fooChanges,).toEqual([
          {
            newValue: '🐴',
            oldValue: FIXTURE_VALUE,
          },
        ],);
        unsubscribeFoo();
        conf.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);
        expect(fooChanges,).toHaveLength(1,);

        conf.set({
          key: 'baz.boo',
          value: FIXTURE_VALUE,
        },);
        /**
         Changes reported for the dotted key `baz.boo`.
         */
        const bazChanges: RecordedChange[] = [];
        /**
         Subscription over the dotted key,
         removed again below.
         */
        const unsubscribeBaz = conf.onDidChange({
          key: 'baz.boo',
          callback: function recordBazChange(change: RecordedChange,): void {
            bazChanges.push(change,);
          },
        },);
        conf.set({
          key: 'baz.boo',
          value: '🐴',
        },);
        expect(bazChanges,).toEqual([
          {
            newValue: '🐴',
            oldValue: FIXTURE_VALUE,
          },
        ],);
        unsubscribeBaz();
        conf.set({
          key: 'baz.boo',
          value: FIXTURE_VALUE,
        },);
        expect(bazChanges,).toHaveLength(1,);
      },
    },),

    it({
      name: 'omits oldValue when a key is set for the first time',
      fn: async () => {
        /**
         Store without any value for the watched key yet.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        /**
         Changes reported for the freshly created key.
         */
        const history: RecordedChange[] = [];
        conf.onDidChange({
          key: 'fresh',
          callback: function recordFreshChange(change: RecordedChange,): void {
            history.push(change,);
          },
        },);
        conf.set({
          key: 'fresh',
          value: 'born',
        },);
        /**
         First reported change,
         which must carry only the new side.
         */
        const [change] = history;
        expect(change,).toEqual({
          newValue: 'born',
        },);
        expect(Object.hasOwn(
          change ?? {},
          'oldValue',
        ),).toBe(false,);
      },
    },),

    it({
      name: 'omits newValue when the watched key is deleted',
      fn: async () => {
        /**
         Store carrying the key that will be deleted.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        conf.set({
          key: 'gone',
          value: FIXTURE_VALUE,
        },);
        /**
         Changes reported for the deleted key.
         */
        const history: RecordedChange[] = [];
        conf.onDidChange({
          key: 'gone',
          callback: function recordDeletion(change: RecordedChange,): void {
            history.push(change,);
          },
        },);
        conf.delete('gone',);
        /**
         First reported change,
         which must carry only the old side.
         */
        const [change] = history;
        expect(change,).toEqual({
          oldValue: FIXTURE_VALUE,
        },);
        expect(Object.hasOwn(
          change ?? {},
          'newValue',
        ),).toBe(false,);
      },
    },),

    it({
      name: 'does not call the key subscription when a write repeats the current value',
      fn: async () => {
        /**
         Store carrying the current value.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        /**
         Changes reported for `foo`.
         */
        const history: RecordedChange[] = [];
        conf.onDidChange({
          key: 'foo',
          callback: function recordChange(change: RecordedChange,): void {
            history.push(change,);
          },
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(history,).toHaveLength(0,);
      },
    },),

    it({
      name: 'does not call the store subscription when a write repeats the current store',
      fn: async () => {
        /**
         Store carrying the current contents.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        /**
         Store changes reported for identical rewrites.
         */
        const history: RecordedChange[] = [];
        conf.onDidAnyChange(function recordStoreChange(change: RecordedChange,): void {
          history.push(change,);
        },);
        conf.store = {
          foo: 'bar',
        };
        expect(history,).toHaveLength(0,);
      },
    },),

    it({
      name: 'reports whole-store before and after pairs on any change',
      fn: async () => {
        /**
         Store carrying one key that will change and one that will be deleted.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        conf.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);
        /**
         Store changes reported from the subscription moment on.
         */
        const history: RecordedChange[] = [];
        /**
         Subscription removed again below.
         */
        const unsubscribe = conf.onDidAnyChange(function recordStoreChange(change: RecordedChange,): void {
          history.push(change,);
        },);
        conf.set({
          key: 'foo',
          value: '🐴',
        },);
        expect(history,).toEqual([
          {
            newValue: {
              foo: '🐴',
            },
            oldValue: {
              foo: FIXTURE_VALUE,
            },
          },
        ],);

        conf.delete('foo',);
        expect(history,).toHaveLength(2,);
        expect(history[1],).toEqual({
          newValue: {},
          oldValue: {
            foo: '🐴',
          },
        },);

        unsubscribe();
        conf.set({
          key: 'foo',
          value: FIXTURE_VALUE,
        },);
        expect(history,).toHaveLength(2,);
      },
    },),

    it({
      name: 'reports one whole-store pair and one key pair when clear() restores defaults',
      fn: async () => {
        /**
         Store whose clear resets everything to its defaults.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          defaults: {
            foo: 42,
            bar: 'hello',
          },
        },);
        conf.set({
          key: 'foo',
          value: 100,
        },);
        conf.set({
          key: 'baz',
          value: 'extra',
        },);

        /**
         Store changes reported across the clear.
         */
        const storeChanges: RecordedChange[] = [];
        /**
         Key changes reported across the clear.
         */
        const keyChanges: RecordedChange[] = [];
        conf.onDidAnyChange(function recordStoreChange(change: RecordedChange,): void {
          storeChanges.push(change,);
        },);
        conf.onDidChange({
          key: 'foo',
          callback: function recordKeyChange(change: RecordedChange,): void {
            keyChanges.push(change,);
          },
        },);

        conf.clear();

        expect(storeChanges,).toEqual([
          {
            newValue: {
              foo: 42,
              bar: 'hello',
            },
            oldValue: {
              foo: 100,
              bar: 'hello',
              baz: 'extra',
            },
          },
        ],);
        expect(keyChanges,).toEqual([
          {
            newValue: 42,
            oldValue: 100,
          },
        ],);
      },
    },),

    it({
      name: 'reports one whole-store pair and one key pair when appendToArray() grows an array',
      fn: async () => {
        /**
         Store whose array key is created by the first append.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);

        /**
         Store changes reported across the appends.
         */
        const storeChanges: RecordedChange[] = [];
        /**
         Key changes reported across the appends.
         */
        const keyChanges: RecordedChange[] = [];
        conf.onDidAnyChange(function recordStoreChange(change: RecordedChange,): void {
          storeChanges.push(change,);
        },);
        conf.onDidChange({
          key: 'items',
          callback: function recordKeyChange(change: RecordedChange,): void {
            keyChanges.push(change,);
          },
        },);

        conf.appendToArray({
          key: 'items',
          value: 'first',
        },);
        expect(keyChanges,).toEqual([
          {
            newValue: [
              'first',
            ],
          },
        ],);
        expect(storeChanges,).toEqual([
          {
            newValue: {
              items: [
                'first',
              ],
            },
            oldValue: {},
          },
        ],);

        conf.appendToArray({
          key: 'items',
          value: 'second',
        },);
        expect(keyChanges,).toHaveLength(2,);
        expect(keyChanges[1],).toEqual({
          newValue: [
            'first',
            'second',
          ],
          oldValue: [
            'first',
          ],
        },);
        expect(storeChanges,).toHaveLength(2,);
        expect(storeChanges[1],).toEqual({
          newValue: {
            items: [
              'first',
              'second',
            ],
          },
          oldValue: {
            items: [
              'first',
            ],
          },
        },);
      },
    },),

    it({
      name: 'throws InvalidKeyError when the watched key is not a string',
      fn: async () => {
        /**
         Store whose subscription receives a mistyped key;
         cast because the runtime check under test is what rejects it.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        const invalidKeyInput = {
          key: 1,
          callback: function ignored(): void {},
        } as unknown as Parameters<typeof conf.onDidChange>[0];
        expect(function subscribeWithInvalidKey(): unknown {
          return conf.onDidChange(invalidKeyInput,);
        },).toThrow(InvalidKeyError,);
      },
    },),

    it({
      name: 'throws InvalidCallbackError when a subscription callback is not a function',
      fn: async () => {
        /**
         Store whose subscriptions receive mistyped callbacks;
         cast because the runtime check under test is what rejects it.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
        },);
        const invalidCallbackInput = {
          key: 'foo',
          callback: 'nope',
        } as unknown as Parameters<typeof conf.onDidChange>[0];
        expect(function subscribeWithInvalidCallback(): unknown {
          return conf.onDidChange(invalidCallbackInput,);
        },).toThrow(InvalidCallbackError,);
        const invalidAnyChangeCallback = 'nope' as unknown as Parameters<typeof conf.onDidAnyChange>[0];
        expect(function subscribeAnyChangeWithInvalidCallback(): unknown {
          return conf.onDidAnyChange(invalidAnyChangeCallback,);
        },).toThrow(InvalidCallbackError,);
      },
    },),
  ],
},);
