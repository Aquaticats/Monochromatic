/**
 TypeScript fork of `p-map`: map over iterables with limited concurrency.
 
 Derived from [`p-map`](https://github.com/sindresorhus/p-map) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Scheduling semantics match `p-map` 7.1.1: up to `concurrency`
 mapper calls run at once over a synchronous or asynchronous input, results
 collect in input order, and `stopOnError`, the abort `signal`, and
 `pMapSkip` behave as upstream defines them.
 
 @module
 */

import {
  closeIterator,
  createPuller,
  selectIterator,
  validateInput,
  type YieldedItem,
} from './map-iterator.ts';
import {
  type MapOptions,
  resolveMapOptions,
  type SettledMapOptions,
  validateConcurrency,
} from './map-options.ts';
import {
  type MapControl,
  type MapInput,
  type Mapper,
  validateMapper,
} from './mapper.ts';
import { pMapSkip, } from './p-map-skip.ts';
import { pMapStop, } from './p-map-stop.ts';

//region Types

/**
 One concurrent-map run: the inputs to walk, the mapper to run per element,
 and the upstream-shaped options.
 
 The single destructured object replaces upstream `p-map`'s
 `pMap(iterable, mapper, options)` positional parameters, which repository
 lint bans; `options` keeps its shape and destructuring failure semantics.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @typeParam NewElement - mapper result type, awaited when thenable
 
 @param iterable - Inputs walked once in iteration order.
 
 @param mapper - Function run per element with its position in the input.
 
 @param options - Concurrency bound, `stopOnError` policy, and optional
 abort signal.
 
 @example
 ```ts
 const run: MapRun<number, number> = {
   iterable: [1, 2, 3,],
   mapper: async function double(value: number,): Promise<number> {
     return value * 2;
   },
   options: { concurrency: 2, },
 };
 ```
 */
export type MapRun<Element, NewElement> = {
  /**
   Inputs walked once in iteration order, each element awaited before its
   mapper call.
   */
  readonly iterable: MapInput<Element>;
  /**
   Function run per element with its position in the input.
   */
  readonly mapper: Mapper<Element, NewElement>;
  /**
   Concurrency bound, `stopOnError` policy, and optional abort signal,
   destructured with upstream `p-map`'s defaults.
   */
  readonly options?: MapOptions;
};

/**
 One settled-map run: the same shape as {@link MapRun}, without
 `stopOnError`, since a mapper failure becomes a rejected result instead of
 rejecting the run.

 @typeParam Element - element type after awaiting an iterated item

 @typeParam NewElement - mapper result type, awaited when thenable

 @example
 ```ts
 const run: SettledMapRun<number, number> = {
   iterable: [1, 2, 3,],
   mapper: async function double(value: number,): Promise<number> {
     return value * 2;
   },
   options: { concurrency: 2, },
 };
 ```
 */
export type SettledMapRun<Element, NewElement> = {
  /**
   Inputs walked once in iteration order, each element awaited before its
   mapper call.
   */
  readonly iterable: MapInput<Element>;
  /**
   Function run per element with its position in the input.
   */
  readonly mapper: Mapper<Element, NewElement>;
  /**
   Concurrency bound and optional abort signal.
   */
  readonly options?: SettledMapOptions;
};

/**
 What the shared engine runs: a run exactly as the caller supplied it, plus
 whether failures become settled results.

 @typeParam Element - element type after awaiting an iterated item

 @typeParam NewElement - mapper result type, awaited when thenable

 @example
 ```ts
 const engineRun: ConcurrentRun<number, number> = {
   run: { iterable: [1,], mapper: identity, },
   isSettled: false,
 };
 ```
 */
export type ConcurrentRun<Element, NewElement> = {
  /**
   The caller's run, destructured inside the engine so a destructuring
   failure rejects the returned promise like upstream's `async` signature.
   */
  readonly run: MapRun<Element, NewElement>;
  /**
   With `true`, a rejected element or mapper gives a
   `{ status: 'rejected', reason }` result instead of an error, like
   `Promise.allSettled()`, so `stopOnError` does not apply.
   */
  readonly isSettled: boolean;
};

//endregion Types

//region Run

/**
 Whether one staged result is a real mapper result rather than the skip
 sentinel.

 @param value - Staged result.

 @returns `false` for the skip sentinel.

 @example
 ```ts
 isNotSkip(1,); // => true
 ```
 */
function isNotSkip(value: unknown,): boolean {
  return value !== pMapSkip;
}

/* oxlint-disable eslint/require-await -- `mapConcurrently` stays `async` so a destructuring failure rejects the caller's promise like upstream `p-map`'s `async` signature, and returning the deferred promise without awaiting it preserves upstream's promise-adoption tick (measured: an `await` here settles the caller one microtask early) */
/**
 Runs one concurrent map over upstream `p-map`'s run structure, the engine
 behind `pMap`, `pMapSettled`, `pMapKeyed`, and `pMapSettledKeyed`: option
 destructuring first, then the input, mapper, and concurrency checks in
 upstream's order, then detached per-element runners pulling the source
 under the concurrency bound.

 The run body sits in one `try` that funnels synchronous failures into the
 returned promise, mirroring upstream's `new Promise(executor)`: a hostile
 input, mapper, or iterator becomes a rejection at the same point in the
 run's lifetime.

 @typeParam Element - element type after awaiting an iterated item

 @typeParam NewElement - mapper result type, awaited when thenable

 @typeParam Result - element type of the resolved array: the mapper results,
 or their settled forms

 @param run - The caller's inputs, mapper, and options, destructured here so
 a failure rejects the returned promise.

 @param isSettled - Whether failures become rejected results.

 @returns Promise settling with every result in input order, or rejecting
 with the first failure (`stopOnError`) or an `AggregateError` of every
 failure; with `isSettled`, only the source, the signal, or the checks reject.

 @throws InvalidInputError when the input is neither an `Iterable` nor an
 `AsyncIterable`; rejected through the returned promise like upstream.

 @throws MapperRequiredError when the mapper is not a function; rejected
 through the returned promise like upstream.

 @throws InvalidConcurrencyError when `concurrency` is not a safe integer
 from 1 and up and not `Number.POSITIVE_INFINITY`; rejected through the
 returned promise like upstream.

 @example
 ```ts
 const results = await mapConcurrently({
   run: { iterable: ['a',], mapper: identity, },
   isSettled: false,
 },);
 ```
 */
export async function mapConcurrently<Element, NewElement, Result>(
  {
    run,
    isSettled,
  }: ConcurrentRun<Element, NewElement>,
): Promise<Result[]> {
  /**
   The caller's run destructured inside the async body, so a non-object run
   rejects the returned promise.
   */
  const {
    iterable,
    mapper,
    options,
  } = run;

  /**
   Deferred settlement of the run's promise, funneling upstream
   `p-map`'s executor-style synchronous failures into it.
   */
  const settlement = Promise.withResolvers<Result[]>();

  /**
   Runs the whole map synchronously up to its detached continuations, so any
   synchronous failure funnels into the caller's `catch` exactly like
   upstream `p-map`'s promise executor throwing.
   
   @throws Nothing observable; synchronous failures reach the run's
   settlement through the caller's `catch`.
   */
  function executeRun(): void {
    /**
     Run configuration destructured with upstream `p-map`'s defaults, before
     every check like upstream's parameter destructuring.
     */
    const {
      concurrency,
      stopOnError,
      signal,
    } = resolveMapOptions(options,);
    validateInput(iterable,);
    validateMapper(mapper,);
    validateConcurrency(concurrency,);

    /**
     Results by input position, including `pMapSkip` entries staged for
     removal.
     */
    const result: unknown[] = [];
    /**
     Failures staged by input position like `result`, so the
     `stopOnError: false` `AggregateError` reports them in input order.
     */
    const errors: unknown[] = [];
    /**
     Positions pulled from the source whose element or mapper call has not
     finished yet, so settling waits for every earlier one.
     */
    const pendingIndexes = new Set<number>();
    /**
     Mutable run state, kept in one record because function-root `let`
     bindings are banned by repository lint.
     */
    const state = {
      /**
       Mapper results equal to `pMapSkip` so far, so a run without skips can
       hand over its dense result array untouched.
       */
      skippedCount: 0,
      /**
       Whether the run settled, checked by every detached continuation.
       */
      isResolved: false,
      /**
       Whether the source reported `done`, halting further pulls.
       */
      isIterableDone: false,
      /**
       Lowest position whose pull reported `done` or whose mapper returned
       `pMapStop`, or infinity while none has. Pulls and mapper results at or
       past it do not affect the result.
       */
      doneIndex: Number.POSITIVE_INFINITY,
      /**
       Position handed to the next pull, taken when the pull starts.
       */
      currentIndex: 0,
    };
    /**
     Source iterator, created after every check like upstream `p-map`.
     */
    const iterator = selectIterator(iterable,);

    /**
     Removes the abort listener, called whenever the run settles.
     */
// mutation-test-disable-next-line block, string -- cleanup only: the abort listener is registered `once` and no public surface observes its removal after the run settles
    function cleanup(): void {
      signal?.removeEventListener(
        // mutation-test-disable-next-line string -- cleanup only: the abort listener is registered `once` and no public surface observes which event name it detaches from
        'abort',
        signalListener,
      );
    }

    /**
     Settles the run with its collected results and detaches the abort
     listener.

     @param value - Results handed to the caller's promise.
     */
    function resolve(value: unknown[],): void {
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- every value reaching this resolver is proven a non-skip mapper result: either no result equals the skip sentinel, or the caller built the value list excluding every one
      settlement.resolve(value as Result[],);
      cleanup();
    }

    /**
     Rejects the run once, closes the source when it never reported `done`,
     and detaches the abort listener.

     @param reason - Failure handed to the caller's promise.
     */
    function reject(reason: unknown,): void {
      if (state.isResolved)
        return;

// mutation-test-disable-next-line boolean -- spawn-loop control only: the loop pulls race the rejection and the run settlement is guarded elsewhere
      state.isResolved = true;
      settlement.reject(reason,);
      cleanup();

      if (!state.isIterableDone)
        void closeIterator(iterator,);
    }

    /**
     Aborts the run with the signal's reason when the signal fires.

     Registered only while a signal exists, so the optional read below always
     finds one at call time.
     */
    function signalListener(): void {
      /**
       Abort reason read now, after the optional chain proves the signal
       present at registration time.
       */
      const abortReason: unknown = signal?.reason;
      reject(abortReason,);
    }

    // Upstream `p-map` guards signal handling with `if (signal)`, so any
    // falsy `signal` value (including `null` and other non-signals) skips it
    // entirely instead of failing.
    if (signal) {
      if (signal.aborted) {
        /**
         Reason of the already-aborted signal, rejecting the run before any
         pull happens.
         */
        const abortReason: unknown = signal.reason;
        reject(abortReason,);
        return;
      }

      signal.addEventListener(
        'abort',
        signalListener,
// mutation-test-disable-next-line object -- listener registration shape only: the run settles once, so a repeated abort callback is guarded and unobservable
        {
// mutation-test-disable-next-line boolean -- listener registration shape only: the run settles once, so a repeated abort callback is guarded and unobservable
          once: true,
        },
      );
    }

    /**
     Source puller that latches `done`, so a source like a queue is never
     pulled again after it reported `done`.
     */
    const pull = createPuller(iterator,);

    /**
     Settles the run once the source is done and every earlier pull and mapper
     call has finished.

     A pull before the `done` can still produce an element, while pulls after
     it do not affect the result.
     */
    function settleIfFinished(): void {
      if ((state.isResolved) || (state.doneIndex === Number.POSITIVE_INFINITY))
        return;

      for (const pendingIndex of pendingIndexes)
        if (pendingIndex < state.doneIndex)
          return;

      if (!stopOnError) {
        /**
         Failures of the positions before the `done`, in input order.
         */
        const collectedErrors: unknown[] = [];

        for (let position = 0; position < Math.min(
          state.doneIndex,
          errors.length,
        ); position += 1)
          if (position in errors)
            collectedErrors.push(errors[position],);

        if (collectedErrors.length > 0) {
          // oxlint-disable-next-line unicorn/error-message -- upstream `p-map` builds its AggregateError message-less; the differential oracle compares `.message` between both implementations
          reject(new AggregateError(collectedErrors,),);
          return;
        }
      }

      state.isResolved = true;

      // A stop ended the result before the source did, so close it, like
      // `for await` does on a `break`. Not at the stop itself, as a source
      // that settles its `next()` calls out of call order may still owe an
      // earlier element, and closing could drop it.
// mutation-test-disable-next-line conditional, block -- close bookkeeping only: closing an already-ended source is a no-op the source cannot observe through results
      if (!state.isIterableDone)
        void closeIterator(iterator,);

      /**
       Results before the `done`: it is final, like `for await`, so a `done`
       that lands between two elements ends the result there and everything
       the source produced past it is dropped.
       */
      const lastIndex = Math.min(
        state.doneIndex,
        result.length,
      );

      // Every position below `lastIndex` has a value: a position only goes
      // unassigned when its pull reported `done`, and the lowest such position
      // is `doneIndex`. So the array is dense below `lastIndex`, and without
      // skips it can be handed over untouched.
// mutation-test-disable-next-line conditional, block, equality -- result-shaping equivalence: with no skips and nothing cut off the filtered pass resolves the identical values
      if ((lastIndex === result.length) && (state.skippedCount === 0)) {
        resolve(result,);
        return;
      }

      // Otherwise this drops the skipped results and whatever the `done` cut
      // off.
      resolve(result.slice(
        0,
        lastIndex,
      )
        .filter(isNotSkip,),);
    }

    /**
     Pulls the next source item, runs its detached mapper continuation, and
     settles the run when the source reports `done` with nothing earlier
     still pending.

     Named `pullNext` rather than upstream's `next` so repository lint does
     not read the calls as node-style callbacks. Self-calling through the
     detached continuation keeps one `pullNext` frame alive at a time: each
     call returns before its successor starts, so the pull chain is
     event-driven rather than a growing recursion.

     @throws Whatever the source throws before its `done`, observed by the
     caller's `await` of this function.
     */
    async function pullNext(): Promise<void> {
      // After a `done`, a pull would take an item nobody is going to map,
      // which a one-shot source, like a queue, would lose.
// mutation-test-disable-next-line conditional, logical, equality -- spawn-loop control only: pulls after `done` or settlement are answered with `done` without touching the source, so pull counts and settlement are identical
      if ((state.isResolved) || (state.currentIndex >= state.doneIndex))
        return;

      // Taken when the pull starts, not when it resolves, so a source that
      // settles its `next()` calls out of call order still gets the right
      // position.
      /**
       Position of this element in the input, assigned before every mapper
       call.
       */
      const index = state.currentIndex;
      state.currentIndex += 1;
      pendingIndexes.add(index,);

      /**
       Slot for the item pulled for this position, filled by the `try`
       below; its initial `done` marker is replaced before anything reads it.
       */
      const pulled: { item: IteratorResult<Element | Promise<Element>, unknown>; } = {
        item: {
          done: true,
          value: undefined,
        },
      };
      try {
        pulled.item = await pull();
      }
      catch (error) {
        pendingIndexes.delete(index,);

        // A failure from a pull after the end of the source is dropped, as
        // `done` is final, so it must have no effect.
        if (index < state.doneIndex)
          throw error;

        settleIfFinished();
        return;
      }

      /**
       Source item pulled for this position.
       */
      const nextItem = pulled.item;

      // oxlint-disable-next-line typescript/strict-boolean-expressions -- upstream `p-map` checks `if (nextItem.done)` with truthiness, so a non-conforming source's truthy `done` must terminate the pull exactly like upstream
      if (nextItem.done) {
        pendingIndexes.delete(index,);
        state.doneIndex = Math.min(
          state.doneIndex,
          index,
        );
        state.isIterableDone = true;
        settleIfFinished();
        return;
      }

      /**
       Element source item for this call, narrowed past the `done` marker above
       and passed as a parameter: narrowing never reaches the detached
       continuation's function declaration, and the value getter must run
       inside that continuation's `try` exactly as upstream `p-map`'s detached
       runner reads it.
       */
      const elementItem: YieldedItem<Element> = nextItem;

      /**
       Awaits this element, runs its mapper call, then pulls the next item
       from the tail of this detached continuation, exactly as upstream
       `p-map`'s detached runner does.

       @throws Nothing; every failure is routed to the run's settlement.
       */
      async function runElement(
        {
          item,
          position,
        }: {
          /**
           Narrowed source item whose value getter runs inside the `try`.
           */
          readonly item: YieldedItem<Element>;
          /**
           Position of this element in the input.
           */
          readonly position: number;
        },
      ): Promise<void> {
        /**
         Whether this runner goes on to pull the next source item: false when
         the run settled, a `done` cut this position off, or a mapper failure
         rejected the run.
         */
        const flow = { pullsOnward: true, };

        try {
          /**
           Element after awaiting the source's yielded value, matching
           upstream `p-map`'s per-item await.
           */
          const element = await item.value;

// mutation-test-disable-next-line conditional, logical, equality -- post-settle continuation only: the run promise is already settled and the result array it would write is discarded
          if ((state.isResolved) || (position >= state.doneIndex))
            flow.pullsOnward = false;
          else {
            /**
             Mapper result for this element, awaited when thenable.
             */
            const value = await mapper(
              element,
              position,
            );

            // Ignore an element produced after an earlier pull reported
            // `done` or an earlier mapper returned `pMapStop`.
// mutation-test-disable-next-line conditional, equality -- post-done continuation only: the result is cut at `doneIndex` when it settles, so a stored value past it is never observed
            if (position >= state.doneIndex)
              flow.pullsOnward = false;
            else if (value === pMapStop) {
              // A stop ends the result here, like a `done` from the source,
              // so settling waits only for the earlier elements.
              state.doneIndex = position;
              flow.pullsOnward = false;
            }
            else {
              if (value === pMapSkip)
                state.skippedCount += 1;

              result[position] = (isSettled && (value !== pMapSkip))
                ? {
                  status: 'fulfilled',
                  value,
                }
                : value;
            }
          }
        }
        catch (error) {
          // A failure past the end of the source is dropped, as `done` is
          // final.
          if (position >= state.doneIndex)
            flow.pullsOnward = false;
          else if (isSettled)
            result[position] = {
              status: 'rejected',
              reason: error,
            };
          else if (stopOnError) {
            reject(error,);
            flow.pullsOnward = false;
          }
          else
            errors[position] = error;
        }

        pendingIndexes.delete(position,);
        settleIfFinished();

        if (!flow.pullsOnward)
          return;

        try {
          await pullNext();
        }
        catch (error) {
          // The iterable threw mid-iteration, so the run cannot continue
          // regardless of `stopOnError`: an iterable is likely to keep
          // throwing after its first failure.
          reject(error,);
        }
      }

      void runElement({
        item: elementItem,
        position: index,
      },);
    }

    /**
     Starts the concurrent runners detached, stopping the spawn loop the
     moment the source is exhausted or the run settles (upstream `p-map`
     spins up runners this way so an async input never spawns unbounded
     `next()` calls).

     @throws Nothing; every failure is routed to the run's settlement.
     */
    async function spawnRunners(): Promise<void> {
      /**
       Spawn slot counter; one runner is spawned per iteration until the
       bound is reached or the source is done.
       */
      for (let spawnIndex = 0; spawnIndex < concurrency; spawnIndex += 1) {
        try {
          /* oxlint-disable no-await-in-loop -- upstream `p-map` awaits each runner spawn in sequence before starting the next one */
          await pullNext();
          /* oxlint-enable no-await-in-loop */
        }
// mutation-test-disable-next-line block -- race-masked: the detached runner tail catch rejects the run when this catch block is emptied
        catch (error) {
          reject(error,);
          break;
        }

        if ((state.isResolved) || (state.currentIndex >= state.doneIndex))
          break;
      }
    }

    void spawnRunners();
  }

  try {
    executeRun();
  }
  catch (error) {
    // Synchronous setup failures (destructuring, input probe, mapper and
    // bound checks) land in the run's promise here, mirroring upstream
    // `p-map`'s promise executor throwing.
    settlement.reject(error,);
  }

  return settlement.promise;
}
/* oxlint-enable eslint/require-await */

/**
 Runs one concurrent map: walks a synchronous or asynchronous input, awaits
 each iterated element, and runs the mapper under the `concurrency` bound.

 @typeParam Element - element type after awaiting an iterated item

 @typeParam NewElement - mapper result type, awaited when thenable

 @param run - Inputs walked once in iteration order, the mapper run per
 element with its position, and the options: concurrency bound,
 `stopOnError` policy, and optional abort signal.

 @returns Promise settling with every non-skip mapper result in input order,
 or rejecting with the first mapper failure (`stopOnError`) or an
 `AggregateError` of every failure in input order. A mapper returning
 `pMapStop` ends the result before that element, without waiting for the
 elements after it.

 @throws InvalidInputError when the input is neither an `Iterable` nor an
 `AsyncIterable`, `null` and `undefined` included; rejected through the
 returned promise like upstream.

 @throws MapperRequiredError when the mapper is not a function; rejected
 through the returned promise like upstream.

 @throws InvalidConcurrencyError when `concurrency` is not a safe integer
 from 1 and up and not `Number.POSITIVE_INFINITY`; rejected through the
 returned promise like upstream.

 @example
 ```ts
 import { pMap, } from '\@monochromatic-dev/upstream-fork-p-map';

 const results = await pMap({
   iterable: [
     'a',
     'b',
   ],
   mapper: async function fetchUser(id: string): Promise<string> {
     return `user-${id}`;
   },
   options: { concurrency: 2, },
 });
 ```
 */
export function pMap<Element, NewElement>(
  run: MapRun<Element, NewElement>,
): Promise<Exclude<NewElement, MapControl>[]> {
  return mapConcurrently<Element, NewElement, Exclude<NewElement, MapControl>>({
    run,
    isSettled: false,
  },);
}

/**
 Runs one concurrent map like {@link pMap}, for
 `Promise.allSettled()`: an error from the mapper, or an element of the input
 that rejects, does not reject the run.

 An element that rejects gets a `{ status: 'rejected', reason }` result and the
 mapper is not called for it. A `pMapSkip` result is left out, and `pMapStop`
 stops like in {@link pMap}. The run still rejects when iterating the input
 throws, and when the `signal` aborts.

 @typeParam Element - element type after awaiting an iterated item

 @typeParam NewElement - mapper result type, awaited when thenable

 @param run - Inputs walked once in iteration order, the mapper run per
 element with its position, and the options: concurrency bound and optional
 abort signal.

 @returns Promise settling with one `PromiseSettledResult` per non-skip
 element, in input order.

 @throws InvalidInputError when the input is neither an `Iterable` nor an
 `AsyncIterable`; rejected through the returned promise.

 @throws MapperRequiredError when the mapper is not a function; rejected
 through the returned promise.

 @throws InvalidConcurrencyError when `concurrency` is invalid; rejected
 through the returned promise.

 @example
 ```ts
 import { pMapSettled, } from '\@monochromatic-dev/upstream-fork-p-map';

 const results = await pMapSettled({
   iterable: urls,
   mapper: async function readJson(url: string): Promise<unknown> {
     return (await fetch(url,)).json();
   },
   options: { concurrency: 4, },
 });
 ```
 */
export function pMapSettled<Element, NewElement>(
  run: SettledMapRun<Element, NewElement>,
): Promise<PromiseSettledResult<Exclude<NewElement, MapControl>>[]> {
  return mapConcurrently<Element, NewElement, PromiseSettledResult<Exclude<NewElement, MapControl>>>({
    run,
    isSettled: true,
  },);
}

//endregion Run
