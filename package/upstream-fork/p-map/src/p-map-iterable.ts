/**
 TypeScript fork of `p-map`'s streaming variant: map over iterables with
 limited concurrency and backpressure, yielding results in order.
 
 Derived from [`p-map`](https://github.com/sindresorhus/p-map) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Streaming semantics match `p-map` 7.1.1: mapper calls stay
 within `concurrency`,
 resolved-but-unconsumed results stay within
 `backpressure`,
 skips are dropped rather than yielded,
 and mapper failures
 surface as a throw from the async iterator instead of a rejection.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  closeIterator,
  createPuller,
  selectIterator,
  type SourceIterator,
  validateInput,
} from './map-iterator.ts';
import {
  type IterableMapOptions,
  resolveIterableMapOptions,
  validateBackpressure,
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
 One streaming-map run: the inputs to walk,
 the mapper to run per element,
 and the upstream-shaped options.
 
 The single destructured object replaces upstream `p-map`'s
 `pMapIterable(iterable, mapper, options)` positional parameters, which
 repository lint bans; `options` keeps its shape and destructuring failure
 semantics.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @typeParam NewElement - mapper result type, awaited when thenable
 
 @example
 ```ts
 const run: IterableMapRun<number, number> = {
   iterable: [1, 2, 3,],
   mapper: async function double(value: number,): Promise<number> {
     return value * 2;
   },
   options: {
     concurrency: 2,
     backpressure: 2,
   },
 };
 ```
 */
export type IterableMapRun<Element, NewElement> = {
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
   Concurrency bound and optional backpressure bound, destructured with
   upstream `p-map`'s defaults.
   */
  readonly options?: IterableMapOptions;
};

/**
 Outcome of one spawned pull, as carried through the spawn queue.
 
 Shaped like upstream `p-map`'s three result forms (`{done}`,
 `{done, value}`,
 and `{error}`),
 discriminated here because the nullish-union ban
 forbids modeling "error carries no done" with optional fields.
 
 @typeParam NewElement - mapper result type, awaited when thenable
 */
type PullOutcome<NewElement> =
  | {
    /**
     Source reported `done`; iteration ends.
     */
    readonly kind: 'done';
  }
    | {
    /**
     One mapped element, possibly {@link pMapSkip} for a dropped result.
     */
      readonly kind: 'value';
    /**
     Mapper result or {@link pMapSkip}.
     */
      readonly value: NewElement | typeof pMapSkip;
    }
    | {
    /**
     The pull or its mapper call failed; carried as a value so an abandoned
     consumer never sees an unhandled rejection.
     */
      readonly kind: 'error';
    /**
     Failure to throw from the async iterator once this pull reaches the
     head of the spawn queue.
     */
      readonly error: unknown;
    };

//endregion Types

//region Run

/**
 Returns the streaming variant of the concurrent map: an async iterable
 yielding each mapper result in input order under the concurrency and
 backpressure bounds.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @typeParam NewElement - mapper result type, awaited when thenable
 
 @param iterable - Inputs walked once in iteration order.
 
 @param mapper - Function run per element with its position in the input.
 
 @param options - Concurrency bound and optional backpressure bound.
 
 @returns Async iterable of non-skip mapper results in input order.
 
 @throws InvalidInputError when the input is neither an `Iterable` nor an
 `AsyncIterable`; thrown synchronously like upstream.
 
 @throws MapperRequiredError when the mapper is not a function; thrown
 synchronously like upstream.
 
 @throws InvalidConcurrencyError when `concurrency` is not a safe integer
 from 1 and up and not `Number.POSITIVE_INFINITY`; thrown synchronously like
 upstream.
 
 @throws InvalidBackpressureError when `backpressure` is not a safe integer
 from `concurrency` and up and not `Number.POSITIVE_INFINITY`; thrown
 synchronously like upstream.
 
 @example
 ```ts
 import { pMapIterable, } from '\@monochromatic-dev/upstream-fork-p-map';
 
 for await (const post of pMapIterable({
   iterable: postIds,
   mapper: getPostMetadata,
   options: { concurrency: 8, },
 }))
   console.log(post,);
 ```
 */
export function pMapIterable<Element, NewElement>(
  {
    iterable,
    mapper,
    options,
  }: IterableMapRun<Element, NewElement>,
): AsyncIterable<Exclude<NewElement, typeof pMapSkip>> {
  /**
   Run configuration destructured with upstream `p-map`'s defaults, before
   every check like upstream's parameter destructuring.
   */
  const {
    concurrency,
    backpressure,
  } = resolveIterableMapOptions(options,);
  validateInput(iterable,);
  validateMapper(mapper,);
  validateConcurrency(concurrency,);
  validateBackpressure({
    backpressure,
    concurrency,
  },);

  return {
    [Symbol.asyncIterator](): AsyncGenerator<Exclude<NewElement, typeof pMapSkip>, void, undefined> {
      /**
       Mutable iteration state, kept in one record because function-root
       `let` bindings are banned by repository lint.
       */
      const state = {
        /**
         Index in `promises` of the oldest entry the consumer has not
         collected, moved forward instead of shifting every entry per element.
         */
        headIndex: 0,
        /**
         Whether iteration stopped, halting every further spawn and dropping
         pending elements.
         */
        isDone: false,
        /**
         Whether the source reported `done`, halting further pulls.
         */
        isIterableDone: false,
        /**
         Position handed to the next pull, taken when the pull starts.
         */
        index: 0,
        /**
         Lowest position whose pull, element, or mapper threw. The stream ends
         there, like a `throw` in a loop: nothing past it is pulled and no
         pending element past it is mapped, while the earlier elements still
         finish and are yielded first.
         */
        stopIndex: Number.POSITIVE_INFINITY,
        /**
         Elements being read, or waiting for a mapper slot. They do not hold a
         mapper slot, so the next element can be read while the mappers run;
         they have their own cap of `concurrency`, so the source never sees
         more than `concurrency` overlapping `next()` calls.
         */
        readingCount: 0,
        /**
         Mapper calls holding a slot, bounded by `concurrency`.
         */
        runningMappersCount: 0,
        /**
         Workers that have started waiting for a mapper slot so far.
         */
        mapperWaiterCount: 0,
        /**
         Waiting workers that a finished mapper has handed its slot to so far.
         */
        wokenMapperWaiterCount: 0,
      };
      /**
       Resolvers of the workers waiting for a mapper slot, in the order they
       started waiting. A finished mapper hands its slot to the oldest one.
       */
      const mapperWaiters = new Map<number, () => void>();
      /**
       Spawned pulls awaiting collection, in yield order; a consumed entry is
       replaced by `consumedSlot` and the consumed prefix is dropped in halves.
       */
      const promises: Promise<PullOutcome<NewElement>>[] = [];
      /**
       Shared placeholder for a collected entry, so the queue releases the
       collected outcome without holding an absent value.
       */
      const consumedSlot = Promise.resolve<PullOutcome<NewElement>>({ kind: 'done', },);
      /**
       Collects the spawned pulls in yield order and yields their results.

       @yields Each non-skip mapper result, in input order.

       @returns Nothing; the stream ends when the source is done.

       @throws The first failure of the stream, at its own position.
       */
      async function* consume(): AsyncGenerator<Exclude<NewElement, typeof pMapSkip>, void, undefined> {
        // Created when the first `next()` arrives, not when the iterable is asked for its iterator, like upstream `p-map`'s generator body.
        /**
         Source iterator, created once per iteration like upstream `p-map`.
         */
        const iterator: SourceIterator<Element> = selectIterator(iterable,);
        /**
         Source puller that latches `done`, so a source like a queue is never
         pulled again after it reported `done`.
         */
        const pull = createPuller(iterator,);

        /**
         Frees one mapper slot: hands it to the oldest waiting worker, or gives
         it back to the pool when none waits.
         */
        function releaseMapperSlot(): void {
          if (state.wokenMapperWaiterCount === state.mapperWaiterCount) {
            state.runningMappersCount -= 1;
            return;
          }

          nonNullishOrThrow(mapperWaiters.get(state.wokenMapperWaiterCount,),)();
          mapperWaiters.delete(state.wokenMapperWaiterCount,);
          state.wokenMapperWaiterCount += 1;
        }

        /**
         Spawns one pull when the reading, concurrency, and backpressure bounds
         allow it, where upstream `p-map` re-spawns: at the start, after each
         source pull, once an element no longer waits for a mapper slot, and
         after each collection.
         */
        function trySpawn(): void {
          // Do not spawn once the consumer is gone, the source is done, or the
          // stream ends at an earlier element: the pull would be wasted work,
          // and for a one-shot source, like a queue, it takes an item nobody is
          // going to ask for.
// mutation-test-disable-next-line conditional, logical, equality -- spawn-gate control only: trySpawn after stop is reached solely from pending continuations whose outcomes are dropped
          if (state.isDone
            || state.isIterableDone
            || (state.index > state.stopIndex))
            return;

          /**
           Whether every bound still allows a spawn. Skipped results stay in the
           queue until collected, so they count toward `backpressure` too.
           */
          const withinBounds = (state.readingCount < concurrency)
            && ((promises.length - state.headIndex) < backpressure);
          if (!withinBounds)
            return;

          state.readingCount += 1;

          // Taken when the pull starts, not when it resolves, so a source that
          // settles its `next()` calls out of call order still gets the right
          // position.
          /**
           Position of this element in the input, assigned before the pull.
           */
          const currentIndex = state.index;
          state.index += 1;

          /**
           Awaits this pull's source item, waits for a mapper slot, runs its
           mapper call, and returns the outcome as a value.

           @returns Outcome of this pull; failures come back as the error
           outcome instead of rejecting, so a promise the consumer abandons after
           an earlier error never becomes an unhandled rejection.
           */
          async function runPull(): Promise<PullOutcome<NewElement>> {
            /**
             Which resources this pull still holds, so every exit gives them
             back exactly once.
             */
            const holds = {
              isReading: true,
              hasMapperSlot: false,
            };

            try {
              /**
               Next source item, or its `done` report.
               */
              const nextItem = await pull();

              // oxlint-disable-next-line typescript/strict-boolean-expressions -- upstream `p-map` checks truthiness of `done`, so a non-conforming source's truthy `done` must terminate the pull exactly like upstream
              if (nextItem.done) {
                state.isIterableDone = true;
                holds.isReading = false;
                state.readingCount -= 1;
                return { kind: 'done', };
              }

              // Spawn if still below the reading and backpressure limits.
              trySpawn();

              /**
               Element after awaiting the source's yielded value.
               */
              const element = await nextItem.value;

              if (state.runningMappersCount < concurrency)
                state.runningMappersCount += 1;
              else {
                // `releaseMapperSlot` hands the slot over without decrementing
                // `runningMappersCount`, so it is already counted.
                /**
                 Deferred wake-up for this pull, resolved when a finished
                 mapper hands its slot over.
                 */
                const slot = Promise.withResolvers<void>();
                mapperWaiters.set(
                  state.mapperWaiterCount,
                  slot.resolve,
                );
                state.mapperWaiterCount += 1;
                await slot.promise;
              }

              holds.hasMapperSlot = true;
              holds.isReading = false;
              state.readingCount -= 1;

              // Spawn now that this element no longer waits for a mapper slot,
              // so the next one is read while this one is mapped.
              trySpawn();

              // The consumer stopped iterating, or the stream ends before this
              // element, while this element was being read or waiting for a
              // mapper slot, so drop it instead of doing work nobody will
              // consume.
// mutation-test-disable-next-line conditional, logical, equality, block -- dropped-work path only: the consumer already left the iteration or the stream ends earlier, so whether the pending mapper runs changes no yielded value, error, or close
              if (state.isDone || (currentIndex > state.stopIndex)) {
                holds.hasMapperSlot = false;
                releaseMapperSlot();
                return {
                  kind: 'value',
                  value: pMapSkip,
                };
              }

              /**
               Mapper result for this element, awaited when thenable.
               */
              const returnValue = await mapper(
                element,
                currentIndex,
              );

              holds.hasMapperSlot = false;
              releaseMapperSlot();

              return {
                kind: 'value',
                value: returnValue,
              };
            }
            catch (error) {
              if (holds.isReading)
                state.readingCount -= 1;

              if (holds.hasMapperSlot)
                releaseMapperSlot();

              // The error is thrown at its own position, after the results
              // before it, so those still finish.
              state.stopIndex = Math.min(
                state.stopIndex,
                currentIndex,
              );
              return {
                kind: 'error',
                error,
              };
            }
          }

          promises.push(runPull(),);
        }

        trySpawn();

        /**
         Source shutdown on every exit from the consumption below: the
         consumer stopping early must end pending pulls and close the source
         exactly the way upstream `p-map`'s `finally` block does. The disposer
         detaches the close like upstream, so a source blocked inside `next()`
         can never block the consumer's exit.
         */
        using sourceShutdown: Disposable = {
          [Symbol.dispose]: function disposeSource(): void {
// mutation-test-disable-next-line boolean -- dispose stop flag only: the consumer already left the iteration and the source close is gated separately
            state.isDone = true;

            if (!state.isIterableDone)
              void closeIterator(iterator,);
          },
        };
        void sourceShutdown;

        /* oxlint-disable no-await-in-loop -- the consumer collects pulls one at a time, in yield order */
// mutation-test-disable-next-line conditional, equality -- loop-shape only: the queue empties through the `done` outcome before the length check is reached
        while (state.headIndex < promises.length) {
          /**
           Oldest spawned pull's outcome; only the head is awaited so results
           leave the queue in yield order.
           */
          const outcome = await nonNullishOrThrow(promises[state.headIndex],);

          // Release the entry instead of leaving it for a `shift()` to move,
          // and drop the consumed prefix once it is at least half the array.
          // That keeps the backing store proportional to the results still in
          // flight rather than to the total ever spawned, which matters for an
          // endless source, while costing one copy per half of the array, so
          // it stays O(1) per element.
          promises[state.headIndex] = consumedSlot;
          state.headIndex += 1;

          if ((state.headIndex * 2) >= promises.length) {
            void promises.splice(
              0,
              state.headIndex,
            );
            state.headIndex = 0;
          }

          if (outcome.kind === 'error')
            throw outcome.error;

          if (outcome.kind === 'done')
            return;

          // Spawn if just dropped below the backpressure limit and still below
          // the reading limit.
          trySpawn();

          // A skipped result stays in the queue and is stepped over here:
          // removing it from the queue in the worker instead would make every
          // skip an O(n) scan and move.
          if (outcome.value === pMapSkip)
            continue;

          // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the skip check above proved the value is not pMapSkip
          yield outcome.value as Exclude<NewElement, typeof pMapSkip>;
        }
        /* oxlint-enable no-await-in-loop */
      }

      /**
       The consuming generator, with `return()` and `throw()` replaced below so
       a stop is seen by the workers at once.
       */
      const generator = consume();

      /**
       The generator's own `return`, called after the stop is recorded.
       */
      const generatorReturn = generator.return
        .bind(generator,);
      /**
       The generator's own `throw`, called after the stop is recorded.
       */
      const generatorThrow = generator.throw
        .bind(generator,);

      // Override `return()` and `throw()` on the generator itself instead of
      // wrapping it, so it keeps what it inherits, like `Symbol.asyncDispose`
      // for `await using`, and `next()` stays the native one. Plain functions,
      // not `async`: the generator already returns a promise, and an `async`
      // function would wrap it in two more promises and two more microtask
      // hops.
      generator.return = function returnAndStop(value: Parameters<typeof generatorReturn>[0],): Promise<IteratorResult<Exclude<NewElement, typeof pMapSkip>, void>> {
        state.isDone = true;
        return generatorReturn(value,);
      };
      generator.throw = function throwAndStop(error: unknown,): Promise<IteratorResult<Exclude<NewElement, typeof pMapSkip>, void>> {
        state.isDone = true;
        return generatorThrow(error,);
      };

      return generator;
    },
  };
}

//endregion Run
