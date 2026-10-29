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
  type MapControl,
  type MapInput,
  type Mapper,
  validateMapper,
} from './mapper.ts';
import { pMapSkip, } from './p-map-skip.ts';
import { pMapStop, } from './p-map-stop.ts';

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
     One mapped element, possibly {@link pMapSkip} for a dropped result or
     {@link pMapStop} for the end of the stream.
     */
      readonly kind: 'value';
    /**
     Mapper result, {@link pMapSkip}, or {@link pMapStop}.
     */
      readonly value: NewElement | MapControl;
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

/**
 Marks a queue slot whose worker has not settled yet, when the results are
 not kept in input order.
 */
// mutation-test-disable-next-line string -- the symbol is module-private and only compared by identity, so its description is never read, logged, or thrown
const emptySlot: unique symbol = Symbol('p-map empty queue slot',);

/**
 One entry of the spawn queue: the pending pull itself with `preserveOrder`;
 without it, an empty slot that the first worker to settle fills with its
 outcome.

 @typeParam NewElement - mapper result type, awaited when thenable
 */
type QueueSlot<NewElement> =
  | Promise<PullOutcome<NewElement>>
    | PullOutcome<NewElement>
    | typeof emptySlot;

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
): AsyncIterable<Exclude<NewElement, MapControl>> {
  /**
   Run configuration destructured with upstream `p-map`'s defaults, before
   every check like upstream's parameter destructuring.
   */
  const {
    concurrency,
    backpressure,
    preserveOrder,
  } = resolveIterableMapOptions(options,);
  validateInput(iterable,);
  validateMapper(mapper,);
  validateConcurrency(concurrency,);
  validateBackpressure({
    backpressure,
    concurrency,
  },);

  return {
    [Symbol.asyncIterator](): AsyncGenerator<Exclude<NewElement, MapControl>, void, undefined> {
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
         Lowest position whose mapper returned `pMapStop`, or, with
         `preserveOrder`, whose pull, element, or mapper threw. The stream ends
         there, like a `break` or a `throw` in a loop: nothing past it is
         pulled and no pending element past it is mapped, while the earlier
         elements still finish and are yielded first.
         */
        stopIndex: Number.POSITIVE_INFINITY,
        /**
         Lowest position whose pull reported `done`. Without
         `preserveOrder`, the pulls after it have their slots given back, so
         their errors are dropped.
         */
        doneIndex: Number.POSITIVE_INFINITY,
        /**
         Without `preserveOrder`, the oldest empty slot in the queue, where
         the next settled worker puts its result.
         */
        fillIndex: 0,
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
      const promises: QueueSlot<NewElement>[] = [];
      /**
       Without `preserveOrder`, the resolver of the consumer while it waits
       on an empty head slot; at most one entry.
       */
      const wakers: ((outcome: PullOutcome<NewElement>) => void)[] = [];
      /**
       Without `preserveOrder`, the positions of the pulls whose worker has
       not filled a slot yet.
       */
      const unfilledIndexes = new Set<number>();
      /**
       Shared placeholder for a collected entry, so the queue releases the
       collected outcome without holding an absent value.
       */
      const consumedSlot: QueueSlot<NewElement> = Promise.resolve<PullOutcome<NewElement>>({ kind: 'done', },);
      /**
       Collects the spawned pulls in yield order and yields their results.

       @yields Each non-skip mapper result, in input order.

       @returns Nothing; the stream ends when the source is done.

       @throws The first failure of the stream, at its own position.
       */
      async function* consume(): AsyncGenerator<Exclude<NewElement, MapControl>, void, undefined> {
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
         Without `preserveOrder`, puts one result in the oldest empty slot and
         wakes the consumer if it waits on that slot.

         @param outcome - Result of a settled worker, or a given-back slot.
         */
        function fillSlot(outcome: PullOutcome<NewElement>,): void {
          promises[state.fillIndex] = outcome;
          state.fillIndex += 1;
          wakers.pop()?.(outcome,);
        }

        /**
         Without `preserveOrder`, fills a slot a tick after the worker settles,
         rather than in the worker itself, for the same reason the consumer
         awaits a filled slot: so the consumer never runs, and pulls, earlier.

         @param worker - The pull's worker; it never rejects, as errors come
         back as a value.

         @param position - Position of the pull in the input.
         */
        async function fillWhenSettled(
          {
            worker,
            position,
          }: {
            /**
             The pull's worker.
             */
            readonly worker: Promise<PullOutcome<NewElement>>;
            /**
             Position of the pull in the input.
             */
            readonly position: number;
          },
        ): Promise<void> {
          /**
           Outcome the worker settled with.
           */
          const outcome = await worker;

          // The slot was already given back, as the pull came after the end of
          // the source.
          if (!unfilledIndexes.delete(position,))
            return;

          if (outcome.kind === 'done')
            // The pulls made after this one came after the end of the source,
            // and a source like a queue may never answer them, so give their
            // slots back now instead of waiting on them.
            for (const unfilledIndex of unfilledIndexes)
// mutation-test-disable-next-line equality -- the line above this loop already deleted `position` from `unfilledIndexes`, so no entry equals it
              if (unfilledIndex > position) {
                unfilledIndexes.delete(unfilledIndex,);
                fillSlot({ kind: 'done', },);
              }

          fillSlot(outcome,);
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
          if (state.isDone
// mutation-test-disable-next-line conditional -- the puller answers any pull after the source reported `done` with a done marker without touching the source, and a spawned worker that gets it ends nothing earlier, so spawning once the source is done is unobservable
            || state.isIterableDone
// mutation-test-disable-next-line equality -- `stopIndex` is the position of a pull that already started, and `index` is incremented when a pull starts, so a finite `stopIndex` is always below `index`
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
// mutation-test-disable-next-line boolean -- `readingCount` only gates `trySpawn`, and every failure closes that gate (an unordered failure before the end of the source sets `isDone`, an ordered one sets `stopIndex` below `index`, and a failure past the end of the source finds `isIterableDone` set), so a missed or doubled decrement is never read
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
                state.doneIndex = Math.min(
                  state.doneIndex,
                  currentIndex,
                );
// mutation-test-disable-next-line boolean -- `holds` is read only by the catch below, and nothing after this assignment can throw before the return
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
// mutation-test-disable-next-line boolean -- `readingCount` only gates `trySpawn`, and every failure closes that gate (an unordered failure before the end of the source sets `isDone`, an ordered one sets `stopIndex` below `index`, and a failure past the end of the source finds `isIterableDone` set), so a missed or doubled decrement is never read
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
// mutation-test-disable-next-line boolean -- `holds` is read only by the catch below, and nothing after this assignment can throw before the return
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

// mutation-test-disable-next-line boolean -- `holds` is read only by the catch below, and nothing after this assignment can throw before the return
              holds.hasMapperSlot = false;
              releaseMapperSlot();

              if (returnValue === pMapStop)
                state.stopIndex = Math.min(
                  state.stopIndex,
                  currentIndex,
                );

              return {
                kind: 'value',
                value: returnValue,
              };
            }
            catch (error) {
// mutation-test-disable-next-line conditional -- `readingCount` only gates `trySpawn`, and every failure closes that gate (an unordered failure before the end of the source sets `isDone`, an ordered one sets `stopIndex` below `index`, and a failure past the end of the source finds `isIterableDone` set), so a missed or doubled decrement is never read
              if (holds.isReading)
                state.readingCount -= 1;

              if (holds.hasMapperSlot)
                releaseMapperSlot();

              // With `preserveOrder`, the error is thrown at its own position,
              // after the results before it, so those still finish. Without
              // it, the error is thrown as soon as it settles, so nothing
              // still pending can be yielded anymore. An error from a pull
              // after the end of the source is dropped, as `done` is final,
              // so it must have no effect: `doneIndex` is set before the
              // worker that saw `done` settles, so its slot hand-back runs
              // before the delivery check of any error that sees it here.
              if (preserveOrder)
                state.stopIndex = Math.min(
                  state.stopIndex,
                  currentIndex,
                );
// mutation-test-disable-next-line equality -- a pull that failed never reported `done`, so its own position cannot equal `doneIndex`
              else if (currentIndex < state.doneIndex)
                state.isDone = true;

              return {
                kind: 'error',
                error,
              };
            }
          }

          /**
           This pull's worker, started now.
           */
          const worker = runPull();

          if (preserveOrder) {
            promises.push(worker,);
            return;
          }

          promises.push(emptySlot,);
          unfilledIndexes.add(currentIndex,);
          void fillWhenSettled({
            worker,
            position: currentIndex,
          },);
        }

        trySpawn();

        /**
         Waits for the first worker to fill the empty head slot, without
         `preserveOrder`.

         @returns The outcome that fills the head slot.
         */
        function waitForFill(): Promise<PullOutcome<NewElement>> {
          /**
           Deferred pair the next `fillSlot` resolves.
           */
          const filled = Promise.withResolvers<PullOutcome<NewElement>>();
          wakers.push(filled.resolve,);
          return filled.promise;
        }

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
           Oldest spawned pull's slot. Without `preserveOrder`, a filled slot
           holds the outcome itself and is still awaited: that takes one tick,
           like awaiting a settled promise, so the consumer, and its
           `trySpawn()`, never run earlier than with a promise in the slot.
           Taking a ready result without the tick can make the consumer pull
           before it sees a stop or the end of the source, and a one-shot
           source, like a queue, would lose that item.
           */
          const headSlot = nonNullishOrThrow(promises[state.headIndex],);
          /**
           The head's outcome; an empty head is waited on until the first
           worker fills it, and only the head is awaited so results leave the
           queue in yield order.
           */
          const outcome = await (headSlot === emptySlot
            ? waitForFill()
            : headSlot);

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
            state.fillIndex -= state.headIndex;
            state.headIndex = 0;
          }

          if (outcome.kind === 'error')
            throw outcome.error;

          if (outcome.kind === 'done') {
            // Without `preserveOrder`, the `done` can arrive while other
            // workers are still running.
            if (preserveOrder)
              return;

            continue;
          }

          // The stream ends where the stop would have been yielded: after the
          // earlier elements with `preserveOrder`, and after the results that
          // were ready before it without.
          if (outcome.value === pMapStop)
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
          yield outcome.value as Exclude<NewElement, MapControl>;
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
      generator.return = function returnAndStop(value: Parameters<typeof generatorReturn>[0],): Promise<IteratorResult<Exclude<NewElement, MapControl>, void>> {
        state.isDone = true;
        return generatorReturn(value,);
      };
      generator.throw = function throwAndStop(error: unknown,): Promise<IteratorResult<Exclude<NewElement, MapControl>, void>> {
        state.isDone = true;
        return generatorThrow(error,);
      };

      return generator;
    },
  };
}

//endregion Run
