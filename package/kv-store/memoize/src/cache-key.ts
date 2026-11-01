/**
 Build full cache key from keyFn output and resolved salt.
 
 Internal helper shared by sync and async memoization; not part of the package public API.
 
 @param argKey - key derived from function arguments via keyFn
 
 @param salt - salt value appended for cache invalidation
 
 @returns composite cache key in `${argKey}:${salt}` form
 
 @example
 ```ts
 buildCacheKey({ argKey: 'arg-key', salt: 'v1' }); // 'arg-key:v1'
 ```
 */
export function buildCacheKey({
  argKey,
  salt,
}: Readonly<{
  argKey: string;
  salt: string;
}>,): string {
  return `${argKey}:${salt}`;
}
