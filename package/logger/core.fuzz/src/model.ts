/**
 Reference model of the logger orchestration.

 The model folds a trace of operations (`log`, `init-complete`, `flush`) and
 fake-sink observations (`verify` settlements) into what the logger's
 contract promises: per sink, the exact records attempted in order and the
 final availability; the dropped-startup count and whether the marker record
 is due; whether each `flush()` settles inside its deadline; and how many
 breadcrumbs the logger must have written. Write and flush outcomes are not
 observed, they are read from the scripts by call index, so the model never
 copies the logger's bookkeeping, only its promises.

 Assumptions the properties keep true: the verify time limit is shorter
 than the flush deadline, so a `never` verify makes init complete before a
 flush deadline can elapse; and every settleable outcome is released before
 the trace is folded, so only `never` outcomes are still pending.

 @module
 */

import type { LogRecord, } from '@monochromatic-dev/logger-core';

import {
  outcomeAt,
  type SinkScript,
  type VerifyOutcome,
} from './fake-sink.ts';

//region Events

/**
 Operation or observation the model folds, in the order it happened.
 */
export type ModelEvent =
  | { readonly kind: 'flush'; }
    | { readonly kind: 'init-complete'; }
    | {
      readonly kind: 'log';
      readonly record: LogRecord
    }
    | {
      readonly kind: 'verify-settled';
      readonly outcome: VerifyOutcome;
      readonly sink: number
    };

//endregion Events

//region State

/**
 Model's view of one sink.
 */
export type SinkModel = {
  /**
   Records the logger must have handed to `write`, in order.
   */
  readonly attempts: LogRecord[];
  /**
   Whether the sink is currently available to receive writes.
   */
  available: boolean;
  /**
   Flush-hook calls made so far, to read the next scripted outcome.
   */
  flushCalls: number;
  /**
   Whether a verify settled true before init completed.
   */
  verified: boolean;
};

/**
 Whole model state after folding a trace.
 */
export type ModelState = {
  /**
   Breadcrumbs (`console.warn` calls) the logger must have written.
   */
  breadcrumbs: number;
  /**
   Whether every `flush` so far settled inside its deadline, per call.
   */
  readonly flushesWithinDeadline: boolean[];
  /**
   Whether a marker record was due at init completion.
   */
  markerDue: boolean;
  /**
   Records dropped from the startup buffer.
   */
  dropped: number;
  initialized: boolean;
  readonly sinks: SinkModel[];
  /**
   Startup buffer as the logger keeps it before init completes.
   */
  readonly startup: LogRecord[];
  /**
   Whether any write attempt whose outcome is `never` is still tracked by the
   logger (not yet abandoned by a deadline hit), per sink attempt index.
   */
  readonly pendingNever: {
    sink: number;
    attempt: number
  }[];
};

//endregion State

//region Fold

/**
 Builds the empty model for a sink list.

 @param sinkCount - Number of sinks the logger was built with.

 @returns Fresh model state.

 @example
 ```ts
 const state = emptyModel({ sinkCount: 2 });
 ```
 */
export function emptyModel({ sinkCount, }: { readonly sinkCount: number; },): ModelState {
  return {
    breadcrumbs: 0,
    dropped: 0,
    flushesWithinDeadline: [],
    initialized: false,
    markerDue: false,
    pendingNever: [],
    sinks: Array.from(
      { length: sinkCount, },
      function toSinkModel(): SinkModel {
        return {
          attempts: [],
          available: false,
          flushCalls: 0,
          verified: false,
        };
      },
    ),
    startup: [],
  };
}

/**
 Records one write attempt on a sink, tracking a `never` outcome as pending.

 @param state - Model state to mutate.

 @param scripts - Sink scripts, to read the write outcome by call index.

 @param sink - Sink index receiving the attempt.

 @param record - Record attempted.
 */
function attempt(
  {
    state,
    scripts,
    sink,
    record,
  }: {
    readonly state: ModelState;
    readonly scripts: readonly SinkScript[];
    readonly sink: number;
    readonly record: LogRecord;
  },
): void {
  /**
   Model of the receiving sink.
   */
  const model = state.sinks[sink];
  /**
   Script of the receiving sink.
   */
  const script = scripts[sink];
  if ((model === undefined) || (script === undefined))
    throw new Error(`model has no sink ${sink}`,);
  /**
   Zero-based write call number this attempt becomes.
   */
  const callIndex = model.attempts
    .length;
  model.attempts
    .push(record,);
  /**
   Scripted outcome of this write call.
   */
  const outcome = outcomeAt({
    script: script.write,
    callIndex,
  },);
  if ((outcome === 'throw') || (outcome === 'reject'))
    state.breadcrumbs += 1;
  if (outcome === 'never') {
    state.pendingNever
      .push({
        sink,
        attempt: callIndex,
      },);
  }
}

/**
 Folds one event into the model.

 @param state - Model state to mutate.

 @param scripts - Sink scripts.

 @param cap - Startup buffer cap the logger was built with.

 @param event - Event to fold.

 @example
 ```ts
 foldEvent({ state, scripts, cap: STARTUP_BUFFER_CAP, event: { kind: 'flush' } });
 ```
 */
export function foldEvent(
  {
    state,
    scripts,
    cap,
    event,
  }: {
    readonly state: ModelState;
    readonly scripts: readonly SinkScript[];
    readonly cap: number;
    readonly event: ModelEvent;
  },
): void {
  if (event.kind === 'log') {
    if (!state.initialized) {
      if (state.startup
        .length
        >= cap) {
        state.startup
          .shift();
        state.dropped += 1;
      }
      state.startup
        .push(event.record,);
    }
    for (const [sink, model,] of state.sinks
      .entries()) {
      if (model.available) {
        attempt({
          state,
          scripts,
          sink,
          record: event.record,
        },);
      }
    }
    return;
  }
  if (event.kind === 'verify-settled') {
    /**
     Model of the sink whose verify settled.
     */
    const model = state.sinks[event.sink];
    if (model === undefined)
      throw new Error(`model has no sink ${event.sink}`,);
    if ((event.outcome === 'reject') || (event.outcome === 'throw')
      || (event.outcome === 'never'))
      state.breadcrumbs += 1;
    if (event.outcome !== 'resolve-true')
      return;
    model.verified = true;
    model.available = true;
    state.startup
      .forEach(function replay(record,) {
        attempt({
          state,
          scripts,
          sink: event.sink,
          record,
        },);
      },);
    return;
  }
  if (event.kind === 'init-complete') {
    state.initialized = true;
    state.startup
      .length = 0;
    if (state.dropped === 0)
      return;
    state.markerDue = true;
    /**
     Marker record the logger writes; the message is asserted separately, so
     only its level matters to the attempt list.
     */
    const marker: LogRecord = {
      level: 'warn',
      message: `${state.dropped} startup record${(state.dropped === 1) ? '' : 's'} dropped before a backend verified (buffer cap ${cap})`,
      timestamp: 0,
    };
    for (const [sink, model,] of state.sinks
      .entries()) {
      if (model.available) {
        attempt({
          state,
          scripts,
          sink,
          record: marker,
        },);
      }
    }
    return;
  }
  foldFlush({
    state,
    scripts,
  },);
}

/**
 Folds a `flush` call: runs the hook of every available sink by script,
 retiring a sink whose hook rejects or throws, and decides whether the flush
 settled inside its deadline (no tracked `never` write and no `never` hook).

 @param state - Model state to mutate.

 @param scripts - Sink scripts.
 */
function foldFlush(
  {
    state,
    scripts,
  }: {
    readonly state: ModelState;
    readonly scripts: readonly SinkScript[];
  },
): void {
  // The logger drains tracked writes before it runs any flush hook, so a
  // write that never settles hits the deadline during the drain and no hook
  // is called at all in that flush; the deadline hit abandons the tracked
  // writes, so the next flush reaches the hooks.
  if (state.pendingNever
    .length
    > 0) {
    state.flushesWithinDeadline
      .push(false,);
    state.breadcrumbs += 1;
    state.pendingNever
      .length = 0;
    return;
  }
  /**
   Whether a flush hook that never settles makes this flush miss its deadline.
   */
  const hung = { value: false, };
  for (const [sink, model,] of state.sinks
    .entries()) {
    /**
     Script of this sink.
     */
    const script = scripts[sink];
    if ((script === undefined) || (!model.available)
      || (script.flush === undefined))
      continue;
    /**
     Scripted outcome of this flush-hook call.
     */
    const outcome = outcomeAt({
      script: script.flush,
      callIndex: model.flushCalls,
    },);
    model.flushCalls += 1;
    if ((outcome === 'reject') || (outcome === 'throw')) {
      state.breadcrumbs += 1;
      model.available = false;
    }
    if (outcome === 'never')
      hung.value = true;
  }
  state.flushesWithinDeadline
    .push(!hung.value,);
  if (hung.value)
    state.breadcrumbs += 1;
}

/**
 Folds a whole trace from the empty model.

 @param scripts - Sink scripts in the logger's sink order.

 @param cap - Startup buffer cap the logger was built with.

 @param events - Trace in the order it happened.

 @returns Model state after every event.

 @example
 ```ts
 const expected = foldTrace({ scripts, cap: STARTUP_BUFFER_CAP, events });
 ```
 */
export function foldTrace(
  {
    scripts,
    cap,
    events,
  }: {
    readonly scripts: readonly SinkScript[];
    readonly cap: number;
    readonly events: readonly ModelEvent[];
  },
): ModelState {
  /**
   Model state accumulated over the trace.
   */
  const state = emptyModel({ sinkCount: scripts.length, },);
  for (const event of events) {
    foldEvent({
      state,
      scripts,
      cap,
      event,
    },);
  }
  return state;
}

//endregion Fold
