/**
 Mutable single-value store shared by the sync and async observable variants.
 
 Both variants hold one value and read it through `getValue`; they differ only in what they do
 with the change handler, so the container and the order of its update live here once.
 
 @module
 */

/**
 Mutable container for one value that hands the replaced value back to its writer.
 */
export type ValueStore<T,> = {
  /**
   Reads the value the store holds now.
   */
  readonly getValue: () => T;
  /**
   Stores `newValue` and returns the value it replaced.
 
   The store holds `newValue` by the time this returns, so a change handler that receives the
   returned value as its previous value also reads the new one through `getValue`.
   */
  readonly replaceValue: (newValue: T,) => T;
};

/**
 Creates a store holding `initialValue`.
 
 @param initialValue - Value `getValue` reports until the first `replaceValue`
 
 @returns {@link ValueStore} whose `replaceValue` reports the value it replaced
 
 @example
 ```ts
 const store = createValueStore(0);
 store.replaceValue(1); // 0, the value it replaced
 store.getValue(); // 1
 ```
 */
export function createValueStore<T,>(initialValue: T,): ValueStore<T> {
  /**
   Internal store backing the methods; held on an object property so mutation avoids a function-root `let`.
   */
  const state: { current: T; } = { current: initialValue, };
  return {
    getValue: function getValue(): T {
      return state.current;
    },
    replaceValue: function replaceValue(newValue: T,): T {
      /**
       Value being replaced, returned so the writer can report it as the previous value.
       */
      const previous = state.current;
      state.current = newValue;
      return previous;
    },
  };
}
