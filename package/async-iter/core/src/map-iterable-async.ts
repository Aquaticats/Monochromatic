/**
 Marks a started mapper's rejection handled from the moment it exists, and drops
 the value.

 `Promise.all` can only attach its handlers once source iteration has finished,
 so a rejection delivered before then counts as unhandled, and Node's default
 `--unhandled-rejections=throw` ends the process over it. This observes the same
 promise immediately instead; the aggregate join over that same promise still
 reports the failure to the caller, and a source that throws mid-iteration
 leaves no rejection behind.

 @param promise - started mapper promise the aggregate join also observes
 */
async function markRejectionHandled(promise: Promise<unknown>,): Promise<void> {
  try {
    await promise;
  }
  catch (error) {
    // The aggregate `Promise.all` over this same promise reports this rejection;
    // this handler only keeps the runtime from treating it as unhandled while
    // iteration continues.
    void error;
  }
}

/**
 Maps every item of a sync or async iterable through an async `fn`, eagerly and
 with unbounded concurrency, collecting results into an array in input order.
 
 Eager, not lazy: every `fn` call is started during iteration, before any result
 is awaited, so all mappers run at the same time. This is a collect-to-array
 mapper, not a lazy async-iterator transform that yields one value at a time.
 
 A mapper rejection reaches the caller as this function's rejection, and never as
 an unhandled rejection, however long the source takes to finish yielding.
 
 @param fn - async mapper invoked once per item; every call starts eagerly so the calls overlap
 
 @param iterable - sync or async source whose items feed `fn`
 
 @returns mapped results in input order, one per source item
 
 @example
 ```ts
 import { mapIterableAsync, } from '\@monochromatic-dev/async-iter-core';
 
 const sizes = await mapIterableAsync({
   fn: async (url,) => (await fetch(url,)).headers.get('content-length',),
   iterable: ['/a', '/b', '/c',],
 },);
 ```
 */
export async function mapIterableAsync<T, R,>({
  fn,
  iterable,
}: {
  readonly fn: (item: T,) => Promise<R>;
  readonly iterable: AsyncIterable<T> | Iterable<T>;
},): Promise<R[]> {
  // Each `fn` call starts executing immediately (promises are eager, not lazy).
  // `Promise.all` only collects already-running results; it does not "activate" them.
  // `Array.fromAsync` is not suitable here: it awaits each mapped value sequentially,
  // which would serialize the work and lose concurrency.
  /**
   Eager mapping promises gathered in input order for a single Promise.all join.
   */
  const promises: Promise<R>[] = [];
  for await (const item of iterable) {
    /**
     Started mapper whose result the join below collects in input order.
     */
    const started = fn(item,);
    promises.push(started,);
    void markRejectionHandled(started,);
  }
  return Promise.all(promises,);
}
