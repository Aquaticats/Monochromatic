/**
 Test-only scheduling helpers shared by this package's unit tests.
 
 Gates make concurrency tests deterministic: each scheduled call blocks on
 its own gate, and the test releases gates one at a time in a chosen order,
 so start and settle sequences are decided by the test rather than by
 scheduler timing.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

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

//region Scripted sources

/**
 Answer a scripted pull settles with.

 @typeParam Value - element type the source yields
 */
export type PullAnswer<Value> = IteratorResult<Value | Promise<Value>>;

/**
 One scripted pull: its answer is decided by the test.

 @typeParam Value - element type the source yields
 */
export type ScriptedPull<Value> = PromiseWithResolvers<PullAnswer<Value>>;

/**
 Source whose `next()` calls each wait for the test to settle them.

 @typeParam Value - element type the source yields
 */
export type ScriptedSource<Value> = {
  /**
   Async iterable handing out one controllable pull per `next()` call.
   */
  readonly iterable: AsyncIterable<Value | Promise<Value>>;
  /**
   Pulls made so far, in call order.
   */
  readonly pulls: ScriptedPull<Value>[];
};

/**
 Builds a source whose pulls stay pending until the test settles them, so the
 test decides the order in which the pulls answer.

 @typeParam Value - element type the source yields

 @returns Source and the list of its pulls so far.

 @example
 ```ts
 const source = scriptedAsyncSource<string>();
 ```
 */
export function scriptedAsyncSource<Value>(): ScriptedSource<Value> {
  /**
   Pulls handed out so far.
   */
  const pulls: ScriptedPull<Value>[] = [];
  return {
    pulls,
    iterable: {
      [Symbol.asyncIterator]: function openScripted(): AsyncIterator<Value | Promise<Value>> {
        return {
          next: function scriptedNext(): Promise<PullAnswer<Value>> {
            /**
             Deferred answer of this pull.
             */
            const pull = Promise.withResolvers<PullAnswer<Value>>();
            pulls.push(pull,);
            return pull.promise;
          },
        };
      },
    },
  };
}

/**
 Reads one scripted pull, failing the test when the run has not made it.

 @typeParam Value - element type the source yields

 @param source - Scripted source to read.

 @param index - Position of the pull in call order.

 @returns The pull's deferred answer.

 @throws Error when the source has fewer pulls.

 @example
 ```ts
 const pull = pullAt({ source, index: 0, },);
 ```
 */
export function pullAt<Value>(
  {
    source,
    index,
  }: {
    /**
     Scripted source to read.
     */
    readonly source: ScriptedSource<Value>;
    /**
     Position of the pull in call order.
     */
    readonly index: number;
  },
): ScriptedPull<Value> {
  return nonNullishOrThrow(source.pulls[index],);
}

/**
 Answers one scripted pull with an element.

 @typeParam Value - element type the source yields

 @param pull - Pull to answer.

 @param value - Element, or promise of one, the pull yields.

 @example
 ```ts
 yieldElement({ pull: pullAt({ source, index: 0, },), value: 'a', },);
 ```
 */
export function yieldElement<Value>(
  {
    pull,
    value,
  }: {
    /**
     Pull to answer.
     */
    readonly pull: ScriptedPull<Value>;
    /**
     Element, or promise of one, the pull yields.
     */
    readonly value: Value | Promise<Value>;
  },
): void {
  pull.resolve({
    done: false,
    value,
  },);
}

/**
 Answers one scripted pull with the end of the source.

 @typeParam Value - element type the source yields

 @param pull - Pull to answer.

 @example
 ```ts
 yieldDone(pullAt({ source, index: 1, },),);
 ```
 */
export function yieldDone<Value>(pull: ScriptedPull<Value>,): void {
  pull.resolve({
    done: true,
    value: undefined,
  },);
}

//endregion Scripted sources
