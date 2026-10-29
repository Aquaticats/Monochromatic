/**
 Scripted fake sink for the logger campaign.

 A `SinkScript` says, per hook, what each call does: the head lists the
 outcomes of the first calls in order and the tail is the outcome every later
 call receives, so "first write rejects, later writes succeed" is
 `{ head: ['reject'], tail: 'resolve' }`. The sink carries a stable index so a
 shrunk counterexample names it (`sink 2: verify [resolve-true*] write
 [reject, resolve*] flush absent`) and appends a trace event when a hook is
 called and when its outcome settles, which the reference model folds.

 Timing is not part of the script: an optional `schedule` gate (fast-check's
 `scheduler().schedule` in the properties) decides when a settleable outcome
 reaches the logger, and `never` is a promise nobody ever resolves.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type {
  LogRecord,
  Sink,
} from '@monochromatic-dev/logger-core';

//region Scripts

/**
 Hook names a `Sink` exposes.
 */
export type HookName = 'flush' | 'verify' | 'write';

/**
 Outcome of one `verify` call. `throw` is a synchronous throw; `never` is a
 promise that never settles, which the logger's verify time limit converts
 into unavailability.
 */
export type VerifyOutcome = 'never' | 'reject' | 'resolve-false' | 'resolve-true' | 'throw';

/**
 Outcome of one `write` or `flush` call.
 */
export type WriteOutcome = 'never' | 'reject' | 'resolve' | 'throw';

/**
 Per-call outcome sequence for one hook: `head` in call order, then `tail`
 forever.
 */
export type HookScript<Outcome extends string,> = {
  readonly head: readonly Outcome[];
  readonly tail: Outcome;
};

/**
 Full script of one fake sink. `flush` absent means the sink exposes no flush
 hook, which the logger treats as "nothing to drain".
 */
export type SinkScript = {
  readonly flush?: HookScript<WriteOutcome>;
  readonly verify: HookScript<VerifyOutcome>;
  readonly write: HookScript<WriteOutcome>;
};

/**
 Reads the outcome a hook script assigns to one call.

 @param script - Hook script to read.

 @param callIndex - Zero-based call number.

 @returns Head entry at that index, or the tail once the head is exhausted.

 @example
 ```ts
 outcomeAt({ script: { head: ['reject'], tail: 'resolve' }, callIndex: 3 }); // 'resolve'
 ```
 */
export function outcomeAt<Outcome extends string,>(
  {
    script,
    callIndex,
  }: {
    readonly script: HookScript<Outcome>;
    readonly callIndex: number;
  },
): Outcome {
  return script.head[callIndex] ?? script.tail;
}

/**
 Renders one hook script the way a counterexample should read.

 @param script - Hook script to render.

 @returns Text such as `[reject, resolve*]`.

 @example
 ```ts
 formatHookScript({ script: { head: ['reject'], tail: 'resolve' } }); // '[reject, resolve*]'
 ```
 */
export function formatHookScript<Outcome extends string,>(
  { script, }: { readonly script: HookScript<Outcome>; },
): string {
  return `[${[
    ...script.head,
    `${script.tail}*`,
  ].join(', ',)}]`;
}

/**
 Renders a whole sink script with its index.

 @param index - Sink index in the logger's sink list.

 @param script - Script to render.

 @returns One line naming the sink and every hook script.

 @example
 ```ts
 formatSinkScript({ index: 2, script });
 // 'sink 2: verify [resolve-true*] write [reject, resolve*] flush absent'
 ```
 */
export function formatSinkScript(
  {
    index,
    script,
  }: {
    readonly index: number;
    readonly script: SinkScript;
  },
): string {
  /**
   Flush column: the hook script, or `absent` when the sink exposes none.
   */
  const flushText = (script.flush === undefined)
    ? 'absent'
    : formatHookScript({ script: script.flush, },);
  return `sink ${index}: verify ${formatHookScript({ script: script.verify, },)} write ${
    formatHookScript({ script: script.write, },)
  } flush ${flushText}`;
}

//endregion Scripts

//region Trace

/**
 One observation the fake sink appends to the shared trace: a hook was
 called, or the outcome it returned settled as the logger sees it (never for
 `never`, immediately for `throw`).
 */
export type SinkTraceEvent = {
  readonly callIndex: number;
  readonly hook: HookName;
  readonly outcome: VerifyOutcome | WriteOutcome;
  readonly phase: 'called' | 'settled';
  /**
   Rejection message when a scripted rejection settled.
   */
  readonly reason?: string;
  readonly record?: LogRecord;
  readonly sink: number;
};

/**
 Gate deciding when a settleable outcome reaches the logger. The identity
 gate returns the promise unchanged; the properties pass a fast-check
 scheduler's `schedule`.
 */
export type ScheduleGate = <Value,>(
  gated: {
    readonly promise: Promise<Value>;
    readonly label: string;
  },
) => Promise<Value>;

/**
 Identity gate: outcomes settle as soon as the microtask queue allows.

 @param promise - Promise to pass through.

 @returns The same promise.

 @example
 ```ts
 const sink = createScriptedSink({ index: 0, script, schedule: identityGate, trace });
 ```
 */
export function identityGate<Value,>({ promise, }: { readonly promise: Promise<Value>; },): Promise<Value> {
  return promise;
}

//endregion Trace

//region Factory

/**
 Scripted sink plus the bookkeeping a property reads back.
 */
export type ScriptedSink = {
  /**
   Records handed to `write`, in call order, regardless of outcome.
   */
  readonly attempts: readonly LogRecord[];
  /**
   Records whose `write` outcome was `resolve`, in call order.
   */
  readonly delivered: readonly LogRecord[];
  readonly index: number;
  /**
   Reasons of every scripted rejection that settled, in settlement order.
   */
  readonly rejections: readonly string[];
  readonly script: SinkScript;
  readonly sink: Sink;
};

/**
 Builds a `Sink` whose hooks follow `script`, appending to `trace`.

 @param index - Sink index in the logger's sink list, used in every trace
 event and in the counterexample text.

 @param script - Per-hook outcome scripts.

 @param schedule - Gate deciding when settleable outcomes reach the logger.

 @param trace - Shared event list the sink appends to.

 @returns Sink plus its attempted and delivered records.

 @example
 ```ts
 const trace: SinkTraceEvent[] = [];
 const fake = createScriptedSink({ index: 0, script, schedule: identityGate, trace });
 const { logger } = createLogger({ sinks: [fake.sink] });
 ```
 */
export function createScriptedSink(
  {
    index,
    script,
    schedule,
    trace,
  }: {
    readonly index: number;
    readonly script: SinkScript;
    readonly schedule: ScheduleGate;
    readonly trace: SinkTraceEvent[];
  },
): ScriptedSink {
  /**
   Records handed to `write`, in call order.
   */
  const attempts: LogRecord[] = [];
  /**
   Records whose write resolved.
   */
  const delivered: LogRecord[] = [];
  /**
   Calls made so far per hook, so each call reads its own script entry.
   */
  const callCounts = {
    flush: 0,
    verify: 0,
    write: 0,
  };

  /**
   Reasons of every scripted rejection that settled, in settlement order.
   */
  const rejections: string[] = [];

  /**
   Appends a trace event.

   @param event - Event to append.
   */
  function note(event: SinkTraceEvent,): void {
    trace.push(event,);
  }

  /**
   Attaches a handler to a raw outcome promise so a scripted rejection is
   never unhandled while a scheduler holds it, recording the reason.

   @param promise - Raw outcome promise.
   */
  async function observeRejection({ promise, }: { readonly promise: Promise<unknown>; },): Promise<void> {
    try {
      await promise;
    }
    catch (error: unknown) {
      rejections.push(caughtValueText(error,),);
    }
  }

  /**
   Appends the `settled` event once `gated` settles, swallowing its rejection
   (the logger observes the rejection through its own handle).

   @param gated - Gated outcome promise.

   @param event - Event fields to reuse with the settled phase.
   */
  async function noteSettled(
    {
      gated,
      event,
    }: {
      readonly gated: Promise<unknown>;
      readonly event: Omit<SinkTraceEvent, 'phase'>;
    },
  ): Promise<void> {
    try {
      await gated;
    }
    catch (error: unknown) {
      // The rejection is the scripted outcome and the logger reports it; the
      // reason rides on the trace so a debugging session can tell a scripted
      // rejection from a real bug.
      note({
        ...event,
        phase: 'settled',
        reason: caughtValueText(error,),
      },);
      return;
    }
    note({
      ...event,
      phase: 'settled',
    },);
  }

  /**
   Runs one hook call: reads the scripted outcome, notes the call, and
   returns (or throws) what the script says.

   @param hook - Hook being called.

   @param outcome - Scripted outcome for this call.

   @param settled - Value a `resolve` outcome produces.

   @param record - Record for write calls, threaded into the trace.

   @returns Gated promise for settleable outcomes.

   @throws Error for the `throw` outcome, synchronously.
   */
  function perform<Value,>(
    {
      hook,
      outcome,
      settled,
      record,
    }: {
      readonly hook: HookName;
      readonly outcome: VerifyOutcome | WriteOutcome;
      readonly settled: Value;
      readonly record?: LogRecord;
    },
  ): Promise<Value> {
    /**
     Zero-based call number for this hook.
     */
    const callIndex = callCounts[hook];
    callCounts[hook] += 1;
    /**
     Trace fields shared by the called and settled events.
     */
    const event: Omit<SinkTraceEvent, 'phase'> = {
      callIndex,
      hook,
      outcome,
      sink: index,
      ...((record === undefined) ? {} : { record, }),
    };
    note({
      ...event,
      phase: 'called',
    },);
    if (outcome === 'throw')
      throw new Error(`scripted ${hook} throw (sink ${index}, call ${callIndex})`,);
    if (outcome === 'never')
      return Promise.withResolvers<Value>()
        .promise;
    /**
     Raw outcome promise before the gate.
     */
    const raw: Promise<Value> = (outcome === 'reject')
      ? Promise.reject(new Error(`scripted ${hook} rejection (sink ${index}, call ${callIndex})`,),)
      : Promise.resolve(settled,);
    // A scheduler may hold the raw promise without a handler until it decides
    // to release it; observing it here keeps a scripted rejection from being
    // reported as unhandled in the meantime.
    void observeRejection({ promise: raw, },);
    /**
     Outcome as the logger sees it, released by the gate.
     */
    const gated = schedule({
      promise: raw,
      label: `sink ${index} ${hook} #${callIndex}`,
    },);
    void noteSettled({
      gated,
      event,
    },);
    return gated;
  }

  /**
   Scripted `verify`.

   @returns Availability the script assigns to this call.
   */
  function verify(): Promise<boolean> {
    /**
     Scripted outcome for this verify call.
     */
    const outcome = outcomeAt({
      script: script.verify,
      callIndex: callCounts.verify,
    },);
    return perform({
      hook: 'verify',
      outcome,
      settled: outcome === 'resolve-true',
    },);
  }

  /**
   Scripted `write`; records the attempt, and the delivery when it resolves.

   @param record - Record the logger hands over.
   */
  function write(record: LogRecord,): Promise<void> {
    /**
     Scripted outcome for this write call.
     */
    const outcome = outcomeAt({
      script: script.write,
      callIndex: callCounts.write,
    },);
    attempts.push(record,);
    if (outcome === 'resolve')
      delivered.push(record,);
    return perform({
      hook: 'write',
      outcome,
      settled: undefined,
      record,
    },);
  }

  /**
   Scripted `flush` hook.
   */
  function flush(): Promise<void> {
    /**
     Scripted outcome for this flush call; the hook exists only when scripted.
     */
    const outcome = outcomeAt({
      script: script.flush ?? {
        head: [],
        tail: 'resolve',
      },
      callIndex: callCounts.flush,
    },);
    return perform({
      hook: 'flush',
      outcome,
      settled: undefined,
    },);
  }

  /**
   Sink handed to the logger; the flush hook is spread in only when scripted,
   because an optional property cannot hold an explicit `undefined`.
   */
  const sink: Sink = {
    ...((script.flush === undefined) ? {} : { flush, }),
    verify,
    write,
  };

  return {
    attempts,
    delivered,
    index,
    rejections,
    script,
    sink,
  };
}

//endregion Factory
