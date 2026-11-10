/**
 Narrows a caught value to a Node error carrying an errno code.
 
 @module
 */

import { isError, } from '@sindresorhus/is';

/**
 Returns whether a thrown value is an error whose `code` names an errno
 condition.
 
 `@sindresorhus/is` has no predicate for this shape, and the hand-written
 copies it replaces disagreed: three tested `Error.isError` alone, one tested a
 non-null object with a `code`, so the same thrown value was read as a missing
 file in one module and rethrown in another. The body taken over from
 `package/wg/cli-quicker` is the strict one, because `NodeJS.ErrnoException`
 extends `Error`: Node's filesystem and process functions throw real errors,
 and a bare object shaped like an errno came from some other thrower, so
 reading its `code` as a missing path would swallow that thrower's failure.
 
 @param error - Caught value, usually a `catch` binding.
 
 @returns Whether the value is an error carrying an errno code.
 
 @example
 ```ts
 const error: NodeJS.ErrnoException = new Error('missing');
 error.code = 'ENOENT';
 isErrnoException(error); // true
 isErrnoException({ code: 'ENOENT' }); // false
 ```
 */
export function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return isError(error,)
    && ('code' in error);
}
