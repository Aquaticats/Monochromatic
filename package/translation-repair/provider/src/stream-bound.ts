import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Stream bound
// CLASS ONE HUNDRED FORTY-EIGHT (hulicaijia29, 2026-09-26). Between 06:05 and
// 06:55 UTC Bedrock's `google.gemma-4-e2b` went from a 1 s stream to 219 to
// 263 s: a first frame at about 15 s, a frame every 15 s after it, content
// trickling or never coming. 24 calls ran to the 360 s deadline, and with
// Synthetic and Hyper dry every round needed that seat for its quorum, so the
// entry took 85 minutes against hulicaijia28's 24. Across the 200 newest pass
// logs no Bedrock Gemma completion without that signature took longer than
// 27,340 ms of 68,235, while completions with it ran 25 to 304 s. No other
// model separates this way (DeepSeek V4.1 Flash, GLM-5.3-Flash, Kimi-K3 and
// Qwen3.8-27B complete healthy streams at 200 to 350 s), so the bound is a
// card's own measurement, not one number for every call.
//
// SEPARATE FROM THE CALL DEADLINE because the two answer different
// questions. The deadline bounds any call at all; this bound says the
// provider has stopped serving the model at its measured pace, which the
// router answers by holding that provider out for that model.

/**
 Reason a call is cut at its card's stream bound.

 @example
 ```ts
 controller.abort(new StreamBoundError({ label: 'google.gemma-4-e2b', boundMs: 60_000, },),);
 ```
 */
export class StreamBoundError extends Error {
  /**
   Declares this message safe to print whole: a model id the pipeline chose
   and a millisecond count.
   */
  readonly messageNamesOnly: true = true;

  /**
   Model the call went to.
   */
  readonly label: string;

  /**
   Bound the call ran past.
   */
  readonly boundMs: number;

  /**
   @param label - model the call went to

   @param boundMs - bound the call ran past
   */
  constructor(
    {
      label,
      boundMs,
    }: {
      readonly label: string;
      readonly boundMs: number;
    },
  ) {
    super(`Bound: ${label} ran past its measured ${String(boundMs,)}ms stream bound`,);
    this.name = 'StreamBoundError';
    this.label = label;
    this.boundMs = boundMs;
  }
}

/**
 Handle over one armed stream bound.
 */
export type StreamBound = Disposable & {
  /**
   Signal the call honours: aborts on the caller's abort or at the bound.
   */
  readonly callSignal: AbortSignal;
};

/**
 Arms a stream bound over one call, forwarding the caller's abort into it.

 @param signal - caller signal whose abort forwards into the call

 @param boundMs - the card's measured bound

 @param label - model the call goes to

 @mutates signal - one abort listener registers and detaches on dispose

 @returns Call signal plus disposal of the timer and the listener

 @example
 ```ts
 using bound = armStreamBound({ signal, boundMs: 60_000, label: servedId, },);
 ```
 */
export function armStreamBound(
  {
    signal,
    boundMs,
    label,
  }: ForeignBorrowed<{
    readonly signal: AbortSignal;
    readonly boundMs: number;
    readonly label: string;
  }>,
): StreamBound {
  /**
   Controller owning this call's teardown at the bound.
   */
  const controller = new AbortController();

  /**
   Forwards the caller's abort, keeping its reason.
   */
  function forwardCallerAbort(): void {
    controller.abort(signal.reason,);
  }
  if (signal.aborted)
    forwardCallerAbort();
  signal.addEventListener(
    'abort',
    forwardCallerAbort,
    { once: true, },
  );

  /**
   Timer cutting the call at the bound.
   */
  const timer = setTimeout(
    function onBound() {
      controller.abort(new StreamBoundError({
        label,
        boundMs,
      },),);
    },
    boundMs,
  );

  return {
    callSignal: controller.signal,
    [Symbol.dispose](): void {
      clearTimeout(timer,);
      signal.removeEventListener(
        'abort',
        forwardCallerAbort,
      );
    },
  };
}

/**
 Whether a failure, or anything in its cause chain, is a stream bound cut.

 @param error - whatever the call raised

 @returns True where the call was cut at its card's bound

 @example
 ```ts
 if (isStreamBoundCut({ error, },)) hold();
 ```
 */
export function isStreamBoundCut({ error, }: { readonly error: unknown; },): boolean {
  /**
   Failure under inspection, walking down the cause chain.
   */
  const cursor: { current: unknown; } = { current: error, };
  /**
   Failures seen, so a cause chain that loops ends.
   */
  const seen = new Set<unknown>();
  while (Error.isError(cursor.current,) && (!seen.has(cursor.current,))) {
    if (cursor.current instanceof StreamBoundError)
      return true;
    /**
     Failure under inspection, narrowed to an error.
     */
    const current: Error = cursor.current;
    seen.add(current,);
    cursor.current = current.cause;
  }
  return false;
}

//endregion Stream bound
