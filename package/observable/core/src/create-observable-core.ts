/**
 One observable value container, shared by the sync and async variants.

 Both variants hold one value, read it through `getValue`, and write it through `setValue`, which
 stores the new value before it reports the change. They differ only in what they do with the
 change handler, so the container and the order of its update live here once, and each public
 factory supplies only its handler discipline.

 @module
 */

/**
 Container over one value whose setter reports each change through a notification strategy.

 `TSetResult` is what the notification returns, so a synchronous strategy yields a `void` setter
 and an awaiting strategy yields a promise-returning one.
 */
export type ObservableContainer<T, TSetResult,> = {
  /**
   Reads the value the container holds now.
   */
  readonly getValue: () => T;
  /**
   Stores `newValue`, then reports it with the value it replaced.

   The container holds `newValue` before the notification runs, so a handler that reads
   `getValue` sees the new value.
   */
  readonly setValue: (newValue: T,) => TSetResult;
};

/**
 Creates a container over `initialValue` that reports each change through `notify`.

 @param initialValue - Value `getValue` reports until the first `setValue`

 @param notify - Change handler called after the container holds the new value, with the new value then the one it replaced

 @returns {@link ObservableContainer} whose `setValue` returns whatever `notify` returns

 @example
 ```ts
 const container = createObservableCore({
   initialValue: 0,
   notify: function notify(next, previous) {
     console.log(`${previous} -> ${next}`);
   },
 });
 container.setValue(1); // logs "0 -> 1"
 ```
 */
export function createObservableCore<T, TSetResult,>(
  {
    initialValue,
    notify,
  }: {
    readonly initialValue: T;
    readonly notify: (
      newValue: T,
      previous: T,
    ) => TSetResult;
  },
): ObservableContainer<T, TSetResult> {
  /**
   Internal store backing the methods; held on an object property so mutation avoids a function-root `let`.
   */
  const state: { current: T; } = { current: initialValue, };
  return {
    getValue: function getValue(): T {
      return state.current;
    },
    setValue: function setValue(newValue: T,): TSetResult {
      /**
       Value being replaced, reported to the notification so it reaches the handler as the previous value.
       */
      const previous = state.current;
      state.current = newValue;
      return notify(
        newValue,
        previous,
      );
    },
  };
}
