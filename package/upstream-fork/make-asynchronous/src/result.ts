/**
 Reply settlement for the make-asynchronous fork: restoring worker-reported
 results and errors on the calling side.
 
 The worker reports a result as `{output}` and a failure as `{error}`, so
 the key tells them apart. A function can throw anything, including
 `undefined`, so the value itself cannot be used for that.
 
 Derived from [`make-asynchronous`](https://github.com/sindresorhus/make-asynchronous)
 by Sindre Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README
 for the full attribution. Behavior matches `make-asynchronous` 2.1.0.
 
 @module
 */

import type { WorkerReply, } from './worker-lifecycle.ts';

//region Settlement

/**
 Restores one worker reply into its result value.
 
 Cloning keeps the name of a built-in error and drops a custom one, so the
 traveling `errorName` is defined back onto the error. Defining (rather
 than assigning) is needed because an error can have a getter-only name,
 like `DOMException.name`, and the same holds for restored extra
 properties.
 
 @param reply - Raw reply envelope from the worker.
 
 @returns Result value carried by the reply.
 
 @throws Whatever the worker reported as its failure.
 
 @mutates reply - `Object.defineProperty` and `Object.defineProperties`
 install the traveling name and extra properties on the cloned error.
 
 @example
 ```ts
 getResult({ id: 1, output: 42, }); // => 42
 ```
 */
export function getResult(reply: WorkerReply,): unknown {
  if (!Object.hasOwn(
    reply,
    'error',
  ))
    return reply.output;

  /**
   Failure value carried by the reply.
   */
  const { error, } = reply;
  /**
   Traveling error name carried beside the cloned error.
   */
  const { errorName, } = reply;
  /**
   Whether the cloned error already reports the traveling name, in which
   case no own property is defined. Non-error failures skip restoration
   entirely: only `Error` instances carry a name slot.
   */
  const nameMatches = Error.isError(error,)
    ? error.name === errorName
    : true;
  if ((errorName !== undefined) && (!nameMatches))
    Object.defineProperty(
      error,
      'name',
      {
        configurable: true,
        value: errorName,
        writable: true,
      },
    );

  /**
   Extra own error properties carried beside the cloned error.
   */
  const { errorProperties, } = reply;
  if (errorProperties !== undefined)
    // Defining is needed because an error can have a getter-only
    // property.
    Object.defineProperties(
      error,
      Object.getOwnPropertyDescriptors(errorProperties,),
    );

  throw error;
}

//endregion Settlement
