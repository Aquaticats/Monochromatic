/**
 Change subscriptions over the store's `change` event.
 
 Both subscription kinds diff snapshots through `isDeepStrictEqual` and
 call back only on real differences,
 matching upstream `conf` 15.1.0's `_handleValueChange` and
 `_handleStoreChange`.
 
 @module
 */

import { isDeepStrictEqual, } from 'node:util';

import type {
  OnDidAnyChangeCallback,
  OnDidChangeCallback,
  StoreChange,
  Unsubscribe,
  ValueChange,
} from './types.ts';

//region Payload shaping

/**
 Builds one key-change payload,
 omitting the side whose value is absent.
 
 @param newValue - Value after the change;
 `undefined` means the key was deleted.
 
 @param oldValue - Value before the change;
 `undefined` means the key never existed.
 
 @returns Payload for {@link OnDidChangeCallback}.
 
 @example
 ```ts
 toValueChange('dark', undefined); // => { newValue: 'dark' }
 ```
 */
function toValueChange<Value>({
  newValue,
  oldValue,
}: {
  readonly newValue: unknown;
  readonly oldValue: unknown;
},): ValueChange<Value> {
  return {
    ...(newValue === undefined ? {} : {
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the change pair arrives as dynamic store state and is re-exposed as the caller's watched value type at this subscription boundary.
      newValue: newValue as Value,
    }),
    ...(oldValue === undefined ? {} : {
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the change pair arrives as dynamic store state and is re-exposed as the caller's watched value type at this subscription boundary.
      oldValue: oldValue as Value,
    }),
  };
}

//endregion Payload shaping

//region Subscriptions

/**
 Subscribes to changes of one key's value.
 
 @param events - Store event target dispatching `change`.
 
 @param getter - Reads the watched value's current state.
 
 @param callback - Receives the changed value pair.
 
 @returns Unsubscribe function stopping the subscription.
 
 @example
 ```ts
 const unsubscribe = subscribeValueChange({
   events: config.events,
   getter: function getter(): unknown { return config.get('theme'); },
   callback: function onChange(change): void { console.log(change.newValue); },
 });
 ```
 */
export function subscribeValueChange<Value>({
  events,
  getter,
  callback,
}: {
  readonly events: EventTarget;
  readonly getter: () => unknown;
  readonly callback: OnDidChangeCallback<Value>;
},): Unsubscribe {
  /**
   Last observed value; starts as whatever the getter reports now.
   */
  const state: {
    currentValue: unknown;
  } = {
    currentValue: getter(),
  };
  /**
   Diffs the fresh value against the last one and reports real changes.
   */
  function onChange(): void {
    /**
     Value before this event.
     */
    const oldValue = state.currentValue;
    /**
     Value after this event.
     */
    const newValue = getter();
    if (isDeepStrictEqual(
      newValue,
      oldValue,
    ))
      return;
    state.currentValue = newValue;
    // oxlint-disable-next-line promise/prefer-await-to-callbacks -- event-subscription API mirroring upstream `conf`'s onDidChange; there is no promise to await, the callback IS the subscription payload.
    callback(toValueChange<Value>({
      newValue,
      oldValue,
    },),);
  }
  events.addEventListener(
    'change',
    onChange,
  );
  return function unsubscribe(): void {
    events.removeEventListener(
      'change',
      onChange,
    );
  };
}

/**
 Subscribes to whole-store changes.
 
 @param events - Store event target dispatching `change`.
 
 @param getSnapshot - Reads the current whole store.
 
 @param callback - Receives the changed store pair.
 
 @returns Unsubscribe function stopping the subscription.
 
 @example
 ```ts
 const unsubscribe = subscribeStoreChange({
   events: config.events,
   getSnapshot: function getSnapshot(): Record<string, unknown> { return config.store; },
   callback: function onChange(change): void { console.log(change.newValue); },
 });
 ```
 */
export function subscribeStoreChange<T extends Record<string, unknown>>({
  events,
  getSnapshot,
  callback,
}: {
  readonly events: EventTarget;
  readonly getSnapshot: () => T;
  readonly callback: OnDidAnyChangeCallback<T>;
},): Unsubscribe {
  /**
   Last observed store; starts as the current snapshot.
   */
  const state: {
    currentSnapshot: T;
  } = {
    currentSnapshot: getSnapshot(),
  };
  /**
   Diffs the fresh store against the last one and reports real changes.
   */
  function onChange(): void {
    /**
     Store before this event.
     */
    const oldValue = state.currentSnapshot;
    /**
     Store after this event.
     */
    const newValue = getSnapshot();
    if (isDeepStrictEqual(
      newValue,
      oldValue,
    ))
      return;
    state.currentSnapshot = newValue;
    // oxlint-disable-next-line promise/prefer-await-to-callbacks -- event-subscription API mirroring upstream `conf`'s onDidAnyChange; there is no promise to await, the callback IS the subscription payload.
    callback({
      newValue,
      oldValue,
    },);
  }
  events.addEventListener(
    'change',
    onChange,
  );
  return function unsubscribe(): void {
    events.removeEventListener(
      'change',
      onChange,
    );
  };
}

//endregion Subscriptions
