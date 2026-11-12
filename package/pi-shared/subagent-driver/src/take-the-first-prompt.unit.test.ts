/**
 Taking a run from a started Pi to the reason it ended: the bound its three start-up answers wait under, what it notes about
 them, and how each ending it answers with comes about. The connection is handed to it, so a test decides what Pi says and reads
 what was asked of Pi.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  COMMAND_ANSWER_MILLISECONDS,
  type CommandAnswer,
  type CommandRequest,
  type ConnectionToPi,
  type DriverOptions,
  type EndReason,
  endReasonOfRun,
  type PiRecord,
  type RecordOfPiRun,
  startRecordOfPiRun,
  START_UP_ANSWER_MILLISECONDS,
} from '../dist/final/node/index.mjs';

/**
 A folder the test throws away.
 */
type DisposableFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a folder for one test.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<DisposableFolder> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-first-prompt-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 What a test's stand-in Pi was asked.
 */
type StandInPi = {
  /**
   The connection to hand to the run.
   */
  readonly connection: ConnectionToPi;

  /**
   Every command sent, in the order Pi received them.
   */
  readonly requests: readonly CommandRequest[];
};

/**
 Reads the feed a run wrote.

 @param feedFile - the feed

 @returns the lines, each as a record

 @example
 ```ts
 const lines = await feedOf('/runs/a-1/feed.jsonl');
 ```
 */
async function feedOf(feedFile: string,): Promise<readonly PiRecord[]> {
  /**
   The feed's text.
   */
  const text = await readFile(
    feedFile,
    'utf8',
  );
  return text.split('\n',)
    .filter(function isLine(line,) {
      return line !== '';
    },)
    .map(function asRecord(line,) {
      return JSON.parse(line,) as PiRecord;
    },);
}

/**
 Builds a stand-in Pi that answers every command, and the prompt the way a test asks.

 @param promptAnswer - what Pi makes of the first prompt

 @returns the connection, and what it was asked

 @example
 ```ts
 const standIn = standInPi({ promptAnswer: { ending: 'answered', record: { success: true, }, }, });
 ```
 */
function standInPi({ promptAnswer, }: { readonly promptAnswer: CommandAnswer },): StandInPi {
  /**
   What Pi was asked, which the connection below appends to.
   */
  const requests: CommandRequest[] = [];
  return {
    requests,
    connection: {
      hasExited: function hasExited() {
        return false;
      },
      exitDetails: function exitDetails() {
        return {};
      },
      write: function write() {
        // A dialog is declined through this, and no test here meets one.
      },
      send: async function send(request: CommandRequest,): Promise<CommandAnswer> {
        requests.push(request,);
        if (request.command.type === 'get_state') {
          return {
            ending: 'answered',
            record: {
              success: true,
              data: {
                model: { id: 'stand-in-model', },
                thinkingLevel: 'xhigh',
              },
            },
          };
        }
        if (request.command.type === 'get_commands') {
          return {
            ending: 'answered',
            record: {
              success: true,
              data: { commands: [{ name: 'bash', },], },
            },
          };
        }
        return (request.command.type === 'prompt')
          ? promptAnswer
          : {
            ending: 'answered',
            record: { success: true, },
          };
      },
      shutDown: async function shutDown() {
        return { stopped: true, };
      },
    },
  };
}

/**
 A prompt Pi accepted, which leaves the run going.
 */
const ACCEPTED: CommandAnswer = {
  ending: 'answered',
  record: {
    success: true,
    data: { disposition: 'started', },
  },
};

/**
 A prompt Pi refused, which ends the run before the watch sends anything else.
 */
const REFUSAL: CommandAnswer = {
  ending: 'answered',
  record: {
    success: false,
    error: 'no key for the provider',
  },
};

/**
 Runs one first prompt against a stand-in Pi.

 @param promptAnswer - what Pi makes of the prompt

 @param millisecondsOfRun - how long the run may last

 @returns why the run ended, what Pi was asked, and the feed the run wrote

 @example
 ```ts
 const first = await firstPromptOf({ promptAnswer: REFUSAL, millisecondsOfRun: 1_000, });
 ```
 */
async function firstPromptOf({
  promptAnswer,
  millisecondsOfRun,
}: {
  readonly promptAnswer: CommandAnswer;
  readonly millisecondsOfRun: number;
}): Promise<{
  readonly endReason: EndReason;
  readonly standIn: StandInPi;
  readonly record: RecordOfPiRun;
  readonly feed: readonly PiRecord[];
}> {
  await using folder = await makeFolder();
  /**
   Where the run writes.
   */
  const runFolder = join(folder.path, 'run',);
  /**
   Where messages for Pi arrive, which the watch reads on every tick.
   */
  const inbox = join(runFolder, 'inbox',);
  await mkdir(inbox, { recursive: true, },);
  /**
   The first prompt.
   */
  const promptFile = join(folder.path, 'prompt.md',);
  await writeFile(promptFile, 'do the work',);
  /**
   The feed of the run.
   */
  const feedFile = join(runFolder, 'feed.jsonl',);
  /**
   What the run was asked for.
   */
  const options: DriverOptions = {
    runFolder,
    workingFolder: folder.path,
    promptFile,
    provider: 'stand-in',
    model: 'stand-in-model',
    piArguments: [],
    wallSeconds: 1,
    idleSeconds: 0,
    turnCap: 2,
    retryCap: 0,
    retryBaseSeconds: 1,
  };
  /**
   The record of the run.
   */
  const record = startRecordOfPiRun({ feedFile, },);
  /**
   What Pi answers.
   */
  const standIn = standInPi({ promptAnswer, },);
  /**
   Why the run ended.
   */
  const { endReason, } = await endReasonOfRun({
    connection: standIn.connection,
    record,
    options,
    inbox,
    deadline: Date.now() + millisecondsOfRun,
  },);
  await record.close();
  return {
    endReason,
    standIn,
    record,
    feed: await feedOf(feedFile,),
  };
}

await describe({
  name: endReasonOfRun.name,
  children: [
    it({
      name: 'waits for the state, the commands and the first prompt under the bound a starting Pi gets, not the one an answer owed mid-run gets',
      fn: async () => {
        /**
         What a run whose prompt Pi refused did, which ends it before the watch sends anything else.
         */
        const { endReason, standIn, } = await firstPromptOf({
          promptAnswer: REFUSAL,
          millisecondsOfRun: 1_000,
        },);
        expect(endReason,).toBe('prompt-rejected',);
        expect(standIn.requests.map(function typeOf(request,) {
          return request.command.type;
        },),).toEqual([
          'get_state',
          'get_commands',
          'prompt',
        ],);
        expect(standIn.requests.map(function boundOf(request,) {
          return request.answerMilliseconds;
        },),).toEqual([
          START_UP_ANSWER_MILLISECONDS,
          START_UP_ANSWER_MILLISECONDS,
          START_UP_ANSWER_MILLISECONDS,
        ],);
        expect(START_UP_ANSWER_MILLISECONDS,).toBeGreaterThan(COMMAND_ANSWER_MILLISECONDS,);
      },
    },),
    it({
      name: 'notes the model and the commands Pi starts with, and sends the prompt file whole',
      fn: async () => {
        /**
         What a run whose prompt Pi refused noted about its start-up.
         */
        const { standIn, feed, } = await firstPromptOf({
          promptAnswer: REFUSAL,
          millisecondsOfRun: 1_000,
        },);
        /**
         What the feed says Pi starts with.
         */
        const startingState = feed.find(function isStartingState(line,) {
          return line.kind === 'state';
        },);
        expect(startingState?.model,).toBe('stand-in-model',);
        expect(startingState?.thinkingLevel,).toBe('xhigh',);
        expect(feed.find(function isCommands(line,) {
          return line.kind === 'commands';
        },)?.names,).toEqual(['bash',],);
        /**
         The prompt Pi was sent.
         */
        const prompt = standIn.requests.find(function isPrompt(request,) {
          return request.command.type === 'prompt';
        },);
        expect(prompt?.command.message,).toBe('do the work',);
        /**
         What the feed says Pi made of the prompt.
         */
        const noted = feed.find(function isPromptSent(line,) {
          return line.kind === 'prompt_sent';
        },);
        expect(noted?.success,).toBe(false,);
        expect(noted?.error,).toBe('no key for the provider',);
        expect(noted?.chars,).toBe('do the work'.length,);
      },
    },),
    it({
      name: 'ends as wall-clock without asking Pi anything when the run has no time left',
      fn: async () => {
        /**
         What a run with no time left did.
         */
        const { endReason, standIn, } = await firstPromptOf({
          promptAnswer: REFUSAL,
          millisecondsOfRun: 0,
        },);
        expect(endReason,).toBe('wall-clock',);
        expect(standIn.requests,).toEqual([],);
      },
    },),
    it({
      name: 'hands a run whose prompt Pi took to the watch, which ends it at its wall clock',
      fn: async () => {
        /**
         What a run whose prompt Pi took did.
         */
        const { endReason, standIn, } = await firstPromptOf({
          promptAnswer: ACCEPTED,
          millisecondsOfRun: 150,
        },);
        expect(endReason,).toBe('wall-clock',);
        expect(standIn.requests.map(function typeOf(request,) {
          return request.command.type;
        },),).toEqual([
          'get_state',
          'get_commands',
          'prompt',
        ],);
      },
    },),
    it({
      name: 'ends as driver-failed, on the feed, when the prompt file cannot be read, instead of throwing',
      fn: async () => {
        await using folder = await makeFolder();
        /**
         Where the run writes.
         */
        const runFolder = join(folder.path, 'run',);
        await mkdir(runFolder, { recursive: true, },);
        /**
         The feed of the run.
         */
        const feedFile = join(runFolder, 'feed.jsonl',);
        /**
         What the run was asked for, naming a prompt file nobody wrote.
         */
        const options: DriverOptions = {
          runFolder,
          workingFolder: folder.path,
          promptFile: join(folder.path, 'no-prompt.md',),
          provider: 'stand-in',
          model: 'stand-in-model',
          piArguments: [],
          wallSeconds: 1,
          idleSeconds: 0,
          turnCap: 2,
          retryCap: 0,
          retryBaseSeconds: 1,
        };
        /**
         The record of the run.
         */
        const record = startRecordOfPiRun({ feedFile, },);
        /**
         Why the run ended.
         */
        const { endReason, } = await endReasonOfRun({
          connection: standInPi({ promptAnswer: REFUSAL, },).connection,
          record,
          options,
          inbox: runFolder,
          deadline: Date.now() + 1_000,
        },);
        await record.close();
        expect(endReason,).toBe('driver-failed',);
        expect((await feedOf(feedFile,)).find(function isFailure(line,) {
          return line.kind === 'driver-failed';
        },)?.failure,).toContain('no-prompt.md',);
      },
    },),
  ],
},);
