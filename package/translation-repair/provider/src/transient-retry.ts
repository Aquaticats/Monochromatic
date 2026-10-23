import { setTimeout as sleepFor, } from 'node:timers/promises';

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  deliveredCharsOf,
  exchangeFailureLogText,
  isSelfEndedStream,
  type ModelTransport,
  ProviderHttpError,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { isStreamBoundCut, } from './stream-bound.ts';
import { retryAfterHeaderMsOf, } from './retry-after-header.ts';
import { retryAfterMsOf, } from './retry-stated-wait.ts';

//region Transient retry
// Transport-level retry over the injected HTTP seam. Two transient failure
// shapes back off and try again on an equal-jitter ladder: retryable
// statuses (timeouts, throttles, upstream and gateway hiccups) and thrown
// transport failures (connection resets mid-stream). Caller aborts always
// propagate untouched: user steering is never weather.

/**
 Request Timeout: the server gave up waiting for the request.
 */
const HTTP_REQUEST_TIMEOUT = 408;

/**
 Too Many Requests: the provider throttled the call.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 Internal Server Error: from an inference stack this is routinely a
 transient upstream failure, not a request defect.
 */
const HTTP_INTERNAL_SERVER_ERROR = 500;

/**
 Bad Gateway: observed live when a 42-stream burst hit the provider.
 */
const HTTP_BAD_GATEWAY = 502;

/**
 Service Unavailable.
 */
const HTTP_SERVICE_UNAVAILABLE = 503;

/**
 Gateway Timeout.
 */
const HTTP_GATEWAY_TIMEOUT = 504;

/**
 Overloaded: Anthropic's own status for a service that cannot take the call
 now (its error list names `overloaded_error` at 529, read 2026-10-22).
 */
const HTTP_OVERLOADED = 529;

/**
 Statuses worth one more try:
 timeouts, throttles, upstream and gateway hiccups.
 A live 42-stream burst drew instant 502s on most calls while identical
 calls succeeded moments later, so these are transient by observation.
 */
const RETRYABLE_STATUSES: ReadonlySet<number> = new Set([
  HTTP_REQUEST_TIMEOUT,
  HTTP_TOO_MANY_REQUESTS,
  HTTP_INTERNAL_SERVER_ERROR,
  HTTP_BAD_GATEWAY,
  HTTP_SERVICE_UNAVAILABLE,
  HTTP_GATEWAY_TIMEOUT,
  HTTP_OVERLOADED,
],);

/**
 Retries granted past the first attempt for transient failures.
 Four retries ride out a burst gate:
 at pack-count concurrency the dispatch burst alone can draw a 502 storm,
 and the equal-jitter ladder spreads the survivors far enough apart.
 */
const TRANSIENT_RETRY_LIMIT = 4;

/**
 Base backoff window before the first retry; doubles per retry.
 */
const RETRY_BACKOFF_BASE_MS = 1_000;

/**
 Retry pacing knobs, injectable so tests run on tiny backoffs.

 @example
 ```ts
 const policy: RetryPolicy = { limit: 2, baseMs: 10, };
 ```
 */
export type RetryPolicy = {
  /**
   Retries granted past the first attempt.
   */
  readonly limit: number;

  /**
   Backoff window before the first retry; doubles per retry.
   */
  readonly baseMs: number;
};

/**
 Production retry pacing.
 */
export const DEFAULT_RETRY_POLICY: RetryPolicy = {
  limit: TRANSIENT_RETRY_LIMIT,
  baseMs: RETRY_BACKOFF_BASE_MS,
};

/**
 Logger root for the transport retry layer.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Computes one equal-jitter backoff:
 half the exponential window fixed, half random,
 so a burst of failing calls decorrelates instead of retrying in
 lockstep and re-triggering the burst gate.

 @param baseMs - full window granted before the first retry

 @param attempt - zero-based index of the attempt that just failed

 @returns Milliseconds to wait before the next attempt

 @example
 ```ts
 const backoffMs = backoffDelayMs({ baseMs: 1_000, attempt: 0, },);
 ```
 */
function backoffDelayMs(
  {
    baseMs,
    attempt,
  }: {
    readonly baseMs: number;
    readonly attempt: number;
  },
): number {
  /**
   Full exponential window for this attempt.
   */
  const windowMs = baseMs * (2 ** attempt);
  return Math.floor(windowMs / 2,)
    + Math.floor(Math.random() * (windowMs / 2),);
}

/**
 Longest stated wait the ladder sleeps in one backoff: the ladder's whole
 span, every retry's full window added together, rounded up to the next
 doubling. A refusal naming a wait past it is the provider naming its
 return, which no retry inside the ladder would live to see.

 NOT THE WIDEST WINDOW OF ANY ONE RETRY, as this said until the ledger's P13
 (2026-09-28): the retry after attempt `a` jitters inside `baseMs * 2 ** a`,
 so the last retry's window is half this reach (8 s under the shipped policy
 against a reach of 16 s). The value is the one the ladder has run on since
 `31e67a100` and the one its suite pins: 277 of 57,803 logged backoffs slept
 a stated wait between the two, each of which the old wording would have
 ended the ladder on and handed to the router's hold of the whole provider.
 No retry line names its model (ledger P12), so whether those retries then
 succeeded is unmeasured, and the behavior stays as it ran.

 @param policy - retry pacing in force

 @returns Milliseconds of the ladder's reach

 @example
 ```ts
 longestBackoffMs({ policy: DEFAULT_RETRY_POLICY, },);
 // => 16_000
 ```
 */
function longestBackoffMs({ policy, }: { readonly policy: RetryPolicy; },): number {
  return policy.baseMs * (2 ** policy.limit);
}

/**
 Sleeps one backoff, ending it the moment the caller aborts.

 LEDGER P13 (the whole-package audit, 2026-09-28): the ladder slept its whole
 backoff on a timer no signal could end and read the abort only after it, so
 a call its caller or its deadline had given up on held its seat for up to
 the ladder's reach, 16 s under the shipped policy.

 @param ms - backoff to sleep

 @param signal - caller's abort, joined with the exchange deadline where one
 is armed

 @throws {@link Error} naming an unreachable state, with the timer's own
 rejection as its cause, when the timer rejects while the caller has not
 aborted: `timers/promises` `setTimeout` rejects for a number of milliseconds
 and an abort signal only when that signal aborts

 @example
 ```ts
 await sleepBackoff({ ms: 2_000, signal, },);
 ```
 */
async function sleepBackoff(
  {
    ms,
    signal,
  }: {
    readonly ms: number;
    readonly signal: AbortSignal;
  },
): Promise<void> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: sleepBackoff.name,
    l,
  },);
  try {
    await sleepFor(
      ms,
      undefined,
      { signal, },
    );
  }
  catch (error) {
    // The caller's abort is the one early end a backoff has; the retry loop
    // reads it next and surfaces the failure that was being retried.
    if (!signal.aborted)
      throw new Error(
        'unreachable: the backoff timer rejected while the caller had not aborted, and a timer given a number '
          + 'of milliseconds and an abort signal rejects only when that signal aborts',
        { cause: error, },
      );
    rl.debug(`backoff of ${String(ms,)}ms ended by the caller's abort: ${String(error,)}`,);
  }
}

/**
 Outcome of one transport attempt with thrown failures captured as data,
 so the retry loop treats bad statuses and dropped connections uniformly.
 */
type ExchangeAttemptOutcome =
  | {
    /**
     The transport answered; the status may still be retryable.
     */
    readonly replied: true;

    /**
     Reply of this attempt.
     */
    readonly reply: Awaited<ReturnType<ModelTransport>>;
  }
    | {
    /**
     The transport threw mid-exchange, e.g. a connection reset while
     draining the stream.
     */
      readonly replied: false;

    /**
     Failure normalized to an Error for rethrow and logging.
     */
      readonly thrown: Error;
    };

/**
 What the ladder hands a caller for one attempt that delivered something and
 then failed.

 @example
 ```ts
 const attempt: AbandonedAttempt = { deliveredChars: 812, error, };
 ```
 */
export type AbandonedAttempt = {
  /**
   Raw wire characters the attempt delivered: a refused reply's whole body,
   or what the error that ended the stream says it had read.
   */
  readonly deliveredChars: number;

  /**
   Failure that ended the attempt.
   */
  readonly error: unknown;
};

/**
 Performs one transport attempt, capturing non-abort throws as data.
 A caller abort rethrows immediately:
 user steering is never a transient failure.

 @param transport - HTTP seam performing the attempt

 @param exchange - request handed to the transport verbatim

 @param verify - caller's read of a reply the status accepted

 @param onAbandonedAttempt - told of this attempt before anything else is
 decided about its failure, when it delivered something

 @mutates exchange - the delegated transport attempt may invoke getters
 while serializing, and the exchange's `signal` rides into the attempt;
 see the transport's own contract

 @returns Reply or captured failure, as data

 @example
 ```ts
 const outcome = await attemptExchange({ transport, exchange, },);
 ```
 */
async function attemptExchange(
  {
    transport,
    exchange,
    verify,
    onAbandonedAttempt,
  }: {
    readonly transport: ModelTransport;
    readonly exchange: ForeignBorrowed<Parameters<ModelTransport>[0]>;
    readonly verify?: (reply: Awaited<ReturnType<ModelTransport>>,) => void;
    readonly onAbandonedAttempt?: (attempt: AbandonedAttempt,) => Promise<void>;
  },
): Promise<ExchangeAttemptOutcome> {
  /**
   The reply the transport returned, kept so a check that refuses it can say
   how much it carried; empty until the transport answers.
   */
  const answered: Awaited<ReturnType<ModelTransport>>[] = [];
  try {
    /**
     Reply this attempt produced, not yet read.
     */
    const reply = await transport(exchange,);
    answered.push(reply,);

    // READ INSIDE THIS TRY ON PURPOSE. A body that is not a whole message is a
    // transport failure wearing a success status: the exchange returned 200 and
    // the message inside it stops early. Running the caller's check here drops
    // it into the same catch as a dropped connection, filtered by the same
    // predicate and paced by the same ladder. Reading it after this function
    // returned is what made a truncated stream the one transport failure that
    // never retried, while an HTTP 503 got the whole ladder.
    verify?.(reply,);

    return {
      replied: true,
      reply,
    };
  }
  catch (error) {
    // EVERY ATTEMPT THAT DELIVERED SOMETHING WAS BILLED (ledger P1,
    // 2026-09-28), and told here, before any of this catch's rethrows, because
    // the attempts this ladder ends without retrying leave by those. A reply
    // the check refused carried its whole body; a stream that ended early
    // says on its error what it had read. A failure that delivered nothing
    // reached no endpoint's meter and is not told. A caller whose telling
    // fails (a ledger that cannot be written) fails the call with that error.
    /**
     Raw characters this attempt delivered, or that nothing says.
     */
    const delivered = (answered[0] === undefined) ? deliveredCharsOf({ error, },) : answered[0]
      .bodyText
      .length;
    if (delivered !== 'nothing-known') {
      await onAbandonedAttempt?.({
        deliveredChars: delivered,
        error,
      },);
    }

    // A caller abort is steering, not weather; it must propagate untouched.
    if (exchange.signal
      .aborted)
      throw error;

    // NEITHER IS A TERMINATION THIS SYSTEM CHOSE. `drainBody` ends a runaway by
    // cancelling the reader and throwing, and it deliberately does NOT abort the
    // caller's signal, because the decision was ours rather than the caller's.
    // That leaves the `exchange.signal.aborted` check blind to it, so without
    // this the retry ladder re-dispatches the runaway once per remaining attempt: measured at five
    // transport calls over twelve seconds of backoff under the production
    // policy. A model that has begun repeating itself will repeat itself again,
    // so every one of those attempts pays the same cost the guard exists to
    // avoid, and the guard ends up multiplying the waste it was built to stop.
    //
    // ASKED THROUGH ONE PREDICATE rather than by naming a class here, because
    // the original defect was this check knowing about fewer guards than the
    // drain could throw. A guard added later updates `isSelfEndedStream` and
    // this site keeps working.
    if (isSelfEndedStream({ error, },))
      throw error;

    // NOR IS A STREAM THE CARD'S BOUND CUT (ledger P13, 2026-09-28). The bound
    // is armed around each attempt, inside the transport, so it never aborts
    // the signal the `exchange.signal.aborted` check reads; a retry would wait
    // in the same queue,
    // and the router has to see the cut to hold the provider out for the
    // model. Asked apart from `isSelfEndedStream` because the drain wraps a
    // bound cut in its cut-short error, which keeps the partial text, and
    // only this walk down the cause chain finds it there.
    if (isStreamBoundCut({ error, },))
      throw error;

    // NOR IS A REFUSAL THE PROVIDER STATED IN THE STREAM. A thrown
    // `ProviderHttpError` whose status no reply is retried on is the refusal
    // a reply of that status would be (a request it declined for what it
    // holds), which the ladder already returns unretried; retrying it here
    // spends every remaining attempt on the same answer.
    if ((error instanceof ProviderHttpError) && (!RETRYABLE_STATUSES.has(error.status,)))
      throw error;

    return {
      replied: false,
      thrown: Error.isError(error,)
        ? error
        : new Error(
          'the transport threw a value that is not an Error',
          { cause: error, },
        ),
    };
  }
}

/**
 Wait a refused reply asks for, from its body or its `retry-after` header, whichever is longer.

 @param reply - the refused attempt's reply

 @returns Milliseconds the reply asks the caller to wait, or zero

 @example
 ```ts
 const waitMs = statedWaitOf({ reply, },);
 ```
 */
function statedWaitOf({ reply, }: { readonly reply: TransportReply; },): number {
  return Math.max(
    retryAfterMsOf({ bodyText: reply.bodyText, },),
    // Conditional spread keeps the member absent instead of undefined.
    retryAfterHeaderMsOf(((reply.headers === undefined) ? {} : { headers: reply.headers, }),),
  );
}

/**
 Performs one exchange with bounded retry on transient failures:
 retryable statuses and thrown transport failures both back off and
 try again on an equal-jitter ladder.
 Success and non-retryable statuses return immediately;
 a caller abort stops retrying at the next boundary.

 @param provider - provider this exchange goes to, carried onto the failure raised when a caller abort stops the
 ladder over a retryable reply, and onto the exchange handed to the transport so the stream guard that cuts a silent
 stream can name whose stream it cut (ledger B804); the ladder cannot tell whose reply it is retrying

 @param transport - HTTP seam performing each attempt

 @param exchange - request repeated verbatim on every attempt

 @param policy - retry pacing; production default retries four times

 @param verify - caller's read of a reply the status accepted, run inside the
 attempt so an incomplete body counts as a failed attempt rather than a
 success the caller has to fail on afterwards. Absent leaves every 200 whole

 @param onAbandonedAttempt - told of every attempt that delivered something
 and then failed, the retried ones included, so a caller can record what the
 endpoint billed for it (ledger P1)

 @mutates exchange - delegated transport attempts may invoke getters while
 serializing, and the exchange's `signal` rides into each attempt;
 see the transport's own contract

 @returns First success or first non-retryable reply

 @throws {@link ProviderHttpError} when a caller abort stops the ladder after a retryable status

 @example
 ```ts
 const reply = await exchangeWithRetry({ provider: 'synthetic', transport, exchange, },);
 ```
 */
export async function exchangeWithRetry(
  {
    provider,
    transport,
    exchange,
    policy = DEFAULT_RETRY_POLICY,
    verify,
    onAbandonedAttempt,
  }: {
    readonly provider: ProviderName;
    readonly transport: ModelTransport;
    readonly exchange: ForeignBorrowed<Parameters<ModelTransport>[0]>;
    readonly policy?: RetryPolicy;
    readonly verify?: (reply: Awaited<ReturnType<ModelTransport>>,) => void;
    readonly onAbandonedAttempt?: (attempt: AbandonedAttempt,) => Promise<void>;
  },
): Promise<Awaited<ReturnType<ModelTransport>>> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: exchangeWithRetry.name,
    l,
  },);

  /**
   The exchange as the transport receives it, naming the provider it goes to
   unless the caller already named one.
   */
  const named = (exchange.provider === undefined)
    ? {
      ...exchange,
      provider,
    }
    : exchange;

  for (
    let attempt = 0;
    attempt <= policy.limit;
    attempt += 1
  ) {
    /* oxlint-disable no-await-in-loop -- attempts are inherently sequential; each retry depends on the previous failure */
    /**
     Reply or captured failure of this attempt.
     */
    const outcome = await attemptExchange({
      transport,
      exchange: named,
      // Conditional spread keeps the check absent instead of undefined, which
      // `exactOptionalPropertyTypes` refuses for an optional property.
      ...(verify === undefined
        ? {}
        : { verify, }),
      ...(onAbandonedAttempt === undefined
        ? {}
        : { onAbandonedAttempt, }),
    },);
    /* oxlint-enable no-await-in-loop */

    /**
     Whether attempts remain after this one.
     */
    const attemptsRemain = attempt < policy.limit;

    /**
     Reply of this attempt when the transport answered.
     */
    const reply = outcome.replied
      ? outcome.reply
      : undefined;

    /**
     Captured failure of this attempt when the transport dropped it.
     */
    const thrown = outcome.replied
      ? undefined
      : outcome.thrown;

    if (reply !== undefined) {
      if (!RETRYABLE_STATUSES.has(reply.status,))
        return reply;

      /**
       Wait the reply names for the provider's return, zero when it names
       none.
       */
      const statedWaitMs = statedWaitOf({ reply, },);

      /**
       Longest stated wait this ladder sleeps in one backoff.
       */
      const reachMs = longestBackoffMs({ policy, },);
      // A WAIT PAST THE LADDER'S REACH IS THE PROVIDER NAMING ITS RETURN, and
      // no retry inside this ladder would live to see it. Hyper's daily limit
      // answered "try again in 2h25m18s" on Huasheng, 2026-09-07; the ladder
      // read no wait in it and retried on its jitter 2,693 times over 2h53m.
      // The reply goes back as it is, for the router to hold the provider out
      // until the named instant.
      if (statedWaitMs > reachMs) {
        rl.warn(
          `${exchange.label}: HTTP ${String(reply.status,)} names its return in ${String(statedWaitMs,)}ms, past this `
            + `ladder's reach of ${String(reachMs,)}ms; ending the ladder`,
        );
        return reply;
      }
      if (!attemptsRemain)
        return reply;
    }
    else if (!attemptsRemain) {
      throw nonNullishOrThrow(thrown,);
    }

    /**
     Failure named in the retry log line.
     */
    const failureLabel = reply === undefined
      ? `transport failure: ${exchangeFailureLogText({ error: thrown, },)}`
      : `HTTP ${String(reply.status,)}`;

    /**
     Equal-jitter backoff for the coming retry, stretched to whatever wait
     the provider's refusal asked for.
     */
    const backoffMs = Math.max(
      backoffDelayMs({
        baseMs: policy.baseMs,
        attempt,
      },),
      (reply === undefined) ? 0 : statedWaitOf({ reply, },),
    );
    // THE CALL IS NAMED (ledger P12): 3,864 "stream ended without its [DONE]
    // terminator ... retrying" lines across the logs named no model, so none
    // could be pinned on a model or a provider.
    rl.warn(
      `${exchange.label}: ${failureLabel}; retrying in ${String(backoffMs,)}ms (attempt ${
        String(attempt + 1,)
      } of ${String(policy.limit + 1,)})`,
    );
    // oxlint-disable-next-line no-await-in-loop -- backoff must complete before the dependent retry
    await sleepBackoff({
      ms: backoffMs,
      signal: exchange.signal,
    },);
    // A caller abort during backoff stops the retry loop here; the aborted
    // signal would otherwise burn an attempt on a guaranteed rejection.
    if (exchange.signal
      .aborted) {
      if (reply !== undefined) {
        throw new ProviderHttpError({
          provider,
          status: reply.status,
          bodyText: reply.bodyText,
        },);
      }
      throw nonNullishOrThrow(thrown,);
    }
  }
  throw new Error('unreachable: retry loop returns or throws on its final attempt',);
}

//endregion Transient retry
