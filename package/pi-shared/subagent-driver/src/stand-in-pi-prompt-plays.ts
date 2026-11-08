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
export const scenario: string = env.STAND_IN_PI_SCENARIO ?? 'normal';

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
 Scenarios that take time over a prompt.
 */
const waitingPromptPlays: ReadonlyMap<string, (id: unknown,) => Promise<void>> = new Map<string, (id: unknown,) => Promise<void>>([
  [
    'crash-later',
    async function crashLater(id,) {
      answer({ id, },);
      await wait(LATE_CRASH_MILLISECONDS,);
      exit(CRASH_STATUS,);
    },
  ],
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
 What the other scenarios do with a prompt. A scenario missing from both maps finishes a normal run.
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
    function rejectLaterPrompts(id,) {
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
    'settle-gap',
    function endWithoutSettling(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      emit({ type: 'agent_end', },);
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
