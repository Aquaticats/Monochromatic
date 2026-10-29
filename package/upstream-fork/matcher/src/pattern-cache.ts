/**
 Bounded compiled-pattern cache shared by every matcher call.
 
 Derived from [`matcher`](https://github.com/sindresorhus/matcher) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Eviction semantics match upstream `matcher` 6.1.0: at most
 1000 entries, keyed by case-sensitivity flag plus raw pattern text, with the
 oldest inserted entry evicted first.
 
 @module
 */

import {
  type CompiledPattern,
  compilePattern,
} from './pattern.ts';

//region Cache

/**
 Maximum compiled patterns held before the oldest entry is evicted.
 */
const MAXIMUM_CACHE_SIZE = 1_000;

/**
 Case-sensitive cache-key prefix, matching upstream `matcher`'s `S` prefix.
 */
const SENSITIVE_PREFIX = 'S';

/**
 Case-insensitive cache-key prefix, matching upstream `matcher`'s `I` prefix.
 */
const INSENSITIVE_PREFIX = 'I';

/**
 Compiled patterns keyed by case-sensitivity prefix plus raw pattern text.
 A `Map` keeps insertion order, so the first key is the oldest.
 */
const patternCache = new Map<string, CompiledPattern>();

/**
 Builds the cache key for one raw pattern and case-sensitivity flag.
 
 @param pattern - Raw pattern text.
 
 @param caseSensitive - Whether matching folds case.
 
 @returns Cache key combining the sensitivity prefix and the raw pattern.
 
 @example
 ```ts
 cacheKey({ pattern: 'a*', caseSensitive: true, }); // => 'Sa*'
 ```
 */
export function cacheKey(
  {
    pattern,
    caseSensitive,
  }: {
    readonly pattern: string;
    readonly caseSensitive: boolean;
  },
): string {
  return `${caseSensitive
    ? SENSITIVE_PREFIX
    : INSENSITIVE_PREFIX}${pattern}`;
}

/**
 Returns the compiled pattern for one raw pattern text, compiling and
 caching it on a miss.
 
 On a miss the compiled entry is stored, evicting the oldest entry first
 when the cache is full.
 
 @param pattern - Raw pattern text.
 
 @param caseSensitive - Whether matching folds case.
 
 @returns Cached or freshly compiled pattern.
 
 @example
 ```ts
 const compiled = getPattern({ pattern: 'a*', caseSensitive: false, });
 ```
 */
export function getPattern(
  {
    pattern,
    caseSensitive,
  }: {
    readonly pattern: string;
    readonly caseSensitive: boolean;
  },
): CompiledPattern {
  /**
   Cache key for this pattern and sensitivity.
   */
  const key = cacheKey({
    pattern,
    caseSensitive,
  },);
  /**
   Cached compilation, when this exact key compiled before.
   */
  const cached = patternCache.get(key,);
  if (cached !== undefined)
    return cached;

  /**
   Freshly compiled pattern stored below.
   */
  const compiled = compilePattern({
    pattern,
    caseSensitive,
  },);

  if (patternCache.size >= MAXIMUM_CACHE_SIZE) {
    /**
     Oldest inserted key, evicted to bound memory.
     */
    const oldest: unknown = patternCache.keys()
      .next()
      .value;
    if ((typeof oldest) === 'string')
      patternCache.delete(oldest,);
  }

  patternCache.set(
    key,
    compiled,
  );
  return compiled;
}

/**
 Number of compiled patterns currently cached.
 
 Exposed for tests proving the eviction bound without reaching into
 module-private state through a back door.
 
 @returns Current cache entry count.
 
 @example
 ```ts
 cacheSize(); // => 0
 ```
 */
export function cacheSize(): number {
  return patternCache.size;
}

/**
 Empties the compiled-pattern cache.
 
 Exposed for tests needing a deterministic cache starting point.
 
 @example
 ```ts
 clearPatternCache();
 ```
 */
export function clearPatternCache(): void {
  patternCache.clear();
}

//endregion Cache
