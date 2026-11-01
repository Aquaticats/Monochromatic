/**
 Asynchronous observable value container with method-based get and set.
 
 @module
 */

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
   Internal store backing the methods; held on an object property so mutation avoids a function-root `let`.
   */
  const state: { current: T; } = { current: initialValue, };
  return {
    getValue: function getValue(): T {
      return state.current;
    },
    setValue: async function setValue(newValue: T,): Promise<void> {
      /**
       Snapshot of the prior value preserved for the change handler call.
       */
      const old = state.current;
      state.current = newValue;
      await onChange(
        newValue,
        old,
      );
    },
  };
}
/* oxlint-enable eslint/require-await, typescript/require-await */
