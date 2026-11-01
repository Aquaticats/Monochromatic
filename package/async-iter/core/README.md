## module-async-iter

Async iterable helpers.

### Helpers

- `mapIterableAsync({ fn, iterable })`:
   maps each item of a sync or async iterable through an async `fn`,
  collecting the results into an array in input order.

### Behavior

`mapIterableAsync` is an eager collect-to-array mapper,
 not a lazy async-iterator transform.
 It does not yield
values one at a time;
 it starts every `fn` call during iteration,
 then joins them with a single `Promise.all`.

- Eager:
   each `fn` call begins immediately,
   before any result is awaited,
   so the mapper calls overlap.
- Unbounded concurrency:
   there is no cap;
   if the iterable has 10000 items,
   10000 mapper calls start at once.
- Order preserving:
   results land in input order,
   regardless of which mapper resolves first.
- Rejection:
   a rejected mapper rejects the returned promise,
   following `Promise.all` semantics.

For bounded concurrency,
 wrap the mapper with a limiter such as `p-limit`;
 this package intentionally does not.

### Usage

```ts
import { mapIterableAsync, } from '@monochromatic-dev/async-iter-core';

const sizes = await mapIterableAsync({
  fn: async (url,) => (await fetch(url,)).headers.get('content-length',),
  iterable: ['/a', '/b', '/c',],
},);
```

Both sync and async iterables are accepted:

```ts
async function* lines() {
  yield 'first';
  yield 'second';
}

const upper = await mapIterableAsync({
  fn: async (line,) => line.toUpperCase(),
  iterable: lines(),
},);
```
