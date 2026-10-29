/**
 Deterministic workload types and failure rendering shared by this
 package's runners and property files.
 
 A workload drives one concurrent map through a mapper adapter: each
 generated call blocks on its own gate, and the runner releases gates one per
 step in a generated order. Everything the mapper does is decided by the
 workload, so two implementations fed the same workload produce directly
 comparable traces.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Types

/**
 One generated call: its stable id and what its mapper call does.
 
 `throwSync` throws from a synchronous function body on entry (never waiting
 for its gate); the other behaviors block on the call's gate first. `stop`
 resolves with the adapter's stop sentinel.
 */
export type CallSpec = {
  /**
   Stable identifier appearing in every trace entry.
   */
  readonly id: string;
  /**
   Outcome the mapper call produces.
   */
  readonly behavior: 'resolve' | 'reject' | 'throwSync' | 'skip' | 'stop';
};

/**
 Fully generated map workload: calls, source shape, bounds, and release
 order.
 */
export type MapWorkload = {
  /**
   Concurrency bound for the run.
   */
  readonly concurrency: number;
  /**
   Whether the run rejects on the first mapper failure.
   */
  readonly stopOnError: boolean;
  /**
   Backpressure bound for the streaming variant.
   */
  readonly backpressure: number;
  /**
   Whether the streaming variant yields results in input order.
   */
  readonly preserveOrder: boolean;
  /**
   Whether the keyed variants map a plain object or a `Map`.
   */
  readonly inputShape: 'object' | 'map';
  /**
   Whether iterated elements are plain values or already-resolved promises.
   */
  readonly elementKind: 'value' | 'promise';
  /**
   Whether the source is walked synchronously or asynchronously.
   */
  readonly sourceKind: 'sync' | 'async';
  /**
   Calls to run, in scheduling order.
   */
  readonly calls: readonly CallSpec[];
  /**
   Ids released one per step; a permutation of the call ids.
   */
  readonly releaseOrder: readonly string[];
};

/**
 Pull and close telemetry recorded from the instrumented source.
 */
export type SourceTelemetry = {
  /**
   Number of `next` calls the run made on the source.
   */
  pulls: number;
  /**
   Number of `return` calls the run made on the source.
   */
  closes: number;
};

/**
 One mapper call as the runner drives it: the awaited element, its position,
 and the outcome it produces.
 */
export type WorkloadMapper = (
  element: string,
  index: number,
) => string | symbol | PromiseLike<string | symbol>;

/**
 Minimal mapper surface the runners drive: one implementation adapter per
 mapper under comparison.
 */
export type MapAdapter = {
  /**
   The implementation's `pMapSkip` sentinel, returned by skip calls.
   */
  readonly skip: symbol;
  /**
   The implementation's `pMapStop` sentinel, returned by stop calls.
   */
  readonly stop: symbol;
  /**
   Runs one concurrent map over the instrumented source and normalizes its
   settlement.
   */
  readonly run: (call: MapAdapterCall,) => Promise<MapAdapterOutcome>;
  /**
   Runs one settled map over the instrumented source and normalizes its
   settlement.
   */
  readonly runSettled: (call: MapAdapterCall,) => Promise<MapAdapterOutcome>;
  /**
   Runs one keyed map, settled or not, and normalizes its settlement.
   */
  readonly runKeyed: (call: KeyedAdapterCall,) => Promise<MapAdapterOutcome>;
  /**
   Runs one streaming map over the instrumented source.
   */
  readonly stream: (call: MapAdapterCall,) => AsyncIterable<string | symbol>;
};

/**
 One adapter-driven concurrent map: source, mapper, and bounds.
 */
export type MapAdapterCall = {
  /**
   Instrumented source walked by the implementation.
   */
  readonly iterable: Iterable<string | Promise<string>> | AsyncIterable<string | Promise<string>>;
  /**
   Workload mapper producing ids, failures, and skip sentinels.
   */
  readonly mapper: WorkloadMapper;
  /**
   Concurrency bound for the run.
   */
  readonly concurrency: number;
  /**
   Whether the run rejects on the first mapper failure.
   */
  readonly stopOnError: boolean;
  /**
   Backpressure bound for the streaming variant.
   */
  readonly backpressure: number;
  /**
   Whether the streaming variant yields results in input order.
   */
  readonly preserveOrder: boolean;
};

/**
 One adapter-driven keyed map: input, mapper, and bounds.
 */
export type KeyedAdapterCall = {
  /**
   Object or `Map` whose values are mapped, keyed by call id.
   */
  readonly input: Record<string, string | Promise<string>> | Map<string, string | Promise<string>>;
  /**
   Workload mapper called with each value and its key.
   */
  readonly mapper: (
    value: string,
    key: string,
  ) => string | symbol | PromiseLike<string | symbol>;
  /**
   Concurrency bound for the run.
   */
  readonly concurrency: number;
  /**
   Whether the run rejects on the first mapper failure.
   */
  readonly stopOnError: boolean;
  /**
   Whether failures become settled results instead of rejecting the run.
   */
  readonly settled: boolean;
};

/**
 Normalized settlement of one concurrent map.
 */
export type MapAdapterOutcome = {
  /**
   Whether the run resolved or rejected.
   */
  readonly status: 'resolved' | 'rejected';
  /**
   Result ids in result order, rendered as `key=id` for keyed results and
   `status:id` for settled ones; empty for a rejected run.
   */
  readonly values: readonly string[];
  /**
   Rejection error name; empty for a resolved run.
   */
  readonly reasonName: string;
  /**
   Rejection error message; empty for a resolved run.
   */
  readonly reasonMessage: string;
  /**
   Messages of every `AggregateError` member; empty unless the run rejected
   with one.
   */
  readonly aggregated: readonly string[];
};

/**
 Observable trace of one concurrent-map workload run.
 */
export type MapTrace = {
  /**
   Call ids in the order their mapper calls started.
   */
  readonly startOrder: readonly string[];
  /**
   Running-call count observed from inside each started mapper, in start
   order.
   */
  readonly activeAtStart: readonly number[];
  /**
   Highest number of simultaneously running mapper calls observed.
   */
  readonly maxOverlap: number;
  /**
   Source pulls and closes observed during the run.
   */
  readonly telemetry: SourceTelemetry;
  /**
   Normalized settlement of the run.
   */
  readonly outcome: MapAdapterOutcome;
};

/**
 Observable trace of one streaming-map workload run.
 */
export type IterableTrace = {
  /**
   Yielded ids in yield order.
   */
  readonly yielded: readonly string[];
  /**
   Error name the iteration threw; empty when it completed.
   */
  readonly thrownName: string;
  /**
   Error message the iteration threw; empty when it completed.
   */
  readonly thrownMessage: string;
  /**
   Source pulls and closes observed during the run.
   */
  readonly telemetry: SourceTelemetry;
  /**
   Highest observed count of mapper results resolved but not yet collected.
   */
  readonly maxBacklog: number;
};

//endregion Types

//region Failures

/**
 Builds the deterministic failure a rejecting mapper call throws, so both
 implementations under comparison fail with text compared exactly.
 
 @param id - Id of the failing call.
 
 @returns Error whose message is derived only from the call id.
 
 @example
 ```ts
 failureFor('c3').message; // => 'failure c3'
 ```
 */
export function failureFor(id: string,): Error {
  return new Error(`failure ${id}`,);
}

/**
 Renders a rejection reason for comparison across implementations.
 
 @param reason - Rejection reason observed by the runner.
 
 @returns Name and message of the reason, normalized for deep comparison.
 
 @example
 ```ts
 describeRejection(new TypeError('bad',),);
 // => { reasonName: 'TypeError', reasonMessage: 'bad' }
 ```
 */
export function describeRejection(reason: unknown,): {
  readonly reasonName: string;
  readonly reasonMessage: string;
} {
  if (Error.isError(reason,))
    return {
      reasonName: reason.name,
      reasonMessage: reason.message,
    };

  return {
    reasonName: 'ThrownValue',
    reasonMessage: caughtValueText(reason,),
  };
}

//endregion Failures
