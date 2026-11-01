/**
 LRU cache configuration parsing and validation.
 
 Accepts upstream `quick-lru`'s constructor options and normalizes them into
 one resolved record, so the cache itself never branches on its input shape.
 Validation keeps upstream `quick-lru`'s acceptance rules, coercion quirks,
 and error message texts exactly: `maxSize` must be truthy and greater than
 `0`, and the global `maxAge` is rejected only when it is exactly the number
 `0`.
 
 @module
 */

import {
  InvalidMaxAgeError,
  InvalidMaxSizeError,
} from './errors.ts';

//region Types

/**
 LRU cache configuration accepted by `createQuickLru`.
 
 Mirrors upstream `quick-lru`'s `Options` with the same defaults and the
 same validation error messages.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 
 @example
 ```ts
 const options: QuickLruOptions<string, number> = {
   maxSize: 100,
   maxAge: 60_000,
   onEviction: function logEvicted(key: string, value: number): void {
     console.log(key, value);
   },
 };
 ```
 */
export type QuickLruOptions<Key, Value> = {
  /**
   Target maximum number of items before evicting the least recently used
   ones. Must be truthy and greater than `0`.
   */
  readonly maxSize: number;
  /**
   Milliseconds an item may remain in the cache before lazy expiry removes
   it. Must not be the number `0`; `Number.POSITIVE_INFINITY` means items
   never expire.
   
   @defaultValue Number.POSITIVE_INFINITY
   */
  readonly maxAge?: number;
  /**
   Called right before an item is evicted by LRU pressure, TTL expiry, or a
   manual `evict` call. Never called for `delete` or `clear`.
   */
  readonly onEviction?: (
    key: Key,
    value: Value
  ) => void;
};

/**
 Validated cache configuration with every lifetime bound present.
 
 `onEviction` is deliberately absent: upstream stores the callback
 unchanged and only checks `typeof === 'function'` at notification time, so
 the cache carries it straight from its input instead of through this
 record.
 
 @example
 ```ts
 const resolved: ResolvedQuickLruOptions = {
   maxSize: 100,
   maxAge: Number.POSITIVE_INFINITY,
 };
 ```
 */
export type ResolvedQuickLruOptions = {
  /**
   Validated `maxSize`, kept in its original runtime form exactly as
   upstream stores it.
   */
  readonly maxSize: number;
  /**
   Validated global lifetime bound, `Number.POSITIVE_INFINITY` when the
   input carries no usable `maxAge`.
   */
  readonly maxAge: number;
};

//endregion Types

//region Validation

/**
 Validates one `maxSize` value, keeping upstream `quick-lru`'s acceptance
 rule (truthy and greater than `0`) and error message.
 
 The input type matches upstream's declared `maxSize: number` while the
 acceptance rule stays as permissive as upstream's runtime, so values like
 `'3'` pass through unchanged exactly as upstream stores them.
 
 @param maxSize - Candidate maximum size of unknown runtime type.
 
 @returns The same value, proven to be an accepted maximum size.
 
 @throws InvalidMaxSizeError when the value is not truthy and greater than
 `0`.
 
 @example
 ```ts
 validateMaxSize(2); // => 2
 ```
 */
export function validateMaxSize(maxSize?: number,): number {
  if ((maxSize === undefined) || (!(maxSize > 0)))
    throw new InvalidMaxSizeError();
  return maxSize;
}

/**
 Resolves the global `maxAge`, keeping upstream `quick-lru`'s acceptance
 rule and coercion: only the number `0` is rejected, any other value is
 stored as given or replaced by `Number.POSITIVE_INFINITY` when falsy.
 
 @param maxAge - Candidate global lifetime of unknown runtime type.
 
 @returns The same value when truthy, otherwise `Number.POSITIVE_INFINITY`.
 
 @throws InvalidMaxAgeError when the value is exactly the number `0`.
 
 @example
 ```ts
 resolveMaxAge(undefined,); // => Number.POSITIVE_INFINITY
 ```
 */
export function resolveMaxAge(maxAge?: number,): number {
  // mutation-test-disable-next-line conditional -- proven equivalent: strict `=== 0` already matches only the number 0 and -0, so the typeof gate cannot change the outcome; kept because it is upstream's exact acceptance shape
  if (((typeof maxAge) === 'number') && (maxAge === 0))
    throw new InvalidMaxAgeError();
  // oxlint-disable-next-line typescript/prefer-nullish-coalescing, typescript/strict-boolean-expressions -- mirrors upstream `options.maxAge || Number.POSITIVE_INFINITY` exactly: every falsy value (NaN and falsy non-number junk included) rewrites to Infinity, while `??` and explicit falsy enumeration both diverge on the junk inputs the differential oracle feeds
  return maxAge || Number.POSITIVE_INFINITY;
}

//endregion Validation

//region Resolution

/**
 Normalizes `createQuickLru`'s input into one validated record.
 
 The input parameter defaults to an empty object and reads its fields by
 property access, mirroring upstream `quick-lru`'s `constructor(options =
 {})` shape: a missing argument reaches the `maxSize` validation and fails
 with the same configuration error, and a `null` argument fails on the same
 property read with the same plain `TypeError`.
 
 Validation order matches upstream: `maxSize` is checked before `maxAge`.
 
 @param options - Cache options as upstream accepts them at runtime.
 
 @returns Validated bounds with every lifetime field present.
 
 @throws InvalidMaxSizeError when `maxSize` is not truthy and greater than
 `0`.
 
 @throws InvalidMaxAgeError when `maxAge` is exactly the number `0`.
 
 @throws TypeError when `options` is `null`, from the same property read
 upstream performs, matching upstream `quick-lru`.
 
 @example
 ```ts
 resolveQuickLruOptions({ maxSize: 10, },);
 // => { maxSize: 10, maxAge: Number.POSITIVE_INFINITY }
 ```
 */
export function resolveQuickLruOptions(
  options: {
    /**
     Candidate target maximum number of items.
     */
    readonly maxSize?: number;
    /**
     Candidate global lifetime in milliseconds.
     */
    readonly maxAge?: number;
  } = {},
): ResolvedQuickLruOptions {
  /**
   Validated maximum size, checked before `maxAge` to preserve upstream
   `quick-lru`'s error precedence.
   */
  const maxSize = validateMaxSize(options.maxSize,);
  return {
    maxSize,
    maxAge: resolveMaxAge(options.maxAge,),
  };
}

//endregion Resolution
