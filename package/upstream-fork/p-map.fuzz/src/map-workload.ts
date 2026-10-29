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
  type MapAdapter,
  type MapAdapterOutcome,
  type MapTrace,
  type MapWorkload,
  type SourceTelemetry,
  describeRejection,
  failureFor,
} from './workload.ts';

//region Runner

/**
 Runs one concurrent-map workload against one mapper adapter and records its
 trace.
 
 @param adapter - Mapper implementation under test.
 
 @param workload - Generated calls, bounds, source shape, and release order.
 
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
  }: {
    readonly adapter: MapAdapter;
    readonly workload: MapWorkload;
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
   Run promise observed once every release step finished.
   */
  const running = adapter.run({
    iterable: instrumentSource({
      values,
      sourceKind: workload.sourceKind,
      telemetry,
    },),
    mapper: function workloadMapper(
      element: string,
      index: number,
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
        return spec.id;
      })();
    },
    concurrency: workload.concurrency,
    stopOnError: workload.stopOnError,
    backpressure: workload.backpressure,
  });

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
     Raw result array settled by the implementation under test.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the adapter's run resolves with the implementation's result array, which comparison renders element by element
    const settled = await run as readonly unknown[];
    /**
     Result values rendered as their string form for comparison.
     */
    const values = settled.map(function renderValue(value: unknown,): string {
      return caughtValueText(value,);
    },);
    return {
      status: 'resolved',
      values,
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
