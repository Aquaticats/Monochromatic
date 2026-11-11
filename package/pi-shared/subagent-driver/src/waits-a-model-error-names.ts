/**
 The waits a provider's failure asks for: the one a server error names as `retry_after`, and the pair a limit on requests running
 at once names as its first delay and its ceiling. Also how long the driver waits before an attempt when a failure names none.

 The wait a failure names is read out of its own text rather than guessed, because the two kinds need different waits: a server
 error is waited out in seconds, and a full pool frees a slot in about the length of another run.

 @module
 */

import {
  MILLISECONDS_PER_SECOND,
  MODEL_ERROR_RETRY_BASE_SECONDS,
  MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS,
} from './driver-constants.ts';
import { secondsNamedBy, } from './scanning-a-failure-text.ts';

/**
 The wait a concurrency limit asked for, when its failure names one, or that it named neither wait nor ceiling.
 */
export type ConcurrencyWait =
  | { readonly asked: false }
    | {
      readonly asked: true;

    /**
     The first wait the failure suggests, in milliseconds.
     */
      readonly firstMilliseconds: number;

    /**
     The longest wait the failure allows, in milliseconds.
     */
      readonly maxMilliseconds: number
    };

/**
 The wait a provider asked for, when its failure names one, or that it named none.
 */
export type WaitAskedByProvider =
  | { readonly asked: false }
    | {
      readonly asked: true;
      readonly milliseconds: number
    };

/**
 The ways a failure writes the wait it asks for: a Cloudflare `520` body carries `"retry_after":60`, and an HTTP header carries
 `Retry-After: 60`.
 */
const WAIT_KEYS: readonly string[] = [
  'retry_after',
  'retry-after',
  'retry after',
];

/**
 The keys a concurrency limit writes the first wait it suggests under.
 */
const CONCURRENCY_FIRST_WAIT_KEYS: readonly string[] = ['suggested_initial_delay_s',];

/**
 The keys a concurrency limit writes the longest wait it allows under.
 */
const CONCURRENCY_MAX_WAIT_KEYS: readonly string[] = ['max_delay_s',];

/**
 Reads the wait a provider asked for, when its failure names one, in seconds.

 @param problem - what the model said when its message ended in an error

 @returns the milliseconds to wait, or that the failure named none

 @example
 ```ts
 const asked = waitAskedByProvider('{"retry_after":60}',);
 ```
 */
export function waitAskedByProvider(problem: string,): WaitAskedByProvider {
  /**
   The seconds the failure names, when it names any.
   */
  const named = secondsNamedBy({
    text: problem.toLowerCase(),
    keys: WAIT_KEYS,
  },);
  return named.named ? {
    asked: true,
    milliseconds: named.seconds * MILLISECONDS_PER_SECOND,
  } : { asked: false, };
}

/**
 The wait a concurrency limit asks for, when its failure names one.

 @param problem - what the model said when its message ended in an error

 @returns the first wait and the ceiling in milliseconds, or that the failure named neither

 @example
 ```ts
 const asked = concurrencyWaitOf('{"retry_strategy":{"suggested_initial_delay_s":10,"max_delay_s":120}}',);
 ```
 */
export function concurrencyWaitOf(problem: string,): ConcurrencyWait {
  /**
   The failure, folded once so every key below is a plain substring.
   */
  const text = problem.toLowerCase();
  /**
   The first wait the failure suggests.
   */
  const first = secondsNamedBy({
    text,
    keys: CONCURRENCY_FIRST_WAIT_KEYS,
  },);
  /**
   The longest wait the failure allows.
   */
  const longest = secondsNamedBy({
    text,
    keys: CONCURRENCY_MAX_WAIT_KEYS,
  },);
  if ((!first.named) && (!longest.named)) {
    return { asked: false, };
  }
  return {
    asked: true,
    firstMilliseconds: (first.named ? first.seconds : MODEL_ERROR_RETRY_BASE_SECONDS) * MILLISECONDS_PER_SECOND,
    maxMilliseconds: longest.named
      ? longest.seconds * MILLISECONDS_PER_SECOND
      : MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS,
  };
}

/**
 How long the driver waits before another attempt at a limit on how many requests run at once: the wait the provider suggests
 first, doubling with each attempt, never past the longest wait the provider allows.

 The growth is what both bodies recorded on 2026-11-11 name: `"backoff":"exponential","backoff_base":2`.

 @param problem - what the model said when its message ended in an error

 @param retryNumber - which attempt this delay comes before, counting from one

 @returns the milliseconds to wait

 @example
 ```ts
 const delay = concurrencyDelayMilliseconds({ problem: concurrencyLimitText, retryNumber: 2, },);
 ```
 */
export function concurrencyDelayMilliseconds({
  problem,
  retryNumber,
}: {
  readonly problem: string;
  readonly retryNumber: number
},): number {
  /**
   What the failure asks for, or the growth every such failure is assumed to want when it names nothing.
   */
  const asked = concurrencyWaitOf(problem,);
  /**
   The first wait, which the failure names or the ordinary first delay stands in for.
   */
  const firstMilliseconds = asked.asked
    ? asked.firstMilliseconds
    : (MODEL_ERROR_RETRY_BASE_SECONDS * MILLISECONDS_PER_SECOND);
  /**
   The longest wait, which the failure names or the ordinary largest delay stands in for.
   */
  const maxMilliseconds = asked.asked ? asked.maxMilliseconds : MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS;
  return Math.min(
    maxMilliseconds,
    firstMilliseconds * (2 ** (retryNumber - 1)),
  );
}

/**
 How long the driver waits before the next attempt: a delay that doubles with each attempt, raised to the wait the provider
 asked for when it named one, and never longer than the largest delay.

 The cap is what keeps the wait from growing without end: a failure naming no wait doubles from the first delay, and a provider
 asking for an hour is still waited out within the cap.

 @param problem - what the model said when its message ended in an error

 @param retryNumber - which attempt this delay comes before, counting from one

 @param baseSeconds - the first delay, in seconds

 @returns the milliseconds to wait

 @example
 ```ts
 const delay = retryDelayMilliseconds({ problem: 'Request timed out.', retryNumber: 2, baseSeconds: 2, },);
 ```
 */
export function retryDelayMilliseconds({
  problem,
  retryNumber,
  baseSeconds,
}: {
  readonly problem: string;
  readonly retryNumber: number;
  readonly baseSeconds: number
},): number {
  /**
   The delay the growth alone asks for, which a large attempt count pushes past every cap.
   */
  const grown = (baseSeconds * MILLISECONDS_PER_SECOND) * (2 ** (retryNumber - 1));
  /**
   The wait the provider asked for.
   */
  const asked = waitAskedByProvider(problem,);
  return Math.min(
    MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS,
    Math.max(
      grown,
      asked.asked ? asked.milliseconds : 0,
    ),
  );
}
