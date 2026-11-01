/**
 Wrapper options for the make-asynchronous fork.
 
 Mirrors upstream `make-asynchronous` 2.1.0's `Options` with the same
 `baseUrl` meaning: the base URL resolving bare dynamic imports inside
 Node.js workers. Call shape follows repository lint (one destructured
 object parameter at every public boundary).
 
 Derived from [`make-asynchronous`](https://github.com/sindresorhus/make-asynchronous)
 by Sindre Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README
 for the full attribution.
 
 @module
 */

//region Types

/**
 Options accepted by both wrapper factories.
 
 @example
 ```ts
 const options: MakeAsynchronousOptions = {
   baseUrl: import.meta.url,
 };
 ```
 */
export type MakeAsynchronousOptions = {
  /**
   Base URL resolving bare dynamic imports in Node.js workers. Pass
   `import.meta.url` when the wrapped function dynamically imports
   dependencies from the module that creates the wrapper.
   */
  readonly baseUrl?: string | URL;
};

//endregion Types
