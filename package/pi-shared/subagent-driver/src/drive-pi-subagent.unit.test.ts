/**
 The driver's bin against a stand-in `pi`, one test for each way a run can go. Each test uses its own disposable folder.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
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
import { NESTED_RUN_VARIABLE, } from './driver-constants.ts';
import type { RunSummary, } from './driver-types.ts';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

const DRIVER = new URL('drive-pi-subagent.ts', import.meta.url,).pathname;
const STAND_IN = new URL('stand-in-pi-for-tests.ts', import.meta.url,).pathname;

/** Seconds a run may last; the wall-clock test waits this long. */
const WALL_SECONDS = 2;
/** Seconds of silence after which a settled run ends. */
const IDLE_SECONDS = 0.2;
/** Turns allowed, below the stand-in's many-turns scenario and above every other. */
const TURN_CAP = 3;
/** Milliseconds between looks at the feed while waiting for the prompt to go out. */
const FEED_POLL_MILLISECONDS = 20;
/** Status of a run that ended on its own terms. */
const CLEAN_STATUS = 0;
/** Status of a run that did not. */
const FAILED_STATUS = 1;

type Finished = {
  readonly status: number;
  readonly standardError: string;
  readonly summary: RunSummary | undefined;
  readonly feed: readonly Record<string, unknown>[];
  readonly commands: readonly string[];
  readonly nestedVariable: string | undefined;
  readonly piArguments: readonly string[];
  readonly inbox: readonly string[];
};

type RunRequest = {
  readonly scenario: string;
  readonly extraOptions?: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
  readonly withoutPi?: boolean;
  readonly afterPromptSent?: (runFolder: string,) => Promise<void>;
};

async function readLines(file: string,): Promise<readonly string[]> {
  try {
    return (await readFile(file, 'utf8',)).split('\n',).filter((line,) => line !== '');
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

async function waitForPromptSent(feedFile: string,): Promise<void> {
  for (;;) {
    if ((await readLines(feedFile,)).some((line,) => line.includes('"kind":"prompt_sent"',))) {
      return;
    }
    await wait(FEED_POLL_MILLISECONDS,);
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

async function runDriver(
  { scenario, extraOptions = [], environment = {}, withoutPi = false, afterPromptSent, }: RunRequest,
): Promise<Finished> {
  const folder = await mkdtemp(join(scratchRoot(), 'pi-subagent-driver-',),);
  try {
    const runFolder = join(folder, 'run',);
    const promptFile = join(folder, 'prompt.md',);
    const log = join(folder, 'commands.log',);
    await writeFile(promptFile, 'hello',);
    if (!withoutPi) {
      await installStandIn(folder,);
    }
    const pathWithStandIn = [folder, dirname(process.execPath,), '/usr/bin',].join(delimiter,);
    const running = execFileAsync(process.execPath, [
      DRIVER,
      `--run-folder=${runFolder}`,
      `--working-folder=${folder}`,
      `--prompt-file=${promptFile}`,
      '--provider=stand-in',
      '--model=stand-in-model',
      `--wall-seconds=${String(WALL_SECONDS,)}`,
      `--idle-seconds=${String(IDLE_SECONDS,)}`,
      `--turn-cap=${String(TURN_CAP,)}`,
      ...extraOptions,
    ], {
      env: { PATH: withoutPi ? dirname(process.execPath,) : pathWithStandIn, HOME: process.env.HOME, STAND_IN_PI_SCENARIO: scenario, STAND_IN_PI_LOG: log, ...environment, },
      encoding: 'utf8',
    },);
    const afterSent = afterPromptSent === undefined
      ? Promise.resolve()
      : (async () => {
        await waitForPromptSent(join(runFolder, 'feed.jsonl',),);
        await afterPromptSent(runFolder,);
      })();
    const [status, standardError,] = await (async (): Promise<readonly [number, string]> => {
      try {
        const result = await running;
        return [CLEAN_STATUS, result.stderr,];
      }
      catch (error) {
        const failure = error as { readonly code: number; readonly stderr: string; };
        return [failure.code, failure.stderr,];
      }
    })();
    await afterSent;
    const logged = await readLines(log,);
    let summary: RunSummary | undefined;
    try {
      summary = JSON.parse(await readFile(join(runFolder, 'summary.json',), 'utf8',),) as RunSummary;
    }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
        throw error;
      }
    }
    return {
      status,
      standardError,
      summary,
      feed: (await readLines(join(runFolder, 'feed.jsonl',),)).map((line,) => JSON.parse(line,) as Record<string, unknown>),
      commands: logged.filter((line,) => !line.startsWith('arguments ',) && !line.startsWith('nested ',)),
      nestedVariable: logged.find((line,) => line.startsWith('nested ',))?.slice('nested '.length,),
      piArguments: logged.filter((line,) => line.startsWith('arguments ',)).flatMap((line,) => JSON.parse(line.slice('arguments '.length,),) as string[]),
      inbox: await readdir(join(runFolder, 'inbox',),).catch(() => []),
    };
  }
  finally {
    await rm(folder, { recursive: true, force: true, },);
  }
}

await describe({
  name: 'drive-pi-subagent',
  children: [
    it({
      name: 'a run that settles and goes quiet ends idle, exits 0 and keeps the reply whole',
      fn: async () => {
        const run = await runDriver({ scenario: 'normal', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary?.endReason,).toBe('idle',);
        expect(run.summary?.lastAssistantText,).toBe('done',);
        expect(run.summary?.settled,).toBe(1,);
        expect(run.summary?.turns,).toBe(1,);
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
        expect(run.summary?.endReason,).toBe('prompt-rejected',);
        expect(run.standardError,).toContain('prompt-rejected',);
      },
    },),
    it({
      name: 'a Pi that crashes at start fails the run and records its status',
      fn: async () => {
        const run = await runDriver({ scenario: 'crash', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary?.piExit?.code,).toBe(3,);
      },
    },),
    it({
      name: 'a Pi that cannot be started fails the run and says why',
      fn: async () => {
        const run = await runDriver({ scenario: 'normal', withoutPi: true, },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary?.piExit?.startError,).toContain('ENOENT',);
      },
    },),
    it({
      name: 'a Pi that exits during the run ends it as pi-exited',
      fn: async () => {
        const run = await runDriver({ scenario: 'crash-later', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary?.endReason,).toBe('pi-exited',);
      },
    },),
    it({
      name: 'a run that never settles ends at the wall clock and is aborted',
      fn: async () => {
        const run = await runDriver({ scenario: 'never-settles', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary?.endReason,).toBe('wall-clock',);
        expect(run.commands,).toContain('abort',);
      },
    },),
    it({
      name: 'a run over its turn cap ends as turn-cap',
      fn: async () => {
        const run = await runDriver({ scenario: 'many-turns', },);
        expect(run.status,).toBe(FAILED_STATUS,);
        expect(run.summary?.endReason,).toBe('turn-cap',);
      },
    },),
    it({
      name: 'a stop file ends the run cleanly',
      fn: async () => {
        const run = await runDriver({
          scenario: 'never-settles',
          afterPromptSent: async (runFolder,) => {
            await writeFile(join(runFolder, 'stop',), '',);
          },
        },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary?.endReason,).toBe('stop-file',);
      },
    },),
    it({
      name: 'a prompt Pi handles without a run ends idle, with no settled event to wait for',
      fn: async () => {
        const run = await runDriver({ scenario: 'handled', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary?.endReason,).toBe('idle',);
        expect(run.feed.some((entry,) => entry.kind === 'settled'),).toBe(false,);
        expect(run.feed.find((entry,) => entry.kind === 'prompt_sent')?.disposition,).toBe('handled',);
      },
    },),
    it({
      name: 'a message dropped between agent_end and agent_settled goes as a steer',
      fn: async () => {
        const run = await runDriver({
          scenario: 'settle-gap',
          afterPromptSent: async (runFolder,) => {
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
          afterPromptSent: async (runFolder,) => {
            await dropInInbox({ runFolder, text: 'more', },);
          },
        },);
        expect(run.feed.find((entry,) => entry.kind === 'inbox')?.sentAs,).toBe('prompt',);
        expect(run.summary?.settled,).toBe(2,);
      },
    },),
    it({
      name: 'a message Pi refuses is kept under a dot name and not sent again',
      fn: async () => {
        const run = await runDriver({
          scenario: 'reject-inbox',
          afterPromptSent: async (runFolder,) => {
            await dropInInbox({ runFolder, text: 'more', },);
          },
        },);
        expect(run.inbox,).toEqual(['.failed-message.txt',],);
        expect(run.commands.filter((command,) => command === 'prompt').length,).toBe(2,);
      },
    },),
    it({
      name: 'declines a dialog so it cannot stall the session, and leaves a notice alone',
      fn: async () => {
        const run = await runDriver({ scenario: 'dialog', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.summary?.dialogs.length,).toBe(1,);
        expect(run.commands.filter((command,) => command === 'extension_ui_response').length,).toBe(1,);
      },
    },),
    it({
      name: 'counts tools by name and keeps the ones that failed',
      fn: async () => {
        const run = await runDriver({ scenario: 'tools', },);
        expect(run.summary?.toolHistogram,).toEqual({ bash: 2, },);
        expect(run.summary?.toolErrors.length,).toBe(1,);
        expect(run.feed.some((entry,) => entry.kind === 'auto_retry_start'),).toBe(true,);
      },
    },),
    it({
      name: 'notes a line that is not JSON and carries on',
      fn: async () => {
        const run = await runDriver({ scenario: 'noise', },);
        expect(run.status,).toBe(CLEAN_STATUS,);
        expect(run.feed.some((entry,) => entry.kind === 'non-json'),).toBe(true,);
        expect(run.summary?.lastAssistantText,).toBe('after the noise',);
      },
    },),
    it({
      name: 'reads a record split inside a character and keeps a Unicode line separator inside it',
      fn: async () => {
        const run = await runDriver({ scenario: 'split-record', },);
        expect(run.summary?.lastAssistantText,).toBe('café line separator',);
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
        const failure = await execFileAsync(process.execPath, [DRIVER,], { encoding: 'utf8', },).catch((error: unknown,) => error as { readonly code: number; readonly stderr: string; });
        expect((failure as { readonly stderr: string; }).stderr,).toContain('--run-folder is required',);
      },
    },),
  ],
},);
