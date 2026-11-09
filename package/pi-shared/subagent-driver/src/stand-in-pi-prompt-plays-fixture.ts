/**
 What the stand-in Pi does with a prompt or a steer in each test scenario, and how it writes records.

 @module
 */

import { stdout, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  FEED_PREVIEW_CHARACTERS,
  POLL_MILLISECONDS,
  type PiRecord,
} from '../dist/final/node/index.mjs';
import {
  scenario,
  seen,
  writeLogLine,
} from './stand-in-pi-log-fixture.ts';

/**
 Status the crashing scenarios exit with.
 */
export const CRASH_STATUS = 3;

/**
 Milliseconds between the halves of a record written in two pieces.
 */
const SPLIT_WRITE_MILLISECONDS = 50;

/**
 How many looks at the inbox a delayed answer spans, so a driver that sends the next message without waiting for this answer has
 sent it before the answer comes.
 */
const LOOKS_A_DELAYED_ANSWER_SPANS = 3;

/**
 Milliseconds a delayed answer takes.
 */
export const SLOW_ANSWER_MILLISECONDS: number = POLL_MILLISECONDS * LOOKS_A_DELAYED_ANSWER_SPANS;

/**
 Turns the many-turns scenario starts, above any cap the tests set.
 */
const MANY_TURNS = 5;

/**
 Bytes into the two-byte character where the record is cut.
 */
const SPLIT_OFFSET_AFTER_CHARACTER_START = 1;

/**
 Code point of the Unicode line separator.
 */
const LINE_SEPARATOR_CODE_POINT = 0x20_28;

/**
 A Unicode line separator, valid inside a JSON string and not a line ending there.
 */
export const LINE_SEPARATOR: string = String.fromCodePoint(LINE_SEPARATOR_CODE_POINT,);

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
 A rate limit, which the coordinator measures apart from other failures.
 */
export const RATE_LIMIT_ERROR_TEXT = '429 {"error":{"message":"Too many requests"}}';

/**
 The text the standard-error scenario writes, so a test can see the copy reach its file.
 */
export const STANDARD_ERROR_TEXT = 'the stand-in wrote this to its standard error';

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
 Writes a finished run.

 @param text - assistant's reply
 */
function finishRun(text: string,): void {
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit({
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
  },);
  emit({ type: 'agent_end', },);
  emit({ type: 'agent_settled', },);
}

/**
 Writes an assistant message that ended in a model error, with no settle after it, so a retry that answers it can follow in the
 same burst. Only a settle takes the error as the run's own; a reply that succeeds after one clears it
 (`record-of-pi-run.unit.test.ts`, "a later reply that succeeded clears a model error Pi settled on"), and a retry after a
 settle is a new run rather than this one recovering.

 @param error - what the model said
 */
function emitErrorWithoutSettling(error: string,): void {
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit({
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
  },);
  emit({ type: 'agent_end', },);
}

/**
 Writes a whole run that ended in a model error, which Pi settles on.

 @param error - what the model said
 */
function finishRunInError(error: string,): void {
  emitErrorWithoutSettling(error,);
  emit({ type: 'agent_settled', },);
}

/**
 Writes a record in two pieces, split inside a two-byte character, holding a Unicode line separator.

 @param text - assistant's reply, which must hold the two-byte character
 */
async function emitFinalReplyInTwoPieces(text: string,): Promise<void> {
  /**
   The record's bytes.
   */
  const bytes = Buffer.from(
    `${JSON.stringify({
      type: 'message_end',
      message: {
        role: 'assistant',
        content: [{
          type: 'text',
          text,
        },],
      },
    },)}\n`,
  );
  /**
   Where the record is cut.
   */
  const cutAt = bytes.indexOf(Buffer.from('é',),) + SPLIT_OFFSET_AFTER_CHARACTER_START;
  stdout.write(bytes.subarray(
    0,
    cutAt,
  ),);
  await wait(SPLIT_WRITE_MILLISECONDS,);
  stdout.write(bytes.subarray(cutAt,),);
}

/**
 Answers a command after a delay, without holding up the reading of the next one, so a test can see whether the driver waited.

 @param id - command's identifier
 */
async function answerSlowly(id: unknown,): Promise<void> {
  await wait(SLOW_ANSWER_MILLISECONDS,);
  seen.answering -= 1;
  answer({ id, },);
}

/**
 Scenarios that take time over a prompt or a steer.
 */
const waitingPromptPlays: ReadonlyMap<string, (id: unknown,) => Promise<void>> = new Map<string, (id: unknown,) => Promise<void>>([
  [
    'split-record',
    async function splitRecord(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      await emitFinalReplyInTwoPieces(`café line${LINE_SEPARATOR}separator`,);
      emit({ type: 'agent_settled', },);
    },
  ],
],);

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
 What the other scenarios do with a prompt or a steer. A scenario missing from both maps finishes a normal run.
 */
const promptPlays: ReadonlyMap<string, (id: unknown,) => void> = new Map<string, (id: unknown,) => void>([
  [
    'reject',
    function rejectPrompt(id,) {
      emit({
        type: 'response',
        id,
        success: false,
        error: 'no key for this provider',
      },);
    },
  ],
  [
    'reject-inbox',
    function rejectLaterMessages(id,) {
      if (seen.prompts > 1) {
        emit({
          type: 'response',
          id,
          success: false,
          error: 'refused',
        },);
        return;
      }
      answer({ id, },);
      finishRun('done',);
    },
  ],
  [
    'handled',
    function handlePrompt(id,) {
      answer({
        id,
        data: { disposition: 'handled', },
      },);
    },
  ],
  [
    'never-settles',
    function neverSettle(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
    },
  ],
  [
    'silent-inbox',
    function acceptThenStaySilent(id,) {
      if (seen.prompts > 1) {
        return;
      }
      answer({ id, },);
      emit({ type: 'agent_start', },);
    },
  ],
  [
    'settle-gap',
    function endWithoutSettling(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      emit({ type: 'agent_end', },);
      void writeLogLine('marker agent_end',);
    },
  ],
  [
    'many-turns',
    function startManyTurns(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      for (let turn = 0; turn < MANY_TURNS; turn += 1) {
        emit({ type: 'turn_start', },);
      }
    },
  ],
  [
    'dialog',
    function askDialog(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      emit({
        type: 'extension_ui_request',
        id: 'dialog-1',
        method: 'select',
        title: 'Pick one',
        message: 'Which?',
      },);
      emit({
        type: 'extension_ui_request',
        id: 'notice-1',
        method: 'notify',
        message: 'hello',
      },);
    },
  ],
  [
    'tools',
    function useTools(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      emit({
        type: 'tool_execution_start',
        toolName: 'bash',
        args: { command: 'ls', },
      },);
      emit({
        type: 'tool_execution_end',
        toolName: 'bash',
        isError: false,
        result: 'ok',
      },);
      emit({
        type: 'tool_execution_start',
        toolName: 'bash',
        args: { command: 'false', },
      },);
      emit({
        type: 'tool_execution_end',
        toolName: 'bash',
        isError: true,
        result: 'failed',
      },);
      emit({ type: 'auto_retry_start', },);
      emit({ type: 'agent_settled', },);
    },
  ],
  [
    'noise',
    function writeNoise(id,) {
      answer({ id, },);
      stdout.write('this line is not JSON\n',);
      finishRun('after the noise',);
    },
  ],
  [
    'long-reply',
    function finishLong(id,) {
      answer({ id, },);
      finishRun(LONG_REPLY,);
    },
  ],
  [
    'model-error',
    function endInError(id,) {
      answer({ id, },);
      finishRunInError(MODEL_ERROR_TEXT,);
    },
  ],
  [
    'transient-error-then-success',
    function failOnceThenRecover(id,) {
      answer({ id, },);
      if (seen.prompts === 1) {
        finishRunInError(TRANSIENT_ERROR_TEXT,);
        return;
      }
      finishRun(RECOVERED_REPLY,);
    },
  ],
  [
    'always-transient-error',
    function failEveryAttempt(id,) {
      answer({ id, },);
      finishRunInError(TRANSIENT_ERROR_TEXT,);
    },
  ],
  [
    'permanent-error',
    function failEveryAttemptForGood(id,) {
      answer({ id, },);
      finishRunInError(PERMANENT_ERROR_TEXT,);
    },
  ],
  [
    'rate-limit-then-success',
    function hitARateLimitOnceThenRecover(id,) {
      answer({ id, },);
      if (seen.prompts === 1) {
        finishRunInError(RATE_LIMIT_ERROR_TEXT,);
        return;
      }
      finishRun(RECOVERED_REPLY,);
    },
  ],
  [
    'recovered-error',
    function retryAfterError(id,) {
      answer({ id, },);
      // The retry answers the error before Pi settles on it, which is what a recovering retry is: a settle in between would
      // leave the run's failure standing until the retry's reply cleared it, and a watch looking in that gap would end the run.
      emitErrorWithoutSettling(MODEL_ERROR_TEXT,);
      emit({ type: 'auto_retry_start', },);
      finishRun(RECOVERED_REPLY,);
    },
  ],
  [
    'crash-on-steer',
    function startThenWait(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
    },
  ],
  [
    'slow-answer',
    function answerAfterWaiting(id,) {
      seen.answering += 1;
      emit({ type: 'agent_start', },);
      void answerSlowly(id,);
    },
  ],
],);

/**
 Plays out a prompt or a steer according to the scenario.

 @param id - command's identifier

 @example
 ```ts
 await playPrompt('c1',);
 ```
 */
export async function playPrompt(id: unknown,): Promise<void> {
  /**
   The scenario's play if it takes time.
   */
  const waitingPlay = waitingPromptPlays.get(scenario,);
  if (waitingPlay !== undefined) {
    await waitingPlay(id,);
    return;
  }
  /**
   The scenario's play if it has one.
   */
  const play = promptPlays.get(scenario,);
  if (play !== undefined) {
    play(id,);
    return;
  }
  answer({ id, },);
  finishRun('done',);
}
