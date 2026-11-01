/**
 Streaming-map workload runner: drives one generated workload through a
 mapper adapter's streaming surface and records its observable trace.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { instrumentSource, } from './workload-source.ts';
import {
  type CallSpec,
  type IterableTrace,
  type MapAdapter,
  type MapWorkload,
  type SourceTelemetry,
  describeRejection,
  failureFor,
} from './workload.ts';

//region Runner

/**
 Runs one streaming-map workload against one mapper adapter and records its
 trace: yields, the iteration's throw, telemetry, and the resolved backlog.
 
 @param adapter - Mapper implementation under test.
 
 @param workload - Generated calls, bounds, source shape, and release order.
 
 @returns Trace of yields, throw, telemetry, and observed backlog.
 
 @example
 ```ts
 const trace = await runIterableWorkload({ adapter, workload, });
 ```
 */
export async function runIterableWorkload(
  {
    adapter,
    workload,
  }: {
    readonly adapter: MapAdapter;
    readonly workload: MapWorkload;
  },
): Promise<IterableTrace> {
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
  /**
   Resolved-but-unconsumed backlog counters for this run.
   */
  const backlog = {
    resolved: 0,
    collected: 0,
    maxObserved: 0,
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
   Consumes the adapter's stream, collecting yields and the throw.
   
   @returns Yields in order and the rendered throw, if any.
   */
  async function collectStream(): Promise<{
    readonly yielded: readonly string[];
    readonly thrownName: string;
    readonly thrownMessage: string;
  }> {
    /**
     Yielded ids in yield order.
     */
    const yielded: string[] = [];
    try {
      for await (const value of adapter.stream({
        iterable: instrumentSource({
          values,
          sourceKind: workload.sourceKind,
          telemetry,
        },),
        mapper: function workloadMapper(element: string,): Promise<string | symbol> {
          /**
           Generated call behind this mapper invocation, found by element id
           and by input position so both lookup surfaces stay exercised.
           */
          const spec = behaviors.get(element,)
            ?? nonNullishOrThrow(calls.find(function byId(candidate: CallSpec,): boolean {
              return candidate.id === element;
            },),);

          if (spec.behavior === 'throwSync')
            throw failureFor(spec.id,);

          return (async function gatedOutcome(): Promise<string | symbol> {
            await gates
              .get(spec.id,)
              ?.promise;

            if (spec.behavior === 'reject')
              throw failureFor(spec.id,);
            if (spec.behavior === 'skip')
              return adapter.skip;
            if (spec.behavior === 'stop')
              return adapter.stop;

            backlog.resolved += 1;
            backlog.maxObserved = Math.max(
              backlog.maxObserved,
              backlog.resolved - backlog.collected,
            );
            return spec.id;
          })();
        },
        concurrency: workload.concurrency,
        stopOnError: workload.stopOnError,
        backpressure: workload.backpressure,
        preserveOrder: workload.preserveOrder,
      })) {
        yielded.push(caughtValueText(value,),);
        backlog.collected += 1;
      }
    }
    catch (error) {
      /**
       Rendered iteration failure.
       */
      const described = describeRejection(error,);
      return {
        yielded,
        thrownName: described.reasonName,
        thrownMessage: described.reasonMessage,
      };
    }

    return {
      yielded,
      thrownName: '',
      thrownMessage: '',
    };
  }

  /**
   Collection observed once every release step finished.
   */
  const collecting = collectStream();

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
   Collected yields and throw from the run under test.
   */
  const collected = await collecting;
  await wait(
    0,
  );

  return {
    yielded: collected.yielded,
    thrownName: collected.thrownName,
    thrownMessage: collected.thrownMessage,
    telemetry,
    maxBacklog: backlog.maxObserved,
  };
}

//endregion Runner
