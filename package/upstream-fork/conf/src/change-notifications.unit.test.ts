/**
 Change-subscription behavior of `subscribeValueChange` and
 `subscribeStoreChange` over a plain `EventTarget`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type StoreChange,
  type ValueChange,
  subscribeStoreChange,
  subscribeValueChange,
} from '../dist/final/neutral/index.mjs';

/**
 Pulls the single recorded change out of a list,
 failing loudly when the subscription recorded a different count,
 so absence assertions read from a non-optional value.
 
 @param changes - Changes recorded by the subscription under test.
 
 @returns The only recorded change.
 
 @example
 ```ts
 const change = onlyChange(changes);
 ```
 */
function onlyChange<Value>(changes: readonly ValueChange<Value>[],): ValueChange<Value> {
  /**
   The single recorded change,
   absent when the count is not exactly one.
   */
  const [change,] = changes;
  if ((changes.length !== 1) || (change === undefined))
    throw new Error(`Expected exactly one recorded change, got ${String(changes.length,)}`,);
  return change;
}

await describe({
  name: 'change subscriptions',
  children: [
    it({
      name: 'reports newValue and oldValue when the watched value changes',
      fn: async () => {
        /**
         Event target standing in for the store's change emitter.
         */
        const events = new EventTarget();
        /**
         Mutable watched state the getter reads.
         */
        const state = {
          value: 'light' as unknown,
        };
        /**
         Changes recorded by the subscription callback.
         */
        const changes: ValueChange<string>[] = [];
        /**
         Subscription under test over the watched state.
         */
        const unsubscribe = subscribeValueChange<string>({
          events,
          getter: function readWatchedValue(): unknown {
            return state.value;
          },
          callback: function onChange(change,): void {
            changes.push(change,);
          },
        },);
        state.value = 'dark';
        events.dispatchEvent(new Event('change',),);
        expect(changes,).toEqual([
          {
            newValue: 'dark',
            oldValue: 'light',
          },
        ],);
        state.value = 'midnight';
        events.dispatchEvent(new Event('change',),);
        expect(changes,).toEqual([
          {
            newValue: 'dark',
            oldValue: 'light',
          },
          {
            newValue: 'midnight',
            oldValue: 'dark',
          },
        ],);
        unsubscribe();
      },
    },),

    it({
      name: 'omits oldValue when the key was missing before the change',
      fn: async () => {
        /**
         Event target standing in for the store's change emitter.
         */
        const events = new EventTarget();
        /**
         Mutable watched state starting absent.
         */
        const state: {
          value: unknown;
        } = {
          value: undefined,
        };
        /**
         Changes recorded by the subscription callback.
         */
        const changes: ValueChange<string>[] = [];
        /**
         Subscription under test over the watched state.
         */
        const unsubscribe = subscribeValueChange<string>({
          events,
          getter: function readWatchedValue(): unknown {
            return state.value;
          },
          callback: function onChange(change,): void {
            changes.push(change,);
          },
        },);
        state.value = 'dark';
        events.dispatchEvent(new Event('change',),);
        /**
         Recorded change for the first set.
         */
        const change = onlyChange(changes,);
        expect('newValue' in change,).toBe(true,);
        expect('oldValue' in change,).toBe(false,);
        expect(change.newValue,).toBe('dark',);
        unsubscribe();
      },
    },),

    it({
      name: 'omits newValue when the key was deleted',
      fn: async () => {
        /**
         Event target standing in for the store's change emitter.
         */
        const events = new EventTarget();
        /**
         Mutable watched state starting present.
         */
        const state = {
          value: 'dark' as unknown,
        };
        /**
         Changes recorded by the subscription callback.
         */
        const changes: ValueChange<string>[] = [];
        /**
         Subscription under test over the watched state.
         */
        const unsubscribe = subscribeValueChange<string>({
          events,
          getter: function readWatchedValue(): unknown {
            return state.value;
          },
          callback: function onChange(change,): void {
            changes.push(change,);
          },
        },);
        state.value = undefined;
        events.dispatchEvent(new Event('change',),);
        /**
         Recorded change for the deletion.
         */
        const change = onlyChange(changes,);
        expect('newValue' in change,).toBe(false,);
        expect('oldValue' in change,).toBe(true,);
        expect(change.oldValue,).toBe('dark',);
        unsubscribe();
      },
    },),

    it({
      name: 'stays quiet when the watched value is deep-equal to the previous one',
      fn: async () => {
        /**
         Event target standing in for the store's change emitter.
         */
        const events = new EventTarget();
        /**
         Mutable watched state holding freshly-built equal objects.
         */
        const state: {
          value: unknown;
        } = {
          value: {
            mode: 'dark',
          },
        };
        /**
         Changes recorded by the subscription callback.
         */
        const changes: ValueChange<Record<string, string>>[] = [];
        /**
         Subscription under test over the watched state.
         */
        const unsubscribe = subscribeValueChange<Record<string, string>>({
          events,
          getter: function readWatchedValue(): unknown {
            return state.value;
          },
          callback: function onChange(change,): void {
            changes.push(change,);
          },
        },);
        state.value = {
          mode: 'dark',
        };
        events.dispatchEvent(new Event('change',),);
        expect(changes,).toHaveLength(0,);
        state.value = {
          mode: 'light',
        };
        events.dispatchEvent(new Event('change',),);
        expect(changes,).toEqual([
          {
            newValue: {
              mode: 'light',
            },
            oldValue: {
              mode: 'dark',
            },
          },
        ],);
        unsubscribe();
      },
    },),

    it({
      name: 'stops delivering watched-value changes after unsubscribe',
      fn: async () => {
        /**
         Event target standing in for the store's change emitter.
         */
        const events = new EventTarget();
        /**
         Mutable watched state the getter reads.
         */
        const state = {
          value: 'light' as unknown,
        };
        /**
         Changes recorded by the subscription callback.
         */
        const changes: ValueChange<string>[] = [];
        /**
         Subscription under test over the watched state.
         */
        const unsubscribe = subscribeValueChange<string>({
          events,
          getter: function readWatchedValue(): unknown {
            return state.value;
          },
          callback: function onChange(change,): void {
            changes.push(change,);
          },
        },);
        unsubscribe();
        state.value = 'dark';
        events.dispatchEvent(new Event('change',),);
        expect(changes,).toHaveLength(0,);
      },
    },),

    it({
      name: 'reports whole-store pairs when the store snapshot changes',
      fn: async () => {
        /**
         Event target standing in for the store's change emitter.
         */
        const events = new EventTarget();
        /**
         Mutable whole-store state the snapshot getter reads.
         */
        const state = {
          store: {
            theme: 'light',
          },
        };
        /**
         Store changes recorded by the subscription callback.
         */
        const changes: StoreChange<Record<string, unknown>>[] = [];
        /**
         Store subscription under test over the mutable state.
         */
        const unsubscribe = subscribeStoreChange<Record<string, unknown>>({
          events,
          getSnapshot: function readStoreSnapshot(): Record<string, unknown> {
            return state.store;
          },
          callback: function onChange(change,): void {
            changes.push(change,);
          },
        },);
        state.store = {
          theme: 'dark',
        };
        events.dispatchEvent(new Event('change',),);
        expect(changes,).toEqual([
          {
            newValue: {
              theme: 'dark',
            },
            oldValue: {
              theme: 'light',
            },
          },
        ],);
        unsubscribe();
      },
    },),

    it({
      name: 'stays quiet when the store snapshot is deep-equal to the previous one',
      fn: async () => {
        /**
         Event target standing in for the store's change emitter.
         */
        const events = new EventTarget();
        /**
         Mutable whole-store state rebuilt into equal snapshots.
         */
        const state = {
          store: {
            theme: 'dark',
          },
        };
        /**
         Store changes recorded by the subscription callback.
         */
        const changes: StoreChange<Record<string, unknown>>[] = [];
        /**
         Store subscription under test over the mutable state.
         */
        const unsubscribe = subscribeStoreChange<Record<string, unknown>>({
          events,
          getSnapshot: function readStoreSnapshot(): Record<string, unknown> {
            return state.store;
          },
          callback: function onChange(change,): void {
            changes.push(change,);
          },
        },);
        state.store = {
          theme: 'dark',
        };
        events.dispatchEvent(new Event('change',),);
        expect(changes,).toHaveLength(0,);
        unsubscribe();
      },
    },),

    it({
      name: 'stops delivering whole-store changes after unsubscribe',
      fn: async () => {
        /**
         Event target standing in for the store's change emitter.
         */
        const events = new EventTarget();
        /**
         Mutable whole-store state the snapshot getter reads.
         */
        const state = {
          store: {
            theme: 'light',
          },
        };
        /**
         Store changes recorded by the subscription callback.
         */
        const changes: StoreChange<Record<string, unknown>>[] = [];
        /**
         Store subscription under test over the mutable state.
         */
        const unsubscribe = subscribeStoreChange<Record<string, unknown>>({
          events,
          getSnapshot: function readStoreSnapshot(): Record<string, unknown> {
            return state.store;
          },
          callback: function onChange(change,): void {
            changes.push(change,);
          },
        },);
        unsubscribe();
        state.store = {
          theme: 'dark',
        };
        events.dispatchEvent(new Event('change',),);
        expect(changes,).toHaveLength(0,);
      },
    },),
  ],
},);
