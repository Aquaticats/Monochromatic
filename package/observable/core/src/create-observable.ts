/**
 Synchronous observable value container with method-based get and set.
 
 @module
 */

/**
 Observable value container with method-based access.
 
 Calling `setValue` stores the new value and triggers the registered change
 handler synchronously; `getValue` reads the current value.
 */
export type Observable<T,> = {
  /**
   Reads the current observed value.
   */
  getValue: () => T;
  /**
   Stores a new value and triggers the change handler synchronously.
   */
  setValue: (newValue: T,) => void;
};

/**
 Creates an observable value container that calls a synchronous handler when the value is set.
 
 State updates before `onChange` runs, so a handler that reads `getValue()` sees the new value.
 `setValue` returns `void` and propagates a thrown error from `onChange` to the caller synchronously.
 
 @param initialValue - Starting value, returned by `getValue` until the first `setValue`
 
 @param onChange - Synchronous callback invoked after each value change with new then previous value
 
 @returns {@link Observable} container whose `setValue` triggers onChange synchronously
 
 @throws Whatever `onChange` throws; `setValue` does not catch, so the error reaches the caller
 
 @example
 ```ts
 const counter = createObservable({
   initialValue: 0,
   onChange: function onChange(next, prev) {
     console.log(`Changed from ${prev} to ${next}`);
   },
 });
 counter.setValue(1); // logs "Changed from 0 to 1"
 counter.getValue(); // 1
 ```
 */
export function createObservable<T,>(
  {
    initialValue,
    onChange,
  }: {
    readonly initialValue: T;
    readonly onChange: (
      newValue: T,
      oldValue: T,
    ) => void;
  },
): Observable<T> {
  /**
   Internal store backing the methods; held on an object property so mutation avoids a function-root `let`.
   */
  const state: { current: T; } = { current: initialValue, };
  return {
    getValue: function getValue(): T {
      return state.current;
    },
    setValue: function setValue(newValue: T,): void {
      /**
       Snapshot of the prior value preserved for the change handler call.
       */
      const old = state.current;
      state.current = newValue;
      onChange(
        newValue,
        old,
      );
    },
  };
}
