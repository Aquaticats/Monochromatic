/**
 TypeScript fork of `p-map`'s streaming variant: map over iterables with
 limited concurrency and backpressure, yielding results in order.
 
 Derived from [`p-map`](https://github.com/sindresorhus/p-map) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Streaming semantics match `p-map` 7.0.8: mapper calls stay
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
    async *[Symbol.asyncIterator](): AsyncGenerator<Exclude<NewElement, typeof pMapSkip>, void, undefined> {
      /**
       Source iterator, created once per iteration like upstream `p-map`.
       */
      const iterator: SourceIterator<Element> = selectIterator(iterable,);
      /**
       Spawned pulls awaiting collection, in yield order.
       */
      const promises: Promise<PullOutcome<NewElement>>[] = [];
      /**
       Mutable iteration state, kept in one record because function-root
       `let` bindings are banned by repository lint.
       */
      const state = {
        /**
         Pulls spawned and not yet finished, bounded by `concurrency`.
         */
        pendingPromisesCount: 0,
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
         Position handed to the next element's mapper call.
         */
        index: 0,
      };

      /**
       Spawns one pull when the concurrency and backpressure bounds allow it,
       exactly where upstream `p-map` re-spawns: at the start,
       after each
       source pull,
       after each mapper completion,
       and after each collection.
       */
      function trySpawn(): void {
        /**
         Whether iteration stopped or the source reported `done`.
         */
// mutation-test-disable-next-line conditional, logical -- spawn-gate control only: trySpawn after stop is reached solely from pending continuations whose outcomes are dropped
        const stopped = state.isDone || state.isIterableDone;
        /**
         Whether both bounds still allow a spawn, matching upstream `p-map`'s
         gate exactly.
         */
        const withinBounds = (state.pendingPromisesCount < concurrency)
          && (promises.length < backpressure);
        if (stopped || (!withinBounds))
          return;

        state.pendingPromisesCount += 1;

        /**
         Deferred outcome of this pull; its promise exists before the pull
         body starts, so the body can locate itself in the spawn queue
         without capturing its own binding.
         */
        const pull = Promise.withResolvers<PullOutcome<NewElement>>();

        /**
         Awaits this pull's source item,
         runs its mapper call,
         then settles
         the deferred at the points upstream `p-map`'s spawned continuation
         returns its result forms.
         
         @throws Nothing; failures settle the deferred with the error outcome
         instead, so a promise the consumer abandons after an earlier error
         never becomes an unhandled rejection.
         */
        async function runPull(): Promise<void> {
          try {
            /**
             Next source item, or its `done` report.
             */
            const nextItem: IteratorResult<Element | Promise<Element>, unknown> = await iterator.next();

            // oxlint-disable-next-line typescript/strict-boolean-expressions -- upstream `p-map` checks `if (nextItem.done)` with truthiness, so a non-conforming source's truthy `done` must terminate the pull exactly like upstream
            if (nextItem.done) {
              state.isIterableDone = true;
              state.pendingPromisesCount -= 1;
              pull.resolve({
                kind: 'done',
              },);
              return;
            }

            // Spawn if still below concurrency and backpressure limit.
            trySpawn();

            /**
             Position of this element in the input, assigned before the
             element's await like upstream `p-map`.
             */
            const currentIndex = state.index;
            state.index += 1;

            /**
             Element after awaiting the source's yielded value.
             */
            const element = await nextItem.value;

            // The consumer stopped iterating or an earlier mapper failed
            // while this input was pending, so drop it instead of doing work
            // nobody will consume.
// mutation-test-disable-next-line conditional, block -- dropped-work path only: the consumer already left the iteration, so whether the pending mapper runs changes no yielded value, error, or close
            if (state.isDone) {
              state.pendingPromisesCount -= 1;
              pull.resolve({
                kind: 'value',
                value: pMapSkip,
              },);
              return;
            }

            /**
             Mapper result for this element, awaited when thenable.
             */
            const returnValue = await mapper(
              element,
              currentIndex,
            );

            state.pendingPromisesCount -= 1;

// mutation-test-disable-next-line conditional, block -- backpressure-capacity bookkeeping only: the skip outcome is dropped at the consumer either way, with identical yields and errors
            if (returnValue === pMapSkip) {
              /**
               Own position in the spawn queue, matching upstream `p-map`'s
               self-removal: a skipped pull leaves the queue so the consumer
               never collects it.
               */
              const spawnedPosition = promises.indexOf(pull.promise,);
// mutation-test-disable-next-line conditional -- backpressure-capacity bookkeeping only: the skip outcome is dropped at the consumer either way, with identical yields and errors
              if (spawnedPosition > 0)
                void promises.splice(
                  spawnedPosition,
                  1,
                );
            }

            // Spawn if still below backpressure limit and just dropped below
            // the concurrency limit.
            trySpawn();

            pull.resolve({
              kind: 'value',
              value: returnValue,
            },);
          }
          catch (error) {
            state.pendingPromisesCount -= 1;
// mutation-test-disable-next-line boolean -- error-path stop flag only: the failing outcome is already queued and the consumer throws it regardless
            state.isDone = true;
            // Failures settle as a value instead of rejecting: a promise the
            // consumer abandons after an earlier error would otherwise become
            // an unhandled rejection.
            pull.resolve({
              kind: 'error',
              error,
            },);
          }
        }

        void runPull();
        promises.push(pull.promise,);
      }

      trySpawn();

      /**
       Source shutdown on every exit from the consumption below: the consumer
       stopping early must end pending pulls and close the source exactly the
       way upstream `p-map`'s `finally` block does. The disposer detaches the
       close like upstream, so a source blocked inside `next()` can never
       block the consumer's exit.
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
      while (promises.length > 0) {
        /**
         Oldest spawned pull's outcome; only the head is awaited so results
         leave the queue in yield order.
         */
        const outcome = await nonNullishOrThrow(promises[0],);
        void promises.shift();

        if (outcome.kind === 'error')
          throw outcome.error;

        if (outcome.kind === 'done')
          return;

        // Spawn if just dropped below backpressure limit and below the
        // concurrency limit.
        trySpawn();

        if (outcome.value === pMapSkip)
          continue;

        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the skip check above proved the value is not pMapSkip
        yield outcome.value as Exclude<NewElement, typeof pMapSkip>;
      }
      /* oxlint-enable no-await-in-loop */
    },
  };
}

//endregion Run
