/**
 Asynchronous observable value container with method-based get and set.
 
 @module
 */

import { createValueStore, } from './value-store.ts';

/**
 Observable value container with method-based access and an awaitable setter.
 
 `getValue` reads the current value synchronously; `setValue` stores the new
 value, then awaits the registered change handler before resolving.
 */
export type ObservableAsync<T,> = {
  /**
   Reads the current observed value synchronously.
   */
  getValue: () => T;
  /**
   Stores a new value, then awaits the change handler before resolving.
   */
  setValue: (newValue: T,) => Promise<void>;
};

/**
 Result of a handler that may run synchronously or asynchronously: either a
 direct `T` or a promise of `T`. Expressed as a generic so the `void` use site
 stays a type-reference argument rather than a `T | void` union escape hatch.
 
 @example
 ```ts
 const handler: (n: number,) => MaybePromise<void> = async function handler(n,) {
   await persist(n,);
 };
 ```
 */
export type MaybePromise<T,> = T | Promise<T>;

/* oxlint-disable eslint/require-await, typescript/require-await -- async wrapper enables top-level await of construction even though construction itself is synchronous */
/**
 Creates an observable value container that awaits a handler when the value is set.
 
 Arrangement with `createObservable`: the two sides share one core. Each wraps
 {@link createValueStore}, which owns the value container and the order of its update; this side
 adds only the awaited handler discipline, and neither side is generated from the other.
 
 Construction is synchronous; the async wrapper lets callers `await` it in top-level module code.
 State updates before `onChange` runs, so a handler that reads `getValue()` sees the new value.
 `setValue` awaits `onChange`, so a rejected handler rejects the `setValue` promise the caller awaits.
 
 @param initialValue - Starting value, returned by `getValue` until the first `setValue`
 
 @param onChange - Callback invoked after each value change with new then previous value; may be async
 
 @returns {@link ObservableAsync} container whose `setValue` awaits onChange before resolving
 
 @example
 ```ts
 const feeds = await createObservableAsync({
   initialValue: [] as readonly string[],
   onChange: async function onChange(next, prev) {
     await persist(next);
   },
 });
 await feeds.setValue(['a', 'b',]); // resolves only after persist completes
 feeds.getValue(); // ['a', 'b']
 ```
 */
export async function createObservableAsync<T,>(
  {
    initialValue,
    onChange,
  }: {
    readonly initialValue: T;
    readonly onChange: (
      newValue: T,
      oldValue: T,
    ) => MaybePromise<void>;
  },
): Promise<ObservableAsync<T>> {
  /**
   Store the container reads and writes; the sync variant wraps the same one.
   */
  const store = createValueStore(initialValue,);
  return {
    getValue: store.getValue,
    setValue: async function setValue(newValue: T,): Promise<void> {
      /**
       The store already holds the new value when it reports the one it replaced.
       */
      await onChange(
        newValue,
        store.replaceValue(newValue,),
      );
    },
  };
}
/* oxlint-enable eslint/require-await, typescript/require-await */
