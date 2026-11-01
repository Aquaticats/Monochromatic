/**
 Deterministic coverage driver: exercises every exported function and its
 error paths with fixed inputs, so the V8 coverage it produces is
 reproducible. Run under `NODE_V8_COVERAGE` by the `fuzz:coverage` task,
 then summarized by `coverage-report.ts`.
 
 @module
 */

import {
  limitFunction,
  pLimit,
} from '@monochromatic-dev/upstream-fork-p-limit/ts';

//region Fixtures

/**
 Result the synchronous answer call must resolve with.
 */
const ANSWER_VALUE = 42;

/**
 Number of inputs the fixed mapper fixture walks.
 */
const MAP_INPUT_COUNT = 3;

/**
 Number of depth calls exercising queue compaction over a consumed prefix.
 */
const DEPTH_CALL_COUNT = 12;

/**
 Failure thrown by the synchronous-throw call fixture.
 */
const THROWN_FAILURE = new Error('thrown',);

/**
 Failure rejected by the rejecting-call and mapper-failure fixtures.
 */
const BOOM_FAILURE = new Error('boom',);

/**
 Failure thrown by the failing-iterable fixture.
 */
const ITERABLE_FAILURE = new Error('iterable failed',);

/**
 Fixed input list the ordered-mapper fixture walks.
 */
const MAP_INPUTS: readonly number[] = [
  0,
  1,
  2,
].map(function plusOne(value: number,): number {
  return value + 1;
},);

/**
 Synchronous answer call resolving with {@link ANSWER_VALUE}.
 
 @returns {@link ANSWER_VALUE}.
 */
function answerCall(): number {
  return ANSWER_VALUE;
}

/**
 Rejecting call: returns a rejected promise without becoming `async`, so the
 rejection path is driven without a `require-await` exemption.
 
 The returned promise is already rejected with {@link BOOM_FAILURE}.
 */
function rejectingCall(): Promise<never> {
  return Promise.reject(BOOM_FAILURE,);
}

/**
 Synchronous-throwing call: throws from a plain function body.
 
 @throws {@link THROWN_FAILURE} on every call.
 */
function throwingCall(): never {
  throw THROWN_FAILURE;
}

/**
 Uncapped-limiter call resolving a fixed string.
 
 @returns Fixed string identifying the fixture.
 */
function uncappedCall(): string {
  return 'uncapped';
}

/**
 Mapper doubling its input.
 
 @param value - Input to double.
 
 @returns Doubled input.
 */
function doubled(value: number,): number {
  return value * 2;
}

/**
 Mapper that must never run for the empty-iterable fixture.
 
 @param value - Input handed back unchanged.
 
 @returns Same value it was called with.
 */
function unusedMapper(value: unknown,): unknown {
  return value;
}

/**
 Mapper failing on every input with a message naming its input.
 
 @param value - Input named in the failure message.
 
 @throws Error naming the input, on every call.
 */
function failingMapperEntry(value: number,): never {
  throw new Error(`mapper failed ${String(value,)}`,);
}

/**
 Mapper returning its input unchanged.
 
 @param value - Input handed back unchanged.
 
 @returns Same value it was called with.
 */
function survivingMapper(value: number,): number {
  return value;
}

/**
 Iterable yielding one value then throwing {@link ITERABLE_FAILURE}.
 
 @returns Nothing; the generator always throws after its first yield.
 
 @yields Exactly one value before throwing.
 
 @throws {@link ITERABLE_FAILURE} after the first yield.
 */
function* oneThenThrow(): Generator<number> {
  yield 1;
  throw ITERABLE_FAILURE;
}

/**
 Call fixture dropped from the queue before it ever starts.
 
 @returns Fixed string identifying the fixture.
 */
function droppedCall(): string {
  return 'dropped';
}

/**
 Wrapped function joining its two destructured parts.
 
 @param left - Left part of the joined result.
 
 @param right - Right part of the joined result.
 
 @returns Both parts concatenated.
 */
function join(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): string {
  return left + right;
}

//endregion Fixtures

//region Helpers

/**
 Runs a thunk that is expected to throw, swallowing `Error` outcomes so the
 driver keeps exercising remaining paths. Re-throws anything that is not an
 `Error`.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad() {
   pLimit(0,);
 });
 ```
 */
function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    if (!(Error.isError(error,)))
      throw error;
  }
}

/**
 Awaits a thunk that is expected to reject, swallowing `Error` outcomes so
 the driver keeps exercising remaining paths. Re-throws anything that is not
 an `Error`.
 
 @param thunk - Async operation expected to reject.
 
 @example
 ```ts
 await swallowAsync(async function bad(): Promise<void> {
   await Promise.reject(new Error('boom',),);
 });
 ```
 */
async function swallowAsync(thunk: () => Promise<void>,): Promise<void> {
  try {
    await thunk();
  }
  catch (error: unknown) {
    if (!(Error.isError(error,)))
      throw error;
  }
}

/**
 Drives one drop-pending call: schedules a blocking call on a fresh limiter,
 queues a second call, then clears the queue in the requested mode.
 
 @param rejectOnClear - Whether the clear rejects the dropped call.
 
 @example
 ```ts
 await runClearable({ rejectOnClear: true, });
 ```
 */
async function runClearable({ rejectOnClear, }: {
  readonly rejectOnClear: boolean;
},): Promise<void> {
  /**
   Fresh limiter holding exactly one slot.
   */
  const limit = pLimit({
    concurrency: 1,
    rejectOnClear,
  },);
  /**
   Gate holding the single slot until the queue is cleared.
   */
  const blocker = Promise.withResolvers<void>();

  void limit({
    fn: async function blockingCall(): Promise<string> {
      await blocker.promise;
      return 'blocker';
    },
    args: [],
  },);
  /**
   Queued call dropped by the clear below.
   */
  const dropped = limit({
    fn: droppedCall,
    args: [],
  },);

  limit.clearQueue();
  blocker.resolve();

  if (rejectOnClear)
    await swallowAsync(async function observeDropped(): Promise<void> {
      await dropped;
    },);
}

//endregion Helpers

//region Exercise

/**
 Exercises option resolution, scheduling, counters, accessor mutation,
 `clearQueue` in both modes, `map`'s success and failure paths, queue
 compaction under depth, and `limitFunction` end to end.
 
 @example
 ```ts
 await exercise();
 ```
 */
async function exercise(): Promise<void> {
  //region Option resolution

  swallow(function invalidConcurrency() {
    pLimit(0,);
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime options mirror a mis-typed caller */
  /**
   Options carrying a non-boolean `rejectOnClear`, fed past the static type.
   */
  const invalidRejectOnClearOptions = {
    concurrency: 1,
    rejectOnClear: 'yes',
  } as never;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  swallow(function invalidRejectOnClear() {
    pLimit(invalidRejectOnClearOptions,);
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime options mirror a mis-typed caller */
  /**
   Options invalid in both fields, checking upstream's error precedence.
   */
  const invalidPrecedenceOptions = {
    concurrency: 0,
    rejectOnClear: 'yes',
  } as never;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  swallow(function invalidPrecedence() {
    pLimit(invalidPrecedenceOptions,);
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- a non-object constructor input mirrors a mis-typed caller */
  /**
   Non-object constructor input, driving the destructuring failure path.
   */
  const nonObjectInput = null as never;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  swallow(function destructuringFailure() {
    pLimit(nonObjectInput,);
  },);

  //endregion Option resolution

  //region Scheduling

  /**
   Two-slot limiter with reject-on-clear enabled, driving the busy paths.
   */
  const limit = pLimit({
    concurrency: 2,
    rejectOnClear: true,
  },);
  /**
   Uncapped limiter, driving the `Number.POSITIVE_INFINITY` path.
   */
  const uncapped = pLimit(Number.POSITIVE_INFINITY,);

  /**
   Result of a successful call.
   */
  const resolved = await limit({
    fn: answerCall,
    args: [],
  },);
  if (resolved !== ANSWER_VALUE)
    throw new Error('coverage driver resolved value diverged; driver inputs are stale',);

  await swallowAsync(async function rejectedCall(): Promise<void> {
    await limit({
      fn: rejectingCall,
      args: [],
    },);
  },);
  await swallowAsync(async function syncThrowingCall(): Promise<void> {
    await limit({
      fn: throwingCall,
      args: [],
    },);
  },);

  /**
   Depth calls exercising queue compaction over a consumed prefix.
   */
  const depthCalls = Array.from(
    {
      length: DEPTH_CALL_COUNT,
    },
    function scheduleDepth(
      _value: number,
      index: number,
    ): Promise<string> {
      return limit({
        fn: function depthCall(): string {
          return `depth-${String(index,)}`;
        },
        args: [],
      },);
    },
  );
  await Promise.all(depthCalls,);

  /**
   Successful call through the uncapped limiter.
   */
  await uncapped({
    fn: uncappedCall,
    args: [],
  },);

  //endregion Scheduling

  //region Accessors

  limit.concurrency = 4;
  limit.concurrency = 1;
  swallow(function invalidSetter() {
    limit.concurrency = 0;
  },);
  /**
   Counter reads driving the accessor getters.
   */
  const counters = {
    active: limit.activeCount,
    pending: limit.pendingCount,
    bound: limit.concurrency,
  };
  if ((counters.active < 0) || (counters.pending < 0))
    throw new Error('coverage driver counters diverged; driver inputs are stale',);

  //endregion Accessors

  //region clearQueue

  await runClearable({ rejectOnClear: false, },);
  await runClearable({ rejectOnClear: true, },);

  //endregion clearQueue

  //region map

  /**
   Ordered mapper results over a fixed input list.
   */
  const mapped = await limit.map({
    iterable: MAP_INPUTS,
    mapper: doubled,
  },);
  if (mapped.length !== MAP_INPUT_COUNT)
    throw new Error('coverage driver map diverged; driver inputs are stale',);

  await limit.map({
    iterable: [],
    mapper: unusedMapper,
  },);

  await swallowAsync(async function failingMapper(): Promise<void> {
    await limit.map({
      iterable: [
        1,
        2,
      ],
      mapper: failingMapperEntry,
    },);
  },);

  await swallowAsync(async function failingIterable(): Promise<void> {
    await limit.map({
      iterable: oneThenThrow(),
      mapper: survivingMapper,
    },);
  },);

  //endregion map

  //region limitFunction

  /**
   Wrapped function driven through success and clearQueue.
   */
  const limited = limitFunction({
    fn: join,
    options: {
      concurrency: 1,
    },
  },);
  /**
   Joined result through the wrapped function.
   */
  const joined = await limited({
    args: [
      {
        left: 'a',
        right: 'b',
      },
    ],
  },);
  if (joined !== 'ab')
    throw new Error('coverage driver limited call diverged; driver inputs are stale',);
  void limited({
    args: [
      {
        left: 'queued',
        right: 'dropped',
      },
    ],
  },);
  limited.clearQueue();

  //endregion limitFunction
}

//endregion Exercise

await exercise();
