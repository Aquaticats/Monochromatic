/**
 A command that says nothing: the line the driver writes the first time, the end of the command past the second, and a command
 that reports output steadily, which is never ended.

 Two kinds of test here. The watch's own decisions are judged against a record whose clock a test moves, so a silence is measured
 rather than waited for and a loaded host cannot decide an outcome. The controls that need an answer from a Pi go through the bin
 against the stand-in, where a command's records are real lines on a channel.

 @module
 */

import {
  type ChildProcess,
  execFile,
} from 'node:child_process';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  aSilenceInTheRun,
  declaredBoundOf,
  type CommandAnswer,
  type CommandRequest,
  type ConnectionToPi,
  type DriverOptions,
  endedCommandMessageOf,
  fieldsOf,
  isRecord,
  MASKING_NOTHING,
  recordsIn,
  keptCommand,
  POLL_MILLISECONDS,
  type RecordOfPiRun,
  type RunSummary,
  SILENT_BOUND_GRACE_SECONDS,
  SILENT_ENDING_SECONDS,
  SILENT_NOTICE_SECONDS,
  startRecordOfPiRun,
  theOperationIsDueToEnd,
  type WatchRequest,
  watchUntilEnded,
} from '../dist/final/node/index.mjs';
import {
  failureCodeOf,
  SILENT_COMMAND,
  WENT_ON,
} from './stand-in-pi-silence-plays-fixture.ts';

/**
 Milliseconds in a second, so a bound a test states in seconds is derived the way the driver derives it.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Seconds a control states for both times when it reads the machine's clock and wants the first look to be past them: any look
 is 300 ms away, so a millisecond of silence is past both and the control does not wait for a bound it did not state.
 */
const PAST_BOTH_TIMES_SECONDS = 0.001;

/**
 The code the operating system gives a process that is not there.
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 Milliseconds the run under the watch's own test may last: far beyond every silence it states, because the watch's deadline is
 not what such a test asks about.
 */
const RUN_MILLISECONDS = 600 * MILLISECONDS_PER_SECOND;

/**
 Milliseconds at which a command of the watch's own tests reports one chunk of output, well past its start, so a silence measured
 from the report is unmistakably shorter than one measured from the command's start.
 */
const OUTPUT_REPORTED_AT_MILLISECONDS = 60 * MILLISECONDS_PER_SECOND;

/**
 Seconds of silence after which a settled run ends, in the runs that go through the bin: shorter than the silence bounds those
 runs state, so the run ends as one that finished its turn rather than on its wall clock.
 */
const IDLE_SECONDS = 0.2;

/**
 Seconds a run that goes through the bin may last: above its start-up, its turns and its silence bounds, and short so the test is
 quick.
 */
const RUN_WALL_SECONDS = 20;

/**
 Seconds a test allows a driver beyond the run's own wall limit, so a stalled driver fails rather than hangs it.
 */
const TEST_TEARDOWN_ALLOWANCE_SECONDS = 120;

/**
 Seconds of silence after which a run through the bin writes its line: short enough that a test waits only a look or two for it.
 */
const NOTICE_SECONDS = 0.1;

/**
 Seconds a command through the bin may say nothing before the driver ends it: above the notice and far below a real command.
 */
const ENDING_SECONDS = 0.2;

/**
 Seconds of silence the command that reports output steadily never reaches. Its output reports thirty to a look, and this bound is
 twenty looks, so a host that delivers one report twenty looks late is still inside it.
 */
const STEADY_NOTICE_SECONDS = (POLL_MILLISECONDS * 20) / MILLISECONDS_PER_SECOND;

/**
 Seconds the driver may wait before it ends a command that reports output steadily, above the notice so a silence there would be
 both named and ended if it came.
 */
const STEADY_ENDING_SECONDS = (POLL_MILLISECONDS * 25) / MILLISECONDS_PER_SECOND;

/**
 Seconds a run is given when the driver's silence end, and not the wall clock, is expected to move it on. It sits above every
 stretch a stand-in takes on a loaded host, so the run ends on what a test asks about rather than on its own limit.
 */
const ENDING_RUN_WALL_SECONDS = 60;

/**
 Seconds a run is given when its subject is the notice alone: the command never ends, so the wall clock is what ends the run. It
 sits far above the start-up a loaded host may spend, so the watch is reached before the run's own limit.
 */
const NOTICE_RUN_WALL_SECONDS = 8;

/**
 Status of a run that ended on its own terms.
 */
const CLEAN_STATUS = 0;

/**
 Turns a run through the bin may start: the driver's own prompt, the message that follows an ended command, and room to spare so
 no test ends on the turn cap instead of on what it asks about.
 */
const TURN_CAP = 5;

/**
 Milliseconds a test waits for a driver's process to leave before it ends it, which no test here should need.
 */
const STOP_GRACE_MILLISECONDS = 5_000;

/**
 Milliseconds a test waits for a line its run was told to write: the feed is a stream, so a line is written a moment after the
 record that asked for it returned.
 */
const FEED_WRITE_MILLISECONDS = 2_000;

/**
 Milliseconds between two looks for such a line.
 */
const LOOK_MILLISECONDS = 10;

/**
 The command a test's record reports, which holds the text a line and a message are read for.
 */
const COMMAND_ARGS = { command: SILENT_COMMAND, };

/**
 The bound a test declares for a command that may run long, in the bash tool's own argument.
 */
const DECLARED_BOUND_SECONDS = 7_200;

/**
 The driver's bin, run against the stand-in Pi.
 */
const DRIVER = new URL('drive-pi-subagent.ts', import.meta.url,).pathname;

/**
 The stand-in Pi the bin starts.
 */
const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

/**
 A folder the test throws away.
 */
type DisposableFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 One run's record and the clock a test moves.

 The record reads the time from `now`, so a silence is measured: moving the clock past a bound is what a test does instead of
 waiting for one.
 */
type RunAtAClock = {
  readonly record: RecordOfPiRun;
  readonly folder: string;
  readonly clock: { at: number };
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a folder the test throws away.

 @returns the folder
 */
async function makeFolder(): Promise<DisposableFolder> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-silence-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Starts a run's record on a clock a test moves.

 @returns the record, its folder, and the clock
 */
async function startRunAtAClock(): Promise<RunAtAClock> {
  /**
   The folder the run's feed goes in.
   */
  const folder = await makeFolder();
  await mkdir(join(folder.path, 'inbox',),);
  /**
   Where the record reads the time, which starts at zero so every figure in a test is the run's own.
   */
  const clock = { at: 0, };
  /**
   The record of the run.
   */
  const record = startRecordOfPiRun({
    feedFile: join(folder.path, 'feed.jsonl',),
    now: function now() {
      return clock.at;
    },
    mask: MASKING_NOTHING,
  },);
  return {
    record,
    folder: folder.path,
    clock,
    async [Symbol.asyncDispose]() {
      await record.close();
      await folder[Symbol.asyncDispose]();
    },
  };
}

/**
 Starts a run's record on the machine's own clock, which is what a test that reaches the idle path needs: that path asks how long
 the run has been quiet, and a clock a test moves reaches it only if the test moves it too.

 @returns the record, its folder, and a clock that stays where the machine has it
 */
async function startRunAtTheRealClock(): Promise<RunAtAClock> {
  /**
   The folder the run's feed goes in.
   */
  const folder = await makeFolder();
  await mkdir(join(folder.path, 'inbox',),);
  /**
   The record of the run.
   */
  const record = startRecordOfPiRun({
    feedFile: join(folder.path, 'feed.jsonl',),
    mask: MASKING_NOTHING,
  },);
  return {
    record,
    folder: folder.path,
    clock: { at: Date.now(), },
    async [Symbol.asyncDispose]() {
      await record.close();
      await folder[Symbol.asyncDispose]();
    },
  };
}

/**
 Declines a dialog a test's events never ask, so a fixture mistake fails the test instead of passing quietly.
 */
function refuseDialog(): void {
  throw new Error('no dialog was expected',);
}

/**
 Observes a record of Pi on a run.

 @param run - the run at its clock

 @param event - the record Pi wrote
 */
function observe(run: RunAtAClock, event: Record<string, unknown>,): void {
  run.record.observe({
    event,
    declineDialog: refuseDialog,
  },);
}

/**
 Observes a command starting, as Pi reports it.

 @param run - the run at its clock

 @param boundSeconds - the bound the command declared, when a test declares one
 */
function observeCommandStart(run: RunAtAClock, boundSeconds?: number,): void {
  observe(run, {
    type: 'tool_execution_start',
    toolName: 'bash',
    args: {
      ...COMMAND_ARGS,
      ...(boundSeconds === undefined ? {} : { timeout: boundSeconds, }),
    },
  },);
}

/**
 Observes a command ending, which is the run saying something again.

 @param run - the run at its clock
 */
function observeCommandEnd(run: RunAtAClock,): void {
  observe(run, {
    type: 'tool_execution_end',
    toolName: 'bash',
    isError: false,
    result: { content: [], },
  },);
}

/**
 A connection that ends the operation as Pi does: the abort is answered after the session settles, and the recovery message is
 taken, so the run goes on.

 @param sent - where the commands the watch sends are recorded

 @param run - the run whose record the settle is written on

 @param toolCallId - the call the abort ends

 @returns the connection a watch reads

 @example
 ```ts
 const connection = endingConnectionFor({ sent, run, toolCallId: 'call-silent' },);
 ```
 */
function endingConnectionFor({
  sent,
  run,
  toolCallId,
}: {
  readonly sent: Record<string, unknown>[];
  readonly run: RunAtAClock;
  readonly toolCallId: string
},): ConnectionToPi {
  return endingConnection({
    sent,
    abort: 'done',
    message: 'done',
    before: {
      abort: function settle() {
        observeAbortedCall(run, toolCallId,);
      },
    },
  },);
}

/**
 A connection whose every command is answered at once, which records what the watch sent.

 @param sent - where the commands the watch sends are recorded

 @param refuses - whether Pi refuses each message instead of taking it

 @returns the connection a watch reads
 */
function recordingConnection({
  sent,
  refuses = false,
}: {
  readonly sent: Record<string, unknown>[];
  readonly refuses?: boolean
},): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send({ command, }: CommandRequest,) {
      sent.push(command,);
      return refuses
        ? {
          ending: 'answered',
          record: {
            success: false,
            error: 'refused',
          },
        }
        : {
          ending: 'answered',
          record: { success: true, },
        };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 The options of a run whose silence bounds a test states.

 @param runFolder - folder of the run

 @param silentSeconds - seconds of silence after which the driver writes its line

 @param silentEndSeconds - seconds of silence past which the driver ends the command

 @returns options of this run
 */
function optionsOf({
  runFolder,
  silentSeconds,
  silentEndSeconds,
  idleSeconds = 0,
  milliseconds = RUN_MILLISECONDS,
}: {
  readonly runFolder: string;
  readonly silentSeconds: number;
  readonly silentEndSeconds: number;
  readonly idleSeconds?: number;
  readonly milliseconds?: number
},): DriverOptions {
  return {
    runFolder,
    workingFolder: runFolder,
    promptFile: join(runFolder, 'prompt.md',),
    provider: 'stand-in',
    model: 'stand-in-model',
    piArguments: [],
    wallSeconds: milliseconds / MILLISECONDS_PER_SECOND,
    idleSeconds,
    silentSeconds,
    silentEndSeconds,
    turnCap: Number.MAX_SAFE_INTEGER,
    retryCap: 0,
    retryBaseSeconds: 1,
    startPauseSeconds: 0,
  };
}

/**
 A watch over a run whose clock a test moves.

 @param run - the run at its clock

 @param connection - the Pi the watch sends to

 @param silentSeconds - seconds of silence after which the driver writes its line

 @param silentEndSeconds - seconds of silence past which the driver ends the command

 @returns what the watch reads
 */
function watchOf({
  run,
  connection,
  silentSeconds,
  silentEndSeconds,
}: {
  readonly run: RunAtAClock;
  readonly connection: ConnectionToPi;
  readonly silentSeconds: number;
  readonly silentEndSeconds: number
},): WatchRequest {
  return {
    connection,
    record: run.record,
    options: optionsOf({
      runFolder: run.folder,
      silentSeconds,
      silentEndSeconds,
    },),
    inbox: join(run.folder, 'inbox',),
    deadline: Date.now() + RUN_MILLISECONDS,
  };
}

/**
 One feed line as a record, so a test reads the feed the way the driver wrote it and nothing else.

 @param line - text of one feed line

 @returns the record it holds
 */
function recordOf(line: string,): Record<string, unknown> {
  const parsed: unknown = JSON.parse(line,);
  if (!isRecord(parsed,)) {
    throw new Error(`the feed line ${line} is not a record`,);
  }
  return parsed;
}

/**
 What a run's feed holds.

 @param feedFile - the run's feed

 @returns the records, in the order they were written
 */
async function feedOf(feedFile: string,): Promise<readonly Record<string, unknown>[]> {
  try {
    return (await readFile(feedFile, 'utf8',)).split('\n',)
      .filter((line,) => line !== '')
      .map(recordOf,);
  }
  catch (error) {
    // A run that wrote no feed yet has nothing to show, which a test asserts about as an empty feed.
    if (((error as { readonly code?: unknown }).code) === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

/**
 The feed lines of one kind.

 @param feed - the run's feed

 @param kind - the kind a line carries

 @returns the lines of that kind
 */
function linesOfKind(feed: readonly Record<string, unknown>[], kind: string,): readonly Record<string, unknown>[] {
  return feed.filter(function ofKind(line,) {
    return line.kind === kind;
  },);
}

/**
 The lines of one kind a run's feed holds, waited for, so a test reads what the feed's stream wrote rather than what it was asked to
 write. The wait is bounded, and a line that never came is left to the assertion that follows to name.

 @param feedFile - the run's feed

 @param kind - the kind a line carries

 @param count - how many lines of that kind the test needs

 @returns the lines of that kind
 */
async function linesOfKindWhenWritten(
  feedFile: string,
  kind: string,
  count = 1,
): Promise<readonly Record<string, unknown>[]> {
  /**
   When the wait is over.
   */
  const deadline = Date.now() + FEED_WRITE_MILLISECONDS;
  for (;;) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the feed is read again after each wait, which is the whole point of the loop.
    const lines = linesOfKind(await feedOf(feedFile,), kind,);
    if ((lines.length >= count) || (Date.now() >= deadline)) {
      return lines;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- one feed to look at, and a wait that cannot outlast the deadline above.
    await wait(LOOK_MILLISECONDS,);
  }
}

/**
 What a run's summary says about its silences, which is the list a test reads.

 @param record - the run's record

 @returns the summary
 */
function summaryOf(record: RecordOfPiRun,): RunSummary {
  return record.summarize({
    endReason: 'idle',
    piExit: { code: 0, },
    piHasExited: true,
    sessionStats: null,
  },);
}

/**
 Installs the stand-in as `pi` in a folder a test puts on `PATH`.

 @param folder - the folder the launcher goes in
 */
async function installStandIn(folder: string,): Promise<void> {
  /**
   The launcher the driver finds as `pi`.
   */
  const launcher = join(folder, 'pi',);
  await writeFile(launcher, `#!/usr/bin/env bash\nexec '${process.execPath}' '${STAND_IN}' "$@"\n`,);
  await chmod(launcher, 0o700,);
}

/**
 How the driver ended and what it wrote to its standard error.

 @param running - the driver's own promise

 @returns its status and what it printed
 */
async function endOf(running: ReturnType<typeof execFileAsync>,): Promise<{
  readonly status: number;
  readonly standardError: string
}> {
  try {
    /**
     The driver's own end.
     */
    const finished = await running;
    return {
      status: CLEAN_STATUS,
      standardError: String(finished.stderr,),
    };
  }
  catch (error) {
    /**
     What the operating system said about the driver's end, read field by field because a killed process has no exit code.
     */
    const failure = error as { readonly code?: unknown; readonly stderr?: unknown };
    return {
      status: ((typeof failure.code) === 'number') ? failure.code : 1,
      standardError: ((typeof failure.stderr) === 'string') ? failure.stderr : '',
    };
  }
}

/**
 The lines the stand-in logged, which is nothing when it never started.

 @param log - the stand-in's log

 @returns its lines
 */
async function loggedLinesOf(log: string,): Promise<readonly string[]> {
  try {
    return (await readFile(log, 'utf8',)).split('\n',).filter((line,) => line !== '');
  }
  catch (error) {
    // A run that never started its stand-in wrote no line, which is what a test reads as nothing said.
    if (((error as { readonly code?: unknown }).code) !== MISSING_PATH_CODE) {
      throw error;
    }
    return [];
  }
}

/**
 Tells whether a process is there.

 @param pid - the process number

 @returns whether a signal of nothing reached it
 */
function aProcessIsAlive(pid: number,): boolean {
  try {
    process.kill(pid, 0,);
    return true;
  }
  catch (error) {
    // A process that is gone is what this asks about; any other refusal is a failure of the test rather than an answer.
    if (failureCodeOf(error,) !== NO_SUCH_PROCESS_CODE) {
      throw error;
    }
    return false;
  }
}

/**
 Ends a driver a test left running, bounded, so its folder can go and no fixture outlives the test.

 @param child - the driver's process
 */
async function stopBounded(child: ChildProcess,): Promise<void> {
  if ((child.exitCode !== null) || (child.signalCode !== null)) {
    return;
  }
  child.kill('SIGKILL',);
  /**
   When the wait for it is over.
   */
  const deadline = Date.now() + STOP_GRACE_MILLISECONDS;
  while (((child.exitCode === null) && (child.signalCode === null)) && (Date.now() < deadline)) {
    /* oxlint-disable-next-line eslint/no-await-in-loop -- one process to end, and a wait that cannot outlast the deadline above. */
    await new Promise(function tick(resolve,) {
      setTimeout(resolve, 10,);
    },);
  }
}

/**
 What one run through the bin left.

 @param summary - the run's summary, or the fields of one nobody wrote

 @param feed - the run's feed

 @param commands - every command the stand-in was sent, in order

 @param messages - the text of every prompt and steer the stand-in was sent, in order

 @param status - the driver's exit status

 @param standardError - what the driver wrote to its standard error

 @param descendants - the process numbers the stand-in named as descendants holding its output

 @returns what a test asserts about
 */
type RunThroughTheBin = {
  readonly summary: RunSummary;
  readonly feed: readonly Record<string, unknown>[];
  readonly commands: readonly string[];
  readonly messages: readonly string[];
  readonly status: number;
  readonly standardError: string;
  readonly descendants: readonly number[];
};

/**
 What a test sees when a run wrote no summary; no field of it can pass for a real run's.
 */
const ABSENT_SUMMARY: RunSummary = {
  endReason: 'driver-failed',
  totalMilliseconds: 0,
  turns: 0,
  settled: 0,
  toolHistogram: {},
  toolErrors: [],
  dialogs: [],
  sessionStats: null,
  lastAssistantText: '',
  retries: 0,
  rateLimitErrors: 0,
  piStartAttempts: 1,
};

/**
 Reads a run's summary.

 @param file - the summary file

 @returns the summary, or the fields of one nobody wrote
 */
async function readSummary(file: string,): Promise<RunSummary> {
  try {
    return JSON.parse(await readFile(file, 'utf8',),) as RunSummary;
  }
  catch (error) {
    if (((error as { readonly code?: unknown }).code) === 'ENOENT') {
      return ABSENT_SUMMARY;
    }
    throw error;
  }
}

/**
 Runs the driver's bin against the stand-in Pi, with the silence bounds a test states.

 @param scenario - the stand-in's behaviour

 @param silentSeconds - seconds of silence after which the driver writes its line

 @param silentEndSeconds - seconds of silence past which the driver ends the command

 @param wallSeconds - seconds the run may last

 @returns what the run left
 */
async function runTheBin({
  scenario,
  silentSeconds,
  silentEndSeconds,
  wallSeconds,
}: {
  readonly scenario: string;
  readonly silentSeconds: number;
  readonly silentEndSeconds: number;
  readonly wallSeconds: number
},): Promise<RunThroughTheBin> {
  await using folder = await makeFolder();
  /**
   Where the run writes its feed and its summary.
   */
  const runFolder = join(folder.path, 'run',);
  /**
   The first prompt.
   */
  const promptFile = join(folder.path, 'prompt.md',);
  await writeFile(promptFile, 'do the work',);
  /**
   Where the stand-in writes every command it was sent.
   */
  const log = join(folder.path, 'commands.log',);
  await installStandIn(folder.path,);
  /**
   The driver's process.
   */
  const running = execFileAsync(process.execPath, [
    DRIVER,
    `--run-folder=${runFolder}`,
    `--working-folder=${folder.path}`,
    `--prompt-file=${promptFile}`,
    '--provider=stand-in',
    '--model=stand-in-model',
    `--wall-seconds=${String(wallSeconds,)}`,
    `--idle-seconds=${String(IDLE_SECONDS,)}`,
    `--turn-cap=${String(TURN_CAP,)}`,
    `--silent-seconds=${String(silentSeconds,)}`,
    `--silent-end-seconds=${String(silentEndSeconds,)}`,
  ], {
    env: {
      PATH: [
        folder.path,
        dirname(process.execPath,),
        '/usr/bin',
      ].join(delimiter,),
      HOME: process.env.HOME,
      STAND_IN_PI_SCENARIO: scenario,
      STAND_IN_PI_LOG: log,
    },
    encoding: 'utf8',
    timeout: (wallSeconds + TEST_TEARDOWN_ALLOWANCE_SECONDS) * MILLISECONDS_PER_SECOND,
    killSignal: 'SIGKILL',
  },);
  /**
   How the driver ended, which is an end of its own rather than a failure to watch for.
   */
  const ending = await endOf(running,);
  await stopBounded(running.child,);
  /**
   The stand-in's own lines, once it has ended.
   */
  const logged = await loggedLinesOf(log,);
  /**
   Every command the stand-in was sent, in order.
   */
  const sentCommands = logged.filter((line,) => line.startsWith('command ',))
    .map((line,) => JSON.parse(line.slice('command '.length,),) as Record<string, unknown>);
  return {
    summary: await readSummary(join(runFolder, 'summary.json',),),
    feed: await feedOf(join(runFolder, 'feed.jsonl',),),
    commands: sentCommands.map((command,) => String(command.type,),),
    messages: sentCommands.filter((command,) => (command.type === 'prompt') || (command.type === 'steer'))
      .map((command,) => String(command.message,),),
    status: ending.status,
    standardError: ending.standardError,
    descendants: logged.filter((line,) => line.startsWith('marker descendant ',))
      .flatMap((line,) => line.slice('marker descendant '.length,).split(' ',))
      .map(Number,),
  };
}

await describe({
  name: 'a command that says nothing',
  children: [
    it({
      name: 'writes one line naming the command and how long, keeps it in the summary, and does not count it as the run speaking',
      fn: async () => {
        await using run = await startRunAtAClock();
        /**
         What the watch sent Pi, which a silence notice must leave empty.
         */
        const sent: Record<string, unknown>[] = [];
        const watch = watchOf({
          run,
          connection: recordingConnection({ sent, },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        observe(run, { type: 'agent_start', },);
        observeCommandStart(run,);
        // The command says nothing, so the run's silence is the command's whole age.
        run.clock.at = SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND;
        expect(await aSilenceInTheRun({
          watch,
          waited: {},
        },),).toBe('still-running',);
        /**
         The lines the run's feed holds for the first notice.
         */
        const notices = await linesOfKindWhenWritten(join(run.folder, 'feed.jsonl',), 'silence',);
        expect(notices,).toHaveLength(1,);
        expect(notices[0]?.silentMilliseconds,).toBe(SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND,);
        /**
         The command the first notice names.
         */
        const named = fieldsOf(recordsIn(notices[0]?.commands,)[0],);
        expect(named.tool,).toBe('bash',);
        expect(String(named.args,),).toContain('rg -l',);
        // The notice is written down without counting as the run's own activity, so the silence it names keeps growing: a line
        // that reset the clock would keep the end of the command from ever coming due.
        expect(run.record.silentMilliseconds(),).toBeGreaterThanOrEqual(
          SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND,
        );
        expect(sent,).toHaveLength(0,);
        expect(summaryOf(run.record,).silences,).toHaveLength(1,);
        expect(summaryOf(run.record,).silences?.[0]?.abort,).toBeUndefined();
      },
    },),
    it({
      name: 'names one stretch once, and names the next one after the run has spoken',
      fn: async () => {
        await using run = await startRunAtAClock();
        const watch = watchOf({
          run,
          connection: recordingConnection({ sent: [], },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        /**
         Where the driver stands with this run's silence.
         */
        const waited = {};
        observe(run, { type: 'agent_start', },);
        observeCommandStart(run,);
        run.clock.at = SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND;
        await aSilenceInTheRun({
          watch,
          waited,
        },);
        // A second look at the same stretch finds the silence longer, and is still the one already named.
        run.clock.at = 2 * SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND;
        await aSilenceInTheRun({
          watch,
          waited,
        },);
        /**
         The lines of one kind the feed holds after two looks at one stretch.
         */
        const afterOneStretch = await linesOfKindWhenWritten(join(run.folder, 'feed.jsonl',), 'silence',);
        expect(afterOneStretch,).toHaveLength(1,);
        // The run speaks, so the next silence is a new stretch with a line of its own.
        observeCommandEnd(run,);
        run.clock.at = 3 * SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND;
        await aSilenceInTheRun({
          watch,
          waited,
        },);
        /**
         The lines of that kind once the second stretch has its own.
         */
        const afterTwo = await linesOfKindWhenWritten(join(run.folder, 'feed.jsonl',), 'silence', 2,);
        expect(afterTwo,).toHaveLength(2,);
        expect(summaryOf(run.record,).silences,).toHaveLength(2,);
      },
    },),
    it({
      name: 'ends the command past the second time, then sends the agent a message of its own naming what was ended',
      fn: async () => {
        await using run = await startRunAtAClock();
        /**
         Every command the watch sent Pi, in order.
         */
        const sent: Record<string, unknown>[] = [];
        const watch = watchOf({
          run,
          connection: endingConnectionFor({
            sent,
            run,
            toolCallId: '',
          },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        observe(run, { type: 'agent_start', },);
        observeCommandStart(run,);
        run.clock.at = SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND;
        expect(await aSilenceInTheRun({
          watch,
          waited: {},
        },),).toBe('still-running',);
        expect(sent.map((command,) => command.type,),).toEqual([
          'abort',
          'prompt',
        ],);
        /**
         What the agent was told.
         */
        const told = String(sent[1]?.message,);
        expect(told,).toContain('The driver ended the operation you were in, after it had said nothing for',);
        expect(told,).toContain('rg -l --fixed-strings something',);
        expect(told,).toContain(`${String(SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,)} ms`,);
        expect(told,).toContain('no bound of its own',);
        expect(told,).toContain('give the bash tool its `timeout` argument',);
        /**
         The lines the feed holds for the ending.
         */
        const endings = await linesOfKindWhenWritten(join(run.folder, 'feed.jsonl',), 'silence-ended',);
        expect(endings,).toHaveLength(1,);
        expect(endings[0]?.abort,).toBe('done',);
        expect(endings[0]?.message,).toBe('done',);
        /**
         What the summary keeps of it.
         */
        const kept = summaryOf(run.record,).silences ?? [];
        expect(kept.at(-1,)?.abort,).toBe('done',);
        expect(kept.at(-1,)?.message,).toBe('done',);
        expect(kept.at(-1,)?.commands?.[0]?.tool,).toBe('bash',);
        // The ending is the driver talking, so the clock starts again from it rather than staying hours behind.
        expect(run.record.silentMilliseconds(),).toBe(0,);
      },
    },),
    it({
      name: 'counts a command output report as the run saying something, so a silence is measured from that report',
      fn: async () => {
        await using run = await startRunAtAClock();
        /**
         Every command the watch sent Pi, which this silence must leave empty.
         */
        const sent: Record<string, unknown>[] = [];
        const watch = watchOf({
          run,
          connection: recordingConnection({ sent, },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        observe(run, { type: 'agent_start', },);
        observeCommandStart(run,);
        // The command reports one chunk of output, which is the run saying something rather than the command being silent.
        run.clock.at = OUTPUT_REPORTED_AT_MILLISECONDS;
        observe(run, {
          type: 'tool_execution_update',
          toolName: 'bash',
          args: COMMAND_ARGS,
          partialResult: { content: [], },
        },);
        /**
         When the report is old enough to be a silence the notice names, which is the notice counted from that report.
         */
        const noticeAt = OUTPUT_REPORTED_AT_MILLISECONDS
          + (SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND);
        // A silence measured from the command's start is far past the notice here; measured from its output it is not.
        run.clock.at = noticeAt - 1;
        expect(await aSilenceInTheRun({
          watch,
          waited: {},
        },),).toBe('still-running',);
        expect(linesOfKind(await feedOf(join(run.folder, 'feed.jsonl',),), 'silence',),).toHaveLength(0,);
        // One millisecond past the notice is a silence, so the check above is a bound rather than a notice that never comes.
        run.clock.at = noticeAt;
        await aSilenceInTheRun({
          watch,
          waited: {},
        },);
        expect(await linesOfKindWhenWritten(join(run.folder, 'feed.jsonl',), 'silence',),).toHaveLength(1,);
        expect(sent,).toHaveLength(0,);
      },
    },),
    it({
      name: 'leaves a command alone until the bound it declared has passed, and ends it a minute past it',
      fn: async () => {
        await using run = await startRunAtAClock();
        /**
         Every command the watch sent Pi, in order.
         */
        const sent: Record<string, unknown>[] = [];
        const watch = watchOf({
          run,
          connection: endingConnectionFor({
            sent,
            run,
            toolCallId: '',
          },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        observe(run, { type: 'agent_start', },);
        observeCommandStart(run, DECLARED_BOUND_SECONDS,);
        // Past the driver's own stated end, and well inside the bound this command declared for itself.
        run.clock.at = SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND;
        await aSilenceInTheRun({
          watch,
          waited: {},
        },);
        expect(sent,).toHaveLength(0,);
        // Past that bound by the grace, which is where a command its own bound could not end is the driver's to end.
        run.clock.at = (DECLARED_BOUND_SECONDS + SILENT_BOUND_GRACE_SECONDS) * MILLISECONDS_PER_SECOND;
        await aSilenceInTheRun({
          watch,
          waited: {},
        },);
        expect(sent.map((command,) => command.type,),).toEqual([
          'abort',
          'prompt',
        ],);
        expect(String(sent[1]?.message,),).toContain(
          `a bound of ${String(DECLARED_BOUND_SECONDS,)} seconds of its own`,
        );
      },
    },),
    it({
      name: 'ends nothing when the run is not sitting in a command, which is a model that has answered nothing',
      fn: async () => {
        await using run = await startRunAtAClock();
        /**
         Every command the watch sent Pi, in order.
         */
        const sent: Record<string, unknown>[] = [];
        const watch = watchOf({
          run,
          connection: recordingConnection({ sent, },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        observe(run, { type: 'agent_start', },);
        run.clock.at = 3 * SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND;
        expect(await aSilenceInTheRun({
          watch,
          waited: {},
        },),).toBe('still-running',);
        expect(sent,).toHaveLength(0,);
        /**
         The lines the feed holds for a silence with no command in it.
         */
        const notices = await linesOfKindWhenWritten(join(run.folder, 'feed.jsonl',), 'silence',);
        expect(notices,).toHaveLength(1,);
        expect(notices[0]?.command,).toBeUndefined();
      },
    },),
    it({
      name: 'ends the run as prompt-rejected when Pi refuses the message that says what was ended',
      fn: async () => {
        await using run = await startRunAtAClock();
        const watch = watchOf({
          run,
          connection: endingConnection({
            sent: [],
            abort: 'done',
            message: 'refused',
            before: {
              abort: function settle() {
                observeAbortedCall(run, '',);
              },
            },
          },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        observe(run, { type: 'agent_start', },);
        observeCommandStart(run,);
        run.clock.at = SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND;
        expect(await aSilenceInTheRun({
          watch,
          waited: {},
        },),).toBe('prompt-rejected',);
        /**
         What the summary says about the ending nobody heard.
         */
        const kept = summaryOf(run.record,).silences ?? [];
        expect(kept.at(-1,)?.message,).toBe('refused',);
        expect(kept.at(-1,)?.problems,).toContain('Pi refused the message: refused',);
      },
    },),
    it({
      name: 'writes the notice for a command that says nothing through the bin, and lets its run go on',
      fn: async () => {
        const run = await runTheBin({
          scenario: 'silent-command',
          silentSeconds: NOTICE_SECONDS,
          // Above every wall clock this run has, so nothing but the run's own end moves it.
          silentEndSeconds: 600,
          wallSeconds: NOTICE_RUN_WALL_SECONDS,
        },);
        /**
         The lines the driver wrote for the silence.
         */
        const notices = linesOfKind(run.feed, 'silence',);
        expect(notices.length,).toBeGreaterThanOrEqual(1,);
        /**
         The command the first notice names.
         */
        const named = fieldsOf(recordsIn(notices[0]?.commands,)[0],);
        expect(named.tool,).toBe('bash',);
        expect(String(named.args,),).toContain('rg -l',);
        // One prompt is the driver's own, and no second went out: a notice ends nothing, so the agent was never told.
        expect(run.commands.filter((command,) => command === 'prompt',),).toHaveLength(1,);
        expect(linesOfKind(run.feed, 'silence-ended',),).toHaveLength(0,);
        expect(run.summary.silences?.[0]?.abort,).toBeUndefined();
        expect(run.summary.endReason,).toBe('wall-clock',);
      },
    },),
    it({
      name: 'ends the command through the bin past the second time, and the agent goes on in a turn of its own',
      fn: async () => {
        const run = await runTheBin({
          scenario: 'silent-command',
          silentSeconds: NOTICE_SECONDS,
          silentEndSeconds: ENDING_SECONDS,
          wallSeconds: ENDING_RUN_WALL_SECONDS,
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        // The abort ends the operation in flight, and only after it was answered does the message go.
        expect(run.commands.filter((command,) => command === 'abort',).length,).toBeGreaterThanOrEqual(1,);
        expect(run.messages,).toHaveLength(2,);
        expect(run.messages[1],).toContain('The driver ended the operation you were in',);
        expect(run.messages[1],).toContain('rg -l --fixed-strings something',);
        expect(run.messages[1],).toContain('give the bash tool its `timeout` argument',);
        // The agent's next turn is the run going on, and its reply is what the run ends with.
        expect(linesOfKind(run.feed, 'silence-ended',),).toHaveLength(1,);
        expect(linesOfKind(run.feed, 'turn_start',).length,).toBeGreaterThanOrEqual(2,);
        expect(run.summary.lastAssistantText,).toBe(WENT_ON,);
        expect(run.summary.endReason,).toBe('idle',);
        /**
         What the summary keeps of the command the driver ended, which is the one silence it ended.
         */
        const ended = (run.summary.silences ?? []).filter((silence,) => silence.abort === 'done');
        expect(ended,).toHaveLength(1,);
        expect(ended[0]?.message,).toBe('done',);
        expect(ended[0]?.commands?.[0]?.tool,).toBe('bash',);
      },
    },),
    it({
      name: 'never ends a command that reports output steadily through the bin',
      fn: async () => {
        const run = await runTheBin({
          scenario: 'silent-command-reports-output',
          silentSeconds: STEADY_NOTICE_SECONDS,
          silentEndSeconds: STEADY_ENDING_SECONDS,
          wallSeconds: ENDING_RUN_WALL_SECONDS,
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(linesOfKind(run.feed, 'silence',),).toHaveLength(0,);
        expect(linesOfKind(run.feed, 'silence-ended',),).toHaveLength(0,);
        expect(run.messages,).toHaveLength(1,);
        expect(run.summary.silences,).toBeUndefined();
        expect(run.summary.endReason,).toBe('idle',);
      },
    },),
    it({
      name: 'ends a command whose own process is gone while a descendant holds the output, and kills that descendant',
      fn: async () => {
        const run = await runTheBin({
          scenario: 'silent-command-descendant-holds-the-pipe',
          silentSeconds: NOTICE_SECONDS,
          silentEndSeconds: ENDING_SECONDS,
          wallSeconds: ENDING_RUN_WALL_SECONDS,
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.descendants.length,).toBeGreaterThanOrEqual(1,);
        expect(run.messages,).toHaveLength(2,);
        expect(run.messages[1],).toContain('The driver ended the operation you were in',);
        expect(linesOfKind(run.feed, 'silence-ended',),).toHaveLength(1,);
        expect(run.summary.lastAssistantText,).toBe(WENT_ON,);
        expect(run.summary.endReason,).toBe('idle',);
        // The abort reaches what the command started, so the descendant that held the output is gone rather than outliving the run.
        for (const pid of run.descendants) {
          expect(aProcessIsAlive(pid,),).toBe(false,);
        }
      },
    },),
  ],
},);

await describe({
  name: theOperationIsDueToEnd.name,
  children: [
    it({
      name: 'is due only once the run has said nothing for the stated time as well as every call having run that long',
      fn: async () => {
        /**
         One call that has run as long as the stated time.
         */
        const command = {
          tool: 'bash',
          args: COMMAND_ARGS,
          milliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
        };
        expect(theOperationIsDueToEnd({
          silentMilliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
          silentEndMilliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
          graceSeconds: SILENT_BOUND_GRACE_SECONDS,
          commands: [command,],
        },),).toBe(true,);
        expect(theOperationIsDueToEnd({
          silentMilliseconds: (SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND) - 1,
          silentEndMilliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
          graceSeconds: SILENT_BOUND_GRACE_SECONDS,
          commands: [command,],
        },),).toBe(false,);
      },
    },),
    it({
      name: 'waits for a call whose own bound has not passed, and is due a grace after that bound',
      fn: async () => {
        /**
         The call that declared a bound longer than the driver's own stated end, beside one that declared none.
         */
        const longBound = {
          tool: 'bash',
          args: COMMAND_ARGS,
          milliseconds: DECLARED_BOUND_SECONDS * MILLISECONDS_PER_SECOND,
          boundSeconds: DECLARED_BOUND_SECONDS,
        };
        /**
         The call that declared no bound, which is past the stated end.
         */
        const unbounded = {
          tool: 'bash',
          args: COMMAND_ARGS,
          milliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
        };
        expect(theOperationIsDueToEnd({
          silentMilliseconds: (SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND) + 1,
          silentEndMilliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
          graceSeconds: SILENT_BOUND_GRACE_SECONDS,
          commands: [longBound, unbounded,],
        },),).toBe(false,);
        expect(theOperationIsDueToEnd({
          silentMilliseconds: (DECLARED_BOUND_SECONDS + SILENT_BOUND_GRACE_SECONDS) * MILLISECONDS_PER_SECOND,
          silentEndMilliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
          graceSeconds: SILENT_BOUND_GRACE_SECONDS,
          commands: [
            {
              ...longBound,
              milliseconds: (DECLARED_BOUND_SECONDS + SILENT_BOUND_GRACE_SECONDS) * MILLISECONDS_PER_SECOND,
            },
            unbounded,
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'is never due without a call in flight, which is a silence no command is sitting in',
      fn: async () => {
        expect(theOperationIsDueToEnd({
          silentMilliseconds: 10 * SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
          silentEndMilliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
          graceSeconds: SILENT_BOUND_GRACE_SECONDS,
          commands: [],
        },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: declaredBoundOf.name,
  children: [
    it({
      name: 'reads the bound a command declared, and nothing else',
      fn: async () => {
        expect(declaredBoundOf({ command: 'mise run test', timeout: DECLARED_BOUND_SECONDS, },),)
          .toEqual({ boundSeconds: DECLARED_BOUND_SECONDS, },);
        expect(declaredBoundOf({ command: 'mise run test', },),).toEqual({},);
        expect(declaredBoundOf({ command: 'x', timeout: 'six hundred', },),).toEqual({},);
        expect(declaredBoundOf({ command: 'x', timeout: 0, },),).toEqual({},);
        expect(declaredBoundOf('a string',),).toEqual({},);
        expect(declaredBoundOf({},),).toEqual({},);
      },
    },),
  ],
},);

await describe({
  name: endedCommandMessageOf.name,
  children: [
    it({
      name: 'names the command, how long it said nothing, whether it declared a bound, and what to do next',
      fn: async () => {
        /**
         What the agent is told about a command that declared no bound.
         */
        const told = endedCommandMessageOf({
          commands: [{
            tool: 'bash',
            args: COMMAND_ARGS,
            milliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
          },],
          silentMilliseconds: SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND,
        },);
        expect(told,).toContain('The driver ended the operation you were in, after it had said nothing for',);
        expect(told,).toContain('rg -l --fixed-strings something',);
        expect(told,).toContain('no bound of its own',);
        expect(told,).toContain('A command that may run long needs a bound of its own',);
        expect(told,).toContain('Go on from here',);
      },
    },),
    it({
      name: 'says a declared bound had passed when the command outlived one',
      fn: async () => {
        /**
         What the agent is told about a command that declared a bound.
         */
        const told = endedCommandMessageOf({
          commands: [{
            tool: 'bash',
            args: COMMAND_ARGS,
            milliseconds: DECLARED_BOUND_SECONDS * MILLISECONDS_PER_SECOND,
            boundSeconds: DECLARED_BOUND_SECONDS,
          },],
          silentMilliseconds: DECLARED_BOUND_SECONDS * MILLISECONDS_PER_SECOND,
        },);
        expect(told,).toContain(`a bound of ${String(DECLARED_BOUND_SECONDS,)} seconds of its own`,);
      },
    },),
  ],
},);

await describe({
  name: keptCommand.name,
  children: [
    it({
      name: 'cuts the arguments to the length one feed line carries and keeps the bound',
      fn: async () => {
        /**
         A command whose arguments are longer than a feed line keeps.
         */
        const kept = keptCommand({
          command: {
            tool: 'bash',
            args: { command: 'x'.repeat(2_000,), },
            milliseconds: 5,
            boundSeconds: DECLARED_BOUND_SECONDS,
          },
        },);
        expect(typeof kept.args,).toBe('string',);
        expect(String(kept.args,),).toContain('...[',);
        expect(kept.boundSeconds,).toBe(DECLARED_BOUND_SECONDS,);
        expect(kept.milliseconds,).toBe(5,);
      },
    },),
  ],
},);

/**
 What one step of ending a silent command got: an answer that did it, a refusal, or no answer at all.
 */
type StepAnswer = 'done' | 'refused' | 'unanswered';

/**
 The answer a step of ending a silent command gets, in the shape a wait on Pi carries.

 @param answer - what that step got

 @returns Pi's answer, as the connection reports it

 @example
 ```ts
 const answer = answerOf('refused',);
 ```
 */
function answerOf(answer: StepAnswer,): CommandAnswer {
  if (answer === 'unanswered') {
    return {
      ending: 'silence',
      problem: 'Pi answered nothing',
    };
  }
  return (answer === 'refused')
    ? {
      ending: 'answered',
      record: {
        success: false,
        error: 'refused',
      },
    }
    : {
      ending: 'answered',
      record: { success: true, },
    };
}

/**
 A connection whose two steps of ending a silent command answer as a test states, and which lets each answer carry the events a
 real Pi writes before it, so the settle the driver reads is the one Pi made.

 @param sent - where the commands the watch sends are recorded

 @param abort - what the abort gets

 @param message - what the recovery message gets

 @param before - what Pi writes before it answers a command, by the command's type

 @returns the connection a watch reads
 */
function endingConnection({
  sent,
  abort,
  message,
  before = {},
}: {
  readonly sent: Record<string, unknown>[];
  readonly abort: StepAnswer;
  readonly message: StepAnswer;
  readonly before?: Readonly<Record<string, () => void>>
},): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send({ command, }: CommandRequest,) {
      sent.push(command,);
      before[String(command.type,)]?.();
      return answerOf(((command.type === 'abort') ? abort : message),);
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 Writes the records a real Pi writes when an abort ends the operation in flight: the tool's end and the settle after it.

 @param run - the run at its clock

 @param toolCallId - the call the abort ended
 */
function observeAbortedCall(run: RunAtAClock, toolCallId: string,): void {
  observe(run, {
    type: 'tool_execution_end',
    toolCallId,
    toolName: 'bash',
    isError: true,
    result: { content: [], },
  },);
  observe(run, { type: 'agent_settled', aborted: true, },);
}

/**
 Starts a run that is sitting in one silent command, which is the state every control of ending one begins in.

 @returns the run at its clock
 */
async function startRunInASilentCommand(): Promise<RunAtAClock> {
  /**
   The run.
   */
  const run = await startRunAtAClock();
  observe(run, { type: 'agent_start', },);
  observe(run, {
    type: 'tool_execution_start',
    toolCallId: 'call-silent',
    toolName: 'bash',
    args: COMMAND_ARGS,
  },);
  run.clock.at = (SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND) + 1;
  return run;
}

await describe({
  name: 'ending a silent command',
  children: [
    it({
      name: 'answers three things of the abort: done, refused, and no answer at all',
      fn: async () => {
        /**
         Each run, its commands, and what the watch answered, one per answer the abort can give.
         */
        const outcomes: Record<string, unknown>[] = [];
        for (const abort of ['done', 'refused', 'unanswered'] as const) {
          /* oxlint-disable eslint/no-await-in-loop -- one run per answer, ended before the next is started. */
          await using run = await startRunInASilentCommand();
          /**
           Every command the watch sent Pi.
           */
          const sent: Record<string, unknown>[] = [];
          const watch = watchOf({
            run,
            connection: endingConnection({
              sent,
              abort,
              message: 'done',
              before: {
                abort: function settle() {
                  observeAbortedCall(run, 'call-silent',);
                },
              },
            },),
            silentSeconds: SILENT_NOTICE_SECONDS,
            silentEndSeconds: SILENT_ENDING_SECONDS,
          },);
          /**
           Why the run cannot go on, or that it goes on.
           */
          const progress = await aSilenceInTheRun({
            watch,
            waited: {},
          },);
          /**
           What the summary keeps of the attempt.
           */
          const kept = summaryOf(run.record,).silences?.at(-1,);
          outcomes.push({
            abort,
            progress,
            keptAbort: kept?.abort,
            commandTypes: sent.map((command,) => command.type,),
          },);
          // oxlint-enable eslint/no-await-in-loop
        }
        expect(outcomes,).toEqual([
          {
            abort: 'done',
            progress: 'still-running',
            keptAbort: 'done',
            commandTypes: ['abort', 'prompt'],
          },
          {
            abort: 'refused',
            progress: 'prompt-rejected',
            keptAbort: 'refused',
            commandTypes: ['abort'],
          },
          {
            abort: 'unanswered',
            progress: 'silent-command-unanswered',
            keptAbort: 'unanswered',
            commandTypes: ['abort'],
          },
        ],);
      },
    },),
    it({
      name: 'answers three things of the recovery message: done, refused, and no answer at all',
      fn: async () => {
        /**
         What the watch answered for each answer the message can give.
         */
        const outcomes: Record<string, unknown>[] = [];
        for (const message of ['done', 'refused', 'unanswered'] as const) {
          /* oxlint-disable eslint/no-await-in-loop -- one run per answer, ended before the next is started. */
          await using run = await startRunInASilentCommand();
          /**
           Every command the watch sent Pi.
           */
          const sent: Record<string, unknown>[] = [];
          const watch = watchOf({
            run,
            connection: endingConnection({
              sent,
              abort: 'done',
              message,
              before: {
                abort: function settle() {
                  observeAbortedCall(run, 'call-silent',);
                },
              },
            },),
            silentSeconds: SILENT_NOTICE_SECONDS,
            silentEndSeconds: SILENT_ENDING_SECONDS,
          },);
          outcomes.push({
            message,
            progress: await aSilenceInTheRun({
              watch,
              waited: {},
            },),
            keptMessage: summaryOf(run.record,).silences?.at(-1,)?.message,
          },);
          // oxlint-enable eslint/no-await-in-loop
        }
        expect(outcomes,).toEqual([
          {
            message: 'done',
            progress: 'still-running',
            keptMessage: 'done',
          },
          {
            message: 'refused',
            progress: 'prompt-rejected',
            keptMessage: 'refused',
          },
          {
            message: 'unanswered',
            progress: 'silent-command-unanswered',
            keptMessage: 'unanswered',
          },
        ],);
      },
    },),
    it({
      name: 'sends no recovery message to a session that did not settle, and says the settle was what it waited for',
      fn: async () => {
        await using run = await startRunInASilentCommand();
        /**
         Every command the watch sent Pi.
         */
        const sent: Record<string, unknown>[] = [];
        // Pi answers the abort but writes no settle, which is a session still at work whatever its answer said.
        const watch = watchOf({
          run,
          connection: endingConnection({
            sent,
            abort: 'done',
            message: 'done',
          },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        expect(await aSilenceInTheRun({
          watch,
          waited: {},
        },),).toBe('silent-command-unanswered',);
        expect(sent.map((command,) => command.type,),).toEqual(['abort',],);
        expect(summaryOf(run.record,).silences?.at(-1,)?.settled,).toBe(false,);
      },
    },),
    it({
      name: 'keeps the silence going when the abort ended nothing, so the next look is not an hour away',
      fn: async () => {
        await using run = await startRunInASilentCommand();
        /**
         How long the run had said nothing when the abort was asked.
         */
        const silenceBefore = run.record.silentMilliseconds();
        const watch = watchOf({
          run,
          connection: endingConnection({
            sent: [],
            abort: 'unanswered',
            message: 'unanswered',
          },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        await aSilenceInTheRun({
          watch,
          waited: {},
        },);
        expect(run.record.silentMilliseconds(),).toBeGreaterThanOrEqual(silenceBefore,);
        expect(summaryOf(run.record,).silences?.at(-1,)?.abort,).toBe('unanswered',);
      },
    },),
    it({
      name: 'does not end the run as idle when the recovery message was never answered',
      fn: async () => {
        // The machine's clock, as the review's own reproduction had it: the idle path is a run that has settled and stayed quiet,
        // and a clock a test moves reaches it only by being moved past the idle wait as well.
        await using run = await startRunAtTheRealClock();
        observe(run, { type: 'agent_start', },);
        observe(run, {
          type: 'tool_execution_start',
          toolCallId: 'call-silent',
          toolName: 'bash',
          args: COMMAND_ARGS,
        },);
        /**
         Every command the watch sent Pi.
         */
        const sent: Record<string, unknown>[] = [];
        const endReason = await watchUntilEnded({
          connection: endingConnection({
            sent,
            abort: 'done',
            message: 'unanswered',
            before: {
              abort: function settle() {
                observeAbortedCall(run, 'call-silent',);
              },
            },
          },),
          record: run.record,
          options: optionsOf({
            runFolder: run.folder,
            silentSeconds: PAST_BOTH_TIMES_SECONDS,
            silentEndSeconds: PAST_BOTH_TIMES_SECONDS,
            milliseconds: RUN_MILLISECONDS,
          },),
          inbox: join(run.folder, 'inbox',),
          deadline: Date.now() + RUN_MILLISECONDS,
        },);
        expect(sent.map((command,) => command.type,),).toEqual(['abort', 'prompt',],);
        expect(endReason,).toBe('silent-command-unanswered',);
      },
    },),
    it({
      name: 'takes a call that ended for that call alone, not for the one started after it',
      fn: async () => {
        await using run = await startRunAtAClock();
        /**
         Every command the watch sent Pi.
         */
        const sent: Record<string, unknown>[] = [];
        observe(run, { type: 'agent_start', },);
        observe(run, {
          type: 'tool_execution_start',
          toolCallId: 'call-A',
          toolName: 'bash',
          args: { command: 'command-A', },
        },);
        run.clock.at = SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND;
        observe(run, {
          type: 'tool_execution_start',
          toolCallId: 'call-B',
          toolName: 'bash',
          args: {
            command: 'command-B',
            timeout: DECLARED_BOUND_SECONDS,
          },
        },);
        run.clock.at = 2 * SILENT_NOTICE_SECONDS * MILLISECONDS_PER_SECOND;
        observe(run, {
          type: 'tool_execution_end',
          toolCallId: 'call-A',
          toolName: 'bash',
          isError: false,
          result: { content: [], },
        },);
        /**
         The calls the run is still sitting in.
         */
        const tracked = run.record.commandsInFlight().map(function named(command,) {
          return fieldsOf(command.args,).command;
        },);
        expect(tracked,).toEqual(['command-B',],);
        const watch = watchOf({
          run,
          connection: endingConnection({
            sent,
            abort: 'done',
            message: 'done',
          },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        // The call still running declared a bound of its own that has not run out, so nothing is ended yet.
        run.clock.at = SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND;
        await aSilenceInTheRun({
          watch,
          waited: {},
        },);
        expect(sent,).toHaveLength(0,);
      },
    },),
    it({
      name: 'ends nothing while a call in flight has not outlived its own bound, and names every call when it is due',
      fn: async () => {
        await using run = await startRunAtAClock();
        /**
         Every command the watch sent Pi.
         */
        const sent: Record<string, unknown>[] = [];
        observe(run, { type: 'agent_start', },);
        observe(run, {
          type: 'tool_execution_start',
          toolCallId: 'long-bound-call',
          toolName: 'bash',
          args: {
            command: 'long-bound-command',
            timeout: DECLARED_BOUND_SECONDS,
          },
        },);
        run.clock.at = 1_000;
        observe(run, {
          type: 'tool_execution_start',
          toolCallId: 'unbounded-call',
          toolName: 'bash',
          args: { command: 'unbounded-command', },
        },);
        const watch = watchOf({
          run,
          connection: endingConnection({
            sent,
            abort: 'done',
            message: 'done',
            before: {
              abort: function endBoth() {
                observe(run, {
                  type: 'tool_execution_end',
                  toolCallId: 'unbounded-call',
                  toolName: 'bash',
                  isError: true,
                  result: { content: [], },
                },);
                observe(run, {
                  type: 'tool_execution_end',
                  toolCallId: 'long-bound-call',
                  toolName: 'bash',
                  isError: true,
                  result: { content: [], },
                },);
                observe(run, { type: 'agent_settled', aborted: true, },);
              },
            },
          },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        // The command without a bound has outlived the stated end, and its peer has not outlived the bound it declared.
        run.clock.at = (SILENT_ENDING_SECONDS * MILLISECONDS_PER_SECOND) + 1_000;
        expect(await aSilenceInTheRun({
          watch,
          waited: {},
        },),).toBe('still-running',);
        expect(sent,).toHaveLength(0,);
        // Past that bound by the grace, both calls have outlived what each declared, so the operation is due.
        run.clock.at = (DECLARED_BOUND_SECONDS + SILENT_BOUND_GRACE_SECONDS) * MILLISECONDS_PER_SECOND;
        expect(await aSilenceInTheRun({
          watch,
          waited: {},
        },),).toBe('still-running',);
        expect(sent.map((command,) => command.type,),).toEqual(['abort', 'prompt',],);
        /**
         What the agent was told, which names every command that was in flight.
         */
        const told = String(sent[1]?.message,);
        expect(told,).toContain('long-bound-command',);
        expect(told,).toContain('unbounded-command',);
      },
    },),
    it({
      name: 'tells a call by its own identifier, and by the tool\'s name only when Pi sends none',
      fn: async () => {
        await using run = await startRunAtAClock();
        observe(run, { type: 'agent_start', },);
        observe(run, {
          type: 'tool_execution_start',
          toolCallId: 'first',
          toolName: 'bash',
          args: { command: 'first-command', },
        },);
        observe(run, {
          type: 'tool_execution_start',
          toolCallId: 'second',
          toolName: 'bash',
          args: { command: 'second-command', },
        },);
        // Pi reports no identifier at all, which is the only case the fallback is for.
        observe(run, {
          type: 'tool_execution_end',
          toolName: 'bash',
          isError: false,
          result: { content: [], },
        },);
        expect(run.record
          .commandsInFlight()
          .map(function named(command,) {
            return fieldsOf(command.args,).command;
          },),).toEqual(['first-command',],);
      },
    },),
  ],
},);

await describe({
  name: 'what the ending of a silent operation leaves behind',
  children: [
    it({
      name: 'stays up for the turn an accepted recovery message starts, and ends on its wall clock when that turn never comes',
      fn: async () => {
        // The machine's clock: a turn that never comes is a silence, and the idle path is what a silence reaches.
        await using run = await startRunAtTheRealClock();
        observe(run, { type: 'agent_start', },);
        observe(run, {
          type: 'tool_execution_start',
          toolCallId: 'call-silent',
          toolName: 'bash',
          args: COMMAND_ARGS,
        },);
        /**
         Every command the watch sent Pi.
         */
        const sent: Record<string, unknown>[] = [];
        /**
         How long the run may last, which is the bound its silent stretch is meant to reach when no turn answers the message.
         */
        const wallMilliseconds = 900;
        const endReason = await watchUntilEnded({
          // The abort settles the operation, and the recovery message is accepted: what never comes is the turn it asked for.
          connection: endingConnection({
            sent,
            abort: 'done',
            message: 'done',
            before: {
              abort: function settle() {
                observeAbortedCall(run, 'call-silent',);
              },
            },
          },),
          record: run.record,
          options: optionsOf({
            runFolder: run.folder,
            silentSeconds: PAST_BOTH_TIMES_SECONDS,
            silentEndSeconds: PAST_BOTH_TIMES_SECONDS,
            idleSeconds: 0.05,
            milliseconds: wallMilliseconds,
          },),
          inbox: join(run.folder, 'inbox',),
          deadline: Date.now() + wallMilliseconds,
        },);
        expect(sent.map((command,) => command.type,),).toEqual(['abort', 'prompt',],);
        // The one settle is the abort's: no turn answered the message, so the run must not end as one that finished its turn.
        expect(run.record.settledCount(),).toBe(1,);
        expect(endReason,).toBe('wall-clock',);
      },
    },),
    it({
      name: 'names every command the abort ended, including one that started while the abort was on the wire',
      fn: async () => {
        await using run = await startRunInASilentCommand();
        /**
         Every command the watch sent Pi.
         */
        const sent: Record<string, unknown>[] = [];
        const watch = watchOf({
          run,
          connection: endingConnection({
            sent,
            abort: 'done',
            message: 'done',
            before: {
              abort: function endTheOperation() {
                // Work already queued starts while the abort is on the wire, and the same abort then ends both calls.
                observe(run, {
                  type: 'tool_execution_start',
                  toolCallId: 'call-late',
                  toolName: 'bash',
                  args: { command: 'command-late', },
                },);
                observe(run, {
                  type: 'tool_execution_end',
                  toolCallId: 'call-silent',
                  toolName: 'bash',
                  isError: true,
                  result: { content: [], },
                },);
                observe(run, {
                  type: 'tool_execution_end',
                  toolCallId: 'call-late',
                  toolName: 'bash',
                  isError: true,
                  result: { content: [], },
                },);
                observe(run, { type: 'agent_settled', aborted: true, },);
              },
            },
          },),
          silentSeconds: SILENT_NOTICE_SECONDS,
          silentEndSeconds: SILENT_ENDING_SECONDS,
        },);
        await aSilenceInTheRun({
          watch,
          waited: {},
        },);
        /**
         What the agent was told.
         */
        const told = String(sent[1]?.message,);
        // The call the driver took for the operation, and the one that started after it looked, are both work this abort ended.
        expect(told,).toContain('command-late',);
        expect(told,).toContain('rg -l --fixed-strings something',);
      },
    },),
    it({
      name: 'ends the operation at its stated ending time whatever order the two stated times come in',
      fn: async () => {
        /**
         The two stated times the driver ships, which every order below is generated from.
         */
        const stated = [
          SILENT_ENDING_SECONDS,
          SILENT_NOTICE_SECONDS,
        ] as const;
        /**
         Every order of them: the ending before the notice, the two equal, and the ending after the notice.
         */
        const orders = [
          {
            ending: Math.min(...stated,),
            notice: Math.max(...stated,),
          },
          {
            ending: stated[0],
            notice: stated[0],
          },
          {
            ending: Math.max(...stated,),
            notice: Math.min(...stated,),
          },
        ];
        for (const order of orders) {
          /* oxlint-disable eslint/no-await-in-loop -- one order of the two times at a time, each its own run. */
          await using run = await startRunAtAClock();
          observe(run, { type: 'agent_start', },);
          observeCommandStart(run,);
          /**
           Every command the watch sent Pi.
           */
          const sent: Record<string, unknown>[] = [];
          const watch = watchOf({
            run,
            connection: endingConnection({
              sent,
              abort: 'done',
              message: 'done',
              before: {
                abort: function settle() {
                  observeAbortedCall(run, '',);
                },
              },
            },),
            silentSeconds: order.notice,
            silentEndSeconds: order.ending,
          },);
          // A notice that is due before the ending is written, and ends nothing.
          if (order.notice < order.ending) {
            run.clock.at = order.notice * MILLISECONDS_PER_SECOND;
            await aSilenceInTheRun({
              watch,
              waited: {},
            },);
            expect(sent,).toHaveLength(0,);
            expect(await linesOfKindWhenWritten(join(run.folder, 'feed.jsonl',), 'silence',),).toHaveLength(1,);
          }
          // At the ending the operation is due, whatever the notice time is.
          run.clock.at = order.ending * MILLISECONDS_PER_SECOND;
          await aSilenceInTheRun({
            watch,
            waited: {},
          },);
          // oxlint-enable eslint/no-await-in-loop
          expect(sent.map((command,) => command.type,),).toEqual(['abort', 'prompt',],);
        }
      },
    },),
  ],
},);
