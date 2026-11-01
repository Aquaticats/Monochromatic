import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Call deadline
// A per-call deadline armed where execution actually begins. Arming at
// dispatch time starved a whole benchmark run: every fan-out call's timer
// started simultaneously while the per-model limiter ran only a few, so
// queued calls burned their entire budget waiting for a slot and expired in
// one synchronized wall. The client arms this inside the limiter slot
// instead, so only the exchange itself counts against the budget.

/**
 Reason a call is forfeited at its deadline.

 A CLASS RATHER THAN A BARE ERROR, so a bench row or a log line can carry the
 deadline through `refusalText` and still say what happened: the message
 names the call's label, which is a model id or a stage name, and a count.

 @example
 ```ts
 controller.abort(new CallTimeoutError({ label: modelId, timeoutMs: 600_000, },),);
 ```
 */
export class CallTimeoutError extends Error {
  /**
   Declares this message safe to print whole: a label the pipeline chose and
   a millisecond count, nothing read from any text.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param label - what the call was for, a model id or a stage name

   @param timeoutMs - deadline the call exceeded
   */
  constructor(
    {
      label,
      timeoutMs,
    }: {
      readonly label: string;
      readonly timeoutMs: number;
    },
  ) {
    super(`Timeout: ${label} exceeded its ${String(timeoutMs,)}ms deadline`,);
    this.name = 'CallTimeoutError';
  }
}

/**
 Deadline handle armed for one model call.
 */
export type CallDeadline = Disposable & {
  /**
   Signal the call must honor;
   aborts on caller abort or deadline expiry, whichever fires first.
   */
  readonly callSignal: AbortSignal;
};

/**
 Arms one per-call deadline:
 a locally owned controller aborted by a plain timer or by the caller's
 signal, whichever fires first.
 `AbortSignal.any` with an `AbortSignal.timeout` source never aborts on
 Node 26.5.0, and a bare `Promise.race` deadline would orphan the hung
 exchange on its per-model limiter slot,
 so a plain timer aborts the controller and tears the stream down.
 Disposal clears the timer and detaches the caller-abort listener.

 @param signal - caller signal whose abort forwards into the call

 @param timeoutMs - deadline granted to this call

 @param label - names the call in the deadline's abort reason

 @mutates signal - one abort listener registers via
 signal.addEventListener and detaches on dispose via
 signal.removeEventListener, and forwarding an abort into the call
 controller retains the caller's reason per
 DOM commit 5796f716 AbortController abort steps retain reason

 @returns Call signal plus disposal of timer and listener

 @example
 ```ts
 using deadline = armCallDeadline({ signal, timeoutMs: 600_000, label: modelId, },);
 ```
 */
export function armCallDeadline(
  {
    signal,
    timeoutMs,
    label,
  }: ForeignBorrowed<{
    readonly signal: AbortSignal;
    readonly timeoutMs: number;
    readonly label: string;
  }>,
): CallDeadline {
  /**
   Controller owning this call's teardown.
   */
  const callController = new AbortController();

  /**
   Forwards the caller's abort into this call so user steering always
   wins over an in-flight exchange.
   */
  function forwardCallerAbort(): void {
    callController.abort(signal.reason,);
  }
  if (signal.aborted)
    forwardCallerAbort();
  signal.addEventListener(
    'abort',
    forwardCallerAbort,
    { once: true, },
  );

  /**
   Timer forfeiting the call at its deadline.
   */
  const deadline = setTimeout(
    function onDeadline() {
      callController.abort(new CallTimeoutError({
        label,
        timeoutMs,
      },),);
    },
    timeoutMs,
  );

  return {
    callSignal: callController.signal,
    [Symbol.dispose](): void {
      clearTimeout(deadline,);
      signal.removeEventListener(
        'abort',
        forwardCallerAbort,
      );
    },
  };
}

//endregion Call deadline
