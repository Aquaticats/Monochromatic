/**
 Whole-call deadline bounding one complete search or fetch call.

 One call makes several provider requests when the fallback chain runs, and
 every one of them shares a single budget: the deadline here starts at call
 entry and aborts the chain wherever it currently is. A caller `AbortSignal`
 combines with the default timer, so whichever of the two expires first aborts
 the call and no caller can extend a call past the default.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

//region Logger

/**
 Logger root for pi-search-fetch after removing the package log shim.
 */
const wholeCallDeadlineLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'whole-call-deadline',
  l: wholeCallDeadlineLogger,
},);

//endregion Logger

//region Constants

/**
 Default budget for one complete call in milliseconds, 120 s.

 Sized so the 30 s gh child bound plus the 30 s DeepSeek request bound still
 leave a real 60 s tail for the Linkup and Exa fallback hops.
 */
export const WHOLE_CALL_DEADLINE_MILLISECONDS = 120_000;

//endregion Constants

//region Types

/**
 Operation name reported in deadline diagnostics.
 */
type WholeCallOperation = 'search' | 'fetch';

/**
 Error thrown when the whole-call deadline expires before the call settles.
 */
export class WholeCallDeadlineError extends Error {
  /**
   Effective deadline in milliseconds, the budget the call exceeded.
   */
  readonly deadlineMs: number;

  /**
   Build a deadline failure naming the operation and the exceeded budget.

   @param operation - operation name shown in the diagnostic

   @param deadlineMs - effective deadline in milliseconds the call exceeded
   */
  constructor(
    {
      operation,
      deadlineMs,
    }: {
      readonly operation: WholeCallOperation;
      readonly deadlineMs: number;
    },
  ) {
    super(
      `The Search Fetch ${operation} call exceeded its whole-call deadline of ${
        String(deadlineMs)
      } ms (120 s by default). The whole-call deadline bounds one complete call, including every fallback hop, and no caller signal extends it.`,
    );
    this.name = 'WholeCallDeadlineError';
    this.deadlineMs = deadlineMs;
  }
}

/**
 Handle releasing the deadline timer once the bounded call settles.
 */
type DeadlineTimerHandle = Disposable;

//endregion Types

//region Public API

/**
 Run one complete call under the combined caller and whole-call deadline.

 The call receives an `AbortSignal` that aborts when the caller signal fires or
 when the deadline expires, whichever happens first. The rejection surfaces the
 abort reason of whichever source fired first, so deadline expiry is a
 `WholeCallDeadlineError` naming the budget while caller cancellation keeps
 surfacing the caller's own abort reason.

 @param call - work to bound; receives the combined signal for every provider hop

 @param operation - operation name shown in deadline diagnostics

 @param signal - caller-owned cancellation signal, when supplied

 @param deadlineMs - deadline override in milliseconds, primarily for tests

 @returns call result when the call settles inside its budget

 @throws WholeCallDeadlineError when the deadline expires first

 @throws the caller's abort reason when the caller signal fires first

 @example
 ```ts
 await withWholeCallDeadline({
   operation: 'search',
   call: async ({ signal }) => searchWithFallback({ signal }),
 });
 ```
 */
export async function withWholeCallDeadline<T>(
  {
    call,
    operation,
    signal,
    deadlineMs = WHOLE_CALL_DEADLINE_MILLISECONDS,
  }: {
    /**
     Work to bound; receives the combined signal for every provider hop.
     */
    readonly call: (options: { readonly signal: AbortSignal }) => Promise<T>;
    /**
     Operation name shown in deadline diagnostics.
     */
    readonly operation: WholeCallOperation;
    /**
     Caller-owned cancellation signal, when supplied.
     */
    readonly signal?: AbortSignal;
    /**
     Deadline override in milliseconds, primarily for tests.
     */
    readonly deadlineMs?: number;
  },
): Promise<T> {
  /**
   Logger tagged for this bounded call.
   */
  const innerL = tagged({
    tag: withWholeCallDeadline.name,
    l,
  },);
  /**
   Deadline failure this call surfaces when its budget expires first.
   */
  const deadlineError = new WholeCallDeadlineError({
    operation,
    deadlineMs,
  },);
  /**
   Controller aborting the combined signal when the deadline expires.
   */
  const deadlineController = new AbortController();
  /**
   Timer handle releasing the budget once the call settles or fails.
   */
  using deadlineTimer = createDeadlineTimer({
    deadlineMs,
    onExpire: function abortAtWholeCallDeadline() {
      innerL.warn(
        `Search Fetch ${operation} exceeded its whole-call deadline of ${String(deadlineMs)} ms`,
      );
      deadlineController.abort(deadlineError,);
    },
  },);
  /**
   Request signal aborted by either the caller or the whole-call deadline.
   */
  const combinedSignal = signal === undefined
    ? deadlineController.signal
    : AbortSignal.any([
      signal,
      deadlineController.signal,
    ],);
  /**
   Rejection settled by the first abort, ahead of any slower call unwind.
   */
  const abortRejection = rejectOnAbort({ signal: combinedSignal, },);
  return await Promise.race([
    abortRejection,
    call({ signal: combinedSignal, },),
  ],);
}

//endregion Public API

//region Helpers

/**
 Arm the whole-call deadline timer and release it when disposed.

 @param deadlineMs - budget in milliseconds before the timer fires

 @param onExpire - callback aborting the bounded call at the deadline

 @returns disposable handle clearing the timer

 @example
 ```ts
 using deadlineTimer = createDeadlineTimer({ deadlineMs, onExpire });
 ```
 */
function createDeadlineTimer(
  {
    deadlineMs,
    onExpire,
  }: {
    readonly deadlineMs: number;
    readonly onExpire: () => void;
  },
): DeadlineTimerHandle {
  /**
   Underlying timer cleared when the bounded call settles.
   */
  const timer = setTimeout(
    onExpire,
    deadlineMs,
  );
  return {
    [Symbol.dispose]() {
      clearTimeout(timer,);
    },
  };
}

/**
 Bridge one abort event into a promise rejecting with the signal's abort reason.

 Created before the bounded call starts, so a signal already aborted at entry
 rejects ahead of any call progress and the race always reports the first abort
 source rather than a later wrapped provider failure. The description carries
 the rejection this promise settles with once the signal aborts.

 @param signal - combined signal whose abort settles the rejection

 @example
 ```ts
 await Promise.race([rejectOnAbort({ signal }), work]);
 ```
 */
function rejectOnAbort({ signal, }: { readonly signal: AbortSignal; }): Promise<never> {
  /**
   Pending rejection settled once by the abort event.
   */
  const {
    promise,
    reject,
  } = Promise.withResolvers<never>();
  if (signal.aborted) {
    reject(signal.reason,);
    return promise;
  }
  signal.addEventListener(
    'abort',
    function rejectAbortedCall() {
      reject(signal.reason,);
    },
    { once: true, },
  );
  return promise;
}

//endregion Helpers
