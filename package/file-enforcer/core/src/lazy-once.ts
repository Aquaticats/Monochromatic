/**
 Process-lifetime single-value cache computed on first access.
 */
type LazyOnce<T> = Readonly<{
  /**
   Returns the cached value, computing and storing it on the first call.
   */
  get: () => T;

  /**
   Drops the cached value so the next `get` recomputes.
   */
  reset: () => void;
}>;

/**
 Async process-lifetime single-value cache computed on first access.
 */
type LazyOnceAsync<T> = Readonly<{
  /**
   Returns the cached value, awaiting and storing it on the first call.
   */
  get: () => Promise<T>;

  /**
   Drops the cached value so the next `get` recomputes.
   */
  reset: () => void;
}>;

/**
 Memoizes a zero-argument computation for the process lifetime behind
 `get()`/`reset()`. Concentrates the single-key `Map` cell idiom (the
 container `no-module-root-let` sanctions) so call sites stay free of cache
 plumbing. The value is boxed so a stored `undefined` stays distinct from an
 empty cache.
 
 @param compute - Producer invoked once until the next `reset`.
 
 @returns Cache exposing `get()` and `reset()`.
 
 @example
 ```ts
 const rootDetection = lazyOnce({ compute: function detect() { return process.getuid?.() === 0; } });
 const root = rootDetection.get();
 rootDetection.reset();
 ```
 */
export function lazyOnce<const T,>(
  { compute, }: { readonly compute: () => T; },
): LazyOnce<T> {
  /**
   Single-slot holder boxing the computed value.
   */
  const slot = new Map<'value', { readonly value: T; }>();
  return {
    get(): T {
      /**
       Boxed cached value, or undefined when the slot is empty.
       */
      const boxed = slot.get('value',);
      if (boxed !== undefined)
        return boxed.value;

      /**
       Freshly computed value stored before return so later calls short-circuit.
       */
      const value = compute();
      slot.set(
        'value',
        { value, },
      );
      return value;
    },
    reset(): void {
      slot.delete('value',);
    },
  };
}

/**
 Async counterpart of {@link lazyOnce}. Does not deduplicate overlapping
 in-flight calls, so two concurrent first calls both run `compute`, matching
 the hand-rolled detection caches it replaces.
 
 @param compute - Async producer invoked once until the next `reset`.
 
 @returns Cache exposing async `get()` and `reset()`.
 
 @example
 ```ts
 const managerDetection = lazyOnceAsync({ compute: async function detect() { return await probe(); } });
 const manager = await managerDetection.get();
 managerDetection.reset();
 ```
 */
export function lazyOnceAsync<const T,>(
  { compute, }: { readonly compute: () => Promise<T>; },
): LazyOnceAsync<T> {
  /**
   Single-slot holder boxing the computed value.
   */
  const slot = new Map<'value', { readonly value: T; }>();
  return {
    async get(): Promise<T> {
      /**
       Boxed cached value, or undefined when the slot is empty.
       */
      const boxed = slot.get('value',);
      if (boxed !== undefined)
        return boxed.value;

      /**
       Freshly awaited value stored before return so later calls short-circuit.
       */
      const value = await compute();
      slot.set(
        'value',
        { value, },
      );
      return value;
    },
    reset(): void {
      slot.delete('value',);
    },
  };
}
