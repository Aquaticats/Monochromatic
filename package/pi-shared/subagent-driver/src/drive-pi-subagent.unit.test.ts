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
  FEED_PREVIEW_CHARACTERS,
  NESTED_RUN_VARIABLE,
  POLL_MILLISECONDS,
  type RunSummary,
} from '../dist/final/node/index.mjs';
import {
  LONG_REPLY,
  MODEL_ERROR_TEXT,
  RECOVERED_REPLY,
} from './stand-in-pi-prompt-plays-fixture.ts';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

const DRIVER = new URL('drive-pi-subagent.ts', import.meta.url,).pathname;
const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/** Seconds a run may last; long enough that only the test about this limit reaches it. */
const WALL_SECONDS = 60;
/** Seconds the tests about the wall clock allow. */
const SHORT_WALL_SECONDS = 1;
/** Seconds a run that stalls on an inbox answer allows, so start-up cannot outlast it. */
const STALLED_INBOX_WALL_SECONDS = 4;
/** Seconds of silence after which a settled run ends. */
const IDLE_SECONDS = 0.2;
/** Seconds of idle wait in tests that end the run with the stop file. */
const LONG_IDLE_SECONDS = 60;
/** Turns allowed, below the stand-in's many-turns scenario and above every other. */
const TURN_CAP = 3;
/** Milliseconds between looks at a file a test waits on. */
const LOOK_MILLISECONDS = 20;
/** Milliseconds in a second. */
const MILLISECONDS_PER_SECOND = 1_000;
/** Status of a run that ended on its own terms. */
const CLEAN_STATUS = 0;
/** Status of a run that did not. */
const FAILED_STATUS = 1;
/** Seconds a test allows a driver beyond the run's own wall limit, so a stalled driver fails rather than hangs it. */
const TEST_TEARDOWN_ALLOWANCE_SECONDS = 120;
/** Characters in a file name that leave no room for a prefix, which most file systems cap at 255 bytes. */
const LONGEST_FILE_NAME_CHARACTERS = 250;
/** Milliseconds a driver runs before a test's own timeout ends it, which is a safety net and not a bound of the driver. */
const OUTER_DEADLINE_MILLISECONDS = (WALL_SECONDS + TEST_TEARDOWN_ALLOWANCE_SECONDS) * MILLISECONDS_PER_SECOND;
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
   Runs after the folders are made and before the driver starts.
   */
  readonly prepare?: (paths: RunPaths,) => Promise<void>;
  readonly whileRunning?: (paths: RunPaths,) => Promise<void>;
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

async function waitForLine({
  file,
  text,
}: {
  readonly file: string;
  readonly text: string;
},): Promise<void> {
  /**
   When to give up.
   */
  const deadline = Date.now() + (WALL_SECONDS * MILLISECONDS_PER_SECOND);
  while (Date.now() <= deadline) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the file is read again after each wait, which is the whole point of the loop; see doc/lint/sequential-awaits.md
    if ((await readLines(file,)).some((line,) => line.includes(text,))) {
      return;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the run writes as it goes, so the test looks again after waiting; see doc/lint/sequential-awaits.md
    await wait(LOOK_MILLISECONDS,);
  }
  throw new Error(`the file never showed ${text}`,);
}

async function waitForFeed({ runFolder, text, }: { readonly runFolder: string; readonly text: string; },): Promise<void> {
  await waitForLine({ file: join(runFolder, 'feed.jsonl',), text, },);
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
    const failure = error as { readonly code: number; readonly stderr: string; };
    return [failure.code, failure.stderr,];
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
    wallSeconds = WALL_SECONDS,
    idleSeconds = IDLE_SECONDS,
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
    `--turn-cap=${String(TURN_CAP,)}`,
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
    timeout: OUTER_DEADLINE_MILLISECONDS,
    killSignal: 'SIGKILL',
  },);
  await using driver = await owningDriver(running,);
  const [[status, standardError,],] = await Promise.all([
    exitOf(driver.running,),
    whileRunning?.({
      runFolder,
      promptFile,
      log,
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
          whileRunning: async ({ runFolder, },) => {
            await waitForFeed({ runFolder, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder, text: 'a steering message', },);
            await waitForFeed({ runFolder, text: '"preview":"a steering message"', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, log, },) => {
            await waitForLine({ file: log, text: '"message":"hello"', },);
            await dropInInbox({ runFolder, text: 'leave now', },);
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
      name: 'a run that never settles ends at the wall clock and is aborted',
      fn: async () => {
        const run = await runDriver({ scenario: 'never-settles', wallSeconds: SHORT_WALL_SECONDS, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('wall-clock',);
        expect(run.commands,).toContain('abort',);
      },
    },),
    it({
      name: 'an inbox answer that stalls ends at the wall clock, not on a delivery that never returns',
      fn: async () => {
        const run = await runDriver({
          scenario: 'silent-inbox',
          wallSeconds: STALLED_INBOX_WALL_SECONDS,
          prepare: async ({ runFolder, },) => {
            // The message is there before the run starts, so no start-up delay can hide it from the watch.
            await mkdir(join(runFolder, 'inbox',), { recursive: true, },);
            await dropInInbox({ runFolder, text: 'never answered', },);
          },
        },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('wall-clock',);
        expect(run.feed.find((entry,) => entry.kind === 'inbox',)?.success,).toBe(false,);
        expect(String(run.feed.find((entry,) => entry.kind === 'inbox',)?.error,),).toContain('no answer',);
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
          whileRunning: async ({ runFolder, },) => {
            await waitForFeed({ runFolder, text: '"kind":"prompt_sent"', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, log, },) => {
            await waitForLine({ file: log, text: 'marker agent_end', },);
            await dropInInbox({ runFolder, text: 'more', },);
            await waitForFeed({ runFolder, text: '"kind":"inbox"', },);
            await writeFile(join(runFolder, 'stop',), '',);
          },
        },);
        expect(run.feed.find((entry,) => entry.kind === 'inbox')?.sentAs,).toBe('steer',);
        expect(run.commands,).toContain('steer',);
        expect(run.inbox,).toEqual([],);
      },
    },),
    it({
      name: 'a message dropped while Pi rests goes as a new prompt',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async ({ runFolder, },) => {
            await waitForFeed({ runFolder, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder, text: 'more', },);
            await waitForFeed({ runFolder, text: '"kind":"settled","n":2', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, },) => {
            await waitForFeed({ runFolder, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder, text: 'refused once', },);
            await waitForFeed({ runFolder, text: '"kind":"inbox","file":"message.txt"', },);
            // A second message of its own makes the watch look again: a driver that resent the refused one would send it first.
            await dropInInbox({
              runFolder,
              text: 'after the refusal',
              name: 'later.txt',
            },);
            await waitForFeed({ runFolder, text: '"preview":"after the refusal"', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, },) => {
            await waitForFeed({ runFolder, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder, text: 'first kept', },);
            await waitForFeed({ runFolder, text: '"preview":"first kept"', },);
            await dropInInbox({ runFolder, text: 'second kept', },);
            await waitForFeed({ runFolder, text: '"preview":"second kept"', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, log, },) => {
            await waitForLine({ file: log, text: '"message":"hello"', },);
            await dropInInbox({ runFolder, text: 'first sent', },);
            await waitForLine({ file: log, text: '"message":"first sent"', },);
            // The driver is still waiting for Pi's answer to the first message; the second arrives at the same path.
            await dropInInbox({ runFolder, text: 'second arrived', },);
            await waitForFeed({ runFolder, text: '"preview":"second arrived"', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, log, },) => {
            await waitForLine({ file: log, text: '"message":"hello"', },);
            await dropInInbox({
              runFolder,
              text: 'second by name',
              name: 'b.txt',
            },);
            await dropInInbox({
              runFolder,
              text: 'first by name',
              name: 'a.txt',
            },);
            await waitForFeed({ runFolder, text: '"preview":"second by name"', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, },) => {
            await waitForFeed({ runFolder, text: '"kind":"settled","n":1', },);
            await mkdir(join(runFolder, 'inbox', 'folder',),);
            await waitForFeed({ runFolder, text: 'could not deliver folder', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, },) => {
            await waitForFeed({ runFolder, text: '"kind":"settled","n":1', },);
            await dropInInbox({
              runFolder,
              text: 'kept but unnamed',
              name: tooLong,
            },);
            await waitForFeed({ runFolder, text: 'could not keep', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
          whileRunning: async ({ runFolder, },) => {
            await waitForFeed({ runFolder, text: '"kind":"settled","n":1', },);
            await makePipe(join(runFolder, 'inbox', 'pipe.txt',),);
            await waitForFeed({ runFolder, text: 'could not deliver pipe.txt', },);
            await dropInInbox({ runFolder, text: 'after the pipe', },);
            await waitForFeed({ runFolder, text: '"preview":"after the pipe"', },);
            await writeFile(join(runFolder, 'stop',), '',);
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
      name: 'names the option when one is missing',
      fn: async () => {
        const [status, standardError,] = await exitOf(execFileAsync(process.execPath, [DRIVER,], { encoding: 'utf8', },),);
        expect(status,).toBe(FAILED_STATUS,);
        expect(standardError,).toContain('--run-folder is required',);
      },
    },),
  ],
},);
