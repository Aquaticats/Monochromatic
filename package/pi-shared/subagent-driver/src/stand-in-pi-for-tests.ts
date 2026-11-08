/**
 A stand-in for `pi --mode rpc` that speaks just enough of the protocol to take the driver down each path of a real run.
 `STAND_IN_PI_SCENARIO` picks the behaviour; `STAND_IN_PI_LOG` receives the type of every command it gets and its arguments.

 @module
 */

import { appendFileSync, } from 'node:fs';
import {
  argv,
  env,
  exit,
  stdin,
  stdout,
} from 'node:process';
import { createInterface, } from 'node:readline';
import { setTimeout as wait, } from 'node:timers/promises';
import { NESTED_RUN_VARIABLE, } from './driver-constants.ts';

/**
 Status the crashing scenarios exit with.
 */
const CRASH_STATUS = 3;

/**
 Milliseconds between `agent_end` and `agent_settled` in the scenario that leaves a gap.
 */
const SETTLE_GAP_MILLISECONDS = 2_000;

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
 Offset, in the encoded record, that falls inside the two-byte character.
 */
const SPLIT_OFFSET_AFTER_CHARACTER_START = 1;

/**
 The behaviour chosen by the test.
 */
const scenario = env.STAND_IN_PI_SCENARIO ?? 'normal';

/**
 Where commands are logged.
 */
const log = env.STAND_IN_PI_LOG ?? '/dev/null';

appendFileSync(log, `arguments ${JSON.stringify(argv.slice(2,),)}\n`,);
appendFileSync(log, `nested ${env[NESTED_RUN_VARIABLE] ?? 'unset'}\n`,);
if (scenario === 'crash') {
  exit(CRASH_STATUS,);
}

/**
 Writes one record.

 @param record - fields of the record
 */
function emit(record: Readonly<Record<string, unknown>>,): void {
  stdout.write(`${JSON.stringify(record,)}\n`,);
}

/**
 Answers a command successfully.

 @param id - command's identifier

 @param data - answer's data
 */
function answer(id: unknown, data: Readonly<Record<string, unknown>> = {},): void {
  emit({ type: 'response', id, success: true, data, },);
}

/**
 Writes a finished run.

 @param text - assistant's reply
 */
function finishRun(text: string,): void {
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit({ type: 'message_end', message: { role: 'assistant', content: [{ type: 'text', text, },], stopReason: 'stop', usage: { input: 1, output: 1, cost: { total: 0, }, }, }, },);
  emit({ type: 'agent_end', },);
  emit({ type: 'agent_settled', },);
}

/**
 Writes a tool that worked and a tool that failed.
 */
function useTools(): void {
  emit({ type: 'tool_execution_start', toolName: 'bash', args: { command: 'ls', }, },);
  emit({ type: 'tool_execution_end', toolName: 'bash', isError: false, result: 'ok', },);
  emit({ type: 'tool_execution_start', toolName: 'bash', args: { command: 'false', }, },);
  emit({ type: 'tool_execution_end', toolName: 'bash', isError: true, result: 'failed', },);
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
   Where the two-byte character starts.
   */
  const characterAt = bytes.indexOf(Buffer.from('é',),);
  stdout.write(bytes.subarray(0, characterAt + SPLIT_OFFSET_AFTER_CHARACTER_START,),);
  await wait(SPLIT_WRITE_MILLISECONDS,);
  stdout.write(bytes.subarray(characterAt + SPLIT_OFFSET_AFTER_CHARACTER_START,),);
}

/**
 Plays out a prompt according to the scenario.

 @param request - command's identifier and how many prompts have arrived
 */
async function playPrompt({ id, promptsSeen, }: { readonly id: unknown; readonly promptsSeen: number; },): Promise<void> {
  switch (scenario) {
    case 'reject': {
      emit({ type: 'response', id, success: false, error: 'no key for this provider', },);
      return;
    }
    case 'reject-inbox': {
      if (promptsSeen > 1) {
        emit({ type: 'response', id, success: false, error: 'refused', },);
        return;
      }
      break;
    }
    case 'handled': {
      answer(id, { disposition: 'handled', },);
      return;
    }
    case 'crash-later': {
      answer(id,);
      await wait(LATE_CRASH_MILLISECONDS,);
      exit(CRASH_STATUS,);
      break;
    }
    case 'never-settles': {
      answer(id,);
      emit({ type: 'agent_start', },);
      return;
    }
    case 'settle-gap': {
      answer(id,);
      emit({ type: 'agent_start', },);
      emit({ type: 'agent_end', },);
      await wait(SETTLE_GAP_MILLISECONDS,);
      emit({ type: 'agent_settled', },);
      return;
    }
    case 'many-turns': {
      answer(id,);
      emit({ type: 'agent_start', },);
      for (let turn = 0; turn < MANY_TURNS; turn += 1) {
        emit({ type: 'turn_start', },);
      }
      return;
    }
    case 'dialog': {
      answer(id,);
      emit({ type: 'agent_start', },);
      emit({ type: 'extension_ui_request', id: 'dialog-1', method: 'select', title: 'Pick one', message: 'Which?', },);
      emit({ type: 'extension_ui_request', id: 'notice-1', method: 'notify', message: 'hello', },);
      return;
    }
    case 'tools': {
      answer(id,);
      emit({ type: 'agent_start', },);
      useTools();
      emit({ type: 'auto_retry_start', },);
      emit({ type: 'agent_settled', },);
      return;
    }
    case 'noise': {
      answer(id,);
      stdout.write('this line is not JSON\n',);
      finishRun('after the noise',);
      return;
    }
    case 'split-record': {
      answer(id,);
      emit({ type: 'agent_start', },);
      await emitFinalReplyInTwoPieces('café line separator',);
      emit({ type: 'agent_settled', },);
      return;
    }
    default: {
      break;
    }
  }
  answer(id,);
  finishRun('done',);
}

/**
 How many prompts have arrived.
 */
let promptsSeen = 0;
for await (const line of createInterface({ input: stdin, },)) {
  /**
   The command.
   */
  const command = JSON.parse(line,) as { readonly type: string; readonly id?: unknown; readonly cancelled?: boolean; };
  appendFileSync(log, `${command.type}\n`,);
  if (command.type === 'extension_ui_response') {
    emit({ type: 'agent_settled', },);
  }
  else if (command.type === 'prompt') {
    promptsSeen += 1;
    await playPrompt({ id: command.id, promptsSeen, },);
  }
  else {
    answer(command.id, { sessionFile: 'stand-in.jsonl', },);
  }
}
