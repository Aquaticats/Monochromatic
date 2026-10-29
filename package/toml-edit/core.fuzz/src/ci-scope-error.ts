/**
 Bootstrap scope failures must stop verification rather than authorize skipping it.

 @example
 ```ts
 throw new ScopeError('Merge-group comparison is unavailable');
 ```
 */
export class ScopeError extends Error {
  /**
   Stable diagnostic name for workflow logs and subprocess regression tests.
   */
  override readonly name = 'ScopeError';
}
