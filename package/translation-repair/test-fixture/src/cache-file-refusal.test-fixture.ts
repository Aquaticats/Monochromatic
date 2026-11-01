/**
 The refusal the lookup cache and the reference cache raise over a cache path
 that is there and cannot be read, as `String` renders it and as a log line
 repeats it.

 TEST SUPPORT, NOT PACKAGE SOURCE. Both caches read through one reader
 (`cacheFileText` in `lookup-cache.ts`), so both test files pin the same
 words, and two copies of one body fail `duplicate-bodies.unit.test.ts`.

 @module
 */

/**
 The message of the refusal of a cache path holding a directory, the one
 unreadable state a case can build without changing a file's owner, as
 `refusalText` repeats it on a log line: the class declares its message free
 of quoted text.

 @param path - cache path the refusal names

 @returns Whole message

 @example
 ```ts
 expect(logged,).toEqual([`lookup failed: ${directoryRefusalMessage({ path, },)}`,],);
 ```
 */
export function directoryRefusalMessage({ path, }: { readonly path: string; },): string {
  return `cache file is there and could not be read (EISDIR): ${path}`;
}

/**
 What `String` gives for the refusal of a cache path holding a directory.

 @param path - cache path the refusal names

 @returns Class name and whole message

 @example
 ```ts
 expect(String(refusal,),).toBe(directoryRefusalText({ path, },),);
 ```
 */
export function directoryRefusalText({ path, }: { readonly path: string; },): string {
  return `CacheFileUnreadableError: ${directoryRefusalMessage({ path, },)}`;
}
