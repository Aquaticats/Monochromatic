/**
 What the stand-in Pi does with a prompt in the scenarios about a 429: a run Pi's own automatic retry recovers inside its one turn,
 and one it gives up on. Both follow a real Pi 1.5.2 as the feed of
 `pi-runs/kimi-review-pi-shared-subagent-driver-takes-the-launcher-fixes-round-212-1/feed.jsonl` recorded it on 2026-11-11: each
 attempt is a `turn_start` and a failed assistant message, an `auto_retry_start` follows the failure it retries, and Pi settles
 only once its own retry has ended.

 @module
 */

import {
  answer,
  assistantFailure,
  assistantReply,
  emit,
  PI_AUTOMATIC_RETRIES,
  RATE_LIMIT_ERROR_TEXT,
  RECOVERED_REPLY,
} from './stand-in-pi-records-fixture.ts';

/**
 The first delay Pi's own automatic retry uses, which is the `retry.baseDelayMs` default of Pi 1.5.2 (`docs/settings.md`).
 */
const PI_AUTOMATIC_RETRY_BASE_MILLISECONDS = 2_000;

/**
 How many of Pi's own retries answer no failure at all, which is what Pi writes for the attempt its retry ended on.
 */
const NO_RETRY = 0;

/**
 How long Pi waits before one of its own attempts, which doubles with each attempt: `retryDelayMs` in
 `@earendil-works/pi-ai/dist/utils/retry.js`.

 @param attempt - which of Pi's own attempts this delay comes before, counting from one

 @returns the milliseconds Pi waits
 */
function piRetryDelayMilliseconds(attempt: number,): number {
  return PI_AUTOMATIC_RETRY_BASE_MILLISECONDS * (2 ** (attempt - 1));
}

/**
 Writes one attempt of Pi's own run, and the retry it asks for after it when it asks for one.

 @param retriedBy - which of Pi's own retries this failure is answered by, or none when Pi gives up here
 */
function playFailedAttempt(retriedBy: number,): void {
  emit({ type: 'turn_start', },);
  emit(assistantFailure(RATE_LIMIT_ERROR_TEXT,),);
  if (retriedBy === NO_RETRY) {
    return;
  }
  emit({
    type: 'auto_retry_start',
    attempt: retriedBy,
    maxAttempts: PI_AUTOMATIC_RETRIES,
    delayMs: piRetryDelayMilliseconds(retriedBy,),
    errorMessage: RATE_LIMIT_ERROR_TEXT,
  },);
}

/**
 What each scenario about a 429 does with a prompt.
 */
export const rateLimitPlays: ReadonlyMap<string, (id: unknown,) => void> = new Map<string, (id: unknown,) => void>([
  [
    'rate-limit-recovered-by-pi',
    function recoverFromTheRateLimit(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      playFailedAttempt(1,);
      emit({ type: 'turn_start', },);
      emit(assistantReply(RECOVERED_REPLY,),);
      emit({
        type: 'auto_retry_end',
        success: true,
        attempt: 1,
      },);
      emit({ type: 'agent_end', },);
      emit({ type: 'agent_settled', },);
    },
  ],
  [
    'rate-limit-retries-exhausted',
    function spendEveryRetryOnTheRateLimit(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      for (let retriedBy = 1; retriedBy <= PI_AUTOMATIC_RETRIES; retriedBy += 1) {
        playFailedAttempt(retriedBy,);
      }
      // The attempt Pi's last retry made, after which it gives up on the error.
      playFailedAttempt(NO_RETRY,);
      emit({
        type: 'auto_retry_end',
        success: false,
        attempt: PI_AUTOMATIC_RETRIES,
        finalError: RATE_LIMIT_ERROR_TEXT,
      },);
      emit({ type: 'agent_end', },);
      emit({ type: 'agent_settled', },);
    },
  ],
],);
