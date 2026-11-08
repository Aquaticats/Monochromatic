/**
 A stand-in for `pi --mode rpc` that speaks just enough of the protocol to take the driver down each path of a real run.
 `STAND_IN_PI_SCENARIO` picks the behaviour; `STAND_IN_PI_LOG` receives every command it gets, its arguments, and the markers a
 test waits for.

 @module
 */

import { spawn, } from 'node:child_process';
import { appendFile, } from 'node:fs/promises';
import process, {
  argv,
  env,
  exit,
  stdin,
  stdout,
} from 'node:process';
import { createInterface, } from 'node:readline';
import { setTimeout as wait, } from 'node:timers/promises';
import { NESTED_RUN_VARIABLE, } from './driver-constants.ts';
import type { PiRecord, } from './driver-types.ts';
import { isRecord, } from './read-pi-fields.ts';
import {
  answer,
  CRASH_STATUS,
  emit,
  leaveLater,
  playPrompt,
  scenario,
  seen,
  writeLogLine,
  writeToStandardError,
} from './stand-in-pi-prompt-plays.ts';

/**
 Longest a scenario that refuses to leave stays. A test's own teardown kills it far sooner; this only keeps a fixture from a
 killed test process from living on.

 */
const STAY_MILLISECONDS = 300_000;

/**
 Seconds the tool started by `keeps-output-open` lives, so a failed test cannot leave it running.
 */
const TOOL_SECONDS = '30';

/**
 Where commands are logged.
 */
const log = env.STAND_IN_PI_LOG ?? '/dev/null';

/**
 What Pi answers each ordinary command with, so a run's state, its command names and its statistics all have shapes to read.
 */
const ORDINARY_ANSWERS: ReadonlyMap<string, PiRecord> = new Map<string, PiRecord>([
  ['get_state', {
    model: { id: 'stand-in-model', },
    thinkingLevel: 'xhigh',
  },],
  ['get_commands', { commands: [{ name: 'bash', }, { name: 'edit', },], },],
  ['get_session_stats', {
    sessionFile: 'stand-in-session.jsonl',
    cost: 0.5,
  },],
]);

/**
 Writes the lines the connection must cope with, then answers.

 @param id - command's identifier
 */
function writeOddLines(id: unknown,): void {
  stdout.write('\n',);
  stdout.write('[1]\n',);
  emit({
    type: 'response',
    id: 7,
  },);
  emit({
    type: 'response',
    id: 'nobody-asked',
    success: true,
  },);
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
  const tool = spawn(
    'sleep',
    [TOOL_SECONDS,],
    {
      stdio: [
        'ignore',
        'inherit',
        'inherit',
      ],
      detached: true,
    },
  );
  tool.unref();
  await appendFile(
    log,
    `tool ${String(tool.pid,)}\n`,
  );
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
  await appendFile(
    log,
    `command ${JSON.stringify(parsed,)}\n`,
  );
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
  if (scenario === 'closes-input') {
    answer({ id: parsed.id, },);
    stdin.destroy();
    await writeLogLine('marker input-closed',);
    await wait(STAY_MILLISECONDS,);
    return;
  }
  if (parsed.type === 'extension_ui_response') {
    /**
     Only the dialog the scenario asked for, declined, ends the run: any other answer proves the driver declined something else.
     */
    if ((parsed.id === 'dialog-1') && (parsed.cancelled === true)) {
      emit({ type: 'agent_settled', },);
    }
    return;
  }
  if ((parsed.type === 'prompt') || (parsed.type === 'steer')) {
    seen.prompts += 1;
    if (seen.answering > 0) {
      await writeLogLine('overlap',);
    }
    if ((parsed.type === 'steer') && (scenario === 'crash-on-steer')) {
      exit(CRASH_STATUS,);
    }
    await playPrompt(parsed.id,);
    return;
  }
  if ((parsed.type === 'get_session_stats') && (scenario === 'statless')) {
    emit({
      type: 'response',
      id: parsed.id,
      success: true,
    },);
    return;
  }
  answer({
    id: parsed.id,
    data: ORDINARY_ANSWERS.get(String(parsed.type,),) ?? { sessionFile: 'stand-in.jsonl', },
  },);
}

await appendFile(
  log,
  `arguments ${JSON.stringify(argv.slice(2,),)}\n`,
);
await appendFile(
  log,
  `nested ${env[NESTED_RUN_VARIABLE] ?? 'unset'}\n`,
);
if (scenario === 'crash') {
  exit(CRASH_STATUS,);
}
if (scenario === 'stderr-note') {
  writeToStandardError();
}
if (scenario === 'exit-later') {
  await leaveLater();
}
if (scenario === 'ignores-sigterm') {
  process.on(
    'SIGTERM',
    function ignoreSigterm() {
      stdout.write('',);
    },
  );
}
for await (const line of createInterface({ input: stdin, },)) {
  await actOnCommand(line,);
}
if ((scenario === 'ignores-input-close') || (scenario === 'ignores-sigterm')) {
  await wait(STAY_MILLISECONDS,);
}
