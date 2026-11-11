/**
 The records the stand-in Pi writes: one JSON line each, the runs a scenario makes of a prompt, and the texts those runs carry.
 Kept apart from what each scenario does with a prompt, so both files stay within the code budget a file is allowed.

 @module
 */

import { stdout, } from 'node:process';
import {
  FEED_PREVIEW_CHARACTERS,
  type PiRecord,
} from '../dist/final/node/index.mjs';
import { seen, } from './stand-in-pi-log-fixture.ts';

/**
 The error text the model-error scenarios end with.
 */
export const MODEL_ERROR_TEXT = 'no key for this provider';

/**
 The reply of the retry that recovers from a model error.
 */
export const RECOVERED_REPLY = 'recovered';

/**
 A provider failure a retry can fix, shaped as one real run's feed recorded it on 2026-11-09
 (`pi-runs/repo-read-and-index-route-findings-on-main-1/summary.json`).
 */
export const TRANSIENT_ERROR_TEXT = 'Request timed out.';

/**
 A failure no retry can fix, shaped as three real runs' feeds recorded it on 2026-11-09
 (`pi-runs/toml-edit-wrapped-values-validated-1/summary.json`): the provider is not configured, so another attempt cannot
 reach a different answer.
 */
export const PERMANENT_ERROR_TEXT = 'Provider is not configured: neuralwatt';

/**
 A server error that asks for a wait longer than a retry takes, so no attempt is made inside its window.
 */
export const LONG_WAIT_ERROR_TEXT = '503 {"error":{"message":"Service Unavailable"},"retry_after":3600}';

/**
 A rate limit, which the coordinator measures apart from other failures.
 */
export const RATE_LIMIT_ERROR_TEXT = '429 {"error":{"message":"Too many requests"}}';

/**
 The limit on requests running at once, exactly as Neuralwatt sent it, recorded in
 `pi-runs/release-wait-reports-a-passed-run-as-passed-1/summary.json` on 2026-11-11.

 Its own text links a page under `/billing`, which Pi reads as a spent quota and never retries (its
 `NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN` holds `billing`), while the failure says in the same body that a slot frees and the
 request can go again.
 */
export const CONCURRENCY_LIMIT_ERROR_TEXT = '429: {"type":"rate_limit_error","code":"concurrent_budget_exceeded","message":"Concurrent limit reached for deepseek-ai/DeepSeek-V4.1-Flash: 6/6 slots in use. Wait for a request to complete and retry. Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it renews. Turn auto-renew back on to restore your account level\'s limits: https://docs.neuralwatt.com/billing/account-levels#overage-with-auto-renew-off","retry_strategy":{"type":"concurrent_drain","suggested_initial_delay_s":1,"max_delay_s":30,"backoff":"exponential","backoff_base":2,"jitter":true},"retryable":true,"context":{"budget":6,"in_flight":6,"model":"deepseek-ai/DeepSeek-V4.1-Flash","limit_type":"concurrent","tpm_window_tokens":0,"tpm_limit":5242880,"floor":5,"burst":6,"burst_shed":false}}';

/**
 Synthetic's 429, exactly as it was recorded in `pi-runs/kimi-review-merge-route-round-9-1/summary.json` on 2026-11-11: the
 subscription's quota is spent, so another attempt on Synthetic reaches the same answer.
 */
export const SPENT_QUOTA_ERROR_TEXT =
  '429 "You\'ve exceeded your subscription rate limits. Upgrade, or try again later. You can view your usage at https://synthetic.new/billing"';

/**
 The text the standard-error scenario writes, so a test can see the copy reach its file.
 */
export const STANDARD_ERROR_TEXT = 'the stand-in wrote this to its standard error';

/**
 One sentence of the long reply.
 */
const LONG_REPLY_SENTENCE = 'a reply long enough to need cutting.';

/**
 A reply longer than a feed line keeps, so a summary that held only the preview would show.
 */
export const LONG_REPLY: string = `${LONG_REPLY_SENTENCE} `.repeat(
  Math.ceil((FEED_PREVIEW_CHARACTERS.assistantText * 2) / (LONG_REPLY_SENTENCE.length + 1),),
);

/**
 Writes one record.

 @param record - fields of the record

 @example
 ```ts
 emit({ type: 'agent_settled', },);
 ```
 */
export function emit(record: PiRecord,): void {
  stdout.write(`${JSON.stringify(record,)}\n`,);
}

/**
 Answers a command successfully.

 @param id - command's identifier

 @param data - answer's data

 @example
 ```ts
 answer({ id: 'c1', },);
 ```
 */
export function answer({
  id,
  data = {},
}: {
  readonly id: unknown;
  readonly data?: PiRecord
},): void {
  emit({
    type: 'response',
    id,
    success: true,
    data,
  },);
}

/**
 The record of an assistant message that replied with text.

 @param text - assistant's reply

 @returns the `message_end` record

 @example
 ```ts
 emit(assistantReply('done',),);
 ```
 */
export function assistantReply(text: string,): PiRecord {
  return {
    type: 'message_end',
    message: {
      role: 'assistant',
      content: [{
        type: 'text',
        text,
      },],
      stopReason: 'stop',
      usage: {
        input: 1,
        output: 1,
        cost: { total: 0, },
      },
    },
  };
}

/**
 The record of an assistant message that ended in an error.

 @param error - what the model said

 @returns the `message_end` record

 @example
 ```ts
 emit(assistantFailure('Request timed out.',),);
 ```
 */
export function assistantFailure(error: string,): PiRecord {
  return {
    type: 'message_end',
    message: {
      role: 'assistant',
      content: [],
      stopReason: 'error',
      errorMessage: error,
      usage: {
        input: 1,
        output: 0,
        cost: { total: 0, },
      },
    },
  };
}

/**
 Writes a finished run.

 @param text - assistant's reply

 @example
 ```ts
 finishRun('done',);
 ```
 */
export function finishRun(text: string,): void {
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit(assistantReply(text,),);
  emit({ type: 'agent_end', },);
  emit({ type: 'agent_settled', },);
}

/**
 Writes an assistant message that ended in a model error, with no settle after it, so a retry that answers it can follow in the
 same burst. Only a settle takes the error as the run's own; a reply that succeeds after one clears it
 (`record-of-pi-run.unit.test.ts`, "a later reply that succeeded clears a model error Pi settled on"), and a retry after a
 settle is a new run rather than this one recovering.

 @param error - what the model said

 @example
 ```ts
 emitErrorWithoutSettling('Request timed out.',);
 ```
 */
export function emitErrorWithoutSettling(error: string,): void {
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit(assistantFailure(error,),);
  emit({ type: 'agent_end', },);
}

/**
 Writes several records in one write, so a test sees the driver read Pi's answer to a command and the events of the run it started
 in the same read, which is what a real Pi does when a run fails or replies before its answer is read.

 @param records - records to write, in order

 @example
 ```ts
 emitInOneWrite([{ type: 'response', id: 'c1', success: true, }, { type: 'agent_settled', },],);
 ```
 */
export function emitInOneWrite(records: readonly PiRecord[],): void {
  /**
   Each record as a line of its own, which one write carries together.
   */
  const lines = records.map(function asLine(record,) {
    return `${JSON.stringify(record,)}\n`;
  },);
  stdout.write(lines.join('',),);
}

/**
 Writes a whole run that ended in a model error, which Pi settles on.

 @param error - what the model said

 @example
 ```ts
 finishRunInError('Request timed out.',);
 ```
 */
export function finishRunInError(error: string,): void {
  emitErrorWithoutSettling(error,);
  emit({ type: 'agent_settled', },);
}

/**
 Plays a prompt as a run that ended in an error, and as a run that ends well after it when the scenario recovers.

 @param error - what the model said when the attempt failed

 @param recovers - whether the attempt after the first one ends well

 @returns what the scenario does with a prompt or a steer

 @example
 ```ts
 const play = modelErrorPlay({ error: 'Request timed out.', recovers: true, },);
 ```
 */
export function modelErrorPlay({
  error,
  recovers,
}: {
  readonly error: string;
  readonly recovers: boolean
},): (id: unknown,) => void {
  return function playModelError(id,) {
    answer({ id, },);
    if (recovers && (seen.prompts > 1)) {
      finishRun(RECOVERED_REPLY,);
      return;
    }
    finishRunInError(error,);
  };
}
