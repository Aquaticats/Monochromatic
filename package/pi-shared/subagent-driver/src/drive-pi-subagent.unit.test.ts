/**
 The driver's bin against a stand-in `pi`, one test for each way a run can go. Each test uses its own disposable folder. A test
 waits for what the feed or the stand-in's log shows instead of for a length of time, so a loaded host cannot change its outcome,
 and a driver a test leaves running is ended before its folder goes.

 @module
 */

import {
  type ChildProcess,
  execFile,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rename,
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
  continuationPromptOf,
  FEED_PREVIEW_CHARACTERS,
  NESTED_RUN_VARIABLE,
  POLL_MILLISECONDS,
  type RunSummary,
  VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME,
} from '../dist/final/node/index.mjs';
import {
  LONG_REPLY,
  MODEL_ERROR_TEXT,
  PERMANENT_ERROR_TEXT,
  RATE_LIMIT_ERROR_TEXT,
  RECOVERED_REPLY,
  TRANSIENT_ERROR_TEXT,
} from './stand-in-pi-records-fixture.ts';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

const DRIVER = new URL('drive-pi-subagent.ts', import.meta.url,).pathname;
const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/**
 Seconds a run may last when something other than the wall clock ends it: the stop file a test writes, or Pi settling. The wall
 clock is only a backstop here, so it sits far above the events a test waits for. It used to be 60, which a busy host could
 spend on start-up alone: at `CPUQuota=25%` on 2026-11-09 one run took 60 s to show its first settled event, and ended at the
 wall clock while the test was still waiting.
 */
const RUN_WALL_SECONDS = 300;
/**
 Seconds a run may last when its subject is the wall clock ending it. Such a test asserts nothing about a state the run must
 reach first, so start-up delayed past this limit changes how the run got there, never what it ends with.
 */
const SHORT_WALL_SECONDS = 1;
/**
 Seconds a run may last when its subject is a compaction that outlives it: above the start-up and the three commands before the
 prompt, so the wall clock reaches the run while Pi is compacting, and short so the test is quick.
 */
const COMPACTION_WALL_SECONDS = 3;
/** Seconds of silence after which a settled run ends. */
const IDLE_SECONDS = 0.2;
/** Seconds of idle wait in tests that end the run with the stop file. */
const LONG_IDLE_SECONDS = 60;
/** Turns allowed, below the stand-in's many-turns scenario and above every other. */
const TURN_CAP = 3;
/**
 Turns a run may start when its subject is how many retries follow model errors: the default retry cap of three retries makes
 four attempts, and each attempt is a turn.
 */
const TURN_CAP_ABOVE_RETRIES = 8;
/**
 Seconds before the first retry after a model error, small enough that a test's three-growing delays take under half a second
 and large enough that the feed records the growth rather than rounding it away.
 */
const RETRY_BASE_SECONDS = 0.05;
/** Milliseconds between looks at a file a test waits on. */
const LOOK_MILLISECONDS = 20;
/** Milliseconds in a second. */
const MILLISECONDS_PER_SECOND = 1_000;
/**
 Milliseconds of the first retry delay, which `--retry-base-seconds` names in seconds.
 */
const RETRY_BASE_MILLISECONDS = RETRY_BASE_SECONDS * MILLISECONDS_PER_SECOND;
/**
 How many looks at the inbox one retry wait spans, so a message dropped during the wait is there before the attempt goes.
 */
const LOOKS_A_RETRY_WAIT_SPANS = 4;
/**
 Seconds before the first retry in a test whose run must never reach it, far past the wall clock the run itself is given.
 */
const WAIT_PAST_THE_RUN_SECONDS = 600;
/**
 How many looks the driver would need to schedule the attempt it owes and send it, which a test waits out so an attempt that
 should not go has had its chance.
 */
const LOOKS_THAT_WOULD_SEND_A_RETRY = 3;
/** Status of a run that ended on its own terms. */
const CLEAN_STATUS = 0;
/** Status of a run that did not. */
const FAILED_STATUS = 1;
/** Seconds a test allows a driver beyond the run's own wall limit, so a stalled driver fails rather than hangs it. */
const TEST_TEARDOWN_ALLOWANCE_SECONDS = 120;
/** Characters in a file name that leave no room for a prefix, which most file systems cap at 255 bytes. */
const LONGEST_FILE_NAME_CHARACTERS = 250;
/**
 Milliseconds a test waits for a line before giving up on it. A line that cannot come any more is noticed when the driver ends,
 so this is only the bound that keeps a test from waiting for ever on a driver that stays up and says nothing.
 */
const EVENT_WAIT_MILLISECONDS = (RUN_WALL_SECONDS + TEST_TEARDOWN_ALLOWANCE_SECONDS) * MILLISECONDS_PER_SECOND;

/**
 Milliseconds a driver is given for one run before the test kills it: the run's own wall limit, and everything the driver does
 after it.

 @param wallSeconds - the run's own wall limit

 @returns the milliseconds the driver may take

 @example
 ```ts
 const bound = outerDeadlineMilliseconds(SHORT_WALL_SECONDS,);
 ```
 */
function outerDeadlineMilliseconds(wallSeconds: number,): number {
  return (wallSeconds + TEST_TEARDOWN_ALLOWANCE_SECONDS) * MILLISECONDS_PER_SECOND;
}

/**
 A variable name the credential rule removes: the one an agent's `env` dump leaked into a model provider's context.
 */
const CREDENTIAL_SHAPED_VARIABLE = 'MISE_GITHUB_TOKEN';

/**
 A value a test hands the driver, which no command may read. The run reads names, so a leak shows as a name nothing asked for.
 */
const CREDENTIAL_VALUE = 'value-that-must-not-reach-a-command';

/**
 A variable name the rule keeps, so a run cannot pass by handing Pi an empty environment.
 */
const ORDINARY_VARIABLE = 'MONOCHROMATIC_ORDINARY_VARIABLE';

/**
 Every name the allow list keeps, taken from the list so a new entry is covered without editing this file.
 */
const KEPT_NAMES = VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME.map(function toName(kept,) {
  return kept.name;
},);
/** What a test sees when the driver wrote no summary; no field of it can pass for a real run's. */
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
};

type Finished = {
  readonly status: number;
  readonly standardError: string;
  readonly summary: RunSummary;
  readonly feed: readonly Record<string, unknown>[];
  readonly commands: readonly string[];
  readonly sentCommands: readonly Record<string, unknown>[];
  readonly markers: readonly string[];
  readonly nestedVariable: string;

  /**
   The names of the variables Pi started with, never their values.
   */
  readonly environmentNames: readonly string[];
  readonly piArguments: readonly string[];
  readonly inbox: readonly string[];
  readonly inboxContents: Readonly<Record<string, string>>;
  readonly logWasWritten: boolean;
  readonly runFolderWasMade: boolean;
};

type RunPaths = {
  readonly runFolder: string;
  readonly promptFile: string;
  readonly log: string;
};

type RunInProgress = RunPaths & {
  /**
   Whether the driver has ended, so a line that never came cannot come any more.
   */
  readonly driverHasStopped: () => boolean;
};

type RunRequest = {
  readonly scenario: string;
  readonly extraOptions?: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
  readonly withoutPi?: boolean;

  /**
   Text of the prompt file, which no test leaves empty.
   */
  readonly promptText?: string;
  readonly withoutPrompt?: boolean;
  readonly wallSeconds?: number;
  readonly idleSeconds?: number;

  /**
   Turns this run may start. A retry after a model error is a turn of its own, so a test that counts retries sets this above the
   default.
   */
  readonly turnCap?: number;

  /**
   Runs after the folders are made and before the driver starts.
   */
  readonly prepare?: (paths: RunPaths,) => Promise<void>;
  readonly whileRunning?: (paths: RunInProgress,) => Promise<void>;
};

type DisposableFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

function isMissingFile(error: unknown,): boolean {
  return (error as NodeJS.ErrnoException).code === 'ENOENT';
}

async function exists(path: string,): Promise<boolean> {
  try {
    await lstat(path,);
    return true;
  }
  catch (error) {
    if (isMissingFile(error,)) {
      return false;
    }
    throw error;
  }
}

async function makeFolder(): Promise<DisposableFolder> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-subagent-driver-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

async function readLines(file: string,): Promise<readonly string[]> {
  try {
    return (await readFile(file, 'utf8',)).split('\n',).filter((line,) => line !== '');
  }
  catch (error) {
    // A test that makes the feed a folder it cannot write sees no lines, which is what it is after.
    if (isMissingFile(error,) || ((error as NodeJS.ErrnoException).code === 'EISDIR')) {
      return [];
    }
    throw error;
  }
}

async function readSummary(file: string,): Promise<RunSummary> {
  try {
    return JSON.parse(await readFile(file, 'utf8',),) as RunSummary;
  }
  catch (error) {
    if (isMissingFile(error,)) {
      return ABSENT_SUMMARY;
    }
    throw error;
  }
}

async function listInbox(folder: string,): Promise<readonly string[]> {
  try {
    return await readdir(folder,);
  }
  catch (error) {
    if (isMissingFile(error,)) {
      return [];
    }
    throw error;
  }
}

async function contentsOfInbox(folder: string,): Promise<Readonly<Record<string, string>>> {
  /**
   What each entry holds, by name.
   */
  const contents: Record<string, string> = {};
  for (const name of await listInbox(folder,)) {
    /**
     The entry itself, which a test may have made a pipe or a folder.
     */
    const path = join(folder, name,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- each entry is inspected in turn, so a name never stands for another's contents; see doc/lint/sequential-awaits.md
    if ((await lstat(path,)).isFile()) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- the contents of one entry, read once its kind is known
      contents[name] = await readFile(path, 'utf8',);
    }
  }
  return contents;
}

/**
 What a file ended with, cut to the lines a failure message can carry, so a test that waited for a line says what it got instead.

 @param lines - the file's lines, as last read

 @returns the last lines, joined
 */
function endingOf(lines: readonly string[],): string {
  /**
   How many of the last lines a failure message carries.
   */
  const shown = 10;
  return lines.length === 0 ? 'nothing at all' : lines.slice(-shown,).join('\n',);
}

/**
 Waits for a file to hold a line, and gives up when the line can no longer come: the driver has ended, or the wait's own bound
 passed. The driver's own limits end its run well inside that bound, so a wait that reaches it means the driver is up and silent,
 and the failure says what the file ended with.

 @param file - file the run writes as it goes

 @param text - text to wait for

 @param driverHasStopped - whether the driver has ended
 */
async function waitForLine({
  file,
  text,
  driverHasStopped,
}: {
  readonly file: string;
  readonly text: string;
  readonly driverHasStopped: () => boolean;
},): Promise<void> {
  /**
   When to give up.
   */
  const deadline = Date.now() + EVENT_WAIT_MILLISECONDS;
  for (;;) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the file is read again after each wait, which is the whole point of the loop; see doc/lint/sequential-awaits.md
    const lines = await readLines(file,);
    if (lines.some((line,) => line.includes(text,))) {
      return;
    }
    if (driverHasStopped()) {
      throw new Error(`the driver ended before ${file} showed ${text}; the file ended with:\n${endingOf(lines,)}`,);
    }
    if (Date.now() >= deadline) {
      throw new Error(
        `${file} showed no ${text} after ${String(EVENT_WAIT_MILLISECONDS,)} ms, and the driver is still up; the file ended with:\n${endingOf(lines,)}`,
      );
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the run writes as it goes, so the test looks again after waiting; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
}

async function waitForFeed(
  { runFolder, text, driverHasStopped, }: {
    readonly runFolder: string;
    readonly text: string;
    readonly driverHasStopped: () => boolean;
  },
): Promise<void> {
  await waitForLine({
    file: join(runFolder, 'feed.jsonl',),
    text,
    driverHasStopped,
  },);
}

async function dropInInbox({
  runFolder,
  text,
  name = 'message.txt',
}: {
  readonly runFolder: string;
  readonly text: string;
  readonly name?: string;
},): Promise<void> {
  const published = join(runFolder, 'inbox', name,);
  await writeFile(join(runFolder, 'inbox', '.writing',), text,);
  await rename(join(runFolder, 'inbox', '.writing',), published,);
}

/**
 Makes a named pipe, which waits for a writer.

 @param path - where the pipe is made
 */
async function makePipe(path: string,): Promise<void> {
  await execFileAsync('mkfifo', [path,],);
}

async function installStandIn(folder: string,): Promise<void> {
  const launcher = join(folder, 'pi',);
  await writeFile(launcher, `#!/usr/bin/env bash\nexec '${process.execPath}' '${STAND_IN}' "$@"\n`,);
  await chmod(launcher, 0o700,);
}

async function exitOf(running: ReturnType<typeof execFileAsync>,): Promise<readonly [number, string,]> {
  try {
    const result = await running;
    return [CLEAN_STATUS, String(result.stderr,),];
  }
  catch (error) {
    /**
     What the operating system said about the driver's end, read field by field because a killed process has no exit code.
     */
    const failure = error as {
      readonly code?: unknown;
      readonly signal?: unknown;
      readonly stderr?: unknown;
    };
    // A driver the test's own outer deadline killed has no exit code, so the signal that ended it is named in the text a
    // failing test prints, instead of the test reporting a bare null status.
    const endedBySignal = (typeof failure.signal) === 'string';
    const exitCodeIsNumber = (typeof failure.code) === 'number';
    const standardErrorIsText = (typeof failure.stderr) === 'string';
    const signalNote = endedBySignal ? `\n[the driver was ended by ${failure.signal}]` : '';
    return [
      exitCodeIsNumber ? failure.code : FAILED_STATUS,
      `${standardErrorIsText ? failure.stderr : ''}${signalNote}`,
    ];
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
  await Promise.race([
    once(child, 'exit',),
    wait(TEST_TEARDOWN_ALLOWANCE_SECONDS * MILLISECONDS_PER_SECOND, undefined, { ref: false, },),
  ],);
}

/**
 Owns a driver process a test started, so every way out of a test ends it before its folder goes.

 @param running - the driver's own promise, which holds its process

 @returns what the test uses, and a disposer that ends the process
 */
async function owningDriver(
  running: ReturnType<typeof execFileAsync>,
): Promise<{
  readonly running: ReturnType<typeof execFileAsync>;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  return {
    running,
    async [Symbol.asyncDispose]() {
      await stopBounded(running.child,);
    },
  };
}

async function runDriver(
  {
    scenario,
    extraOptions = [],
    environment = {},
    withoutPi = false,
    promptText = 'hello',
    withoutPrompt = false,
    wallSeconds = RUN_WALL_SECONDS,
    idleSeconds = IDLE_SECONDS,
    turnCap = TURN_CAP,
    prepare,
    whileRunning,
  }: RunRequest,
): Promise<Finished> {
  await using folder = await makeFolder();
  const runFolder = join(folder.path, 'run',);
  const promptFile = join(folder.path, 'prompt.md',);
  const log = join(folder.path, 'commands.log',);
  /**
   A folder holding no executable, so a test that takes Pi away cannot reach a real one.
   */
  const withoutExecutables = join(folder.path, 'no-executables',);
  await mkdir(withoutExecutables,);
  if (!withoutPrompt) {
    await writeFile(promptFile, promptText,);
  }
  if (!withoutPi) {
    await installStandIn(folder.path,);
  }
  await prepare?.({
    runFolder,
    promptFile,
    log,
  },);
  const pathWithStandIn = [folder.path, dirname(process.execPath,), '/usr/bin',].join(delimiter,);
  const running = execFileAsync(process.execPath, [
    DRIVER,
    `--run-folder=${runFolder}`,
    `--working-folder=${folder.path}`,
    `--prompt-file=${promptFile}`,
    '--provider=stand-in',
    '--model=stand-in-model',
    `--wall-seconds=${String(wallSeconds,)}`,
    `--idle-seconds=${String(idleSeconds,)}`,
    `--turn-cap=${String(turnCap,)}`,
    ...extraOptions,
  ], {
    env: {
      PATH: withoutPi ? withoutExecutables : pathWithStandIn,
      HOME: process.env.HOME,
      STAND_IN_PI_SCENARIO: scenario,
      STAND_IN_PI_LOG: log,
      ...environment,
    },
    encoding: 'utf8',
    timeout: outerDeadlineMilliseconds(wallSeconds,),
    killSignal: 'SIGKILL',
  },);
  await using driver = await owningDriver(running,);
  const [[status, standardError,],] = await Promise.all([
    exitOf(driver.running,),
    whileRunning?.({
      runFolder,
      promptFile,
      log,
      driverHasStopped: function driverHasStopped() {
        return (driver.running.child.exitCode !== null) || (driver.running.child.signalCode !== null);
      },
    },) ?? Promise.resolve(),
  ],);
  const logged = await readLines(log,);
  const sentCommands = logged.filter((line,) => line.startsWith('command ',))
    .map((line,) => JSON.parse(line.slice('command '.length,),) as Record<string, unknown>);
  return {
    status,
    standardError,
    summary: await readSummary(join(runFolder, 'summary.json',),),
    feed: (await readLines(join(runFolder, 'feed.jsonl',),)).map((line,) => JSON.parse(line,) as Record<string, unknown>),
    commands: sentCommands.map((command,) => String(command.type,),),
    sentCommands,
    markers: logged.filter((line,) => line.startsWith('marker ',)).map((line,) => line.slice('marker '.length,)),
    nestedVariable: logged.find((line,) => line.startsWith('nested ',))?.slice('nested '.length,) ?? '',
    environmentNames: JSON.parse(
      logged.find((line,) => line.startsWith('environment-names ',))?.slice('environment-names '.length,) ?? '[]',
    ) as readonly string[],
    piArguments: logged.filter((line,) => line.startsWith('arguments ',)).flatMap((line,) =>
      JSON.parse(line.slice('arguments '.length,),) as string[]
    ),
    inbox: await listInbox(join(runFolder, 'inbox',),),
    inboxContents: await contentsOfInbox(join(runFolder, 'inbox',),),
    logWasWritten: logged.length > 0,
    runFolderWasMade: await exists(runFolder,),
  };
}

await describe({
  name: 'drive-pi-subagent',
  children: [
    it({
      name: 'a run that settles and goes quiet ends idle, exits 0 and keeps the reply whole',
      fn: async () => {
        const run = await runDriver({ scenario: 'normal', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.endReason,).toBe('idle',);
        expect(run.summary.lastAssistantText,).toBe('done',);
        expect(run.summary.settled,).toBe(1,);
        expect(run.summary.turns,).toBe(1,);
        expect(run.summary.piExit?.code,).toBe(0,);
        expect(run.commands,).toContain('get_session_stats',);
      },
    },),
    it({
      name: 'keeps the whole reply in the summary and only a preview on the feed',
      fn: async () => {
        const run = await runDriver({ scenario: 'long-reply', },);
        expect(run.summary.lastAssistantText,).toBe(LONG_REPLY,);
        const assistant = run.feed.filter((entry,) => entry.kind === 'assistant',).at(-1,);
        const preview = String(assistant?.text,);
        expect(preview.length,).toBeLessThan(LONG_REPLY.length,);
        expect(
          preview.startsWith(LONG_REPLY.slice(0, FEED_PREVIEW_CHARACTERS.assistantText,),),
        ).toBe(true,);
      },
    },),
    it({
      name: 'passes the provider, the model and each extra word to Pi as whole arguments',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          extraOptions: ['--session=/a folder/with spaces.jsonl', '--pi-argument=--extension', '--pi-argument=/a folder/x.ts',],
        },);
        expect(run.piArguments,).toEqual([
          '--mode',
          'rpc',
          '--provider',
          'stand-in',
          '--model',
          'stand-in-model',
          '--session',
          '/a folder/with spaces.jsonl',
          '--extension',
          '/a folder/x.ts',
        ],);
      },
    },),
    it({
      name: 'sends the prompt file and each inbox message whole to Pi',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          promptText: 'the prompt from the file',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'a steering message', },);
            await waitForFeed({ ...running, text: '"preview":"a steering message"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.sentCommands.filter((command,) => command.type === 'prompt',).map((command,) => command.message),)
          .toEqual(['the prompt from the file', 'a steering message',],);
      },
    },),
    it({
      name: 'reads the state, the commands and the statistics Pi starts with',
      fn: async () => {
        const run = await runDriver({ scenario: 'normal', },);
        expect(run.feed.find((entry,) => entry.kind === 'state',),).toMatchObject({
          model: 'stand-in-model',
          thinkingLevel: 'xhigh',
        },);
        expect(run.feed.find((entry,) => entry.kind === 'commands',)?.names,).toEqual(['bash', 'edit',],);
        expect(run.summary.sessionStats,).toEqual({
          sessionFile: 'stand-in-session.jsonl',
          cost: 0.5,
        },);
      },
    },),
    it({
      name: 'leaves the statistics empty when Pi answers with neither data nor an error',
      fn: async () => {
        const run = await runDriver({ scenario: 'statless', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.sessionStats,).toBeNull();
      },
    },),
    it({
      name: 'a rejected first prompt fails at once instead of waiting for the wall clock',
      fn: async () => {
        const run = await runDriver({ scenario: 'reject', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('prompt-rejected',);
        expect(run.standardError,).toContain('prompt-rejected',);
      },
    },),
    it({
      name: 'a Pi whose compaction outlives the run ends at the wall clock, and the summary and the message name the compaction',
      fn: async () => {
        const run = await runDriver({ scenario: 'compaction-never-ends', wallSeconds: COMPACTION_WALL_SECONDS, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('wall-clock',);
        expect(run.feed.some((entry,) => entry.kind === 'compaction_start'),).toBe(true,);
        expect(String(run.summary.compactionProblem,),).toContain('compacting',);
        expect(run.standardError,).toContain('compacting',);
      },
    },),
    it({
      name: 'a Pi that crashes at start ends the run as pi-exited and records its status',
      fn: async () => {
        const run = await runDriver({ scenario: 'crash', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('pi-exited',);
        expect(run.summary.piExit?.code,).toBe(3,);
      },
    },),
    it({
      name: 'a Pi that cannot be started ends the run as pi-exited and says why',
      fn: async () => {
        const run = await runDriver({ scenario: 'normal', withoutPi: true, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('pi-exited',);
        expect(String(run.summary.piExit?.processError,),).toContain('ENOENT',);
      },
    },),
    it({
      name: 'a Pi that exits while the watch runs ends it as pi-exited',
      fn: async () => {
        const run = await runDriver({
          scenario: 'crash-on-steer',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForLine({ ...running, file: running.log, text: '"message":"hello"', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'leave now', },);
          },
        },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('pi-exited',);
        expect(run.feed.find((entry,) => entry.kind === 'inbox',)?.error,).toBe('pi exited',);
      },
    },),
    it({
      name: 'a run whose start-up stalls ends at the wall clock instead of on a rejected prompt',
      fn: async () => {
        const run = await runDriver({ scenario: 'silent', wallSeconds: SHORT_WALL_SECONDS, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('wall-clock',);
        // The statistics ask after the wall clock ends by the teardown's own bound, not by Pi answering it.
        expect(String(run.summary.sessionStats,),).toContain('no answer',);
      },
    },),
    it({
      name: 'a run that never settles ends at the wall clock, and Pi is stopped with it',
      fn: async () => {
        const run = await runDriver({ scenario: 'never-settles', wallSeconds: SHORT_WALL_SECONDS, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('wall-clock',);
        // The teardown asks a Pi the record still holds at work to abort, which is shown without a clock at all in "a message
        // dropped between agent_end and agent_settled goes as a steer". Here the run may reach its wall clock before Pi has
        // started a turn, so no abort is owed; what a run over its wall clock always ends with is Pi stopped and recorded.
        expect(run.summary.piExit,).toBeDefined();
      },
    },),
    it({
      name: 'a Pi that fails while it is asked for its statistics leaves the run pi-exited, not idle',
      fn: async () => {
        const run = await runDriver({ scenario: 'crash-on-stats', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('pi-exited',);
        expect(run.summary.piExit?.code,).toBe(3,);
      },
    },),
    it({
      name: 'a run over its turn cap ends as turn-cap',
      fn: async () => {
        const run = await runDriver({ scenario: 'many-turns', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('turn-cap',);
      },
    },),
    it({
      name: 'a stop file ends the run cleanly',
      fn: async () => {
        const run = await runDriver({
          scenario: 'never-settles',
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"prompt_sent"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.endReason,).toBe('stop-file',);
      },
    },),
    it({
      name: 'a failure in the driver itself still stops Pi and writes a summary that names it',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          withoutPrompt: true,
        },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('driver-failed',);
        expect(String(run.summary.driverError,),).toContain('ENOENT',);
        expect(run.summary.piExit?.code,).toBe(0,);
      },
    },),
    it({
      name: 'a prompt file that is a named pipe fails the run instead of waiting for a writer',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          withoutPrompt: true,
          prepare: async ({ promptFile, },) => {
            await makePipe(promptFile,);
          },
        },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('driver-failed',);
        expect(String(run.summary.driverError,),).toContain('not an ordinary file',);
      },
    },),
    it({
      name: 'a run whose feed cannot be written does not end clean',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          prepare: async ({ runFolder, },) => {
            await mkdir(join(runFolder, 'feed.jsonl',), { recursive: true, },);
          },
        },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('driver-failed',);
        expect(String(run.summary.feedProblem,),).toContain('EISDIR',);
      },
    },),
    it({
      name: 'a final reply that ended in a model error does not end clean',
      fn: async () => {
        const run = await runDriver({ scenario: 'model-error', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('model-error',);
        expect(String(run.summary.modelError,),).toContain(MODEL_ERROR_TEXT,);
      },
    },),
    it({
      name: 'a model error that an automatic retry recovers still ends clean',
      fn: async () => {
        const run = await runDriver({ scenario: 'recovered-error', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.endReason,).toBe('idle',);
        expect(run.summary.lastAssistantText,).toBe(RECOVERED_REPLY,);
      },
    },),
    it({
      name: 'a transient model error is answered with another attempt, and the run goes on',
      fn: async () => {
        const run = await runDriver({
          scenario: 'transient-error-then-success',
          extraOptions: [`--retry-base-seconds=${String(RETRY_BASE_SECONDS,)}`,],
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.endReason,).toBe('idle',);
        expect(run.summary.lastAssistantText,).toBe(RECOVERED_REPLY,);
        expect(run.summary.modelError,).toBeUndefined();
        expect(run.summary.retries,).toBe(1,);
        /**
         The retries this run recorded, which are the attempts after its one model error.
         */
        const retries = run.feed.filter((entry,) => entry.kind === 'retry',);
        expect(retries.length,).toBe(1,);
        expect(retries[0]?.attempt,).toBe(1,);
        expect(retries[0]?.of,).toBe(3,);
        expect(retries[0]?.errorKind,).toBe('transient',);
        expect(String(run.feed.find((entry,) => entry.kind === 'retry')?.error,),).toContain(TRANSIENT_ERROR_TEXT,);
        expect(run.feed.find((entry,) => entry.kind === 'retry_prompt')?.success,).toBe(true,);
        expect(run.sentCommands.filter((command,) => command.type === 'prompt',).length,).toBe(2,);
      },
    },),
    it({
      name: 'a model error that keeps coming back ends the run once the retries run out, naming the attempts',
      fn: async () => {
        const run = await runDriver({
          scenario: 'always-transient-error',
          extraOptions: [`--retry-base-seconds=${String(RETRY_BASE_SECONDS,)}`,],
          turnCap: TURN_CAP_ABOVE_RETRIES,
        },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('model-error',);
        expect(String(run.summary.modelError,),).toContain(TRANSIENT_ERROR_TEXT,);
        expect(run.summary.retries,).toBe(3,);
        expect(run.summary.modelErrorAttempts,).toBe(4,);
        expect(run.feed.filter((entry,) => entry.kind === 'retry',).map((entry,) => entry.delayMilliseconds),)
          .toEqual([RETRY_BASE_MILLISECONDS, 2 * RETRY_BASE_MILLISECONDS, 4 * RETRY_BASE_MILLISECONDS,],);
        expect(run.standardError,).toContain('after 4 attempts',);
      },
    },),
    it({
      name: 'a model error no retry can fix ends the run without an attempt at one',
      fn: async () => {
        const run = await runDriver({ scenario: 'permanent-error', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('model-error',);
        expect(String(run.summary.modelError,),).toContain(PERMANENT_ERROR_TEXT,);
        expect(run.summary.retries,).toBe(0,);
        expect(run.summary.modelErrorAttempts,).toBe(1,);
        expect(run.feed.some((entry,) => entry.kind === 'retry'),).toBe(false,);
        expect(run.sentCommands.filter((command,) => command.type === 'prompt',).length,).toBe(1,);
      },
    },),
    it({
      name: 'an inbox message arriving during a retry wait goes after the retry prompt, not into a run it would start',
      fn: async () => {
        const run = await runDriver({
          scenario: 'transient-error-then-success',
          idleSeconds: LONG_IDLE_SECONDS,
          extraOptions: [`--retry-base-seconds=${String(RETRY_BASE_SECONDS * LOOKS_A_RETRY_WAIT_SPANS,)}`,],
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'a steering message', },);
            await waitForFeed({ ...running, text: '"kind":"retry_prompt"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        // The message goes after the retry prompt, whether Pi is still finishing that attempt's run when the driver hands it
        // over or has settled on it by then: a steer for a run still going, a prompt for a Pi at rest.
        expect(run.sentCommands.filter((command,) => (command.type === 'prompt') || (command.type === 'steer'))
          .map((command,) => command.message),)
          .toEqual([
            'hello',
            continuationPromptOf({ problem: TRANSIENT_ERROR_TEXT, },),
            'a steering message',
          ],);
      },
    },),
    it({
      name: 'a failure that asks for a wait longer than a retry takes ends the run rather than retrying inside its window',
      fn: async () => {
        const run = await runDriver({
          scenario: 'long-wait-error',
          extraOptions: [`--retry-base-seconds=${String(RETRY_BASE_SECONDS,)}`,],
        },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('model-error',);
        expect(run.summary.retries,).toBe(0,);
        expect(run.feed.some((entry,) => entry.kind === 'retry'),).toBe(false,);
        expect(String(run.feed.find((entry,) => entry.kind === 'problem')?.problem,),).toContain('asked for a wait',);
      },
    },),
    it({
      name: 'a rate limit is retried like a transient failure and counted apart',
      fn: async () => {
        const run = await runDriver({
          scenario: 'rate-limit-then-success',
          extraOptions: [`--retry-base-seconds=${String(RETRY_BASE_SECONDS,)}`,],
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.endReason,).toBe('idle',);
        expect(run.summary.retries,).toBe(1,);
        expect(run.summary.rateLimitErrors,).toBe(1,);
        expect(run.feed.find((entry,) => entry.kind === 'retry')?.errorKind,).toBe('throttled',);
      },
    },),
    it({
      name: 'a retry Pi only handles ends the run on the failure rather than as a run that came back',
      fn: async () => {
        const run = await runDriver({
          scenario: 'handled-retry',
          extraOptions: [`--retry-base-seconds=${String(RETRY_BASE_SECONDS,)}`,],
        },);
        // No run started for the prompt and no reply came, so nothing recovered the failure: the run failed, whatever its Pi
        // did with the prompt.
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('model-error',);
        expect(String(run.summary.modelError,),).toContain(TRANSIENT_ERROR_TEXT,);
        expect(run.summary.retries,).toBe(3,);
        expect(run.summary.modelErrorAttempts,).toBe(4,);
        expect(run.feed.filter((entry,) => entry.kind === 'retry_prompt').map((entry,) => entry.disposition),)
          .toEqual(['handled', 'handled', 'handled',],);
      },
    },),
    it({
      name: 'a retry stays held while a run a message from the inbox started is still going',
      fn: async () => {
        const run = await runDriver({
          scenario: 'busy-after-inbox',
          idleSeconds: LONG_IDLE_SECONDS,
          extraOptions: [`--retry-base-seconds=${String(RETRY_BASE_SECONDS,)}`,],
          whileRunning: async (running,) => {
            // The message waits out the retry, which goes first, and the driver then hands it to a resting Pi.
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'a message that starts a run of its own', },);
            await waitForFeed({ ...running, text: '"kind":"inbox"', },);
            // The attempt the driver owes answers a failure the run it started may still recover, so it must not go: a driver
            // that sent it would have sent it by now.
            await wait(POLL_MILLISECONDS * LOOKS_THAT_WOULD_SEND_A_RETRY,);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        // The failure the run settled on was never answered, and a failure stays until a reply succeeds, so a run the stop file
        // ends while that failure stands reports it.
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('model-error',);
        expect(String(run.summary.modelError,),).toContain(TRANSIENT_ERROR_TEXT,);
        expect(run.summary.retries,).toBe(1,);
        /**
         What Pi was sent: the first prompt, the retry that answers the failure, and the message the inbox held. A fourth one
         would have been the retry landing in the run that message started.
         */
        const messages = run.sentCommands
          .filter((command,) => (command.type === 'prompt') || (command.type === 'steer'))
          .map((command,) => command.message);
        expect(messages,).toHaveLength(3,);
        expect(messages[1],).toBe(continuationPromptOf({ problem: TRANSIENT_ERROR_TEXT, },),);
        // The message went to Pi and its run was never interrupted, so nothing of it was kept aside.
        expect(run.inbox,).toEqual([],);
      },
    },),
    it({
      name: 'a wait Pi leaves before leaves the failure standing and counts no retry',
      fn: async () => {
        const run = await runDriver({
          scenario: 'exits-after-error',
          extraOptions: [`--retry-base-seconds=${String(WAIT_PAST_THE_RUN_SECONDS,)}`,],
        },);
        expect(run.summary.endReason,).toBe('pi-exited',);
        // A retry the run never reached is not an attempt it made, and the failure it would have answered stays reported.
        expect(run.summary.retries,).toBe(0,);
        expect(String(run.summary.modelError,),).toContain(TRANSIENT_ERROR_TEXT,);
        expect(run.summary.modelErrorAttempts,).toBe(1,);
        expect(run.feed.some((entry,) => entry.kind === 'retry'),).toBe(false,);
        expect(run.sentCommands.filter((command,) => command.type === 'prompt'),).toHaveLength(1,);
      },
    },),
    it({
      name: 'a run Pi is at work on outlasts the retries the run has spent',
      fn: async () => {
        const run = await runDriver({
          scenario: 'spent-retries-then-inbox',
          idleSeconds: LONG_IDLE_SECONDS,
          turnCap: TURN_CAP_ABOVE_RETRIES,
          extraOptions: [
            '--retry-cap=1',
            `--retry-base-seconds=${String(RETRY_BASE_SECONDS,)}`,
          ],
          whileRunning: async (running,) => {
            // The message waits out the retry, which goes first, and the driver then hands it to a resting Pi.
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'a message that answers the task', },);
            await waitForFeed({ ...running, text: '"kind":"inbox"', },);
          },
        },);
        // The retries were spent, but the run the inbox started replied, and a reply that succeeds recovers the failure.
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.endReason,).toBe('idle',);
        expect(run.summary.lastAssistantText,).toBe(RECOVERED_REPLY,);
        expect(run.summary.retries,).toBe(1,);
        expect(run.summary.modelError,).toBeUndefined();
      },
    },),
    it({
      name: 'counts a retry at the failure it answered, not at the reply its own run produced',
      fn: async () => {
        const run = await runDriver({
          scenario: 'recovers-then-fails',
          turnCap: TURN_CAP_ABOVE_RETRIES,
          extraOptions: [
            '--retry-cap=1',
            `--retry-base-seconds=${String(RETRY_BASE_SECONDS,)}`,
          ],
        },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('model-error',);
        expect(run.summary.retries,).toBe(1,);
        // The reply ended the failure the retry answered, and the failure the run ends on came after it: one attempt at it.
        expect(run.summary.modelErrorAttempts,).toBe(1,);
      },
    },),
    it({
      name: 'a prompt Pi handles without a run ends idle, with no settled event to wait for',
      fn: async () => {
        const run = await runDriver({ scenario: 'handled', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.endReason,).toBe('idle',);
        expect(run.summary.settled,).toBe(1,);
        expect(run.feed.some((entry,) => entry.kind === 'settled'),).toBe(false,);
        expect(run.feed.find((entry,) => entry.kind === 'prompt_sent')?.disposition,).toBe('handled',);
      },
    },),
    it({
      name: 'a message dropped between agent_end and agent_settled goes as a steer',
      fn: async () => {
        const run = await runDriver({
          scenario: 'settle-gap',
          whileRunning: async (running,) => {
            await waitForLine({ ...running, file: running.log, text: 'marker agent_end', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'more', },);
            await waitForFeed({ ...running, text: '"kind":"inbox"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.feed.find((entry,) => entry.kind === 'inbox')?.sentAs,).toBe('steer',);
        expect(run.commands,).toContain('steer',);
        // The record still holds Pi at work (only agent_settled clears it), so the teardown asks Pi to abort. This is the
        // claim the wall-clock test cannot make: it has no way to know Pi started a turn before its clock ran out.
        expect(run.commands,).toContain('abort',);
        expect(run.inbox,).toEqual([],);
      },
    },),
    it({
      name: 'a message dropped while Pi rests goes as a new prompt',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'more', },);
            await waitForFeed({ ...running, text: '"kind":"settled","n":2', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.feed.find((entry,) => entry.kind === 'inbox')?.sentAs,).toBe('prompt',);
        expect(run.summary.settled,).toBe(2,);
      },
    },),
    it({
      name: 'a message Pi refuses is kept under a dot name and not sent again',
      fn: async () => {
        const run = await runDriver({
          scenario: 'reject-inbox',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'refused once', },);
            await waitForFeed({ ...running, text: '"kind":"inbox","file":"message.txt"', },);
            // A second message of its own makes the watch look again: a driver that resent the refused one would send it first.
            await dropInInbox({
              runFolder: running.runFolder,
              text: 'after the refusal',
              name: 'later.txt',
            },);
            await waitForFeed({ ...running, text: '"preview":"after the refusal"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.sentCommands.filter((command,) => command.type === 'prompt',).map((command,) => command.message),)
          .toEqual(['hello', 'refused once', 'after the refusal',],);
        expect(run.inbox.toSorted(),).toEqual(['.failed-later.txt', '.failed-message.txt',],);
      },
    },),
    it({
      name: 'two messages refused under one name are both kept',
      fn: async () => {
        const run = await runDriver({
          scenario: 'reject-inbox',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'first kept', },);
            await waitForFeed({ ...running, text: '"preview":"first kept"', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'second kept', },);
            await waitForFeed({ ...running, text: '"preview":"second kept"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.inbox.toSorted(),).toEqual(['.failed-message.txt', '.failed-message.txt.2',],);
        expect(run.inboxContents['.failed-message.txt'],).toBe('first kept',);
        expect(run.inboxContents['.failed-message.txt.2'],).toBe('second kept',);
      },
    },),
    it({
      name: 'a message published while an earlier one is in flight is delivered, not deleted',
      fn: async () => {
        const run = await runDriver({
          scenario: 'slow-answer',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForLine({ ...running, file: running.log, text: '"message":"hello"', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'first sent', },);
            await waitForLine({ ...running, file: running.log, text: '"message":"first sent"', },);
            // The driver is still waiting for Pi's answer to the first message; the second arrives at the same path.
            await dropInInbox({ runFolder: running.runFolder, text: 'second arrived', },);
            await waitForFeed({ ...running, text: '"preview":"second arrived"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.sentCommands.filter((command,) => command.type === 'steer',).map((command,) => command.message),)
          .toEqual(['first sent', 'second arrived',],);
        expect(run.feed.filter((entry,) => entry.kind === 'inbox',).length,).toBe(2,);
      },
    },),
    it({
      name: 'inbox messages go in name order, one at a time',
      fn: async () => {
        const run = await runDriver({
          scenario: 'slow-answer',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForLine({ ...running, file: running.log, text: '"message":"hello"', },);
            await dropInInbox({
              runFolder: running.runFolder,
              text: 'second by name',
              name: 'b.txt',
            },);
            await dropInInbox({
              runFolder: running.runFolder,
              text: 'first by name',
              name: 'a.txt',
            },);
            await waitForFeed({ ...running, text: '"preview":"second by name"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.feed.filter((entry,) => entry.kind === 'inbox',).map((entry,) => entry.preview),)
          .toEqual(['first by name', 'second by name',],);
        expect(run.markers.filter((marker,) => marker === 'overlap'),).toEqual([],);
      },
    },),
    it({
      name: 'an inbox entry that cannot be read is set aside and the run goes on',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await mkdir(join(running.runFolder, 'inbox', 'folder',),);
            await waitForFeed({ ...running, text: 'could not deliver folder', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.endReason,).toBe('stop-file',);
        expect(run.inbox,).toEqual(['.failed-folder',],);
      },
    },),
    it({
      name: 'a message whose name is too long to keep is held under its claim, and the run goes on',
      fn: async () => {
        /**
         A name that leaves no room for the `.failed-` prefix within the file system's name limit.
         */
        const tooLong = 'n'.repeat(LONGEST_FILE_NAME_CHARACTERS,);
        const run = await runDriver({
          scenario: 'reject-inbox',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await dropInInbox({
              runFolder: running.runFolder,
              text: 'kept but unnamed',
              name: tooLong,
            },);
            await waitForFeed({ ...running, text: 'could not keep', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.inbox.filter((entry,) => entry.startsWith('.claimed-',)).length,).toBe(1,);
      },
    },),
    it({
      name: 'an inbox entry that is a named pipe is set aside instead of waiting for a writer',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (running,) => {
            await waitForFeed({ ...running, text: '"kind":"settled","n":1', },);
            await makePipe(join(running.runFolder, 'inbox', 'pipe.txt',),);
            await waitForFeed({ ...running, text: 'could not deliver pipe.txt', },);
            await dropInInbox({ runFolder: running.runFolder, text: 'after the pipe', },);
            await waitForFeed({ ...running, text: '"preview":"after the pipe"', },);
            await writeFile(join(running.runFolder, 'stop',), '',);
          },
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.inbox.toSorted(),).toEqual(['.failed-pipe.txt',],);
      },
    },),
    it({
      name: 'declines a dialog so it cannot stall the session, and answers no notice',
      fn: async () => {
        const run = await runDriver({ scenario: 'dialog', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.dialogs.length,).toBe(1,);
        expect(run.sentCommands.filter((command,) => command.type === 'extension_ui_response',),)
          .toEqual([{
            type: 'extension_ui_response',
            id: 'dialog-1',
            cancelled: true,
          },],);
      },
    },),
    it({
      name: 'counts tools by name and keeps the one that failed',
      fn: async () => {
        const run = await runDriver({ scenario: 'tools', },);
        expect(run.summary.toolHistogram,).toEqual({ bash: 2, },);
        expect(run.summary.toolErrors.length,).toBe(1,);
        expect(run.summary.toolErrors[0],).toMatchObject({
          tool: 'bash',
          result: 'failed',
        },);
        expect(run.feed.some((entry,) => entry.kind === 'auto_retry_start'),).toBe(true,);
      },
    },),
    it({
      name: 'notes a line that is not JSON and carries on',
      fn: async () => {
        const run = await runDriver({ scenario: 'noise', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.feed.some((entry,) => entry.kind === 'non-json'),).toBe(true,);
        expect(run.summary.lastAssistantText,).toBe('after the noise',);
      },
    },),
    it({
      name: 'reads a record split inside a character and keeps a Unicode line separator inside it',
      fn: async () => {
        const run = await runDriver({ scenario: 'split-record', },);
        expect(run.summary.lastAssistantText,).toBe(`café line${String.fromCodePoint(0x20_28,)}separator`,);
      },
    },),
    it({
      name: 'refuses to run inside a Pi that a driver is running, before starting anything',
      fn: async () => {
        const run = await runDriver({ scenario: 'normal', environment: { [NESTED_RUN_VARIABLE]: '1', }, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.standardError,).toContain('NestedDriverError',);
        expect(run.logWasWritten,).toBe(false,);
        expect(run.runFolderWasMade,).toBe(false,);
      },
    },),
    it({
      name: 'hands Pi the variable that stops a nested driver',
      fn: async () => {
        const run = await runDriver({ scenario: 'normal', },);
        expect(run.nestedVariable,).toBe('1',);
      },
    },),
    it({
      name: 'removes a credential-named variable from what Pi gets and keeps the ordinary ones',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          environment: {
            [CREDENTIAL_SHAPED_VARIABLE]: CREDENTIAL_VALUE,
            [ORDINARY_VARIABLE]: CREDENTIAL_VALUE,
          },
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.environmentNames,).not.toContain(CREDENTIAL_SHAPED_VARIABLE,);
        expect(run.environmentNames,).toContain(ORDINARY_VARIABLE,);
      },
    },),
    it({
      name: 'keeps every variable the allow list names, so a GUI check still reaches its display',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          environment: Object.fromEntries(
            KEPT_NAMES.map(function toVariable(name,) {
              return [name, '/run/user/0/probe-path',];
            },),
          ),
        },);
        for (const name of KEPT_NAMES) {
          expect(run.environmentNames,).toContain(name,);
        }
      },
    },),
    it({
      name: 'notes the names it removed on the feed and no value of them',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          environment: { [CREDENTIAL_SHAPED_VARIABLE]: CREDENTIAL_VALUE, },
        },);
        const noted = run.feed.find((entry,) => entry.kind === 'environment',);
        expect(noted?.removedNames,).toEqual([CREDENTIAL_SHAPED_VARIABLE,],);
        expect(JSON.stringify(run.feed,),).not.toContain(CREDENTIAL_VALUE,);
      },
    },),
    it({
      name: 'names the option when one is missing',
      fn: async () => {
        const [status, standardError,] = await exitOf(execFileAsync(process.execPath, [DRIVER,], { encoding: 'utf8', },),);
        expect(status,).toBe(FAILED_STATUS,);
        expect(standardError,).toContain('--run-folder is required',);
      },
    },),
  ],
},);
