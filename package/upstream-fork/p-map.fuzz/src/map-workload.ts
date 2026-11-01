/**
 Concurrent-map workload runner: drives one generated workload through a
 mapper adapter and records its observable trace.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { instrumentSource, } from './workload-source.ts';
import {
  type CallSpec,
  type KeyedAdapterCall,
  type MapAdapter,
  type MapAdapterCall,
  type MapAdapterOutcome,
  type MapTrace,
  type MapWorkload,
  type SourceTelemetry,
  describeRejection,
  failureFor,
} from './workload.ts';

//region Runner

/**
 Which map variant a workload drives.
 */
export type MapSurface = 'map' | 'settled' | 'keyed' | 'settledKeyed';

/**
 One key with its value, promised or plain.
 */
type KeyedEntry = [
  string,
  string | Promise<string>,
];

/**
 One entry of a result `Map`.
 */
type ResultMapEntry = readonly [
  unknown,
  unknown,
];

/**
 One entry of a result object.
 */
type ResultObjectEntry = readonly [
  string,
  unknown,
];

/**
 Builds the keyed call of one workload: the call ids become the keys of an
 object or a `Map`, and the workload mapper is addressed by key.

 @param calls - Generated calls in scheduling order.

 @param values - Value per call, promised or plain.

 @param workload - Generated bounds and input shape.

 @param surface - Keyed surface, settled or not.

 @param mapper - Workload mapper shared with the other surfaces.

 @returns Keyed call for the adapter.

 @example
 ```ts
 const keyed = toKeyedCall({ calls, values, workload, surface: 'keyed', mapper, },);
 ```
 */
function toKeyedCall(
  {
    calls,
    values,
    workload,
    surface,
    mapper,
  }: {
    /**
     Generated calls in scheduling order.
     */
    readonly calls: readonly CallSpec[];
    /**
     Value per call, promised or plain.
     */
    readonly values: readonly (string | Promise<string>)[];
    /**
     Generated bounds and input shape.
     */
    readonly workload: MapWorkload;
    /**
     Keyed surface, settled or not.
     */
    readonly surface: MapSurface;
    /**
     Workload mapper shared with the other surfaces.
     */
    readonly mapper: (call: {
      readonly element: string;
      readonly index: number;
    },) => Promise<string | symbol>;
  },
): KeyedAdapterCall {
  /**
   Key and value per call, in scheduling order.
   */
  const entries: KeyedEntry[] = calls.map(function toEntry(
    spec: CallSpec,
    position: number,
  ): KeyedEntry {
    return [
      spec.id,
      nonNullishOrThrow(values.at(position,),),
    ];
  },);

  return {
    input: (workload.inputShape === 'map')
      ? new Map(entries,)
      : Object.fromEntries(entries,),
    mapper: function keyedMapper(
      value: string,
      key: string,
    ): Promise<string | symbol> {
      return mapper({
        element: value,
        index: calls.findIndex(function byKey(spec: CallSpec,): boolean {
          return spec.id === key;
        },),
      },);
    },
    concurrency: workload.concurrency,
    stopOnError: workload.stopOnError,
    settled: surface === 'settledKeyed',
  };
}

/**
 Runs one concurrent-map workload against one mapper adapter and records its
 trace.
 
 @param adapter - Mapper implementation under test.
 
 @param workload - Generated calls, bounds, source shape, and release order.
 
 @param surface - Which map variant to drive: the plain map, the settled map,
 or the keyed map, settled or not.
 
 @returns Trace of starts, overlap, telemetry, and the normalized
 settlement.
 
 @example
 ```ts
 const trace = await runMapWorkload({ adapter, workload, });
 ```
 */
export async function runMapWorkload(
  {
    adapter,
    workload,
    surface = 'map',
  }: {
    readonly adapter: MapAdapter;
    readonly workload: MapWorkload;
    readonly surface?: MapSurface;
  },
): Promise<MapTrace> {
  /**
   Call ids in the order their mapper calls started.
   */
  const startOrder: string[] = [];
  /**
   Running-call count observed from inside each started mapper call.
   */
  const activeAtStart: number[] = [];
  /**
   Running-call counter mirrored from the observed starts and completions.
   */
  const overlap = {
    running: 0,
    maxRunning: 0,
  };
  /**
   Gate per call id, released by the workload's release steps.
   */
  const gates = new Map<string, PromiseWithResolvers<void>>();
  /**
   Behavior per call id, looked up when the mapper call starts.
   */
  const behaviors = new Map<string, CallSpec>();
  /**
   Source telemetry for this run.
   */
  const telemetry: SourceTelemetry = {
    pulls: 0,
    closes: 0,
  };

  for (const spec of workload.calls) {
    gates.set(
      spec.id,
      Promise.withResolvers<void>(),
    );
    behaviors.set(
      spec.id,
      spec,
    );
  }

  /**
   Generated calls in scheduling order, destructured so every lookup below
   stays a one-step chain.
   */
  const {
    calls,
  } = workload;
  /**
   Source values in call order, promised or plain per the workload.
   */
  const values: (string | Promise<string>)[] = calls.map(function toValue(spec: CallSpec,): string | Promise<string> {
    return (workload.elementKind === 'promise')
      ? Promise.resolve(spec.id,)
      : spec.id;
  },);

  /**
   Workload mapper shared by every surface: records the start, then behaves
   as the call's spec says.
   
   @returns Promise of the call's id or sentinel.
   */
  function runWorkloadCall(
    {
      element,
      index,
    }: {
      /**
       Awaited element or value the engine hands over.
       */
      readonly element: string;
      /**
       Position of the call in scheduling order.
       */
      readonly index: number;
    },
  ): Promise<string | symbol> {
    /**
     Generated call behind this mapper invocation, found by element id and
     by input position so both lookup surfaces stay exercised.
     */
    const spec = behaviors.get(element,)
      ?? nonNullishOrThrow(calls.at(index,),);
    startOrder.push(spec.id,);
    overlap.running += 1;
    overlap.maxRunning = Math.max(
      overlap.maxRunning,
      overlap.running,
    );
    activeAtStart.push(overlap.running,);

    if (spec.behavior === 'throwSync') {
      overlap.running -= 1;
      throw failureFor(spec.id,);
    }

    return (async function gatedOutcome(): Promise<string | symbol> {
      await gates
        .get(spec.id,)
        ?.promise;
      overlap.running -= 1;

      if (spec.behavior === 'reject')
        throw failureFor(spec.id,);
      if (spec.behavior === 'skip')
        return adapter.skip;
      if (spec.behavior === 'stop')
        return adapter.stop;
      return spec.id;
    })();
  }

  /**
   Call for the plain and settled surfaces: the instrumented source plus the
   workload mapper.
   */
  const call: MapAdapterCall = {
    iterable: instrumentSource({
      values,
      sourceKind: workload.sourceKind,
      telemetry,
    },),
    mapper: function workloadMapper(
      element: string,
      index: number,
    ): Promise<string | symbol> {
      return runWorkloadCall({
        element,
        index,
      },);
    },
    concurrency: workload.concurrency,
    stopOnError: workload.stopOnError,
    backpressure: workload.backpressure,
    preserveOrder: workload.preserveOrder,
  };

  /**
   Run promise observed once every release step finished.
   */
  const running = (surface === 'map')
    ? adapter.run(call,)
    : (surface === 'settled')
      ? adapter.runSettled(call,)
      : adapter.runKeyed(toKeyedCall({
        calls,
        values,
        workload,
        surface,
        mapper: runWorkloadCall,
      },),);

  await wait(
    0,
  );

  /* oxlint-disable no-await-in-loop -- release steps run one at a time: each gate release and the implementation's pull chain have to settle before the next step */
  for (const releaseId of workload.releaseOrder) {
    gates
      .get(releaseId,)
      ?.resolve();
    await wait(
      0,
    );
  }
  /* oxlint-enable no-await-in-loop */

  /**
   Normalized settlement of the run under test.
   */
  const outcome = await running;
  await wait(
    0,
  );

  return {
    startOrder,
    activeAtStart,
    maxOverlap: overlap.maxRunning,
    telemetry,
    outcome,
  };
}

//endregion Runner

//region Settlement

/**
 Whether a value is an object carrying a `status` property.

 @param value - Result element the implementation produced.

 @returns `true` for an object with a `status` property; narrows it.

 @example
 ```ts
 isStatusObject({ status: 'fulfilled', },); // => true
 ```
 */
function isStatusObject(value: unknown,): value is { readonly status: unknown; } {
  if (((typeof value) !== 'object') || (value === null))
    return false;

  return 'status' in value;
}

/**
 Renders one result element for comparison: settled results as
 `status:id` or `status:message`, anything else as its string form.

 @param value - Result element the implementation produced.

 @returns Comparable text.

 @example
 ```ts
 renderResult({ status: 'fulfilled', value: 'c0', },); // => 'fulfilled:c0'
 ```
 */
function renderResult(value: unknown,): string {
  if (isStatusObject(value,)) {
    if ((value.status === 'fulfilled') && ('value' in value))
      return `fulfilled:${caughtValueText(value.value,)}`;
    if ((value.status === 'rejected') && ('reason' in value)) {
      /**
       Rendered rejection reason of the settled result.
       */
      const described = describeRejection(value.reason,);
      return `rejected:${described.reasonMessage}`;
    }
  }

  return caughtValueText(value,);
}

/**
 Renders one settlement for comparison: an array element by element, a
 `Map` or an object as `key=result` pairs in order. A `Map`, a plain object,
 and a null-prototype object differ in their prefix, so the result shape is
 compared too.

 @param settled - Value the implementation resolved with.

 @returns Comparable rendering, one text per element or entry.

 @example
 ```ts
 renderSettlement(['c0',],); // => ['c0']
 ```
 */
function renderSettlement(settled: unknown,): readonly string[] {
  if (Array.isArray(settled,))
    return settled.map(renderResult,);

  if (settled instanceof Map)
    return [
      'Map',
      ...[...settled.entries(),].map(function renderMapEntry(
        entry: ResultMapEntry,
      ): string {
        return `${caughtValueText(entry[0],)}=${renderResult(entry[1],)}`;
      },),
    ];

  if (((typeof settled) === 'object') && (settled !== null))
    return [
      (Object.getPrototypeOf(settled,) === null)
        ? 'null-prototype'
        : 'prototype',
      ...Object.entries(settled,)
        .map(function renderObjectEntry(
          entry: ResultObjectEntry,
        ): string {
          return `${entry[0]}=${renderResult(entry[1],)}`;
        },),
    ];

  return [caughtValueText(settled,)];
}

/**
 Renders every `AggregateError` member message of one rejection, for
 comparison across implementations.
 
 @param reason - Rejection reason observed by the runner.
 
 @returns Member messages in `AggregateError` order; empty otherwise.
 
 @example
 ```ts
 aggregateMessages(new AggregateError([new Error('boom',)],));
 // => ['boom']
 ```
 */
function aggregateMessages(reason: unknown,): readonly string[] {
  if ((!(Error.isError(reason,))) || (!(reason instanceof AggregateError)))
    return [];
  /**
   Aggregated members of the observed rejection.
   */
  const members = reason.errors;
  return members.map(function renderMember(member: unknown,): string {
    return describeRejection(member,)
      .reasonMessage;
  },);
}

/**
 Normalizes one adapter's raw run settlement into the comparable outcome:
 result ids in result order, or the rejection's rendered shape including
 `AggregateError` members.
 
 @param run - Promise returned by the implementation under test.
 
 @returns Normalized settlement for cross-implementation comparison.
 
 @example
 ```ts
 const outcome = await settleRun(pMap({...}));
 ```
 */
export async function settleRun(run: Promise<unknown>,): Promise<MapAdapterOutcome> {
  try {
    /**
     Raw result settled by the implementation under test: an array, an
     object, or a `Map`.
     */
    const settled = await run;
    return {
      status: 'resolved',
      values: renderSettlement(settled,),
      reasonName: '',
      reasonMessage: '',
      aggregated: [],
    };
  }
  catch (error) {
    /**
     Rendered rejection shape.
     */
    const described = describeRejection(error,);
    return {
      status: 'rejected',
      values: [],
      ...described,
      aggregated: aggregateMessages(error,),
    };
  }
}

//endregion Settlement
