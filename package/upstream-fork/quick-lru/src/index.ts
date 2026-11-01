/**
 TypeScript fork of [`quick-lru`](https://github.com/sindresorhus/quick-lru) by
 Sindre Sorhus (MIT): a dual-cache Least Recently Used map with lazy `maxAge`
 expiry, eviction notifications, `resize`, and `evict`.
 
 Rewritten to this repository's TypeScript standards with upstream `quick-lru`
 7.3.0 cache semantics preserved: items roll between a recent map and an old
 map so writes never pay per-item deletes, `size` reports at most `maxSize`
 even though the cache may hold `2 × maxSize`, and expiry is applied lazily
 on read. The only API-shape deviations are lint-mandated: the cache is a
 factory-built object instead of a `Map` subclass instance, `set` takes a
 destructured options object instead of three positional parameters, and
 `forEach` takes one destructured options object.
 
 @example
 ```ts
 import { createQuickLru, } from '\@monochromatic-dev/upstream-fork-quick-lru';
 
 const lru = createQuickLru<string, string>({ maxSize: 1000, });
 lru.set({
   key: '🦄',
   value: '🌈',
 },);
 lru.get('🦄',); // => '🌈'
 ```
 
 @packageDocumentation
 */

export {
  InvalidMaxAgeError,
  InvalidMaxSizeError,
} from './errors.ts';

export {
  /**
   @internal
   */
  hasExpiryStamp,
  type QuickLruItem,
} from './quick-lru-item.ts';

export {
  type QuickLruOptions,
  type ResolvedQuickLruOptions,
} from './quick-lru-options.ts';

export type {
  QuickLru,
  QuickLruForEachOptions,
  QuickLruSetOptions,
  QuickLruVisitor,
} from './quick-lru-types.ts';

export {
  createQuickLru,
} from './quick-lru.ts';
