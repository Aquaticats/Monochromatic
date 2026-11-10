/**
 Key rejection errors raised by the store pair.

 @module
 */

/**
 Error raised when a store operation is given an empty key.

 An empty key is not a usable storage key. The two store variants used to
 disagree about one: the async `set` derived a content-hash key that no read
 could address, while the sync `set` stored the literal empty key, so an
 empty-key write in either variant landed somewhere the caller could not name.
 Both variants now refuse an empty key at the method boundary, for `set`,
 `get`, and `delete` alike, and name the call that received it.

 @example
 ```ts
 throw new EmptyKeyError('set');
 ```
 */
export class EmptyKeyError extends Error {
  /**
   Creates an empty-key error.

   @param operation - store method that received the empty key, named in the message

   @example
   ```ts
   new EmptyKeyError('get');
   ```
   */
  public constructor(operation: string,) {
    super(
      `store.${operation} rejects an empty key: a storage key needs at least one character`,
    );
    this.name = 'EmptyKeyError';
  }
}
