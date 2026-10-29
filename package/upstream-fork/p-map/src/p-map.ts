/**
 TypeScript fork of `p-map`: map over iterables with limited concurrency.
 
 Derived from [`p-map`](https://github.com/sindresorhus/p-map) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Scheduling semantics match `p-map` 7.0.8: up to `concurrency`
 mapper calls run at once over a synchronous or asynchronous input, results
 collect in input order, and `stopOnError`, the abort `signal`, and
 `pMapSkip` behave as upstream defines them.
 
 @module
 */

import {
  closeIterator,
  selectIterator,
  validateInput,
  type YieldedItem,
} from './map-iterator.ts';
import {
  type MapOptions,
  resolveMapOptions,
  validateConcurrency,
} from './map-options.ts';
import {
  type MapInput,
  type Mapper,
  validateMapper,
} from './mapper.ts';
import { pMapSkip, } from './p-map-skip.ts';

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

//endregion Types

//region Run

/* oxlint-disable eslint/require-await -- `pMap` stays `async` so a destructuring failure rejects the caller's promise like upstream `p-map`'s `async` signature, and returning the deferred promise without awaiting it preserves upstream's promise-adoption tick (measured: an `await` here settles the caller one microtask early) */
/**
 Runs one concurrent map over upstream `p-map`'s run structure: option
 destructuring first, then the input, mapper, and concurrency checks in
 upstream's order, then detached per-element runners pulling the source
 under the concurrency bound.
 
 The run body sits in one `try` that funnels synchronous failures into the
 returned promise, mirroring upstream's `new Promise(executor)`: a hostile
 input, mapper, or iterator becomes a rejection at the same point in the
 run's lifetime.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @typeParam NewElement - mapper result type, awaited when thenable
 
 @param iterable - Inputs walked once in iteration order.
 
 @param mapper - Function run per element with its position in the input.
 
 @param options - Concurrency bound, `stopOnError` policy, and optional
 abort signal.
 
 @returns Promise settling with every non-skip mapper result in input order,
 or rejecting with the first mapper failure (`stopOnError`) or an
 `AggregateError` of every failure.
 
 @throws InvalidInputError when the input is neither an `Iterable` nor an
 `AsyncIterable`; rejected through the returned promise like upstream.
 
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
export async function pMap<Element, NewElement>(
  {
    iterable,
    mapper,
    options,
  }: MapRun<Element, NewElement>,
): Promise<(Exclude<NewElement, typeof pMapSkip>)[]> {
  /**
   Deferred settlement of the run's promise, funneling upstream
   `p-map`'s executor-style synchronous failures into it.
   */
  const settlement = Promise.withResolvers<(Exclude<NewElement, typeof pMapSkip>)[]>();

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
     Mapper results in input order, including `pMapSkip` entries staged for
     removal.
     */
    const result: (NewElement | typeof pMapSkip)[] = [];
    /**
     Failures collected for the `stopOnError: false` `AggregateError`.
     */
    const errors: unknown[] = [];
    /**
     Indices whose mapper returned `pMapSkip`, staged like upstream's
     `skippedIndexesMap`.
     */
    const skippedIndexesMap = new Map<number, typeof pMapSkip>();
    /**
     Mutable run state, kept in one record because function-root `let`
     bindings are banned by repository lint.
     */
    const state = {
      /**
       Whether {@link reject} already ran, stopping the runner spawn loop.
       */
      isRejected: false,
      /**
       Whether the run settled, checked by every detached continuation.
       */
      isResolved: false,
      /**
       Whether the source reported `done`, halting further pulls.
       */
      isIterableDone: false,
      /**
       Mapper calls started and not yet settled.
       */
      resolvingCount: 0,
      /**
       Position handed to the next element's mapper call.
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
    function resolve(value: (NewElement | typeof pMapSkip)[],): void {
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- every value reaching this resolver is proven a non-skip mapper result: either the skip-index map is empty, or the caller built the value list excluding every staged index
      settlement.resolve(value as (Exclude<NewElement, typeof pMapSkip>)[],);
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

      state.isRejected = true;
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
     Pulls the next source item, runs its detached mapper continuation, and
     finalizes the run when the source reports `done` with nothing resolving.
     
     Named `pullNext` rather than upstream's `next` so repository lint does
     not read the calls as node-style callbacks. Self-calling through the
     detached continuation keeps one `pullNext` frame alive at a time: each
     call returns before its successor starts, so the pull chain is
     event-driven rather than a growing recursion.
     
     @throws Whatever the source or mapper throws, observed by the caller's
     `await` of this function.
     */
    async function pullNext(): Promise<void> {
// mutation-test-disable-next-line conditional -- post-settle continuation only: the run promise is already settled and the result array it would write is discarded
      if (state.isResolved)
        return;

      /**
       Synthetic `done` marker, `satisfies`-typed so its literal `done`
       discriminant survives for the narrowing below.
       */
      const doneItem = {
        done: true,
        value: undefined,
      } satisfies IteratorReturnResult<unknown>;
      /**
       Next source item, or the synthetic `done` marker once the source
       already reported `done`: a source like a queue may block in `next()`
       after it is exhausted, so pulling again could hang the completion
       below.
       */
// mutation-test-disable-next-line conditional -- spawn-loop control only: pulls after `done` synthesize `done` without touching the source, so pull counts and settlement are identical
      const nextItem: IteratorResult<Element | Promise<Element>, unknown> = state.isIterableDone
        ? doneItem
        : await iterator.next();

      /**
       Position of this element in the input, assigned before every mapper
       call.
       */
      const index = state.currentIndex;
      state.currentIndex += 1;

      /**
       Repeated `done` reports are possible because `iterator.next()` runs in
       parallel; the shutdown below runs once because the skip staging is not
       idempotent.
       */
      // oxlint-disable-next-line typescript/strict-boolean-expressions -- upstream `p-map` checks `if (nextItem.done)` with truthiness, so a non-conforming source's truthy `done` must terminate the pull exactly like upstream
      if (nextItem.done) {
        state.isIterableDone = true;

        if ((state.resolvingCount === 0) && (!state.isResolved)) {
          if ((!stopOnError) && (errors.length > 0)) {
            // oxlint-disable-next-line unicorn/error-message -- upstream `p-map` builds its AggregateError message-less; the differential oracle compares `.message` between both implementations
            reject(new AggregateError(errors,),);
            return;
          }

// mutation-test-disable-next-line boolean -- finalize bookkeeping only: resolution happens when no mapper call is still running, so nothing observes the flag afterwards
          state.isResolved = true;

// mutation-test-disable-next-line conditional, block -- result-shaping equivalence: with an empty skip map the filtered pass resolves the identical values
          if (skippedIndexesMap.size === 0) {
            resolve(result,);
            return;
          }

          /**
           Results with every `pMapSkip` entry dropped.
           */
          const pureResult: (NewElement | typeof pMapSkip)[] = [];

          for (const [
            skippedIndex,
            value,
          ] of result.entries())
            if (skippedIndexesMap.get(skippedIndex,) !== pMapSkip)
              pureResult.push(value,);

          resolve(pureResult,);
        }

        return;
      }

      state.resolvingCount += 1;

      /**
       Element source item for this call, narrowed past the synthetic `done`
       marker above and passed as a parameter: narrowing never reaches the
       detached continuation's function declaration, and the value getter
       must run inside that continuation's `try` exactly as upstream
       `p-map`'s detached runner reads it.
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
        try {
          /**
           Element after awaiting the source's yielded value, matching
           upstream `p-map`'s per-item await.
           */
          const element = await item.value;

// mutation-test-disable-next-line conditional -- post-settle continuation only: the run promise is already settled and the result array it would write is discarded
          if (state.isResolved)
            return;

          /**
           Mapper result for this element, awaited when thenable.
           */
          const value = await mapper(
            element,
            position,
          );

          if (value === pMapSkip)
            skippedIndexesMap.set(
              position,
              pMapSkip,
            );

          result[position] = value;
        }
        catch (error) {
          if (stopOnError) {
            reject(error,);
            return;
          }

          errors.push(error,);
        }

        state.resolvingCount -= 1;

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

        if (state.isIterableDone || state.isRejected)
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

//endregion Run
