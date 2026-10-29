/**
 Mapper adapters exposing one uniform workload surface over this package's
 fork and upstream `p-map`, so the same generated workload can drive both.
 
 @module
 */

import pMapUpstream, {
  type Mapper as UpstreamMapper,
  pMapIterable as pMapIterableUpstream,
  pMapSkip as pMapSkipUpstream,
} from 'p-map';

import {
  type Mapper,
  pMap,
  pMapIterable,
  pMapSkip,
} from '@monochromatic-dev/upstream-fork-p-map/ts';

import { settleRun, } from './map-workload.ts';
import type {
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
    },
  },);
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
    },
  );
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
    run: runFork,
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
    run: runUpstream,
    stream: streamUpstream,
  };
}

//endregion Adapters
