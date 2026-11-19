/**
 A stand-in for `pi --mode rpc` that speaks just enough of the protocol to take the driver down each path of a real run.
 `STAND_IN_PI_SCENARIO` picks the behaviour; `STAND_IN_PI_LOG` receives every command it gets, its arguments, the names of the
 variables it inherited, and the markers a test waits for. It never writes a value from its environment: a log line is a name,
 because everything a command prints reaches a model provider.

 @module
 */

import { spawn, } from 'node:child_process';
import { close, } from 'node:fs';
import { randomUUID, } from 'node:crypto';
import {
  appendFile,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
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
import { ordinaryAnswersOf, } from './ordinary-answers-of-a-stand-in-pi.ts';
import { sessionAnswerOf, } from './session-answers-of-a-stand-in-pi.ts';
import { logTheTemporaryFolderPiResolved, } from './what-a-stand-in-pi-answers-about-its-temporary-folder.ts';
import { logTheNamesOfWhatPiCouldRead, } from './what-a-stand-in-pi-logs-about-its-names.ts';
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
  armTheStandInMasking,
  answerCommands,
  sessionHoldsAValue,
  type StandInMasking,
} from './stand-in-pi-masking-fixture.ts';
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
 The session Pi was told to resume, which is the word after `--session` in its own arguments.
 */
const sessionArgumentAt = argv.indexOf('--session',);

/**
 Names the session file a fresh start makes, which is the file its starter named or a new name in the temporary folder its
 starter named.

 Nothing else places it. Falling back to `HOME` and then to the working folder left thousands of empty
 `stand-in-session-<uuid>.jsonl` files in the real home folder of whoever ran the connection tests, so a starter that names neither
 is stopped here, before anything is written.

 @returns the path of the session file

 @throws Error when the starter named neither `STAND_IN_PI_SESSION_FILE` nor a non-empty `TMPDIR`
 */
function sessionFileOfAFreshStart(): string {
  if (env.STAND_IN_PI_SESSION_FILE !== undefined) {
    return env.STAND_IN_PI_SESSION_FILE;
  }
  /**
   The temporary folder the starter named, where an empty value names none: joined, it would put the file in the working folder.
   */
  const temporaryFolder = env.TMPDIR ?? '';
  if (temporaryFolder === '') {
    throw new Error(
      'the stand-in Pi was started for a fresh session with neither STAND_IN_PI_SESSION_FILE nor a non-empty TMPDIR set. Give it TMPDIR naming a folder the test removes, as the driver does for a real run, so its session file lands in neither the home folder nor the working folder.',
    );
  }
  return join(
    temporaryFolder,
    `stand-in-session-${randomUUID()}.jsonl`,
  );
}

/**
 The session Pi keeps: the one the run handed it, or one this stand-in makes for itself at its own start, as Pi makes a new session
 file for a fresh run.
 */
const sessionFile = (sessionArgumentAt === (-1))
  ? sessionFileOfAFreshStart()
  : argv[sessionArgumentAt + 1];

// A fresh run's session file is made by its Pi, during the run, holding whatever the fixture says Pi's own session already held.
if ((sessionArgumentAt === (-1)) && (sessionFile !== undefined)) {
  try {
    await writeFile(
      sessionFile,
      env.STAND_IN_PI_SESSION_CONTENT ?? '',
      { flag: 'wx', },
    );
  }
  catch (error) {
    // A file that is already there is the fixture's own, which this start keeps rather than writes.
    if (!(Error.isError(error,) && (errorCodeOf(error,) === 'EEXIST'))) {
      throw error;
    }
  }
}

/**
 What Pi answers each ordinary command with, named by the session file this start keeps.
 */
const ORDINARY_ANSWERS = ordinaryAnswersOf({ sessionFile: sessionFile ?? 'stand-in-session.jsonl', },);

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

 @param masking - what this stand-in's Pi armed, which its command list answers with
 */
async function actOnCommand({
  line,
  masking,
}: {
  readonly line: string;
  readonly masking: StandInMasking;
},): Promise<void> {
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
  if (parsed.type === 'get_commands') {
    answerCommands({
      id: parsed.id,
      masking,
    },);
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
  /**
   What a scenario about the session file Pi names answers this command with.
   */
  const sessionAnswer = sessionAnswerOf({
    type: parsed.type,
    scenario,
    ordinary: ORDINARY_ANSWERS.get(String(parsed.type,),) ?? {},
    reported: env.STAND_IN_PI_REPORTED_SESSION ?? 'stand-in-reported-session.jsonl',
  },);
  if ('noData' in sessionAnswer) {
    emit({
      type: 'response',
      id: parsed.id,
      success: true,
    },);
    return;
  }
  if ('answer' in sessionAnswer) {
    answer({
      id: parsed.id,
      data: sessionAnswer.answer,
    },);
    return;
  }
  answer({
    id: parsed.id,
    data: ORDINARY_ANSWERS.get(String(parsed.type,),) ?? { sessionFile: 'stand-in.jsonl', },
  },);
}

/**
 What this stand-in's Pi armed, read before any scenario runs so the descriptor is closed before a tool could inherit it.
 */
const masking: StandInMasking = armTheStandInMasking({ scenario, },);
await logTheNamesOfWhatPiCouldRead({
  log,
  argumentsOfTheStart: argv.slice(2,),
  values: masking.values,
},);
await logTheTemporaryFolderPiResolved({ log, },);
/**
 Whether the session Pi was handed holds an armed value as Pi starts, which is a yes or a no and never a value.
 */
const sessionValue = await sessionHoldsAValue({
  masking,
  ...(sessionFile === undefined ? {} : { sessionFile, }),
},);
if ((typeof sessionValue) === 'string') {
  await appendFile(
    log,
    `marker ${sessionValue}\n`,
  );
}
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
  await actOnCommand({
    line,
    masking,
  },);
}
if ((scenario === 'ignores-input-close') || (scenario === 'ignores-sigterm')) {
  await wait(STAY_MILLISECONDS,);
}
