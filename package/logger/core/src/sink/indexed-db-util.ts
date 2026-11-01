/**
 Promise bridges for the event-based IndexedDB API.
 
 IndexedDB predates promises and signals completion only through `success`
 and `error` events, so awaiting it requires wrapping each request and
 transaction in a promise once, here, instead of scattering listener wiring
 through the sink.
 
 @module
 */

/**
 Resolves with an IndexedDB request's result once it succeeds, rejecting
 with the request's error when it fails.
 
 @param request - Pending IndexedDB request.
 
 @returns Result the request produces.
 
 @throws DOMException - Whatever the backend attaches to a failed request.
 
 @mutates request - `request.addEventListener` registers success and error
 listeners on the host-owned request, and `resolve` hands `request.result`,
 a host-owned value, out to the awaiting caller.
 
 @example
 ```ts
 const key = await awaitRequest(store.add(batch));
 ```
 */
export function awaitRequest<T,>(request: IDBRequest<T>,): Promise<T> {
  // oxlint-disable-next-line promise/avoid-new -- IndexedDB is event-based and exposes no promise API; a constructed promise is the only bridge to await it.
  return new Promise(function bridgeRequest(
    resolve,
    reject,
  ) {
    request.addEventListener(
      'success',
      function resolveResult(): void {
        resolve(request.result,);
      },
    );
    request.addEventListener(
      'error',
      function rejectError(): void {
        reject(request.error ?? new Error('IndexedDB request failed without an error object',),);
      },
    );
  },);
}

/**
 Resolves once an IndexedDB transaction commits, rejecting when it errors or
 aborts, so a caller can await durability of everything queued on it.
 
 @param transaction - Transaction whose settlement is awaited.
 
 @throws DOMException - Whatever the backend attaches to a failed or aborted
 transaction.
 
 @mutates transaction - `transaction.addEventListener` registers complete,
 error, and abort listeners on the host-owned transaction.
 
 @example
 ```ts
 const transaction = database.transaction('batch', 'readwrite');
 transaction.objectStore('batch').add(batch);
 await awaitTransaction(transaction);
 ```
 */
export function awaitTransaction(transaction: IDBTransaction,): Promise<void> {
  // oxlint-disable-next-line promise/avoid-new -- IndexedDB is event-based and exposes no promise API; a constructed promise is the only bridge to await it.
  return new Promise(function bridgeTransaction(
    resolve,
    reject,
  ) {
    transaction.addEventListener(
      'complete',
      function resolveCommit(): void {
        resolve();
      },
    );
    transaction.addEventListener(
      'error',
      function rejectError(): void {
        reject(transaction.error ?? new Error('IndexedDB transaction failed without an error object',),);
      },
    );
    transaction.addEventListener(
      'abort',
      function rejectAbort(): void {
        reject(transaction.error ?? new Error('IndexedDB transaction aborted without an error object',),);
      },
    );
  },);
}
