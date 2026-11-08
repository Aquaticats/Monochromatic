/**
 The driver's bin against a stand-in `pi`, one test for each way a run can go. Each test uses its own disposable folder. Steering
 tests wait for what the feed shows instead of for a length of time, so a loaded host cannot change their outcome.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
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
import {
  setInterval as everyInterval,
  setTimeout as wait,
} from 'node:timers/promises';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  NESTED_RUN_VARIABLE,
  POLL_MILLISECONDS,
  type RunSummary,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

const DRIVER = new URL('drive-pi-subagent.ts', import.meta.url,).pathname;
const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/** Seconds a run may last; long enough that only the test about this limit reaches it. */
const WALL_SECONDS = 60;
/** Seconds the test about the wall clock allows. */
const SHORT_WALL_SECONDS = 1;
/** Seconds of silence after which a settled run ends. */
const IDLE_SECONDS = 0.2;
/** Seconds of idle wait in tests that end the run with the stop file. */
const LONG_IDLE_SECONDS = 60;
/** Turns allowed, below the stand-in's many-turns scenario and above every other. */
const TURN_CAP = 3;
/** Milliseconds between looks at the feed while waiting for a line. */
const FEED_POLL_MILLISECONDS = 20;
/** Milliseconds in a second. */
const MILLISECONDS_PER_SECOND = 1_000;
/** Status of a run that ended on its own terms. */
const CLEAN_STATUS = 0;
/** Status of a run that did not. */
const FAILED_STATUS = 1;
/** Polls of the inbox to leave room for a refused message to be sent again, if that were broken. */
const POLLS_TO_CATCH_A_RESEND = 3;
/** A Unicode line separator, which must survive inside a record. */
const LINE_SEPARATOR = String.fromCodePoint(0x20_28,);
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
  readonly nestedVariable: string;
  readonly piArguments: readonly string[];
  readonly inbox: readonly string[];
};

type RunRequest = {
  readonly scenario: string;
  readonly extraOptions?: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
  readonly withoutPi?: boolean;
  readonly withoutPrompt?: boolean;
  readonly wallSeconds?: number;
  readonly idleSeconds?: number;
  readonly whileRunning?: (runFolder: string,) => Promise<void>;
};

type DisposableFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

function isMissingFile(error: unknown,): boolean {
  return (error as NodeJS.ErrnoException).code === 'ENOENT';
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
    if (isMissingFile(error,)) {
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

async function waitForFeed({ runFolder, text, }: { readonly runFolder: string; readonly text: string; },): Promise<void> {
  const deadline = Date.now() + (WALL_SECONDS * MILLISECONDS_PER_SECOND);
  for await (const feedFile of everyInterval(FEED_POLL_MILLISECONDS, join(runFolder, 'feed.jsonl',),)) {
    if ((await readLines(feedFile,)).some((line,) => line.includes(text,))) {
      return;
    }
    if (Date.now() > deadline) {
      throw new Error(`the feed never showed ${text}`,);
    }
  }
}

async function dropInInbox({ runFolder, text, }: { readonly runFolder: string; readonly text: string; },): Promise<void> {
  await writeFile(join(runFolder, 'inbox', '.message',), text,);
  await rename(join(runFolder, 'inbox', '.message',), join(runFolder, 'inbox', 'message.txt',),);
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

async function runDriver(
  {
    scenario,
    extraOptions = [],
    environment = {},
    withoutPi = false,
    withoutPrompt = false,
    wallSeconds = WALL_SECONDS,
    idleSeconds = IDLE_SECONDS,
    whileRunning,
  }: RunRequest,
): Promise<Finished> {
  await using folder = await makeFolder();
  const runFolder = join(folder.path, 'run',);
  const promptFile = join(folder.path, 'prompt.md',);
  const log = join(folder.path, 'commands.log',);
  if (!withoutPrompt) {
    await writeFile(promptFile, 'hello',);
  }
  if (!withoutPi) {
    await installStandIn(folder.path,);
  }
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
      PATH: withoutPi ? dirname(process.execPath,) : pathWithStandIn,
      HOME: process.env.HOME,
      STAND_IN_PI_SCENARIO: scenario,
      STAND_IN_PI_LOG: log,
      ...environment,
    },
    encoding: 'utf8',
  },);
  const [[status, standardError,],] = await Promise.all([
    exitOf(running,),
    whileRunning === undefined ? Promise.resolve() : whileRunning(runFolder,),
  ],);
  const logged = await readLines(log,);
  return {
    status,
    standardError,
    summary: await readSummary(join(runFolder, 'summary.json',),),
    feed: (await readLines(join(runFolder, 'feed.jsonl',),)).map((line,) => JSON.parse(line,) as Record<string, unknown>),
    commands: logged.filter((line,) => (!line.startsWith('arguments ',)) && (!line.startsWith('nested ',))),
    nestedVariable: logged.find((line,) => line.startsWith('nested ',))?.slice('nested '.length,) ?? '',
    piArguments: logged.filter((line,) => line.startsWith('arguments ',)).flatMap((line,) =>
      JSON.parse(line.slice('arguments '.length,),) as string[]
    ),
    inbox: await listInbox(join(runFolder, 'inbox',),),
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
      name: 'a Pi that exits during the run ends it as pi-exited',
      fn: async () => {
        const run = await runDriver({ scenario: 'crash-later', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('pi-exited',);
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
          whileRunning: async (runFolder,) => {
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
        const run = await runDriver({ scenario: 'normal', withoutPrompt: true, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary.endReason,).toBe('driver-failed',);
        expect(String(run.summary.driverError,),).toContain('ENOENT',);
        expect(run.summary.piExit?.code,).toBe(0,);
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
          whileRunning: async (runFolder,) => {
            await waitForFeed({ runFolder, text: '"kind":"prompt_sent"', },);
            await dropInInbox({ runFolder, text: 'more', },);
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
          whileRunning: async (runFolder,) => {
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
          whileRunning: async (runFolder,) => {
            await waitForFeed({ runFolder, text: '"kind":"settled","n":1', },);
            await dropInInbox({ runFolder, text: 'more', },);
            await waitForFeed({ runFolder, text: '"kind":"inbox"', },);
            await wait(POLL_MILLISECONDS * POLLS_TO_CATCH_A_RESEND,);
            await writeFile(join(runFolder, 'stop',), '',);
          },
        },);
        expect(run.inbox,).toEqual(['.failed-message.txt',],);
        expect(run.commands.filter((command,) => command === 'prompt').length,).toBe(2,);
      },
    },),
    it({
      name: 'an inbox entry that cannot be read is set aside and the run goes on',
      fn: async () => {
        const run = await runDriver({
          scenario: 'normal',
          idleSeconds: LONG_IDLE_SECONDS,
          whileRunning: async (runFolder,) => {
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
      name: 'declines a dialog so it cannot stall the session, and leaves a notice alone',
      fn: async () => {
        const run = await runDriver({ scenario: 'dialog', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary.dialogs.length,).toBe(1,);
        expect(run.commands.filter((command,) => command === 'extension_ui_response').length,).toBe(1,);
      },
    },),
    it({
      name: 'counts tools by name and keeps the ones that failed',
      fn: async () => {
        const run = await runDriver({ scenario: 'tools', },);
        expect(run.summary.toolHistogram,).toEqual({ bash: 2, },);
        expect(run.summary.toolErrors.length,).toBe(1,);
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
        expect(run.summary.lastAssistantText,).toBe(`café line${LINE_SEPARATOR}separator`,);
      },
    },),
    it({
      name: 'refuses to run inside a Pi that a driver is running, before starting anything',
      fn: async () => {
        const run = await runDriver({ scenario: 'normal', environment: { [NESTED_RUN_VARIABLE]: '1', }, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.standardError,).toContain('NestedDriverError',);
        expect(run.commands,).toEqual([],);
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
