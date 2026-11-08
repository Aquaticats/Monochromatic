/**
 A stand-in for `pi --mode rpc` that speaks just enough of the protocol to take the driver down each path of a real run.
 `STAND_IN_PI_SCENARIO` picks the behaviour; `STAND_IN_PI_LOG` receives the type of every command it gets and its arguments.

 @module
 */

import { spawn, } from 'node:child_process';
import { appendFile, } from 'node:fs/promises';
import {
  argv,
  env,
  exit,
  on,
  stdin,
  stdout,
} from 'node:process';
import { createInterface, } from 'node:readline';
import { setTimeout as wait, } from 'node:timers/promises';
import { NESTED_RUN_VARIABLE, } from './driver-constants.ts';
import type { PiRecord, } from './driver-types.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 Status the crashing scenarios exit with.
 */
const CRASH_STATUS = 3;

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
 A Unicode line separator, valid inside a JSON string and not a line ending there.
 */
const LINE_SEPARATOR = String.fromCodePoint(0x20_28,);

/**
 Longest a scenario that refuses to leave stays, so a failed test cannot leave it running.
 */
const STAY_MILLISECONDS = 30_000;

/**
 Seconds the tool started by `keeps-output-open` lives, so a failed test cannot leave it running.
 */
const TOOL_SECONDS = '30';

/**
 The behaviour chosen by the test.
 */
const scenario = env.STAND_IN_PI_SCENARIO ?? 'normal';

/**
 Where commands are logged.
 */
const log = env.STAND_IN_PI_LOG ?? '/dev/null';

/**
 How many prompts have arrived.
 */
const seen = { prompts: 0, };

/**
 Writes one record.

 @param record - fields of the record
 */
function emit(record: PiRecord,): void {
  stdout.write(`${JSON.stringify(record,)}\n`,);
}

/**
 Answers a command successfully.

 @param id - command's identifier

 @param data - answer's data
 */
function answer({ id, data = {}, }: { readonly id: unknown; readonly data?: PiRecord; },): void {
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
const promptPlays: ReadonlyMap<string, (id: unknown,) => Promise<void> | void> = new Map<string, (id: unknown,) => Promise<void> | void>([
  ['reject', function rejectPrompt(id,) {
    emit({ type: 'response', id, success: false, error: 'no key for this provider', },);
  },],
  ['reject-inbox', function rejectLaterPrompts(id,) {
    if (seen.prompts > 1) {
      emit({ type: 'response', id, success: false, error: 'refused', },);
      return;
    }
    answer({ id, },);
    finishRun('done',);
  },],
  ['handled', function handlePrompt(id,) {
    answer({ id, data: { disposition: 'handled', }, },);
  },],
  ['crash-later', async function crashLater(id,) {
    answer({ id, },);
    await wait(LATE_CRASH_MILLISECONDS,);
    exit(CRASH_STATUS,);
  },],
  ['never-settles', function neverSettle(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
  },],
  ['settle-gap', function endWithoutSettling(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    emit({ type: 'agent_end', },);
  },],
  ['many-turns', function startManyTurns(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    for (let turn = 0; turn < MANY_TURNS; turn += 1) {
      emit({ type: 'turn_start', },);
    }
  },],
  ['dialog', function askDialog(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    emit({ type: 'extension_ui_request', id: 'dialog-1', method: 'select', title: 'Pick one', message: 'Which?', },);
    emit({ type: 'extension_ui_request', id: 'notice-1', method: 'notify', message: 'hello', },);
  },],
  ['tools', function useTools(id,) {
    answer({ id, },);
    emit({ type: 'agent_start', },);
    emit({ type: 'tool_execution_start', toolName: 'bash', args: { command: 'ls', }, },);
    emit({ type: 'tool_execution_end', toolName: 'bash', isError: false, result: 'ok', },);
    emit({ type: 'tool_execution_start', toolName: 'bash', args: { command: 'false', }, },);
    emit({ type: 'tool_execution_end', toolName: 'bash', isError: true, result: 'failed', },);
    emit({ type: 'auto_retry_start', },);
    emit({ type: 'agent_settled', },);
  },],
  ['noise', function writeNoise(id,) {
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
 */
async function playPrompt(id: unknown,): Promise<void> {
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

/**
 Writes the lines the connection must cope with, then answers.

 @param id - command's identifier
 */
function writeOddLines(id: unknown,): void {
  stdout.write('\n',);
  stdout.write('[1]\n',);
  emit({ type: 'response', id: 7, },);
  emit({ type: 'response', id: 'nobody-asked', success: true, },);
  answer({ id, },);
}

/**
 Starts a tool that holds this process's output open after this process has gone, and logs its process number.

 @param id - command's identifier
 */
async function startToolHoldingOutput(id: unknown,): Promise<void> {
  /**
   The tool.
   */
  const tool = spawn('sleep', [TOOL_SECONDS,], { stdio: ['ignore', 'inherit', 'inherit',], detached: true, },);
  tool.unref();
  await appendFile(log, `tool ${String(tool.pid,)}\n`,);
  answer({ id, },);
}

/**
 Acts on one command.

 @param line - command as written
 */
async function actOnCommand(line: string,): Promise<void> {
  /**
   The command, of any shape.
   */
  const parsed: unknown = JSON.parse(line,);
  if (!isRecord(parsed,)) {
    return;
  }
  await appendFile(log, `${String(parsed.type,)}\n`,);
  if (scenario === 'silent') {
    return;
  }
  if (scenario === 'exits-silently') {
    exit(CRASH_STATUS,);
  }
  if (scenario === 'odd-lines') {
    writeOddLines(parsed.id,);
    return;
  }
  if (scenario === 'keeps-output-open') {
    await startToolHoldingOutput(parsed.id,);
    return;
  }
  if (parsed.type === 'extension_ui_response') {
    emit({ type: 'agent_settled', },);
    return;
  }
  if (parsed.type === 'prompt') {
    seen.prompts += 1;
    await playPrompt(parsed.id,);
    return;
  }
  answer({ id: parsed.id, data: { sessionFile: 'stand-in.jsonl', }, },);
  if ((parsed.type === 'steer') && (scenario === 'settle-gap')) {
    emit({ type: 'agent_settled', },);
  }
}

await appendFile(log, `arguments ${JSON.stringify(argv.slice(2,),)}\n`,);
await appendFile(log, `nested ${env[NESTED_RUN_VARIABLE] ?? 'unset'}\n`,);
if (scenario === 'crash') {
  exit(CRASH_STATUS,);
}
if (scenario === 'ignores-sigterm') {
  on('SIGTERM', function ignoreSigterm() {
    stdout.write('',);
  },);
}
for await (const line of createInterface({ input: stdin, },)) {
  await actOnCommand(line,);
}
if ((scenario === 'ignores-input-close') || (scenario === 'ignores-sigterm')) {
  await wait(STAY_MILLISECONDS,);
}
