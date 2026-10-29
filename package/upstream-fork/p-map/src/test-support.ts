/**
 Test-only scheduling helpers shared by this package's unit tests.
 
 Gates make concurrency tests deterministic: each scheduled call blocks on
 its own gate, and the test releases gates one at a time in a chosen order,
 so start and settle sequences are decided by the test rather than by
 scheduler timing.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

//region Gates

/**
 One controllable completion gate: a blocked call awaits `open`, the test
 finishes it with `release`.
 
 @example
 ```ts
 const gate = createGate();
 await gate.open;
 ```
 */
export type Gate = {
  /**
   Promise settled by {@link Gate.release}.
   */
  readonly open: Promise<void>;
  /**
   Settles {@link Gate.open}, letting the blocked call continue.
   */
  readonly release: () => void;
};

/**
 Creates one openable gate.
 
 @returns Gate whose promise is pending until released.
 
 @example
 ```ts
 const gate = createGate();
 gate.release();
 await gate.open;
 ```
 */
export function createGate(): Gate {
  /**
   Deferred pair backing the gate's promise and its release.
   */
  const deferred = Promise.withResolvers<void>();
  return {
    open: deferred.promise,
    release: function release(): void {
      deferred.resolve();
    },
  };
}

//endregion Gates

//region Turns

/**
 Yields one macrotask turn.
 
 A `setTimeout` turn runs after every queued microtask, so awaiting this
 guarantees the run's detached mapper continuations and pulls have all run.
 
 @example
 ```ts
 await yieldTurn();
 startedIds; // reflects every mapper call the run started
 ```
 */
export async function yieldTurn(): Promise<void> {
  await wait(
    0,
  );
}

//endregion Turns

//region Observation

/**
 Awaits a promise that must reject and hands back its reason.

 @param promise - Promise expected to reject.

 @returns The rejection reason.

 @throws Error when the promise resolves instead.

 @example
 ```ts
 const reason = await rejectionOf(Promise.reject(new Error('boom',),),);
 ```
 */
export async function rejectionOf(promise: Promise<unknown>,): Promise<unknown> {
  try {
    await promise;
  }
  catch (error) {
    return error;
  }

  throw new Error('expected the promise to reject, but it resolved',);
}

/**
 Collects every value of an async iterable.

 @typeParam Value - value type the iterable yields

 @param iterable - Async iterable to drain.

 @returns The yielded values in order.

 @example
 ```ts
 const values = await collect(stream,);
 ```
 */
export async function collect<Value>(iterable: AsyncIterable<Value>,): Promise<Value[]> {
  /**
   Values yielded so far.
   */
  const values: Value[] = [];
  for await (const value of iterable)
    values.push(value,);
  return values;
}

//endregion Observation

//region Sources

/**
 Counters of an endless numeric source.
 */
export type SourceCounters = {
  /**
   Number of `next()` calls so far; also the next value.
   */
  pulls: number;
  /**
   Number of `return()` calls so far.
   */
  returnCalls: number;
};

/**
 An endless synchronous source with the counters its iterators write.
 */
export type CountingSyncSource = Iterable<number> & {
  /**
   Counters the source's iterators write.
   */
  readonly counters: SourceCounters;
};

/**
 An endless asynchronous source with the counters its iterators write.
 */
export type CountingAsyncSource = AsyncIterable<number> & {
  /**
   Counters the source's iterators write.
   */
  readonly counters: SourceCounters;
};

/**
 Builds an endless synchronous source of 0, 1, 2, and so on that counts its
 pulls and its closes.

 @returns Source whose counters the test reads after the run.

 @example
 ```ts
 const source = countingSyncSource();
 ```
 */
export function countingSyncSource(): CountingSyncSource {
  /**
   Counters shared by every iterator the source opens.
   */
  const counters: SourceCounters = {
    pulls: 0,
    returnCalls: 0,
  };

  /**
   Takes the next value, counting the pull.

   @returns The next value.
   */
  function nextSync(): IteratorResult<number> {
    /**
     Value of this pull.
     */
    const value = counters.pulls;
    counters.pulls += 1;
    return {
      done: false,
      value,
    };
  }

  /**
   Closes the source, counting the close.

   @returns The done result.
   */
  function returnSync(): IteratorResult<number> {
    counters.returnCalls += 1;
    return {
      done: true,
      value: undefined,
    };
  }

  return {
    counters,
    [Symbol.iterator]: function openSync(): Iterator<number> {
      return {
        next: nextSync,
        return: returnSync,
      };
    },
  };
}

/**
 Builds an endless asynchronous source of 0, 1, 2, and so on that counts its
 pulls and its closes.

 @returns Source whose counters the test reads after the run.

 @example
 ```ts
 const source = countingAsyncSource();
 ```
 */
export function countingAsyncSource(): CountingAsyncSource {
  /**
   Counters shared by every iterator the source opens.
   */
  const counters: SourceCounters = {
    pulls: 0,
    returnCalls: 0,
  };

  /**
   Takes the next value, counting the pull.

   @returns Promise of the next value.
   */
  function nextAsync(): Promise<IteratorResult<number>> {
    /**
     Value of this pull.
     */
    const value = counters.pulls;
    counters.pulls += 1;
    return Promise.resolve({
      done: false,
      value,
    },);
  }

  /**
   Closes the source, counting the close.

   @returns Promise of the done result.
   */
  function returnAsync(): Promise<IteratorResult<number>> {
    counters.returnCalls += 1;
    return Promise.resolve({
      done: true,
      value: undefined,
    },);
  }

  return {
    counters,
    [Symbol.asyncIterator]: function openAsync(): AsyncIterator<number> {
      return {
        next: nextAsync,
        return: returnAsync,
      };
    },
  };
}

//endregion Sources
