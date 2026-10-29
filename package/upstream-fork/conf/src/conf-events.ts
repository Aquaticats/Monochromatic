/**
 Change-subscription methods for the config store.
 
 Groups upstream `conf`'s `onDidChange` and `onDidAnyChange`
 implementations behind one factory so the store object in `conf.ts` stays
 a thin wiring layer.
 
 @module
 */

import {
  InvalidCallbackError,
  InvalidKeyError,
} from './errors.ts';
import {
  subscribeStoreChange,
  subscribeValueChange,
} from './change-notifications.ts';
import type {
  OnDidAnyChangeCallback,
  OnDidChangeCallback,
  Unsubscribe,
} from './types.ts';

//region Types

/**
 The store's change-subscription surface as implemented here,
 before the public `Conf` overload typing is applied at the factory edge.
 
 @example
 ```ts
 const eventMethods = createEventMethods(context);
 eventMethods.onDidAnyChange(callback);
 ```
 */
export type EventApi = {
  /**
   Subscribes to changes of one key's value.
   */
  readonly onDidChange: (input: {
    readonly key: string;
    readonly callback: OnDidChangeCallback<unknown>;
  },) => Unsubscribe;
  /**
   Subscribes to whole-store changes.
   */
  readonly onDidAnyChange: (callback: OnDidAnyChangeCallback<Record<string, unknown>>,) => Unsubscribe;
};

/**
 Everything the subscription methods need from the store around them.
 
 @example
 ```ts
 const eventMethods = createEventMethods({
   events,
   getStore: function getStore(): Record<string, unknown> { return {}; },
   readValue: function readValue(): unknown { return undefined; },
 });
 ```
 */
export type EventContext<T extends Record<string, unknown>> = {
  /**
   Store event target dispatching `change`.
   */
  readonly events: EventTarget;
  /**
   Cache-aware whole-store read.
   */
  readonly getStore: () => T;
  /**
   Public-semantics single-key read used to watch one key.
   */
  readonly readValue: (key: string,) => unknown;
};

//endregion Types

//region Factory

/**
 Builds the change-subscription methods bound to one store's context.
 
 @param context - Event target,
 store read,
 and key read closures.
 
 @returns Subscription methods ready to attach to the store object.
 
 @example
 ```ts
 const eventMethods = createEventMethods(context);
 const unsubscribe = eventMethods.onDidChange({ key: 'theme', callback, });
 ```
 */
export function createEventMethods<T extends Record<string, unknown>>(context: EventContext<T>,): EventApi {
  return {
    onDidChange: function onDidChange(input: {
      readonly key: string;
      readonly callback: OnDidChangeCallback<unknown>;
    },): Unsubscribe {
      if ((typeof input.key) !== 'string')
        throw new InvalidKeyError(`Expected \`key\` to be of type \`string\`, got ${typeof input.key}`);
      if ((typeof input.callback) !== 'function')
        throw new InvalidCallbackError(`Expected \`callback\` to be of type \`function\`, got ${typeof input.callback}`);
      return subscribeValueChange({
        events: context.events,
        getter: function readWatchedValue(): unknown {
          return context.readValue(input.key,);
        },
        callback: input.callback,
      },);
    },

    // oxlint-disable-next-line promise/prefer-await-to-callbacks -- event-subscription API mirroring upstream `conf`'s onDidAnyChange; there is no promise to await, the callback IS the subscription payload.
    onDidAnyChange: function onDidAnyChange(callback: OnDidAnyChangeCallback<Record<string, unknown>>,): Unsubscribe {
      if ((typeof callback) !== 'function')
        throw new InvalidCallbackError(`Expected \`callback\` to be of type \`function\`, got ${typeof callback}`);
      return subscribeStoreChange({
        events: context.events,
        getSnapshot: context.getStore,
        callback,
      },);
    },
  };
}

//endregion Factory
