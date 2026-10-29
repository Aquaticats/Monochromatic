/**
 Async iterable helpers.
 
 - `mapIterableAsync({ fn, iterable })`: eager, unbounded, order-preserving
   collect-to-array mapper over a sync or async iterable. Every `fn` call
   starts during iteration, so all mappers overlap; results land in input
   order. Not a lazy async-iterator transform.
 
 @example
 ```ts
 import { mapIterableAsync, } from '\@monochromatic-dev/async-iter-core';
 
 const sizes = await mapIterableAsync({
   fn: async (url,) => (await fetch(url,)).headers.get('content-length',),
   iterable: ['/a', '/b', '/c',],
 },);
 ```
 
 @packageDocumentation
 */

export { mapIterableAsync, } from './map-iterable-async.ts';
