/**
 Deterministic coverage driver: exercises every exported function and its
 error paths with fixed inputs, so the V8 coverage it produces is
 reproducible. Run under `NODE_V8_COVERAGE` by the `fuzz:coverage` task,
 then summarized by `coverage-report.ts`.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  pMap,
  pMapIterable,
  pMapSkip,
  pMapStop,
} from '@monochromatic-dev/upstream-fork-p-map/ts';

import { exerciseControls, } from './coverage-driver-controls.ts';

import {
  bothSlots,
  type CloseState,
  closeCounting,
  droppingMapper,
  failingMapper,
  identityMapper,
  MAP_INPUTS,
  oneThenThrow,
  promisedMapper,
  swallow,
  swallowAsync,
  throwingMapper,
  twoAsyncValues,
} from './coverage-fixtures.ts';

//region Exercise

/**
 Exercises option resolution and validation,
 scheduling over sync,
 async,
 and
 promise-element sources,
 skip staging,
 both failure modes,
 abort paths,
 iterator
 selection and shutdown,
 and the streaming variant's bounds,
 skips,
 failures,
 and
 early exit.
 
 @example
 ```ts
 await exercise();
 ```
 */
async function exercise(): Promise<void> {
  //region Option resolution

  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime options and inputs mirror a mis-typed caller */

  await swallowAsync(async function invalidConcurrencyRejected(): Promise<void> {
    await pMap({
      iterable: [1],
      mapper: identityMapper,
      options: { concurrency: 0, },
    },);
  },);
  await swallowAsync(async function invalidInput(): Promise<void> {
    await pMap({
      iterable: true as never,
      mapper: identityMapper,
    },);
  },);
  await swallowAsync(async function nullInputProbe(): Promise<void> {
    await pMap({
      iterable: null as never,
      mapper: identityMapper,
    },);
  },);
  await swallowAsync(async function missingMapper(): Promise<void> {
    await pMap({
      iterable: [1],
      mapper: undefined as never,
    },);
  },);
  await swallowAsync(async function nonObjectOptions(): Promise<void> {
    await pMap({
      iterable: [1],
      mapper: identityMapper,
      options: null as never,
    },);
  },);
  await swallowAsync(async function symbolBound(): Promise<void> {
    await pMap({
      iterable: [1],
      mapper: identityMapper,
      options: {
        concurrency: Symbol('symbolic concurrency bound value',) as never,
      },
    },);
  },);
  swallow(function invalidBackpressure() {
    pMapIterable({
      iterable: [1],
      mapper: identityMapper,
      options: {
        concurrency: 2,
        backpressure: 1,
      },
    },);
  },);
  swallow(function invalidStreamingOptions() {
    pMapIterable({
      iterable: [1],
      mapper: identityMapper,
      options: null as never,
    },);
  },);
  swallow(function invalidStreamingConcurrency() {
    pMapIterable({
      iterable: [1],
      mapper: identityMapper,
      options: { concurrency: 0, },
    },);
  },);
  swallow(function invalidStreamingInput() {
    pMapIterable({
      iterable: true as never,
      mapper: identityMapper,
    },);
  },);
  swallow(function invalidStreamingMapper() {
    pMapIterable({
      iterable: [1],
      mapper: true as never,
    },);
  },);

  /* oxlint-enable typescript/no-unsafe-type-assertion */

  //endregion Option resolution

  //region Concurrent map

  /**
   Ordered results over a fixed input list.
   */
  const ordered = await pMap({
    iterable: MAP_INPUTS,
    mapper: identityMapper,
    options: {
      concurrency: 2,
      stopOnError: true,
    },
  },);
  if ((ordered.length !== MAP_INPUTS.length) || (ordered[0] !== 0))
    throw new Error('coverage driver map diverged; driver inputs are stale',);

  await pMap({
    iterable: MAP_INPUTS,
    mapper: promisedMapper,
  },);
  await pMap({
    iterable: twoAsyncValues(),
    mapper: identityMapper,
    options: { concurrency: Number.POSITIVE_INFINITY, },
  },);
  await pMap({
    iterable: bothSlots(),
    mapper: identityMapper,
  },);
  /**
   Skip-staged results: the middle input is dropped.
   */
  const skipped = await pMap({
    iterable: MAP_INPUTS,
    mapper: droppingMapper,
  },);
  if ((skipped.length !== 2) || (skipped[0] !== 0))
    throw new Error('coverage driver skip diverged; driver inputs are stale',);
  await pMap({
    iterable: [1],
    mapper: droppingMapper,
  },);

  await swallowAsync(async function rejectingMapper(): Promise<void> {
    await pMap({
      iterable: [
        1,
        2,
      ],
      mapper: failingMapper,
    },);
  },);
  await swallowAsync(async function throwingMapperRun(): Promise<void> {
    await pMap({
      iterable: [1],
      mapper: throwingMapper,
    },);
  },);
  await swallowAsync(async function aggregatedFailures(): Promise<void> {
    await pMap({
      iterable: [
        1,
        2,
      ],
      mapper: failingMapper,
      options: {
        concurrency: 1,
        stopOnError: false,
      },
    },);
  },);
  await swallowAsync(async function failingIterable(): Promise<void> {
    await pMap({
      iterable: oneThenThrow(),
      mapper: identityMapper,
      options: { stopOnError: false, },
    },);
  },);

  //endregion Concurrent map

  //region Abort signal

  /**
   Controller whose signal aborts before the run starts.
   */
  const earlyAbort = new AbortController();
  earlyAbort.abort('stopped early',);
  await swallowAsync(async function abortedBeforeStart(): Promise<void> {
    await pMap({
      iterable: [
        1,
        2,
      ],
      mapper: identityMapper,
      options: {
        signal: earlyAbort.signal,
      },
    },);
  },);

  /**
   Controller whose signal aborts while the mapper is gated.
   */
  const midAbort = new AbortController();
  /**
   Deferred holding the mapper until the abort fires.
   */
  const gate = Promise.withResolvers<void>();
  /**
   Close counter for the aborted run's source.
   */
  const abortCloseState: CloseState = {
    closes: 0,
  };
  /**
   Run under abort observation.
   */
  const aborting = pMap({
    iterable: closeCounting({
      closeState: abortCloseState,
    },),
    mapper: async function gatedMapper(value: number,): Promise<number> {
      await gate.promise;
      return value;
    },
    options: {
      concurrency: 1,
      signal: midAbort.signal,
    },
  },);
  midAbort.abort();
  gate.resolve();
  await swallowAsync(async function abortedMidRun(): Promise<void> {
    await aborting;
  },);
  // The close runs detached, exactly like upstream `p-map`; give it a turn
  // before observing the counter.
  await wait(
    0,
  );
  if (abortCloseState.closes < 1)
    throw new Error('coverage driver abort close diverged; driver inputs are stale',);

  //endregion Abort signal

  //region Streaming map

  /**
   Streamed values over a fixed input list.
   */
  const streamed: number[] = [];
  for await (const value of pMapIterable({
    iterable: MAP_INPUTS,
    mapper: identityMapper,
    options: {
      concurrency: 2,
      backpressure: 2,
    },
  }))
    streamed.push(value,);
  if (streamed.length !== MAP_INPUTS.length)
    throw new Error('coverage driver stream diverged; driver inputs are stale',);

  /**
   Streamed values with the middle input dropped.
   */
  const streamedSkips: number[] = [];
  for await (const value of pMapIterable({
    iterable: MAP_INPUTS,
    mapper: droppingMapper,
    options: {
      concurrency: 1,
      backpressure: 1,
    },
  }))
    streamedSkips.push(value,);
  if ((streamedSkips.length !== 2) || (streamedSkips[0] !== 0))
    throw new Error('coverage driver stream skips diverged; driver inputs are stale',);

  await swallowAsync(async function failingStream(): Promise<void> {
    for await (const value of pMapIterable({
      iterable: [
        1,
        2,
      ],
      mapper: failingMapper,
      options: {
        concurrency: 2,
        backpressure: 2,
      },
    }))
      void value;
  },);

  /**
   Close counter for the early-exit stream's source.
   */
  const breakCloseState: CloseState = {
    closes: 0,
  };
  for await (const value of pMapIterable({
    iterable: closeCounting({
      closeState: breakCloseState,
    },),
    mapper: identityMapper,
    options: {
      concurrency: 1,
      backpressure: 1,
    },
  })) {
    void value;
    break;
  }
  // The close runs detached, exactly like upstream `p-map`; give it a turn
  // before observing the counter.
  await wait(
    0,
  );
  if (breakCloseState.closes < 1)
    throw new Error('coverage driver stream close diverged; driver inputs are stale',);

  /**
   Stream over an asynchronous source, driven fully.
   */
  const asyncStreamed: number[] = [];
  for await (const value of pMapIterable({
    iterable: twoAsyncValues(),
    mapper: promisedMapper,
    options: { concurrency: Number.POSITIVE_INFINITY, },
  }))
    asyncStreamed.push(value,);
  if (asyncStreamed.length !== 2)
    throw new Error('coverage driver async stream diverged; driver inputs are stale',);

  //endregion Streaming map

  await exerciseControls();

  /**
   Sentinel identity check keeping {@link pMapSkip} and {@link pMapStop}
   referenced.
   */
  const skipHeld = pMapSkip;
  /**
   Stop sentinel held alongside the skip sentinel.
   */
  const stopHeld = pMapStop;
  if (((typeof skipHeld) !== 'symbol') || ((typeof stopHeld) !== 'symbol'))
    throw new Error('coverage driver skip sentinel diverged; driver inputs are stale',);
}

//endregion Exercise

await exercise();
