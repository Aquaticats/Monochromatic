/**
 TypeScript fork of [`p-map`](https://github.com/sindresorhus/p-map) by
 Sindre Sorhus (MIT): map over iterables with limited concurrency.
 
 Rewritten to this repository's TypeScript standards with the upstream
 `p-map` 7.1.1 iteration semantics preserved: up to `concurrency` mapper
 calls run at once,
 results collect in input order,
 `pMapSkip` drops a result,
 `pMapStop` ends the iteration,
 `pMapSettled`, `pMapKeyed`, and `pMapSettledKeyed` cover `Promise.allSettled()` and the keyed forms,
 `stopOnError` picks first-failure versus `AggregateError`,
 the abort
 `signal` rejects with its reason,
 and `pMapIterable` streams results under a
 `backpressure` bound. The only API-shape deviation is lint-mandated: one
 destructured object replaces upstream's three positional parameters.
 
 @example
 ```ts
 import { pMap, } from '\@monochromatic-dev/upstream-fork-p-map';
 
 const results = await pMap({
   iterable: [
     'a',
     'b',
   ],
   mapper: async function fetchUser(id: string): Promise<string> {
     return `user-${id}`;
   },
   options: { concurrency: 2, },
 });
 ```
 
 @packageDocumentation
 */

export {
  InvalidBackpressureError,
  InvalidConcurrencyError,
  InvalidInputError,
  InvalidKeyedInputError,
  MapperRequiredError,
} from './errors.ts';

export {
  type MapControl,
  type MapInput,
  type Mapper,
} from './mapper.ts';

export { type SourceIterator, } from './map-iterator.ts';

export {
  type IterableMapOptions,
  type MapOptions,
  type SettledMapOptions,
} from './map-options.ts';

export {
  type KeyedInput,
  type KeyedKey,
  type KeyedMapper,
  type KeyedMapperFunction,
  type KeyedMapRun,
  type KeyedNewValue,
  type KeyedResult,
  type KeyedValue,
  pMapKeyed,
  pMapSettledKeyed,
  type SettledKeyedMapRun,
} from './p-map-keyed.ts';

export {
  type MapRun,
  pMap,
  pMapSettled,
  type SettledMapRun,
} from './p-map.ts';

export {
  type IterableMapRun,
  pMapIterable,
} from './p-map-iterable.ts';

export { pMapSkip, } from './p-map-skip.ts';

export { pMapStop, } from './p-map-stop.ts';
