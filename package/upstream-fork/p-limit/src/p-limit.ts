/**
 TypeScript fork of `p-limit`: run promise-returning functions with limited
 concurrency.
 
 Derived from [`p-limit`](https://github.com/sindresorhus/p-limit) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Scheduling semantics match `p-limit` 7.3.3: calls start
 asynchronously in FIFO order, at most `concurrency` run at once, and each
 call's promise settles with its function's result or failure.
 
 @module
 */

import {
  type LimitOptions,
  resolveLimitOptions,
  validateConcurrency,
} from './limit-options.ts';
import {
  createTaskQueue,
  type ScheduledCall,
} from './task-queue.ts';

//region Types

/**
 One call to schedule: the function to run and the argument tuple to spread
 into it.
 
 The `args` tuple replaces upstream `p-limit`'s `limit(fn, ...args)` rest
 parameter, which repository lint bans; it exists for the same reason as
 upstream's form, letting callers push many calls without allocating a
 closure per call.
 
 @typeParam TArgs - tuple of function argument types
 
 @typeParam TResult - function return type, awaited when thenable
 
 @example
 ```ts
 const task: LimitTask<[string], Response> = {
   fn: fetch,
   args: ['https://example.com'],
 };
 ```
 */
export type LimitTask<TArgs extends readonly unknown[], TResult> = {
  /**
   Function whose result the caller's promise settles with.
   */
  readonly fn: (...args: TArgs) => TResult | PromiseLike<TResult>;
  /**
   Arguments spread into `fn` when its call starts.
   */
  readonly args: TArgs;
};

/**
 Mapper used by {@link LimitFunction}'s `map`: receives one input and its
 position in the iterable.
 
 @typeParam TInput - iterable element type
 
 @typeParam TResult - mapper return type, awaited when thenable
 */
export type LimitMapper<TInput, TResult> = (
  input: TInput,
  index: number,
) => TResult | PromiseLike<TResult>;

/**
 Options for {@link LimitFunction}'s `map`.
 
 @typeParam TInput - iterable element type
 
 @typeParam TResult - mapper return type, awaited when thenable
 
 @example
 ```ts
 const options: LimitMapOptions<number, number> = {
   iterable: [1, 2, 3],
   mapper: async function double(value: number): Promise<number> {
     return value * 2;
   },
 };
 ```
 */
export type LimitMapOptions<TInput, TResult> = {
  /**
   Inputs to process, walked once in iteration order.
   */
  readonly iterable: Iterable<TInput>;
  /**
   Function run per input under the limiter's concurrency bound.
   */
  readonly mapper: LimitMapper<TInput, TResult>;
};

/**
 A concurrency limiter: schedules {@link LimitTask} calls, runs at most
 `concurrency` of them at once, and starts queued calls in FIFO order.
 
 @example
 ```ts
 import { pLimit, } from '\@monochromatic-dev/upstream-fork-p-limit';
 
 const limit = pLimit({ concurrency: 2, });
 const responses = await Promise.all([
   limit({ fn: fetch, args: ['/a'], }),
   limit({ fn: fetch, args: ['/b'], }),
 ]);
 ```
 */
export type LimitFunction = {
  /**
   Schedules one call and returns the promise its result settles.
   
   The call's function is never invoked synchronously: it starts from a
   microtask once the limiter admits it.
   
   @param task - Function to run and arguments to spread into it.
   
   @returns Promise settling with the function's result or failure.
   */
  <const TArgs extends readonly unknown[], TResult>(task: LimitTask<TArgs, TResult>,): Promise<TResult>;
  /**
   Number of calls whose functions have started but not settled.
   */
  readonly activeCount: number;
  /**
   Number of calls queued and not yet started.
   */
  readonly pendingCount: number;
  /**
   Get or set the concurrency bound. Raising it admits queued calls from a
   microtask; lowering it never interrupts calls already running.
   */
  concurrency: number;
  /**
   Discards queued calls that never started. Calls already running are
   untouched; with `rejectOnClear`, each discarded call's promise rejects
   with `AbortSignal.abort().reason` instead of staying pending.
   */
  readonly clearQueue: () => void;
  /**
   Processes an iterable under the concurrency bound, collecting results in
   input order.
   
   @param options - Iterable to walk and mapper to run per input.
   
   @returns Promise settling with every mapper result in input order.
   */
  readonly map: <TInput, TResult>(options: LimitMapOptions<TInput, TResult>,) => Promise<readonly TResult[]>;
};

//endregion Types

//region Limiter

/**
 Invokes one call's function asynchronously, so a synchronous throw becomes
 a rejection and the completion handling in {@link runCall} always runs
 from a suspended continuation (matching upstream `p-limit`'s slot-release
 tick).
 
 @param fn - Function to invoke.
 
 @param args - Arguments spread into `fn`.
 
 @returns Promise settling with the function's result or failure.
 */
async function startCall<TArgs extends readonly unknown[], TResult>(
  {
    fn,
    args,
  }: {
    readonly fn: (...args: TArgs) => TResult | PromiseLike<TResult>;
    readonly args: TArgs;
  },
): Promise<TResult> {
  return await fn(...args,);
}

/**
 Creates a concurrency limiter over upstream `p-limit`'s constructor shapes.
 
 @param concurrencyOrOptions - Bare concurrency number, or options carrying
 `concurrency` and optional `rejectOnClear`.
 
 @returns Callable limiter with `activeCount`, `pendingCount`, `concurrency`,
 `clearQueue`, and `map`.
 
 @throws InvalidConcurrencyError when the concurrency value is not a
 positive integer or `Number.POSITIVE_INFINITY`.
 
 @throws InvalidRejectOnClearError when `rejectOnClear` is present but not a
 boolean.
 
 @throws TypeError when the input is a non-object, non-number value that
 cannot be destructured, matching upstream `p-limit`.
 
 @example
 ```ts
 import { pLimit, } from '\@monochromatic-dev/upstream-fork-p-limit';
 
 const limit = pLimit(1,);
 const result = await limit({
   fn: async function fetchUser(id: string): Promise<string> {
     return `user-${id}`;
   },
   args: ['7'],
 });
 ```
 */
export function pLimit(concurrencyOrOptions: number | LimitOptions,): LimitFunction {
  /**
   Validated options; resolving first keeps every later branch free of input
   shape checks.
   */
  const options = resolveLimitOptions(concurrencyOrOptions,);
  /**
   FIFO queue of calls waiting for a slot.
   */
  const queue = createTaskQueue();
  /**
   Mutable limiter counters and configuration, read and written by the
   closures below.
   */
  const state = {
    /**
     Settled-free slots are re-admitted here; never exceeds `concurrency`.
     */
    activeCount: 0,
    /**
     Current concurrency bound, replaceable through the `concurrency` setter.
     */
    concurrency: options.concurrency,
    /**
     Whether `clearQueue` rejects discarded calls instead of leaving them
     pending.
     */
    rejectOnClear: options.rejectOnClear,
  };

  /**
   Admits queued calls into free slots, deferring each start to a microtask so
   a call's function is never invoked inside the `limit` call that scheduled
   it (upstream `p-limit` defers the same way).
   */
  function drainQueue(): void {
    /**
     Free slots under the current bound; `Number.POSITIVE_INFINITY` leaves
     every queued call admissible.
     */
    const capacity = state.concurrency - state.activeCount;
    // mutation-test-disable-next-line conditional, equality -- fast path only: taking zero or fewer calls admits nothing, so skipping the guard is unobservable
    if (capacity <= 0)
      return;

    /**
     Calls admitted into the freed slots, in FIFO order.
     */
    const admitted = queue.take(capacity,);
    state.activeCount += admitted.length;

    for (const call of admitted)
      queueMicrotask(function startAdmittedCall(): void {
        call.run();
      },);
  }

  /**
   Frees one slot after a call settles and admits the next queued call.
   */
  function complete(): void {
    state.activeCount -= 1;
    drainQueue();
  }

  /**
   Runs one admitted call, settles its deferred with the outcome, then frees
   the slot.
   
   @param fn - Function whose result settles the caller's promise.
   
   @param args - Arguments spread into `fn`.
   
   @param deferred - Promise held by the call's caller.
   */
  async function runCall<TArgs extends readonly unknown[], TResult>(
    {
      fn,
      args,
      deferred,
    }: {
      readonly fn: (...args: TArgs) => TResult | PromiseLike<TResult>;
      readonly args: TArgs;
      readonly deferred: PromiseWithResolvers<TResult>;
    },
  ): Promise<void> {
    try {
      /**
       Result of the call's function; awaited so a thenable return flattens
       exactly once before the caller's promise settles.
       */
      const result = await startCall({
        fn,
        args,
      },);
      deferred.resolve(result,);
    }
    catch (error) {
      // Both a synchronous throw and a rejected return land here; the failure
      // is handed to the call's promise rather than discarded.
      deferred.reject(error,);
    }
    complete();
  }

  /**
   Schedules one call: enqueues it, admits it now when a slot is free, and
   returns the promise its result settles.
   
   @param fn - Function to run once admitted.
   
   @param args - Arguments spread into `fn`.
   
   @returns Promise settling with the function's result or failure.
   
   @example
   ```ts
   await limit({
     fn: compute,
     args: [input,],
   });
   ```
   */
  function limit<const TArgs extends readonly unknown[], TResult>(
    {
      fn,
      args,
    }: LimitTask<TArgs, TResult>,
  ): Promise<TResult> {
    /**
     Promise handed back now, settled when the scheduled call runs.
     */
    const deferred = Promise.withResolvers<TResult>();

    queue.enqueue({
      run: function runQueuedCall(): void {
        void runCall({
          fn,
          args,
          deferred,
        },);
      },
      reject: function rejectQueuedCall(reason: unknown,): void {
        deferred.reject(reason,);
      },
    },);
    drainQueue();
    return deferred.promise;
  }

  /**
   Members attached exactly as upstream `p-limit` attaches them:
   non-enumerable,
   non-configurable accessors and data properties,
   so
   `Object.keys` stays empty and no member can be replaced.
   */
  Object.defineProperties(
    limit,
    {
      activeCount: {
        get: function getActiveCount(): number {
          return state.activeCount;
        },
        enumerable: false,
        configurable: false,
      },
      pendingCount: {
        get: function getPendingCount(): number {
          return queue.size;
        },
        enumerable: false,
        configurable: false,
      },
      concurrency: {
        get: function getConcurrency(): number {
          return state.concurrency;
        },
        set: function setConcurrency(newConcurrency: number): void {
          state.concurrency = validateConcurrency(newConcurrency,);
          queueMicrotask(function drainAfterConcurrencyChange(): void {
            drainQueue();
          },);
        },
        enumerable: false,
        configurable: false,
      },
      clearQueue: {
        value: function clearQueue(): void {
          /**
           Queued calls discarded before they ever started.
           */
          const discarded = queue.clear();
          if (!state.rejectOnClear)
            return;

          /**
           Abort signal whose reason every discarded call shares, matching
           upstream `p-limit`'s `AbortSignal.abort().reason` hand-off.
           */
          const abortSignal = AbortSignal.abort();
          /**
           Shared abort reason handed to every discarded call.
           */
          const abortReason: unknown = abortSignal.reason;
          for (const call of discarded)
            call.reject(abortReason,);
        },
        writable: false,
        enumerable: false,
        configurable: false,
      },
      map: {
        value: async function map<TInput, TResult>(
          {
            iterable,
            mapper,
          }: LimitMapOptions<TInput, TResult>,
        ): Promise<readonly TResult[]> {
          /**
           Result promises in input order, collected while the iterable is
           walked so a mid-iteration failure can still reach them.
           */
          const scheduled: Promise<TResult>[] = [];

          try {
            void Array.from(
              iterable,
              function scheduleInput(
                input: TInput,
                index: number,
              ): Promise<TResult> {
                /**
                 Result promise of this input's scheduled call.
                 */
                const promise = limit({
                  fn: mapper,
                  args: [
                    input,
                    index,
                  ],
                },);
                scheduled.push(promise,);
                return promise;
              },
            );
          }
          catch (error) {
            // The iterable threw mid-iteration, so nothing awaits the calls
            // already scheduled; observe them here so their failures never
            // surface as unhandled promise rejections (upstream `p-limit`
            // attaches empty handlers for the same reason).
            void Promise.allSettled(scheduled,);
            throw error;
          }

          return await Promise.all(scheduled,);
        },
        writable: false,
        enumerable: false,
        configurable: false,
      },
    },
  );

  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- `limit` is extended with activeCount, pendingCount, concurrency, clearQueue, and map properties to satisfy LimitFunction
  return limit as LimitFunction;
}

//endregion Limiter
