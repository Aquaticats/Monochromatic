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
  START_PAUSE_MILLISECONDS,
  START_UP_ANSWER_MILLISECONDS,
  MASKING_ARMED_COMMAND,
  MASKING_NOTHING,
  maskingArmedDescription,
  type StartThatEndedWithoutARun,
} from '../dist/final/node/index.mjs';

/**
 Milliseconds in a second, so a limit the options state in seconds is derived from the value the driver ships.
 */
const MILLISECONDS_PER_SECOND = 1_000;

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
 Pi's own state, answered.
 */
const ANSWERED_STATE: CommandAnswer = {
  ending: 'answered',
  record: {
    success: true,
    data: {
      model: { id: 'stand-in-model', },
      thinkingLevel: 'xhigh',
    },
  },
};

/**
 Pi's command list, answered with the commands Pi starts with.
 */
const ANSWERED_COMMANDS: CommandAnswer = {
  ending: 'answered',
  record: {
    success: true,
    data: { commands: [{ name: 'bash', },], },
  },
};

/**
 Pi's command list, answered with the masking command one value arms, which is what a run needs before it may send a prompt.
 */
const ANSWERED_COMMANDS_WITH_MASKING: CommandAnswer = {
  ending: 'answered',
  record: {
    success: true,
    data: {
      commands: [
        { name: 'bash', },
        {
          name: MASKING_ARMED_COMMAND,
          description: maskingArmedDescription({ count: 1, },),
        },
      ],
    },
  },
};

/**
 A command list Pi answered without the masking command, which is a start that sends no prompt.
 */
const COMMANDS_WITHOUT_MASKING: CommandAnswer = {
  ending: 'answered',
  record: {
    success: true,
    data: { commands: [{ name: 'bash', },], },
  },
};

/**
 A wait whose silence ran out, as the driver settles it.
 */
function silenceAt({ answerMilliseconds, }: { readonly answerMilliseconds: number },): CommandAnswer {
  return {
    ending: 'silence',
    problem: `no answer in ${String(answerMilliseconds,)} ms`,
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
 Builds a stand-in Pi that answers each command the way a test asks: with an answer, with the bound running out, or by having gone.

 @param stateAnswer - what Pi makes of the state ask

 @param commandsAnswer - what Pi makes of the command-list ask

 @param promptAnswer - what Pi makes of the first prompt

 @param exit - how Pi had gone, which a wait that ended because Pi left reports

 @returns the connection, and what it was asked

 @example
 ```ts
 const standIn = standInPi({ promptAnswer: { ending: 'answered', record: { success: true, }, }, });
 ```
 */
function standInPi({
  stateAnswer = ANSWERED_STATE,
  commandsAnswer = ANSWERED_COMMANDS,
  promptAnswer,
  exit = {},
}: {
  readonly stateAnswer?: CommandAnswer;
  readonly commandsAnswer?: CommandAnswer;
  readonly promptAnswer: CommandAnswer;
  readonly exit?: { readonly code?: number; readonly signal?: string };
},): StandInPi {
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
        return exit;
      },
      write: function write() {
        // A dialog is declined through this, and no test here meets one.
      },
      send: async function send(request: CommandRequest,): Promise<CommandAnswer> {
        requests.push(request,);
        if (request.command.type === 'get_state') {
          return stateAnswer;
        }
        if (request.command.type === 'get_commands') {
          return commandsAnswer;
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
  stateAnswer = ANSWERED_STATE,
  commandsAnswer = ANSWERED_COMMANDS,
  exit,
  startUpAnswerMilliseconds,
  handedOver,
}: {
  readonly promptAnswer: CommandAnswer;
  readonly millisecondsOfRun: number;
  readonly stateAnswer?: CommandAnswer;
  readonly commandsAnswer?: CommandAnswer;
  readonly exit?: { readonly code?: number; readonly signal?: string };
  readonly startUpAnswerMilliseconds?: number;
  readonly handedOver?: number;
}): Promise<{
  readonly endReason: EndReason;
  readonly standIn: StandInPi;
  readonly record: RecordOfPiRun;
  readonly feed: readonly PiRecord[];
  readonly startThatEndedWithoutARun?: StartThatEndedWithoutARun;
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
    startPauseSeconds: START_PAUSE_MILLISECONDS / MILLISECONDS_PER_SECOND,
  };
  /**
   The record of the run.
   */
  const record = startRecordOfPiRun({
    feedFile,
    mask: MASKING_NOTHING,
  },);
  /**
   What Pi answers.
   */
  const standIn = standInPi({
    stateAnswer,
    commandsAnswer,
    promptAnswer,
    ...(exit === undefined ? {} : { exit }),
  },);
  /**
   Why the run ended, and what a start that ended without a run has to say.
   */
  const { endReason, startThatEndedWithoutARun, } = await endReasonOfRun({
    connection: standIn.connection,
    record,
    options,
    inbox,
    deadline: Date.now() + millisecondsOfRun,
    ...(startUpAnswerMilliseconds === undefined ? {} : { startUpAnswerMilliseconds }),
    ...(handedOver === undefined ? {} : { secretValuesHandedOver: handedOver }),
  },);
  await record.close();
  return {
    endReason,
    standIn,
    record,
    feed: await feedOf(feedFile,),
    ...(startThatEndedWithoutARun === undefined ? {} : { startThatEndedWithoutARun }),
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
          startPauseSeconds: START_PAUSE_MILLISECONDS / MILLISECONDS_PER_SECOND,
        };
        /**
         The record of the run.
         */
        const record = startRecordOfPiRun({
          feedFile,
          mask: MASKING_NOTHING,
        },);
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

await describe({
  name: 'a start that ended without a run',
  children: [
    it({
      name: 'names the two asks that went unanswered when Pi answered nothing, with each bound and how long each had waited',
      fn: async () => {
        /**
         The bound each ask is given in this test, short because the stand-in answers at once.
         */
        const BOUND_MILLISECONDS = 40;
        /**
         What a run whose Pi answered nothing has to say of its start.
         */
        const { endReason, startThatEndedWithoutARun, } = await firstPromptOf({
          stateAnswer: silenceAt({ answerMilliseconds: BOUND_MILLISECONDS, },),
          commandsAnswer: silenceAt({ answerMilliseconds: BOUND_MILLISECONDS, },),
          promptAnswer: silenceAt({ answerMilliseconds: BOUND_MILLISECONDS, },),
          millisecondsOfRun: 5_000,
          startUpAnswerMilliseconds: BOUND_MILLISECONDS,
          handedOver: 1,
        },);
        expect(endReason,).toBe('prompt-rejected',);
        /**
         The waits the note names, which are the two asks that went unanswered.
         */
        const waits = startThatEndedWithoutARun?.waits ?? [];
        expect(waits,).toHaveLength(2,);
        expect(waits[0]?.wait,).toContain('get_state',);
        expect(waits[1]?.wait,).toContain('get_commands',);
        expect(waits.map((wait,) => wait.bound,),).toEqual([
          `${String(BOUND_MILLISECONDS,)} ms`,
          `${String(BOUND_MILLISECONDS,)} ms`,
        ],);
        for (const wait of waits) {
          expect(wait.waitedMilliseconds,).toBeGreaterThanOrEqual(0,);
        }
        expect(startThatEndedWithoutARun?.what,).toContain('sent no prompt',);
      },
    },),
    it({
      name: 'names only the command list when Pi answered its state and then nothing',
      fn: async () => {
        const { endReason, startThatEndedWithoutARun, } = await firstPromptOf({
          commandsAnswer: silenceAt({ answerMilliseconds: 40, },),
          promptAnswer: silenceAt({ answerMilliseconds: 40, },),
          millisecondsOfRun: 5_000,
          startUpAnswerMilliseconds: 40,
          handedOver: 1,
        },);
        expect(endReason,).toBe('prompt-rejected',);
        const waits = startThatEndedWithoutARun?.waits ?? [];
        expect(waits,).toHaveLength(1,);
        expect(waits[0]?.wait,).toContain('get_commands',);
        expect(startThatEndedWithoutARun?.what,).toContain('masks this machine\'s secret values',);
      },
    },),
    it({
      name: 'names the first prompt when Pi answered its state and its commands with the masking in place, and then nothing',
      fn: async () => {
        const { endReason, startThatEndedWithoutARun, } = await firstPromptOf({
          commandsAnswer: ANSWERED_COMMANDS_WITH_MASKING,
          promptAnswer: silenceAt({ answerMilliseconds: 40, },),
          millisecondsOfRun: 5_000,
          startUpAnswerMilliseconds: 40,
          handedOver: 1,
        },);
        expect(endReason,).toBe('prompt-rejected',);
        const waits = startThatEndedWithoutARun?.waits ?? [];
        expect(waits,).toHaveLength(1,);
        expect(waits[0]?.wait,).toContain('first prompt',);
        expect(waits[0]?.bound,).toBe('40 ms',);
        expect(startThatEndedWithoutARun?.what,).toContain('took no prompt',);
      },
    },),
    it({
      name: 'names the command list with no bound of its own when Pi answered it without the masking',
      fn: async () => {
        const { endReason, startThatEndedWithoutARun, } = await firstPromptOf({
          commandsAnswer: COMMANDS_WITHOUT_MASKING,
          promptAnswer: ACCEPTED,
          millisecondsOfRun: 100,
          handedOver: 1,
        },);
        expect(endReason,).toBe('masking-not-armed',);
        const waits = startThatEndedWithoutARun?.waits ?? [];
        expect(waits,).toHaveLength(1,);
        expect(waits[0]?.wait,).toContain('command list',);
        // Pi answered, so no bound ran out: the note says there is none rather than naming one.
        expect(waits[0]?.bound,).toBeUndefined();
        expect(startThatEndedWithoutARun?.what,).toContain('armed the masking',);
        expect(startThatEndedWithoutARun?.next,).toContain('stderr.log',);
      },
    },),
    it({
      name: 'names the prompt ask that ended when Pi went, and says which status Pi left with',
      fn: async () => {
        /**
         What Pi's waits answer once it has gone, which is what the connection settles them with.
         */
        const GONE: CommandAnswer = {
          ending: 'pi-exited',
          problem: 'pi exited',
        };
        const { endReason, startThatEndedWithoutARun, } = await firstPromptOf({
          stateAnswer: GONE,
          commandsAnswer: GONE,
          promptAnswer: GONE,
          exit: { code: 1, },
          millisecondsOfRun: 5_000,
          handedOver: 1,
        },);
        expect(endReason,).toBe('pi-exited',);
        const waits = startThatEndedWithoutARun?.waits ?? [];
        expect(waits[0]?.wait,).toContain('which ended when Pi went',);
        expect(waits[0]?.bound,).toBeUndefined();
        expect(startThatEndedWithoutARun?.what,).toContain('Pi exited with status 1',);
      },
    },),
    it({
      name: 'says nothing for a start whose prompt Pi took, which is a run that went on',
      fn: async () => {
        // A run started by hand hands Pi no values, so the question of the masking is not asked of it.
        const { endReason, startThatEndedWithoutARun, } = await firstPromptOf({
          promptAnswer: ACCEPTED,
          millisecondsOfRun: 100,
        },);
        expect(endReason,).toBe('wall-clock',);
        expect(startThatEndedWithoutARun,).toBeUndefined();
      },
    },),
  ],
},);
