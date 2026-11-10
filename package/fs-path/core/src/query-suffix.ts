/**
 The path spelling the ensure and empty families share.

 A path a caller builds for a bundler may carry a `?query` suffix
 (`dist/x.js?raw`, `dist/style.css?inline`). Nothing in that suffix names a
 filesystem entry, so every function in both families strips it before it
 inspects or touches the disk: one spelling has to mean one file or directory,
 or clearing a path a caller just ensured would truncate a different one.

 @module
 */

/**
 Character that starts a query suffix.
 */
const QUERY_SEPARATOR = '?';

/**
 Removes the query suffix from a path.

 @param path - Path that may carry a `?query` suffix

 @returns the path without everything from its first `?`, or the path itself when it carries none

 @example
 ```ts
 stripQuerySuffix('dist/x.js?raw'); // 'dist/x.js'
 stripQuerySuffix('dist/x.js');     // 'dist/x.js'
 ```
 */
export function stripQuerySuffix(path: string,): string {
  /**
   Position of the first query separator, or `-1` when the path carries none.
   */
  const queryIndex = path.indexOf(QUERY_SEPARATOR,);

  if (queryIndex === (-1))
    return path;

  return path.slice(
    0,
    queryIndex,
  );
}
