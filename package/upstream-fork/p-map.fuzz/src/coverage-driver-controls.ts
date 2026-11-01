/**
 Deterministic coverage driver for the stop sentinel, the settled map, the
 keyed variants, and the unordered stream. Called from the main driver, so
 its V8 coverage lands in the same run.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  pMap,
  pMapIterable,
  pMapKeyed,
  pMapSettled,
  pMapSettledKeyed,
  pMapSkip,
  pMapStop,
} from '@monochromatic-dev/upstream-fork-p-map/ts';

import {
  type CloseState,
  closeCounting,
  droppingMapper,
  failingMapper,
  identityMapper,
  MAP_INPUTS,
  promisedMapper,
  stoppingMapper,
  swallowAsync,
  twoAsyncValues,
} from './coverage-fixtures.ts';

//region Exercise

/**
 Number of settled results the driver's settled run produces.
 */
const SETTLED_COUNT = 3;

/**
 Mapper throwing for the value two.

 @param value - Input under test.

 @returns The input unchanged.

 @throws Error for the value two.

 @example
 ```ts
 settleMapper(1,); // => 1
 ```
 */
function settleMapper(value: number,): number {
  if (value === 2)
    throw new Error('mapper failed',);
  return value;
}

/**
 Builds the keyed mapper that drops the key `'skipped'` and ends at the key
 `'stopped'`.

 @returns Keyed mapper returning the value, `pMapSkip`, or `pMapStop`.

 @example
 ```ts
 const mapper = createKeyedControlMapper();
 ```
 */
function createKeyedControlMapper(): (
  value: number,
  key: string,
) => number | typeof pMapSkip | typeof pMapStop {
  return function keyedControl(
    value: number,
    key: string,
  ): number | typeof pMapSkip | typeof pMapStop {
    if (key === 'skipped')
      return pMapSkip;
    if (key === 'stopped')
      return pMapStop;
    return value;
  };
}

/**
 Exercises the stop sentinel in the concurrent and streaming maps, the
 settled map, both keyed maps over an object and a `Map`, and the unordered
 stream with skips, stops, failures, and an early exit.

 @example
 ```ts
 await exerciseControls();
 ```
 */
export async function exerciseControls(): Promise<void> {
  /**
   Keyed mapper shared by the keyed runs.
   */
  const keyedControl = createKeyedControlMapper();

  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime inputs mirror a mis-typed caller */


  /**
   Results before the stop, from a sync source that is closed by the stop.
   */
  const stopped = await pMap({
    iterable: MAP_INPUTS,
    mapper: stoppingMapper,
    options: { concurrency: 1, },
  },);
  if ((stopped.length !== 1) || (stopped[0] !== 0))
    throw new Error('coverage driver stop diverged; driver inputs are stale',);

  /**
   Close counter for the stopped run's endless source.
   */
  const stopCloseState: CloseState = {
    closes: 0,
  };
  await pMap({
    iterable: closeCounting({
      closeState: stopCloseState,
    },),
    mapper: stoppingMapper,
    options: { concurrency: 1, },
  },);
  await wait(
    0,
  );
  if (stopCloseState.closes < 1)
    throw new Error('coverage driver stop close diverged; driver inputs are stale',);

  /**
   Settled results: fulfilled, rejected by the mapper, and rejected by the
   element itself.
   */
  const settled = await pMapSettled({
    iterable: [
      1,
      2,
      Promise.reject(new Error('element failed',),),
    ],
    mapper: settleMapper,
  },);
  if (settled.length !== SETTLED_COUNT)
    throw new Error('coverage driver settled diverged; driver inputs are stale',);
  await pMapSettled({
    iterable: MAP_INPUTS,
    mapper: droppingMapper,
    options: { concurrency: 2, },
  },);
  await pMapSettled({
    iterable: MAP_INPUTS,
    mapper: stoppingMapper,
  },);
  await swallowAsync(async function invalidSettledInput(): Promise<void> {
    await pMapSettled({
      iterable: null as never,
      mapper: identityMapper,
    },);
  },);

  /**
   Keyed results over an object with a skipped and a stopped key.
   */
  const keyed = await pMapKeyed({
    input: {
      first: 1,
      skipped: 2,
      stopped: 3,
      last: 4,
    },
    mapper: keyedControl,
    options: { concurrency: 1, },
  },);
  /**
   Keys the keyed run kept.
   */
  const keptKeys = Object.keys(keyed,);
  if (keptKeys.length !== 1)
    throw new Error('coverage driver keyed diverged; driver inputs are stale',);
  await pMapKeyed({
    input: new Map<string, number | Promise<number>>([
      [
        'first',
        1,
      ],
      [
        'skipped',
        Promise.resolve(2,),
      ],
    ],),
    mapper: keyedControl,
  },);
  await swallowAsync(async function rejectingKeyedValue(): Promise<void> {
    await pMapKeyed({
      input: { first: Promise.reject(new Error('value failed',),), },
      mapper: identityMapper,
    },);
  },);
  await swallowAsync(async function invalidKeyedInput(): Promise<void> {
    await pMapKeyed({
      input: null as never,
      mapper: identityMapper,
    },);
  },);
  await swallowAsync(async function invalidKeyedMapper(): Promise<void> {
    await pMapKeyed({
      input: { first: 1, },
      mapper: undefined as never,
    },);
  },);
  await pMapSettledKeyed({
    input: {
      first: 1,
      second: Promise.reject(new Error('value failed',),),
      skipped: 3,
    },
    mapper: keyedControl,
  },);
  await pMapSettledKeyed({
    input: new Map([
      [
        'first',
        1,
      ],
    ],),
    mapper: failingMapper,
  },);

  /**
   Unordered stream: settle order, a skip, a stop, and a failure.
   */
  const unordered: number[] = [];
  for await (const value of pMapIterable({
    iterable: MAP_INPUTS,
    mapper: droppingMapper,
    options: {
      concurrency: 2,
      preserveOrder: false,
    },
  }))
    unordered.push(value,);
  if (unordered.length !== 2)
    throw new Error('coverage driver unordered diverged; driver inputs are stale',);
  for await (const value of pMapIterable({
    iterable: MAP_INPUTS,
    mapper: stoppingMapper,
    options: { preserveOrder: false, },
  }))
    void value;
  for await (const value of pMapIterable({
    iterable: MAP_INPUTS,
    mapper: stoppingMapper,
    options: { concurrency: 1, },
  }))
    void value;
  for await (const value of pMapIterable({
    iterable: twoAsyncValues(),
    mapper: promisedMapper,
    options: {
      concurrency: Number.POSITIVE_INFINITY,
      preserveOrder: false,
    },
  }))
    void value;
  await swallowAsync(async function failingUnorderedStream(): Promise<void> {
    for await (const value of pMapIterable({
      iterable: [
        1,
        2,
      ],
      mapper: failingMapper,
      options: { preserveOrder: false, },
    }))
      void value;
  },);
  /**
   Close counter for the unordered stream's early exit.
   */
  const unorderedCloseState: CloseState = {
    closes: 0,
  };
  for await (const value of pMapIterable({
    iterable: closeCounting({
      closeState: unorderedCloseState,
    },),
    mapper: identityMapper,
    options: {
      concurrency: 1,
      backpressure: 2,
      preserveOrder: false,
    },
  })) {
    void value;
    break;
  }
  await wait(
    0,
  );


  /* oxlint-enable typescript/no-unsafe-type-assertion */
}

//endregion Exercise
