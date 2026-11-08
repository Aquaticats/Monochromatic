/**
 What the stand-in Pi does with a prompt in each test scenario, and how it writes records.

 @module
 */

import {
  env,
  exit,
  stdout,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import type { PiRecord, } from './driver-types.ts';

/**
 Status the crashing scenarios exit with.
 */
export const CRASH_STATUS = 3;

/**
 Milliseconds before the later crash, so the driver has started watching.
 */
const LATE_CRASH_MILLISECONDS = 300;

/**
 Milliseconds between the halves of a record written in two pieces.
 */
const SPLIT_WRITE_MILLISECONDS = 50;

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
const LINE_SEPARATOR = String.fromCodePoint(LINE_SEPARATOR_CODE_POINT,);

/**
 The behaviour chosen by the test.
 */
export const scenario = env.STAND_IN_PI_SCENARIO ?? 'normal';

/**
 How many prompts have arrived.
 */
export const seen = { prompts: 0, };

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
export function answer({ id, data = {}, }: { readonly id: unknown; readonly data?: PiRecord; },): void {
  emit({ type: 'response', id, success: true, data, },);
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
      content: [{ type: 'text', text, },],
      stopReason: 'stop',
      usage: { input: 1, output: 1, cost: { total: 0, }, },
    },
  },);
  emit({ type: 'agent_end', },);
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
    `${JSON.stringify({ type: 'message_end', message: { role: 'assistant', content: [{ type: 'text', text, },], }, },)}\n`,
  );
  /**
   Where the record is cut.
   */
  const cutAt = bytes.indexOf(Buffer.from('é',),) + SPLIT_OFFSET_AFTER_CHARACTER_START;
  stdout.write(bytes.subarray(0, cutAt,),);
  await wait(SPLIT_WRITE_MILLISECONDS,);
  stdout.write(bytes.subarray(cutAt,),);
}

/**
 What each scenario does with a prompt. A scenario missing here finishes a normal run.
 */
const promptPlays: ReadonlyMap<string, (id: unknown,) => Promise<void>> = new Map<string, (id: unknown,) => Promise<void>>([
  ['reject', async function rejectPrompt(id,) {
    emit({ type: 'response', id, success: false, error: 'no key for this provider', },);
  },],
  ['reject-inbox', async function rejectLaterPrompts(id,) {
    if (seen.prompts > 1) {
      emit({ type: 'response', id, success: false, error: 'refused', },);
      return;
    }
    answer({ id, },);
    finishRun('done',);
  },],
  ['handled', async function handlePrompt(id,) {
    answer({ id, data: { disposition: 'handled', }, },);
  },],
  ['crash-later', async function crashLater(id,) {
    answer({ id, },);
    await wait(LATE_CRASH_MILLISECONDS,);
    exit(CRASH_STATUS,);
  },],
  ['never-settles', async function neverSettle(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
  },],
  ['settle-gap', async function endWithoutSettling(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    emit({ type: 'agent_end', },);
  },],
  ['many-turns', async function startManyTurns(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    for (let turn = 0; turn < MANY_TURNS; turn += 1) {
      emit({ type: 'turn_start', },);
    }
  },],
  ['dialog', async function askDialog(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    emit({ type: 'extension_ui_request', id: 'dialog-1', method: 'select', title: 'Pick one', message: 'Which?', },);
    emit({ type: 'extension_ui_request', id: 'notice-1', method: 'notify', message: 'hello', },);
  },],
  ['tools', async function useTools(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    emit({ type: 'tool_execution_start', toolName: 'bash', args: { command: 'ls', }, },);
    emit({ type: 'tool_execution_end', toolName: 'bash', isError: false, result: 'ok', },);
    emit({ type: 'tool_execution_start', toolName: 'bash', args: { command: 'false', }, },);
    emit({ type: 'tool_execution_end', toolName: 'bash', isError: true, result: 'failed', },);
    emit({ type: 'auto_retry_start', },);
    emit({ type: 'agent_settled', },);
  },],
  ['noise', async function writeNoise(id,) {
    answer({ id, },);
    stdout.write('this line is not JSON\n',);
    finishRun('after the noise',);
  },],
  ['split-record', async function splitRecord(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    await emitFinalReplyInTwoPieces(`café line${LINE_SEPARATOR}separator`,);
    emit({ type: 'agent_settled', },);
  },],
],);

/**
 Plays out a prompt according to the scenario.

 @param id - command's identifier

 @example
 ```ts
 await playPrompt('c1',);
 ```
 */
export async function playPrompt(id: unknown,): Promise<void> {
  /**
   The scenario's play, if it has one.
   */
  const play = promptPlays.get(scenario,);
  if (play !== undefined) {
    await play(id,);
    return;
  }
  answer({ id, },);
  finishRun('done',);
}
