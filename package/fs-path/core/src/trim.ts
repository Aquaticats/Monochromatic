/**
 Removes a trailing `/` from a path, unless the path is root (`/`).
 
 @param path - POSIX path to trim
 
 @returns Path without trailing slash
 
 @example
 ```ts
 trimTrailingSlash('/foo/bar/'); // '/foo/bar'
 trimTrailingSlash('/');          // '/'
 trimTrailingSlash('foo');        // 'foo'
 ```
 */
export function trimTrailingSlash(path: string,): string {
  return (path !== '/') && path
    .endsWith('/',)
    ? path.slice(
      0,
      -1,
    )
    : path;
}

/**
 Removes a leading `/` from a path, unless the path is root (`/`).
 
 @param path - POSIX path to trim
 
 @returns Path without leading slash
 
 @example
 ```ts
 trimLeadingSlash('/foo/bar'); // 'foo/bar'
 trimLeadingSlash('/');         // '/'
 trimLeadingSlash('foo');       // 'foo'
 ```
 */
export function trimLeadingSlash(path: string,): string {
  return (path !== '/') && path
    .startsWith('/',) ? path.slice(1,) : path;
}
