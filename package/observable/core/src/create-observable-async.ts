/**
 Asynchronous observable value container with method-based get and set.
 
 @module
 */

import { createObservableCore, } from './create-observable-core.ts';

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
 {@link createObservableCore}, which owns the container and the order of its update; this side
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
  return createObservableCore({
    initialValue,
    notify: async function notify(
      newValue: T,
      previous: T,
    ): Promise<void> {
      await onChange(
        newValue,
        previous,
      );
    },
  },);
}
/* oxlint-enable eslint/require-await, typescript/require-await */
