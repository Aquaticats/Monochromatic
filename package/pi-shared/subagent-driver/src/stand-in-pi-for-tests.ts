/**
 A stand-in for `pi --mode rpc` that speaks just enough of the protocol to take the driver down each path of a real run.
 `STAND_IN_PI_SCENARIO` picks the behaviour; `STAND_IN_PI_LOG` receives every command it gets, its arguments, the names of the
 variables it inherited, and the markers a test waits for. It never writes a value from its environment: a log line is a name,
 because everything a command prints reaches a model provider.

 @module
 */

import { spawn, } from 'node:child_process';
import { close, } from 'node:fs';
import {
  appendFile,
  readFile,
  writeFile,
} from 'node:fs/promises';
import process, {
  argv,
  env,
  exit,
  stderr,
  stdin,
  stdout,
} from 'node:process';
import { createInterface, } from 'node:readline';
import { setTimeout as wait, } from 'node:timers/promises';
import { promisify, } from 'node:util';
import { NESTED_RUN_VARIABLE, } from './driver-constants.ts';
import { errorCodeOf, } from './error-code-of.ts';
import type { PiRecord, } from './driver-types.ts';
import { isRecord, } from './read-pi-fields.ts';
import {
  LOG,
  scenario,
  seen,
  writeLogLine,
} from './stand-in-pi-log-fixture.ts';
import {
  CRASH_STATUS,
  playPrompt,
} from './stand-in-pi-prompt-plays-fixture.ts';
import {
  answer,
  answerAndExit,
  emit,
  START_FAILURE_ADVICE,
  START_FAILURE_TEXT,
  STANDARD_ERROR_TEXT,
} from './stand-in-pi-records-fixture.ts';

/**
 Longest a scenario that refuses to leave stays. A test's own teardown kills it far sooner; this only keeps a fixture from a
 killed test process from living on.

 */
const STAY_MILLISECONDS = 300_000;

/**
 Milliseconds after starting at which the late-exit scenario leaves, longer than any answer bound a test asks for and shorter
 than the answer bound the driver ships.
 */
const LATE_EXIT_MILLISECONDS = 1_000;

/**
 Status a stand-in that will not start leaves with, which is what a real Pi leaves when an extension cannot be loaded.
 */
const START_FAILURE_STATUS = 1;

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Seconds the tool started by `keeps-output-open` lives, so a failed test cannot leave it running.
 */
const TOOL_SECONDS = '30';

/**
 Where commands are logged.
 */
const log = env.STAND_IN_PI_LOG ?? '/dev/null';

/**
 Closes a descriptor the process no longer wants, such as its own input.
 */
const closeDescriptor = promisify(close,);

/**
 What Pi answers each ordinary command with, so a run's state, its command names and its statistics all have shapes to read.
 */
const ORDINARY_ANSWERS: ReadonlyMap<string, PiRecord> = new Map<string, PiRecord>([
  [
    'get_state',
    {
      model: { id: 'stand-in-model', },
      thinkingLevel: 'xhigh',
    },
  ],
  [
    'get_commands',
    { commands: [
      { name: 'bash', },
      { name: 'edit', },
    ], },
  ],
  [
    'get_session_stats',
    {
      sessionFile: 'stand-in-session.jsonl',
      cost: 0.5,
    },
  ],
]);

/**
 Counts this start among the starts of one run, in a file the several stand-in processes of one run share.

 @returns which start this is, counting from one

 @throws Error when the run gave the stand-in no file to count them in
 */
async function countStarts(): Promise<number> {
  /**
   Where the count is kept, which the test names for the run.
   */
  const file = env.STAND_IN_PI_START_COUNT_FILE;
  if (file === undefined) {
    throw new Error('a scenario that counts starts needs STAND_IN_PI_START_COUNT_FILE',);
  }
  /**
   How many starts the run made before this one.
   */
  let before = 0;
  try {
    before = Number(await readFile(
      file,
      'utf8',
    ),);
  }
  catch (error) {
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
  }
  /**
   How many starts the run has made, this one included.
   */
  const started = before + 1;
  await writeFile(
    file,
    String(started,),
  );
  return started;
}

/**
 Leaves before the prompt is read, as a Pi whose extensions would not load does, writing Pi's own words to its standard error
 first.

 The `once` scenario leaves that way only on the run's first start, so a test sees a start that failed and the one after it, and
 `always` leaves that way on every start, which is a condition that stands. A start that does not leave goes on as any other.
 */
async function leaveBeforeThePrompt(): Promise<void> {
  /**
   Which start this is, counting from one.
   */
  const started = await countStarts();
  if ((scenario === 'exits-before-prompt-always') || (started === 1)) {
    stderr.write(`${START_FAILURE_TEXT}\n${START_FAILURE_ADVICE}\n`,);
    exit(START_FAILURE_STATUS,);
  }
}

/**
 Leaves after a delay, for a Pi that answers nothing and leaves before the answer bound the driver ships.
 */
async function leaveLater(): Promise<void> {
  await wait(LATE_EXIT_MILLISECONDS,);
  exit(CRASH_STATUS,);
}

/**
 Writes the text a test looks for in the captured standard error.
 */
function writeToStandardError(): void {
  stderr.write(`${STANDARD_ERROR_TEXT}\n`,);
}

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
    // Closing the descriptor is what gives the driver an EPIPE: destroying the stream leaves it open.
    stdin.pause();
    await closeDescriptor(0,);
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
      await writeLogLine('marker overlap',);
    }
    if ((parsed.type === 'steer') && (scenario === 'crash-on-steer')) {
      exit(CRASH_STATUS,);
    }
    if (scenario === 'exits-after-prompt') {
      answerAndExit({
        id: parsed.id,
        status: CRASH_STATUS,
      },);
      return;
    }
    await playPrompt(parsed.id,);
    return;
  }
  if ((parsed.type === 'get_session_stats') && (scenario === 'crash-on-stats')) {
    exit(CRASH_STATUS,);
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
// Names only: a test learns what Pi's commands could read without any value reaching a log, and so a model provider.
await appendFile(
  log,
  `environment-names ${JSON.stringify(Object.keys(env,)
    .toSorted(),)}\n`,
);
if (scenario === 'crash') {
  exit(CRASH_STATUS,);
}
if ((scenario === 'exits-before-prompt-once') || (scenario === 'exits-before-prompt-always')) {
  await leaveBeforeThePrompt();
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
