/**
 Maps every item of a sync or async iterable through an async `fn`, eagerly and
 with unbounded concurrency, collecting results into an array in input order.
 
 Eager, not lazy: every `fn` call is started during iteration, before any result
 is awaited, so all mappers run at the same time. This is a collect-to-array
 mapper, not a lazy async-iterator transform that yields one value at a time.
 
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
  for await (const item of iterable)
    promises.push(fn(item,),);
  return Promise.all(promises,);
}
