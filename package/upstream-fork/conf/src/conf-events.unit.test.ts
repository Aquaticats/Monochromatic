/**
 Validation and delivery behavior of `createEventMethods`'s `onDidChange`
 and `onDidAnyChange` over a real `createConf` store context.
 
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
  createEventMethods,
  type EventContext,
  InvalidCallbackError,
  InvalidKeyError,
  type StoreChange,
  type ValueChange,
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
 Builds one real store plus its event-method surface,
 so subscriptions read values through production wiring.
 
 @returns Store under test and its event methods.
 
 @example
 ```ts
 const fixture = createEventFixture();
 fixture.eventMethods.onDidChange({ key: 'theme', callback, });
 ```
 */
function createEventFixture(): {
  readonly conf: ReturnType<typeof createConf>;
  readonly eventMethods: ReturnType<typeof createEventMethods>;
} {
  /**
   Real store whose events and reads back the subscriptions.
   */
  const conf = createConf({
    cwd: createTempDirectory(),
  },);
  /**
   Event-method context wired to the real store's events,
   store getter,
   and single-key read.
   */
  const context: EventContext<Record<string, unknown>> = {
    events: conf.events,
    getStore: function getStore(): Record<string, unknown> {
      return conf.store;
    },
    readValue: function readValue(key: string,): unknown {
      return conf.get(key,);
    },
  };
  return {
    conf,
    eventMethods: createEventMethods(context,),
  };
}

await describe({
  name: createEventMethods.name,
  children: [
    it({
      name: 'throws InvalidKeyError when the watched key is not a string',
      fn: async () => {
        /**
         Event-method fixture under test.
         */
        const fixture = createEventFixture();
        const invalidKeyInput = {
          key: 1,
          callback: function ignored(): void {},
        } as unknown as Parameters<typeof fixture.eventMethods.onDidChange>[0];
        /**
         Error thrown while subscribing with the non-string key.
         */
        const error = captureThrown(function subscribeWithInvalidKey(): unknown {
          return fixture.eventMethods
            .onDidChange(invalidKeyInput,);
        },);
        expect(error,).toBeInstanceOf(InvalidKeyError,);
        expect(caughtMessage(error,),).toBe('Expected `key` to be of type `string`, got number',);
      },
    },),

    it({
      name: 'throws InvalidCallbackError when the change callback is not a function',
      fn: async () => {
        /**
         Event-method fixture under test.
         */
        const fixture = createEventFixture();
        const invalidCallbackInput = {
          key: 'theme',
          callback: 'nope',
        } as unknown as Parameters<typeof fixture.eventMethods.onDidChange>[0];
        /**
         Error thrown while subscribing with the non-function callback.
         */
        const error = captureThrown(function subscribeWithInvalidCallback(): unknown {
          return fixture.eventMethods
            .onDidChange(invalidCallbackInput,);
        },);
        expect(error,).toBeInstanceOf(InvalidCallbackError,);
        expect(caughtMessage(error,),).toBe('Expected `callback` to be of type `function`, got string',);
      },
    },),

    it({
      name: 'reports watched-key changes through the change object',
      fn: async () => {
        /**
         Event-method fixture under test.
         */
        const fixture = createEventFixture();
        /**
         Changes recorded by the watched-key subscription.
         */
        const changes: ValueChange<unknown>[] = [];
        /**
         Subscription under test on the `theme` key.
         */
        const unsubscribe = fixture.eventMethods
          .onDidChange({
            key: 'theme',
            callback: function onChange(change,): void {
              changes.push(change,);
            },
          },);
        fixture.conf
          .set({
            key: 'theme',
            value: 'dark',
          },);
        fixture.conf
          .set({
            key: 'theme',
            value: 'light',
          },);
        unsubscribe();
        expect(changes,).toEqual([
          {
            newValue: 'dark',
          },
          {
            newValue: 'light',
            oldValue: 'dark',
          },
        ],);
      },
    },),

    it({
      name: 'ignores changes to keys the subscription is not watching',
      fn: async () => {
        /**
         Event-method fixture under test.
         */
        const fixture = createEventFixture();
        /**
         Changes recorded by the watched-key subscription.
         */
        const changes: ValueChange<unknown>[] = [];
        /**
         Subscription under test on the `theme` key.
         */
        const unsubscribe = fixture.eventMethods
          .onDidChange({
            key: 'theme',
            callback: function onChange(change,): void {
              changes.push(change,);
            },
          },);
        fixture.conf
          .set({
            key: 'locale',
            value: 'en',
          },);
        unsubscribe();
        expect(changes,).toHaveLength(0,);
      },
    },),

    it({
      name: 'stops reporting watched-key changes after unsubscribe',
      fn: async () => {
        /**
         Event-method fixture under test.
         */
        const fixture = createEventFixture();
        /**
         Changes recorded by the watched-key subscription.
         */
        const changes: ValueChange<unknown>[] = [];
        /**
         Subscription under test on the `theme` key.
         */
        const unsubscribe = fixture.eventMethods
          .onDidChange({
            key: 'theme',
            callback: function onChange(change,): void {
              changes.push(change,);
            },
          },);
        unsubscribe();
        fixture.conf
          .set({
            key: 'theme',
            value: 'dark',
          },);
        expect(changes,).toHaveLength(0,);
      },
    },),

    it({
      name: 'throws InvalidCallbackError when the whole-store callback is not a function',
      fn: async () => {
        /**
         Event-method fixture under test.
         */
        const fixture = createEventFixture();
        const invalidCallback = 'nope' as unknown as Parameters<typeof fixture.eventMethods.onDidAnyChange>[0];
        /**
         Error thrown while subscribing with the non-function callback.
         */
        const error = captureThrown(function subscribeAnyChangeWithInvalidCallback(): unknown {
          return fixture.eventMethods
            .onDidAnyChange(invalidCallback,);
        },);
        expect(error,).toBeInstanceOf(InvalidCallbackError,);
        expect(caughtMessage(error,),).toBe('Expected `callback` to be of type `function`, got string',);
      },
    },),

    it({
      name: 'reports whole-store changes as store pairs',
      fn: async () => {
        /**
         Event-method fixture under test.
         */
        const fixture = createEventFixture();
        /**
         Store changes recorded by the whole-store subscription.
         */
        const changes: StoreChange<Record<string, unknown>>[] = [];
        /**
         Whole-store subscription under test.
         */
        const unsubscribe = fixture.eventMethods
          .onDidAnyChange(function onChange(change,): void {
            changes.push(change,);
          },);
        fixture.conf
          .set({
            key: 'theme',
            value: 'dark',
          },);
        unsubscribe();
        expect(changes,).toEqual([
          {
            newValue: {
              theme: 'dark',
            },
            oldValue: {},
          },
        ],);
      },
    },),

    it({
      name: 'stops reporting whole-store changes after unsubscribe',
      fn: async () => {
        /**
         Event-method fixture under test.
         */
        const fixture = createEventFixture();
        /**
         Store changes recorded by the whole-store subscription.
         */
        const changes: StoreChange<Record<string, unknown>>[] = [];
        /**
         Whole-store subscription under test.
         */
        const unsubscribe = fixture.eventMethods
          .onDidAnyChange(function onChange(change,): void {
            changes.push(change,);
          },);
        unsubscribe();
        fixture.conf
          .set({
            key: 'theme',
            value: 'dark',
          },);
        expect(changes,).toHaveLength(0,);
      },
    },),
  ],
},);
