/**
 Mapper adapters exposing one uniform workload surface over this package's
 fork and upstream `p-map`, so the same generated workload can drive both.
 
 @module
 */

import pMapUpstream, {
  type Mapper as UpstreamMapper,
  pMapIterable as pMapIterableUpstream,
  pMapKeyed as pMapKeyedUpstream,
  pMapSettled as pMapSettledUpstream,
  pMapSettledKeyed as pMapSettledKeyedUpstream,
  pMapSkip as pMapSkipUpstream,
  pMapStop as pMapStopUpstream,
} from 'p-map';

import {
  type Mapper,
  pMap,
  pMapIterable,
  pMapKeyed,
  pMapSettled,
  pMapSettledKeyed,
  pMapSkip,
  pMapStop,
} from '@monochromatic-dev/upstream-fork-p-map/ts';

import { settleRun, } from './map-workload.ts';
import type {
  KeyedAdapterCall,
  MapAdapter,
  MapAdapterCall,
  MapAdapterOutcome,
} from './workload.ts';

//region Fork surface

/**
 Runs one concurrent map through this package's fork.
 
 @param call - Workload source, mapper, and bounds.
 
 @returns Normalized settlement of the fork's run.
 
 @example
 ```ts
 const outcome = await runFork(call,);
 ```
 */
function runFork(call: MapAdapterCall,): Promise<MapAdapterOutcome> {
  return settleRun(pMap({
    iterable: call.iterable,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's skip sentinel; the fork's Mapper narrows that sentinel to its own pMapSkip symbol
    mapper: call.mapper as Mapper<string, string>,
    options: {
      concurrency: call.concurrency,
      stopOnError: call.stopOnError,
    },
  },),);
}

/**
 Streams one map through this package's fork.
 
 @param call - Workload source, mapper, and bounds.
 
 @returns Async iterable of the fork's mapper results.
 
 @example
 ```ts
 const stream = streamFork(call,);
 ```
 */
function streamFork(call: MapAdapterCall,): AsyncIterable<string | symbol> {
  return pMapIterable({
    iterable: call.iterable,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's skip sentinel; the fork's Mapper narrows that sentinel to its own pMapSkip symbol
    mapper: call.mapper as Mapper<string, string>,
    options: {
      concurrency: call.concurrency,
      backpressure: call.backpressure,
      preserveOrder: call.preserveOrder,
    },
  },);
}

/**
 Runs one settled map through this package's fork.

 @param call - Workload source, mapper, and bounds.

 @returns Normalized settlement of the fork's run.

 @example
 ```ts
 const outcome = await runForkSettled(call,);
 ```
 */
function runForkSettled(call: MapAdapterCall,): Promise<MapAdapterOutcome> {
  return settleRun(pMapSettled({
    iterable: call.iterable,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's sentinels; the fork's Mapper narrows those to its own symbols
    mapper: call.mapper as Mapper<string, string>,
    options: { concurrency: call.concurrency, },
  },),);
}

/**
 Runs one keyed map, settled or not, through this package's fork.

 @param call - Workload input, mapper, and bounds.

 @returns Normalized settlement of the fork's run.

 @example
 ```ts
 const outcome = await runForkKeyed(call,);
 ```
 */
function runForkKeyed(call: KeyedAdapterCall,): Promise<MapAdapterOutcome> {
  if (call.settled)
    return settleRun(pMapSettledKeyed({
      input: call.input,
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's sentinels; the fork's mapper type narrows those to its own symbols
      mapper: call.mapper as never,
      options: { concurrency: call.concurrency, },
    },),);

  return settleRun(pMapKeyed({
    input: call.input,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's sentinels; the fork's mapper type narrows those to its own symbols
    mapper: call.mapper as never,
    options: {
      concurrency: call.concurrency,
      stopOnError: call.stopOnError,
    },
  },),);
}

//endregion Fork surface

//region Upstream surface

/**
 Runs one concurrent map through upstream `p-map`.
 
 @param call - Workload source, mapper, and bounds.
 
 @returns Normalized settlement of upstream's run.
 
 @example
 ```ts
 const outcome = await runUpstream(call,);
 ```
 */
function runUpstream(call: MapAdapterCall,): Promise<MapAdapterOutcome> {
  return settleRun(pMapUpstream(
    call.iterable,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's skip sentinel; upstream's Mapper narrows that sentinel to its own pMapSkip symbol
    call.mapper as UpstreamMapper<string, string>,
    {
      concurrency: call.concurrency,
      stopOnError: call.stopOnError,
    },
  ),);
}

/**
 Streams one map through upstream `p-map`.
 
 @param call - Workload source, mapper, and bounds.
 
 @returns Async iterable of upstream's mapper results.
 
 @example
 ```ts
 const stream = streamUpstream(call,);
 ```
 */
function streamUpstream(call: MapAdapterCall,): AsyncIterable<string | symbol> {
  return pMapIterableUpstream(
    call.iterable,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's skip sentinel; upstream's Mapper narrows that sentinel to its own pMapSkip symbol
    call.mapper as UpstreamMapper<string, string>,
    {
      concurrency: call.concurrency,
      backpressure: call.backpressure,
      preserveOrder: call.preserveOrder,
    },
  );
}

/**
 Runs one settled map through upstream `p-map`.

 @param call - Workload source, mapper, and bounds.

 @returns Normalized settlement of upstream's run.

 @example
 ```ts
 const outcome = await runUpstreamSettled(call,);
 ```
 */
function runUpstreamSettled(call: MapAdapterCall,): Promise<MapAdapterOutcome> {
  return settleRun(pMapSettledUpstream(
    call.iterable,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's sentinels; upstream's Mapper narrows those to its own symbols
    call.mapper as UpstreamMapper<string, string>,
    { concurrency: call.concurrency, },
  ),);
}

/**
 Runs one keyed map, settled or not, through upstream `p-map`.

 @param call - Workload input, mapper, and bounds.

 @returns Normalized settlement of upstream's run.

 @example
 ```ts
 const outcome = await runUpstreamKeyed(call,);
 ```
 */
function runUpstreamKeyed(call: KeyedAdapterCall,): Promise<MapAdapterOutcome> {
  if (call.settled)
    return settleRun(pMapSettledKeyedUpstream(
      call.input,
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's sentinels; upstream's mapper type narrows those to its own symbols
      call.mapper as never,
      { concurrency: call.concurrency, },
    ),);

  return settleRun(pMapKeyedUpstream(
    call.input,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the workload mapper returns strings plus the adapter's sentinels; upstream's mapper type narrows those to its own symbols
    call.mapper as never,
    {
      concurrency: call.concurrency,
      stopOnError: call.stopOnError,
    },
  ),);
}

//endregion Upstream surface

//region Adapters

/**
 Adapts this package's fork to the workload surface.
 
 @returns Adapter driving the fork's `pMap` and `pMapIterable`.
 
 @example
 ```ts
 const adapter = createForkAdapter();
 ```
 */
export function createForkAdapter(): MapAdapter {
  return {
    skip: pMapSkip,
    stop: pMapStop,
    run: runFork,
    runSettled: runForkSettled,
    runKeyed: runForkKeyed,
    stream: streamFork,
  };
}

/**
 Adapts upstream `p-map` to the workload surface.
 
 @returns Adapter driving upstream `pMap` and `pMapIterable`.
 
 @example
 ```ts
 const adapter = createUpstreamAdapter();
 ```
 */
export function createUpstreamAdapter(): MapAdapter {
  return {
    skip: pMapSkipUpstream,
    stop: pMapStopUpstream,
    run: runUpstream,
    runSettled: runUpstreamSettled,
    runKeyed: runUpstreamKeyed,
    stream: streamUpstream,
  };
}

//endregion Adapters
