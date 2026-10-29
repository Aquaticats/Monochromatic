/**
 Deterministic workload model and runner shared by this package's property
 files.
 
 A workload schedules generated calls against a limiter adapter, then walks
 generated release steps: each step optionally applies an action (raising or
 lowering the bound, clearing the queue) and then releases one call's gate.
 Everything the limiter does is decided by the workload, so two limiters fed
 the same workload produce directly comparable traces.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Types

/**
 One generated call: its stable id and what its function does once started.
 
 `throwSync` throws from a synchronous function body on entry (never waiting
 for its gate); the other behaviors block on the call's gate first.
 */
export type CallSpec = {
  /**
   Stable identifier appearing in every trace entry.
   */
  readonly id: string;
  /**
   Outcome the call's function produces.
   */
  readonly behavior: 'resolve' | 'reject' | 'throwSync';
};

/**
 One generated in-flight action applied before a release step.
 */
export type WorkloadAction =
  | {
    /**
     No action at this step.
     */
    readonly kind: 'none';
  }
    | {
    /**
     Clear the limiter's queue at this step.
     */
      readonly kind: 'clearQueue';
    }
    | {
    /**
     Change the limiter's bound at this step.
     */
      readonly kind: 'setConcurrency';
    /**
     New bound to apply.
     */
      readonly concurrency: number;
    };

/**
 Fully generated workload: calls, release order, and interleaved actions.
 */
export type Workload = {
  /**
   Initial concurrency bound.
   */
  readonly concurrency: number;
  /**
   Whether the limiter rejects calls dropped by `clearQueue`.
   */
  readonly rejectOnClear: boolean;
  /**
   Calls to schedule, in scheduling order.
   */
  readonly calls: readonly CallSpec[];
  /**
   Ids released one per step; a permutation of the call ids.
   */
  readonly releaseOrder: readonly string[];
  /**
   Action applied before each release step; parallel to `releaseOrder`.
   */
  readonly actions: readonly WorkloadAction[];
};

/**
 How one call's promise ended, as observed by the runner.
 */
export type CallSettlement = {
  /**
   Call id this settlement belongs to.
   */
  readonly id: string;
  /**
   Settlement state after the workload finished.
   */
  readonly status: 'resolved' | 'rejected' | 'unsettled';
  /**
   Error name of the rejection, empty for resolved or unsettled calls.
   */
  readonly reasonName: string;
  /**
   Error message of the rejection, empty for resolved or unsettled calls.
   */
  readonly reasonMessage: string;
};

/**
 Observable trace of one workload run.
 */
export type WorkloadTrace = {
  /**
   Call ids in the order their functions started.
   */
  readonly startOrder: readonly string[];
  /**
   `activeCount` observed from inside each started call, in start order.
   */
  readonly activeAtStart: readonly number[];
  /**
   `concurrency` observed from inside each started call, in start order.
   */
  readonly boundAtStart: readonly number[];
  /**
   Highest number of simultaneously running calls observed.
   */
  readonly maxOverlap: number;
  /**
   Per-call settlements, in observation order (unsettled calls appended last).
   */
  readonly settlements: readonly CallSettlement[];
  /**
   `pendingCount` observed after each release step.
   */
  readonly pendingAfterStep: readonly number[];
  /**
   `activeCount` once the workload finished.
   */
  readonly finalActiveCount: number;
  /**
   `pendingCount` once the workload finished.
   */
  readonly finalPendingCount: number;
};

/**
 Minimal limiter surface the runner drives: one implementation adapter per
 limiter under comparison.
 */
export type LimiterAdapter = {
  /**
   Schedules one task function and returns the call's promise.
   */
  readonly schedule: (task: () => string | Promise<string>,) => Promise<string>;
  /**
   Reads the limiter's `activeCount`.
   */
  readonly activeCount: () => number;
  /**
   Reads the limiter's `pendingCount`.
   */
  readonly pendingCount: () => number;
  /**
   Reads the limiter's `concurrency`.
   */
  readonly concurrency: () => number;
  /**
   Writes the limiter's `concurrency`.
   */
  readonly setConcurrency: (concurrency: number,) => void;
  /**
   Runs the limiter's `clearQueue`.
   */
  readonly clearQueue: () => void;
};

//endregion Types

//region Failures

/**
 Builds the deterministic failure a rejecting call throws, so both limiters
 under comparison reject with text that can be compared exactly.
 
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
 
 @mutates reason - Rendering a non-Error value runs JavaScript string
 conversion, which may invoke getters, proxies, `Symbol.toPrimitive`,
 `toString`, or `valueOf` through `caughtValueText`.
 
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

//region Runner

/**
 Runs one workload against one limiter adapter and records its trace.
 
 @param adapter - Limiter surface under test.
 
 @param workload - Generated calls, release order, and actions.
 
 @returns Trace of starts, settlements, counters, and overlap.
 
 @example
 ```ts
 const trace = await runWorkload({ adapter, workload, });
 ```
 */
export async function runWorkload(
  {
    adapter,
    workload,
  }: {
    readonly adapter: LimiterAdapter;
    readonly workload: Workload;
  },
): Promise<WorkloadTrace> {
  /**
   Call ids in the order their functions started.
   */
  const startOrder: string[] = [];
  /**
   `activeCount` observed from inside each started call.
   */
  const activeAtStart: number[] = [];
  /**
   `concurrency` observed from inside each started call.
   */
  const boundAtStart: number[] = [];
  /**
   Per-call settlements, appended as observers finish.
   */
  const settlements: CallSettlement[] = [];
  /**
   `pendingCount` observed after each release step.
   */
  const pendingAfterStep: number[] = [];
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
   Settlement observer per call id, created at scheduling time so every
   rejection is observed even when the call is later dropped from the queue.
   */
  const observers = new Map<string, Promise<void>>();

  for (const spec of workload.calls) {
    /**
     Gate this call blocks on until its release step.
     */
    const gate = Promise.withResolvers<void>();
    gates.set(
      spec.id,
      gate,
    );

    /**
     This call's scheduled task and its observed settlement.
     */
    const scheduled = adapter.schedule(function workloadCall(): string | Promise<string> {
      startOrder.push(spec.id,);
      activeAtStart.push(adapter.activeCount(),);
      boundAtStart.push(adapter.concurrency(),);
      overlap.running += 1;
      overlap.maxRunning = Math.max(
        overlap.maxRunning,
        overlap.running,
      );

      if (spec.behavior === 'throwSync') {
        overlap.running -= 1;
        throw failureFor(spec.id,);
      }

      return (async function gatedOutcome(): Promise<string> {
        await gate.promise;
        overlap.running -= 1;
        if (spec.behavior === 'reject')
          throw failureFor(spec.id,);
        return spec.id;
      })();
    },);

    observers.set(
      spec.id,
      observeSettlement({
        id: spec.id,
        promise: scheduled,
        settlements,
      },),
    );
  }

  /**
   Quiescence turn after the synchronous scheduling batch: queued starts fire
   before any action runs, so each call's per-start observations match its
   admission moment.
   */
  await wait(
    0,
  );

  /* oxlint-disable no-await-in-loop */
  // Release steps must run one at a time: each step's gate release and the
  // limiter's drain have to settle before the next step observes counters.
  /**
   Generated actions and release order, destructured so the per-step lookups
   stay two-step chains.
   */
  const {
    actions,
    releaseOrder,
  } = workload;
  for (const [stepIndex, releaseId,] of releaseOrder.entries()) {
    applyAction({
      action: actions[stepIndex]
        ?? {
          kind: 'none',
        },
      adapter,
    },);
    gates
      .get(releaseId,)
      ?.resolve();
    await wait(
      0,
    );
    pendingAfterStep.push(adapter.pendingCount(),);
  }
  /* oxlint-enable no-await-in-loop */

  /**
   Observers of every call whose function started; each settles once its
   gate was released.
   */
  const startedObservers = startOrder.map(function observerForStarted(id: string,): Promise<void> {
    return observers
      .get(id,)
      ?? Promise.resolve();
  });
  await Promise.allSettled(startedObservers,);
  await wait(
    0,
  );

  for (const spec of workload.calls)
    if (!settlements.some(function alreadyRecorded(entry: CallSettlement,): boolean {
      return entry.id === spec.id;
    },))
      settlements.push({
        id: spec.id,
        status: 'unsettled',
        reasonName: '',
        reasonMessage: '',
      },);

  return {
    startOrder,
    activeAtStart,
    boundAtStart,
    maxOverlap: overlap.maxRunning,
    settlements,
    pendingAfterStep,
    finalActiveCount: adapter.activeCount(),
    finalPendingCount: adapter.pendingCount(),
  };
}

/**
 Records one call's settlement once its promise settles.
 
 @param id - Id of the observed call.
 
 @param promise - Promise returned by the limiter for this call.
 
 @param settlements - Trace list this observation appends to.
 
 @example
 ```ts
 const observer = observeSettlement({ id, promise, settlements, });
 ```
 */
async function observeSettlement(
  {
    id,
    promise,
    settlements,
  }: {
    readonly id: string;
    readonly promise: Promise<string>;
    readonly settlements: CallSettlement[];
  },
): Promise<void> {
  try {
    await promise;
  }
  catch (error) {
    settlements.push({
      id,
      status: 'rejected',
      ...describeRejection(error,),
    },);
    return;
  }

  settlements.push({
    id,
    status: 'resolved',
    reasonName: '',
    reasonMessage: '',
  },);
}

/**
 Applies one generated in-flight action to the limiter under test.
 
 @param action - Action selected for the current release step.
 
 @param adapter - Limiter surface under test.
 
 @example
 ```ts
 applyAction({
   action: { kind: 'clearQueue' },
   adapter,
 });
 ```
 */
function applyAction(
  {
    action,
    adapter,
  }: {
    readonly action: WorkloadAction;
    readonly adapter: LimiterAdapter;
  },
): void {
  if (action.kind === 'setConcurrency') {
    adapter.setConcurrency(action.concurrency,);
    return;
  }

  if (action.kind === 'clearQueue')
    adapter.clearQueue();
}

//endregion Runner
