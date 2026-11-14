/**
 The watch of a run, after its prompt was sent, against a connection and a feed a test owns. The bounds that keep a stalled
 delivery from holding a run are judged here, where no process starts and no start-up can spend the run's own clock, so a busy
 host cannot decide the outcome.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  answerModelError,
  type CommandRequest,
  type ConnectionToPi,
  COMMAND_ANSWER_MILLISECONDS,
  type DriverOptions,
  EMPTY_ANSWER_ASK_LIMIT,
  isRecord,
  type PiRecord,
  POLL_MILLISECONDS,
  type RecordOfPiRun,
  type RetryWaiting,
  startRecordOfPiRun,
  START_PAUSE_MILLISECONDS,
  type WatchRequest,
  watchUntilEnded,
} from '../dist/final/node/index.mjs';

/**
 Milliseconds in a second, so a limit the options state in seconds is derived from the deadline a test passes.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Milliseconds a run here may last: enough for the watch to take one look, deliver one message and take another look. The watch
 looks every `POLL_MILLISECONDS`, so no part of this is a sleep whose length a test guessed.
 */
const RUN_MILLISECONDS = 2 * POLL_MILLISECONDS;

/**
 Milliseconds a run here may last when the watch must take a further look: one look to see the failure, one to answer it, and
 one to look again. Derived from the poll so no part of it is a sleep a test guessed.
 */
const HOLD_MILLISECONDS = 3 * POLL_MILLISECONDS;

/**
 Seconds of silence after which a settled run would end. Every run here is one Pi is at work on, so this never decides anything;
 the wall clock, which each test sets, is what ends them.
 */
const IDLE_SECONDS = 0;

/**
 Text of the message a test drops in the inbox.
 */
const MESSAGE_TEXT = 'a message nothing answers';

/**
 Name the message is published under.
 */
const MESSAGE_NAME = 'message.txt';

/**
 The failure a test's attempt ended on: the timeout one real run recorded, which another attempt may fix, so the watch has a
 retry to schedule for it.
 */
const FAILURE_TEXT = 'Request timed out.';

/**
 The failure no attempt can fix, shaped as three real runs' feeds recorded it on 2026-11-09: the provider is not configured, so
 another attempt reaches the same answer.
 */
const PERMANENT_FAILURE_TEXT = 'Provider is not configured: neuralwatt';

/**
 The 429 a real run met on 2026-11-11, in the shape gateways send it: a rate limit Pi's own automatic retry answers, which the
 watch has nothing of its own to do about.
 */
const RATE_LIMIT_FAILURE_TEXT = '429 {"error":{"message":"Too many requests"}}';

/**
 How many attempts may follow a model error, for a run whose retries are spent before the error even settles.
 */
const NO_RETRIES = 0;

/**
 Seconds before the first attempt, longer than one look and shorter than two, so a reply can land between the look that
 schedules the attempt and the look that would send it.
 */
const WAIT_LONGER_THAN_ONE_LOOK_SECONDS = ((POLL_MILLISECONDS * 3) / 2) / MILLISECONDS_PER_SECOND;

/**
 Seconds before the first attempt at a failure, for a test that wants the wait over on the very next look.
 */
const NO_WAIT_SECONDS = 0;

/**
 Seconds before the first attempt at a failure, chosen to outlast every run here, so no attempt is ever reached.
 */
const WAIT_PAST_THE_RUN_SECONDS = 600;

/**
 Seconds of silence after which a run that rests with nothing answering its prompt ends: shorter than one look, so the look
 after the one that sent the prompt is the one that sees the silence.
 */
const RESTING_IDLE_SECONDS = (POLL_MILLISECONDS / 2) / MILLISECONDS_PER_SECOND;

type DisposableFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 One run's record, its feed, and the folder they live in.
 */
type RunUnderTest = {
  readonly runFolder: string;
  readonly inbox: string;
  readonly record: RecordOfPiRun;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

async function makeFolder(): Promise<DisposableFolder> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-watch-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The options of a run, which the watch reads for its turn cap, its retry bounds and its idle limit; the wall clock a test
 passes is the deadline the watch keeps, and these carry the same number so a reader cannot find two answers to one question.

 @param runFolder - folder of the run

 @param milliseconds - how long the run may last, in milliseconds

 @param retryCap - how many attempts may follow a model error

 @param retryBaseSeconds - seconds before the first attempt at a model error

 @param idleSeconds - seconds of silence after which a settled run ends

 @returns options of this run
 */
function optionsOf({
  runFolder,
  milliseconds = RUN_MILLISECONDS,
  retryCap = 0,
  retryBaseSeconds = 1,
  idleSeconds = IDLE_SECONDS,
}: {
  readonly runFolder: string;
  readonly milliseconds?: number;
  readonly retryCap?: number;
  readonly retryBaseSeconds?: number;
  readonly idleSeconds?: number
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
    turnCap: Number.MAX_SAFE_INTEGER,
    retryCap,
    retryBaseSeconds,
    startPauseSeconds: START_PAUSE_MILLISECONDS / MILLISECONDS_PER_SECOND,
  };
}

/**
 Starts a run's record, and a folder to hold its feed and its inbox.

 @returns the run, and the folder that goes with it
 */
async function startRun(): Promise<RunUnderTest> {
  const folder = await makeFolder();
  const runFolder = folder.path;
  const inbox = join(runFolder, 'inbox',);
  await mkdir(inbox,);
  const record = startRecordOfPiRun({ feedFile: join(runFolder, 'feed.jsonl',), },);
  return {
    runFolder,
    inbox,
    record,
    async [Symbol.asyncDispose]() {
      await record.close();
      await folder[Symbol.asyncDispose]();
    },
  };
}

/**
 Starts a run whose Pi is at work on the prompt the driver sent.

 @returns the run, and the folder that goes with it
 */
async function startRunAtWork(): Promise<RunUnderTest> {
  const run = await startRun();
  run.record.observe({
    event: { type: 'agent_start', },
    declineDialog: refuseDialog,
  },);
  return run;
}

/**
 Starts a run whose Pi is at work, with a message waiting in the inbox.

 @returns the run, and the folder that goes with it
 */
async function startRunWithAMessage(): Promise<RunUnderTest> {
  const run = await startRunAtWork();
  await writeFile(join(run.inbox, MESSAGE_NAME,), MESSAGE_TEXT,);
  return run;
}

/**
 Starts a run whose Pi ended its turn with nothing open, which is the state a run may end from.

 @returns the run, and the folder that goes with it

 @example
 ```ts
 await using run = await startRunThatSettled();
 ```
 */
async function startRunThatSettled(): Promise<RunUnderTest> {
  const run = await startRunAtWork();
  run.record.observe({
    event: { type: 'agent_settled', },
    declineDialog: refuseDialog,
  },);
  return run;
}

/**
 Starts a run whose Pi settled on a failure another attempt may fix, and then is at work again, which is what a message it took
 from the inbox starts.

 @param atWork - whether Pi is at work on a run that did not produce an assistant message

 @param text - what the model said when the message ended in the error

 @returns the run, and the folder that goes with it
 */
async function startRunThatFailed({
  atWork,
  text = FAILURE_TEXT,
}: {
  readonly atWork: boolean;
  readonly text?: string
},): Promise<RunUnderTest> {
  const run = await startRunAtWork();
  observeFailedAttempt(run.record, text,);
  if (atWork) {
    run.record.observe({
      event: { type: 'agent_start', },
      declineDialog: refuseDialog,
    },);
  }
  return run;
}

/**
 Observes an attempt that ended in a failure Pi settled on, which is what the watch answers with another attempt.

 @param record - the run's record

 @param text - what the model said when the message ended in the error
 */
function observeFailedAttempt(record: RecordOfPiRun, text: string,): void {
  record.observe({
    event: {
      type: 'message_end',
      message: {
        role: 'assistant',
        content: [],
        stopReason: 'error',
        errorMessage: text,
      },
    },
    declineDialog: refuseDialog,
  },);
  record.observe({
    event: { type: 'agent_settled', },
    declineDialog: refuseDialog,
  },);
}

/**
 Observes the reply that recovers a failure: an assistant message that did not end in an error, and the settle after it.

 @param record - the run's record
 */
function observeSuccessfulReply(record: RecordOfPiRun,): void {
  record.observe({
    event: {
      type: 'message_end',
      message: {
        role: 'assistant',
        content: [{
          type: 'text',
          text: 'recovered',
        },],
        stopReason: 'stop',
      },
    },
    declineDialog: refuseDialog,
  },);
  record.observe({
    event: { type: 'agent_settled', },
    declineDialog: refuseDialog,
  },);
}

/**
 Declines a dialog a test's events never ask, so a fixture mistake fails the test instead of passing quietly.
 */
function refuseDialog(): void {
  throw new Error('no dialog was expected',);
}

/**
 A connection that never answers, stalling for exactly the bound a wait has, and recording that bound.

 @param boundsAsked - where the bounds the wait was given are recorded

 @returns the connection a watch reads
 */
function stallingConnection({ boundsAsked, }: { readonly boundsAsked: number[]; },): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function stall({ answerMilliseconds = COMMAND_ANSWER_MILLISECONDS, deadline, }: CommandRequest,) {
      // A wait with no deadline could last for ever, which is exactly what this file is about.
      if (!Number.isFinite(deadline,)) {
        throw new Error('the watch asked for an answer with no deadline',);
      }
      /**
       The bound this stall waits out: the silence the watch allows, or what the run has left, whichever ends first.
       */
      const bound = Math.min(answerMilliseconds, deadline - Date.now(),);
      boundsAsked.push(bound,);
      if (bound <= 0) {
        return {
          ending: 'deadline',
          problem: 'no answer before the deadline',
        };
      }
      // The timer holds this process up while it stalls, so a wait given a long bound is waited out and judged, never left to a
      // process that ran out of work and exited.
      await wait(bound,);
      return {
        ending: 'silence',
        problem: `no answer in ${String(bound,)} ms`,
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 A connection whose commands are all answered at once, which records the messages it was asked to send.

 @param sent - where the messages the watch sends are recorded

 @param disposition - what Pi says it did with each prompt, or nothing when it says nothing about it

 @param refuses - whether Pi refuses each prompt instead of taking it

 @returns the connection a watch reads
 */
function answeringConnection({
  sent = [],
  disposition,
  refuses = false,
}: {
  readonly sent?: string[];
  readonly disposition?: string;
  readonly refuses?: boolean
} = {},): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send({ command, }: CommandRequest,) {
      const { message } = command;
      sent.push((typeof message) === 'string' ? message : '',);
      if (refuses) {
        return {
          ending: 'answered',
          record: {
            success: false,
            error: 'refused',
          },
        };
      }
      return {
        ending: 'answered',
        record: {
          success: true,
          data: disposition === undefined ? {} : { disposition, },
        },
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 A connection whose every command is answered at once, and whose Pi then shows the reply it produced, which is what a run sent
 back to work it can now read does.

 @param sent - where the messages the watch sends are recorded

 @param record - the run's record, which the reply is observed on

 @returns the connection a watch reads
 */
function connectionWhosePiReplies({
  sent,
  record,
}: {
  readonly sent: string[];
  readonly record: RecordOfPiRun
},): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send({ command, }: CommandRequest,) {
      const { message } = command;
      sent.push((typeof message) === 'string' ? message : '',);
      // Pi takes the message and starts a turn on it, which is a reply and the settle after it.
      observeSuccessfulReply(record,);
      return {
        ending: 'answered',
        record: { success: true, },
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 One feed line as a record, so a test reads the feed the way the driver wrote it and nothing else.

 @param line - text of one feed line

 @returns the record it holds
 */
function recordOf(line: string,): PiRecord {
  const parsed: unknown = JSON.parse(line,);
  if (!isRecord(parsed,)) {
    throw new Error(`the feed line ${line} is not a record`,);
  }
  return parsed;
}

/**
 What the feed holds, once the run is over.

 @param runFolder - folder of the run

 @returns the feed's records, in the order they were written
 */
async function feedOf(runFolder: string,): Promise<readonly PiRecord[]> {
  return (await readFile(join(runFolder, 'feed.jsonl',), 'utf8',)).split('\n',)
    .filter((line,) => line !== '')
    .map(recordOf,);
}

/**
 The words the run here began with, which a summary carries as the agent's last when every answer after them says nothing.
 */
const FIRST_WORDS = 'the first words of the run';

/**
 Observes a first turn that said something and called a tool, which is where the words a later summary carries come from.

 @param record - the run's record
 */
function observeFirstTurnWithWords(record: RecordOfPiRun,): void {
  record.observe({
    event: { type: 'turn_start', },
    declineDialog: refuseDialog,
  },);
  record.observe({
    event: {
      type: 'message_end',
      message: {
        role: 'assistant',
        content: [{ type: 'text', text: FIRST_WORDS, }, { type: 'toolCall', },],
        stopReason: 'toolUse',
      },
    },
    declineDialog: refuseDialog,
  },);
}

/**
 Observes an assistant message that answered with nothing, and the settle after it.

 @param record - the run's record
 */
function observeEmptyAnswer(record: RecordOfPiRun,): void {
  record.observe({
    event: { type: 'turn_start', },
    declineDialog: refuseDialog,
  },);
  record.observe({
    event: {
      type: 'message_end',
      message: {
        role: 'assistant',
        content: [{ type: 'thinking', thinking: 'hm', },],
        stopReason: 'stop',
      },
    },
    declineDialog: refuseDialog,
  },);
  record.observe({
    event: { type: 'agent_settled', },
    declineDialog: refuseDialog,
  },);
}

/**
 Starts a run whose Pi worked a turn with words and then ended another turn with an answer that said nothing.

 @returns the run, and the folder that goes with it
 */
async function startRunThatAnsweredNothing(): Promise<RunUnderTest> {
  const run = await startRunAtWork();
  observeFirstTurnWithWords(run.record,);
  observeEmptyAnswer(run.record,);
  return run;
}

/**
 Starts a run whose Pi answered a turn with words, and then rested with nothing of its own running.

 @returns the run, and the folder that goes with it
 */
async function startRunWhoseLastAnswerHadWords(): Promise<RunUnderTest> {
  const run = await startRunAtWork();
  observeFirstTurnWithWords(run.record,);
  run.record.observe({
    event: { type: 'agent_settled', },
    declineDialog: refuseDialog,
  },);
  return run;
}

/**
 A connection whose every message is taken at once, and whose Pi then answers with nothing again.

 @param sent - where the messages the watch sends are recorded

 @param record - the run's record, which the answer is observed on

 @returns the connection a watch reads
 */
function connectionWhosePiAnswersNothing({
  sent,
  record,
}: {
  readonly sent: string[];
  readonly record: RecordOfPiRun
},): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send({ command, }: CommandRequest,) {
      const { message, } = command;
      sent.push((typeof message) === 'string' ? message : '',);
      observeEmptyAnswer(record,);
      return {
        ending: 'answered',
        record: { success: true, },
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 A connection whose every message is refused while the run's record says Pi is at work, as Pi refuses a prompt that reaches it
 mid-turn, and taken while Pi rests.

 @param sent - where the messages the watch sends are recorded

 @param record - the run's record, which tells whether Pi is at work

 @param beforeAnswer - what happens as the message arrives, before Pi answers it

 @returns the connection a watch reads
 */
function connectionThatRefusesWhilePiWorks({
  sent,
  record,
  beforeAnswer,
}: {
  readonly sent: string[];
  readonly record: RecordOfPiRun;
  readonly beforeAnswer?: () => void
},): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send({ command, }: CommandRequest,) {
      const { message, } = command;
      sent.push((typeof message) === 'string' ? message : '',);
      beforeAnswer?.();
      if (record.isRunning()) {
        return {
          ending: 'answered',
          record: {
            success: false,
            error: 'agent is already running',
          },
        };
      }
      return {
        ending: 'answered',
        record: { success: true, },
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 A connection whose first message arrives while Pi begins a turn, so Pi refuses it in the words Pi uses for a Pi at work, and whose
 later messages are taken. The turn that began settles before the refusal is read, so the refusal words are what tell the driver
 that Pi was at work, not the state left behind.

 @param sent - where the messages the watch sends are recorded

 @param record - the run's record, which the scripted Pi writes its events to

 @returns the connection a watch reads
 */
function connectionWhoseFirstMessageMeetsATurn({
  sent,
  record,
}: {
  readonly sent: string[];
  readonly record: RecordOfPiRun
},): ConnectionToPi {
  /**
   Whether a message has gone yet: the first one meets the turn, every later one is taken.
   */
  let sentAny = false;
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send({ command, }: CommandRequest,) {
      const { message, } = command;
      sent.push((typeof message) === 'string' ? message : '',);
      if (!sentAny) {
        sentAny = true;
        // Pi begins a turn as the message arrives, so the message meets a Pi at work and is refused. The turn ends on a timer, so
        // the refusal is read while Pi works and the run rests on a later look.
        record.observe({
          event: { type: 'agent_start', },
          declineDialog: refuseDialog,
        },);
        queueMicrotask(function endTheTurn() {
          record.observe({
            event: { type: 'turn_start', },
            declineDialog: refuseDialog,
          },);
          record.observe({
            event: {
              type: 'message_end',
              message: {
                role: 'assistant',
                content: [{
                  type: 'text',
                  text: 'working on the steer',
                },],
                stopReason: 'stop',
              },
            },
            declineDialog: refuseDialog,
          },);
          record.observe({
            event: { type: 'agent_settled', },
            declineDialog: refuseDialog,
          },);
        },);
        return {
          ending: 'answered',
          record: {
            success: false,
            error: 'agent is already running',
          },
        };
      }
      // Taken: the turn it starts answers with words and settles.
      observeSuccessfulReply(record,);
      return {
        ending: 'answered',
        record: { success: true, },
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**

 @param ending - how the wait ends

 @param sent - where the messages the watch sends are recorded

 @returns the connection a watch reads
 */
function connectionWhoseWaitEnds({
  ending,
  sent,
}: {
  readonly ending: 'deadline' | 'pi-exited';
  readonly sent: string[]
},): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function send({ command, }: CommandRequest,) {
      const { message, } = command;
      sent.push((typeof message) === 'string' ? message : '',);
      return {
        ending,
        problem: (ending === 'deadline') ? 'no answer before the deadline' : 'pi exited',
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

await describe({
  name: watchUntilEnded.name,
  children: [
    it({
      name: 'bounds a stalled delivery by what the run has left, and still ends at the wall clock',
      fn: async () => {
        await using run = await startRunWithAMessage();
        /**
         The bounds the watch asked Pi for, in the order it asked.
         */
        const boundsAsked: number[] = [];
        /**
         When the run must be over, counted from before the watch starts so the whole run is inside it.
         */
        const deadline = Date.now() + RUN_MILLISECONDS;
        /**
         What the run had left when the watch started, which no bound the watch asks for can exceed.
         */
        const leftAtStart = deadline - Date.now();
        const endReason = await watchUntilEnded({
          connection: stallingConnection({ boundsAsked, },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline,
        },);
        expect(endReason,).toBe('wall-clock',);
        // One message, asked for within the run's own limit: a delivery that waited for its answer by an ordinary command
        // bound would outlast the run it is part of.
        expect(boundsAsked,).toHaveLength(1,);
        expect(boundsAsked[0],).toBeLessThanOrEqual(leftAtStart,);
        /**
         What the feed says about the delivery.
         */
        const delivered = (await feedOf(run.runFolder,)).find((entry,) => entry.kind === 'inbox',);
        expect(delivered?.success,).toBe(false,);
        expect(String(delivered?.error,),).toContain('no answer',);
        // The keep-it-aside path is what makes a stalled message neither lost nor sent again.
        expect(await readdir(run.inbox,),).toEqual([`.failed-${MESSAGE_NAME}`,],);
        expect(await readFile(join(run.inbox, `.failed-${MESSAGE_NAME}`,), 'utf8',),).toBe(MESSAGE_TEXT,);
      },
    },),
    it({
      name: 'takes what Pi answered off the inbox and goes on watching',
      fn: async () => {
        await using run = await startRunWithAMessage();
        /**
         When the run must be over.
         */
        const deadline = Date.now() + RUN_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection(),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline,
        },);
        expect(endReason,).toBe('wall-clock',);
        /**
         What the feed says about the delivery.
         */
        const delivered = (await feedOf(run.runFolder,)).find((entry,) => entry.kind === 'inbox',);
        expect(delivered?.success,).toBe(true,);
        expect(await readdir(run.inbox,),).toEqual([],);
      },
    },),
    it({
      name: 'holds a retry back while Pi is at work on a run the retry did not start',
      fn: async () => {
        await using run = await startRunThatFailed({ atWork: true, },);
        /**
         The messages the watch sent, which a retry sent into the run Pi is at work on would join.
         */
        const sent: string[] = [];
        /**
         When the run must be over.
         */
        const deadline = Date.now() + HOLD_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: HOLD_MILLISECONDS,
            retryCap: 3,
            retryBaseSeconds: NO_WAIT_SECONDS,
          },),
          inbox: run.inbox,
          deadline,
        },);
        expect(endReason,).toBe('wall-clock',);
        // A prompt sent to a Pi that is at work is refused or queued behind the run it interrupts, and either way the attempt
        // answers an error that run may yet recover.
        expect(sent,).toEqual([],);
      },
    },),
    it({
      name: 'keeps the failure standing when the retry Pi was sent only handles the prompt',
      fn: async () => {
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         When the run must be over: after the looks that send the prompt and see Pi resting with nothing answering it.
         */
        const deadline = Date.now() + (2 * HOLD_MILLISECONDS);
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ disposition: 'handled', },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: 2 * HOLD_MILLISECONDS,
            retryCap: 1,
            retryBaseSeconds: NO_WAIT_SECONDS,
            idleSeconds: RESTING_IDLE_SECONDS,
          },),
          inbox: run.inbox,
          deadline,
        },);
        // No run started for the prompt and no reply came, so nothing recovered the failure: the run ends on it, not idle.
        expect(endReason,).toBe('model-error',);
        const summary = run.record.summarize({
          endReason: 'model-error',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.modelError,).toBe(FAILURE_TEXT,);
        expect(summary.retries,).toBe(1,);
        expect(summary.modelErrorAttempts,).toBe(2,);
      },
    },),
    it({
      name: 'ends a run on a 429 with no attempt of its own, whatever retries the run would allow',
      fn: async () => {
        await using run = await startRunThatFailed({
          atWork: false,
          text: RATE_LIMIT_FAILURE_TEXT,
        },);
        /**
         The messages the watch sent, which a 429 leaves to Pi's own retry.
         */
        const sent: string[] = [];
        /**
         When the run must be over: past the looks an attempt would have gone in, had the watch scheduled one.
         */
        const deadline = Date.now() + HOLD_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: HOLD_MILLISECONDS,
            retryCap: 3,
            retryBaseSeconds: NO_WAIT_SECONDS,
          },),
          inbox: run.inbox,
          deadline,
        },);
        expect(endReason,).toBe('model-error',);
        expect(sent,).toEqual([],);
        expect(run.record.retriesTaken(),).toBe(0,);
        /**
         What the run's feed holds.
         */
        const feed = await feedOf(run.runFolder,);
        expect(feed.some((entry,) => entry.kind === 'retry'),).toBe(false,);
        // The hand-off is written plainly, so a person reading the run sees why the driver made no attempt at it.
        expect(String(feed.find((entry,) => entry.kind === 'problem')?.problem,),).toContain('rate limit',);
        const summary = run.record.summarize({
          endReason: 'model-error',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.modelError,).toBe(RATE_LIMIT_FAILURE_TEXT,);
        expect(summary.retries,).toBe(0,);
        expect(summary.rateLimitErrors,).toBe(1,);
      },
    },),
    it({
      name: 'a wait the run ends before leaves the failure standing and counts no retry',
      fn: async () => {
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         The messages the watch sent, which a wait cut off before its end never puts one in.
         */
        const sent: string[] = [];
        /**
         When the run must be over, before the wait it scheduled is.
         */
        const deadline = Date.now() + HOLD_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: HOLD_MILLISECONDS,
            retryCap: 3,
            retryBaseSeconds: WAIT_PAST_THE_RUN_SECONDS,
          },),
          inbox: run.inbox,
          deadline,
        },);
        expect(endReason,).toBe('wall-clock',);
        expect(sent,).toEqual([],);
        // A retry the run never reached is not an attempt it made.
        expect(run.record.retriesTaken(),).toBe(0,);
        expect((await feedOf(run.runFolder,)).some((entry,) => entry.kind === 'retry'),).toBe(false,);
        const summary = run.record.summarize({
          endReason: 'wall-clock',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.retries,).toBe(0,);
        expect(summary.modelError,).toBe(FAILURE_TEXT,);
        expect(summary.modelErrorAttempts,).toBe(1,);
      },
    },),
    it({
      name: 'sends no attempt that fell due once the run wall clock had passed, which is the moment a look can straddle',
      fn: async () => {
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         The messages the watch sent, which an attempt sent past the run's own clock would join.
         */
        const sent: string[] = [];
        /**
         Where the watch stands: one attempt owed to the failure, and due now.
         */
        const retry: RetryWaiting = {
          owed: {
            at: Date.now() - 1,
            problem: FAILURE_TEXT,
          },
        };
        /**
         The watch, whose run's wall clock has already passed: a look reads the limits and then answers the error it found, and
         a folder read that straddles the deadline is what let one go on 2026-11-12.
         */
        const watch: WatchRequest = {
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: HOLD_MILLISECONDS,
            retryCap: 3,
          },),
          inbox: run.inbox,
          deadline: Date.now() - 1,
        };
        expect(await answerModelError({
          watch,
          retry,
        },),).toBe('wall-clock',);
        expect(sent,).toEqual([],);
        // A prompt nobody sent is no attempt the run made.
        expect(run.record.retriesTaken(),).toBe(0,);
      },
    },),
    it({
      name: 'holds the ending on a failure while Pi is at work on a run that may still reply',
      fn: async () => {
        await using run = await startRunThatFailed({ atWork: true, },);
        /**
         The messages the watch sent, which no attempt at this failure is left to send.
         */
        const sent: string[] = [];
        /**
         When the run must be over.
         */
        const deadline = Date.now() + HOLD_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: HOLD_MILLISECONDS,
            retryCap: NO_RETRIES,
          },),
          inbox: run.inbox,
          deadline,
        },);
        // The retries are spent, but the run Pi is at work on may still reply, and a reply that succeeds recovers the error:
        // ending the run now would abort work whose message is already off the inbox.
        expect(endReason,).toBe('wall-clock',);
        expect(sent,).toEqual([],);
      },
    },),
    it({
      name: 'drops an owed attempt whose failure a reply recovered before it went',
      fn: async () => {
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         The messages the watch sent, which a recovered failure owes nothing to.
         */
        const sent: string[] = [];
        /**
         When the run must be over.
         */
        const deadline = Date.now() + (2 * HOLD_MILLISECONDS);
        /**
         What the watch does, which runs while the reply arrives.
         */
        const watching = watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: 2 * HOLD_MILLISECONDS,
            retryCap: 3,
            retryBaseSeconds: WAIT_LONGER_THAN_ONE_LOOK_SECONDS,
          },),
          inbox: run.inbox,
          deadline,
        },);
        // The attempt is scheduled on the first look and would go on the next one, so the reply lands in between.
        await wait(POLL_MILLISECONDS + (POLL_MILLISECONDS / 2),);
        observeSuccessfulReply(run.record,);
        const endReason = await watching;
        expect(endReason,).toBe('idle',);
        expect(sent,).toEqual([],);
        expect(run.record.retriesTaken(),).toBe(0,);
      },
    },),
    it({
      name: 'drops an owed attempt whose failure another one replaced, and reads the new one afresh',
      fn: async () => {
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         The messages the watch sent, which the failure that replaced the first one earns none of.
         */
        const sent: string[] = [];
        /**
         When the run must be over.
         */
        const deadline = Date.now() + (2 * HOLD_MILLISECONDS);
        /**
         What the watch does, which runs while the second failure arrives.
         */
        const watching = watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: 2 * HOLD_MILLISECONDS,
            retryCap: 3,
            retryBaseSeconds: WAIT_LONGER_THAN_ONE_LOOK_SECONDS,
          },),
          inbox: run.inbox,
          deadline,
        },);
        await wait(POLL_MILLISECONDS + (POLL_MILLISECONDS / 2),);
        observeFailedAttempt(run.record, PERMANENT_FAILURE_TEXT,);
        const endReason = await watching;
        // What stands now is a failure no attempt can fix, whatever the failure the owed attempt was scheduled for.
        expect(endReason,).toBe('model-error',);
        expect(sent,).toEqual([],);
        expect(run.record.retriesTaken(),).toBe(0,);
      },
    },),
    it({
      name: 'keeps a run whose own work still runs, then sends the agent back when that work ends',
      fn: async () => {
        await using run = await startRunThatSettled();
        /**
         The messages the watch sent, which the one that sends the agent back to its work joins.
         */
        const sent: string[] = [];
        /**
         How many times the work was asked about, which is how the test ends it after two looks.
         */
        let looks = 0;
        /**
         The line that names the work the run left running.
         */
        const work = ['proc_7adb (native-build-2) 486667 systemd-run --user --scope'];
        const endReason = await watchUntilEnded({
          connection: connectionWhosePiReplies({ sent, record: run.record, },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 100),
          workLeftRunning: async function workLeftRunning() {
            looks += 1;
            return (looks <= 2) ? work : [];
          },
        },);
        // The run was kept for as long as its work ran, sent a message once it ended, and ended after the reply to it.
        expect(endReason,).toBe('idle',);
        expect(sent.length,).toBe(1,);
        expect(sent[0],).toContain('proc_7adb (native-build-2) 486667 systemd-run --user --scope',);
        /**
         What the run's feed holds.
         */
        const feed = await feedOf(run.runFolder,);
        expect(feed.filter((entry,) => entry.kind === 'waiting').length,).toBe(1,);
        expect(String(feed.find((entry,) => entry.kind === 'waiting')?.problem,),).toContain('native-build-2',);
        expect(feed.some((entry,) => (entry.kind === 'wake') && (entry.success === true)),).toBe(true,);
      },
    },),
    it({
      name: 'ends on its wall clock while its work still runs, and its feed says what it waited for',
      fn: async () => {
        await using run = await startRunThatSettled();
        /**
         The messages the watch sent, which a kept run must not send while it waits.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + HOLD_MILLISECONDS,
          workLeftRunning: async function workLeftRunning() {
            return ['proc_7adb (native-build-2) 486667 systemd-run --user --scope',];
          },
        },);
        expect(endReason,).toBe('wall-clock',);
        expect(sent,).toEqual([],);
        /**
         What the run's feed holds.
         */
        const feed = await feedOf(run.runFolder,);
        expect(feed.some((entry,) => entry.kind === 'waiting'),).toBe(true,);
        // A run the wall clock ended while it waited for work did not end with nothing to do, and the feed says so.
        expect(String(feed.at(-1)?.problem,),).toContain('waited for the work it left running',);
      },
    },),
    it({
      name: 'does not end a run as idle before the agent answers the message that woke it, and ends it on the wall clock'
        + ' instead',
      fn: async () => {
        await using run = await startRunThatSettled();
        /**
         The messages the watch sent, which is the one that wakes the agent.
         */
        const sent: string[] = [];
        /**
         How many times the work was asked about, which is how the test ends it after one look.
         */
        let looks = 0;
        const endReason = await watchUntilEnded({
          // Pi takes the message and says nothing more, which is the state a driver that ended a woken run as idle left it
          // in: the agent's turn never opened, so the report it would have written was never read.
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: HOLD_MILLISECONDS * 2,
          },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 2),
          workLeftRunning: async function workLeftRunning() {
            looks += 1;
            return (looks <= 1) ? ['proc_7adb (native-build-2) 486667 systemd-run --user --scope'] : [];
          },
        },);
        // The message went, and the run stayed up after it rather than ending as one that had nothing left to do.
        expect(sent.length,).toBe(1,);
        expect(endReason,).toBe('wall-clock',);
        /**
         What the run's feed holds.
         */
        const feed = await feedOf(run.runFolder,);
        expect(feed.some((entry,) => (entry.kind === 'wake') && (entry.success === true)),).toBe(true,);
      },
    },),
    it({
      name: 'ends a run as idle when Pi handled the message that woke it without starting a turn',
      fn: async () => {
        await using run = await startRunThatSettled();
        /**
         The messages the watch sent, which is the one that wakes the agent.
         */
        const sent: string[] = [];
        /**
         How many times the work was asked about, which is how the test ends it after one look.
         */
        let looks = 0;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({
            sent,
            disposition: 'handled',
          },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: HOLD_MILLISECONDS * 2,
          },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 2),
          workLeftRunning: async function workLeftRunning() {
            looks += 1;
            return (looks <= 1) ? ['proc_7adb (native-build-2) 486667 systemd-run --user --scope'] : [];
          },
        },);
        // Nothing was started, so nothing is owed a reply: waiting for one would keep the run up for no reason.
        expect(sent.length,).toBe(1,);
        expect(endReason,).toBe('idle',);
      },
    },),
    it({
      name: 'ends a run whose wake message Pi refused as one Pi refused a prompt, not as one that finished',
      fn: async () => {
        await using run = await startRunThatSettled();
        /**
         The messages the watch sent, which is the one that wakes the agent.
         */
        const sent: string[] = [];
        /**
         How many times the work was asked about, which is how the test ends it after one look.
         */
        let looks = 0;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({
            sent,
            refuses: true,
          },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: HOLD_MILLISECONDS * 2,
          },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 2),
          workLeftRunning: async function workLeftRunning() {
            looks += 1;
            return (looks <= 1) ? ['proc_7adb (native-build-2) 486667 systemd-run --user --scope'] : [];
          },
        },);
        // The agent was never told its work ended, so the run is not one that came back with nothing to say.
        expect(sent.length,).toBe(1,);
        expect(endReason,).toBe('prompt-rejected',);
        /**
         What the run's feed holds.
         */
        const feed = await feedOf(run.runFolder,);
        expect(feed.find((entry,) => entry.kind === 'wake')?.success,).toBe(false,);
      },
    },),
    it({
      name: 'ends a run with nothing running as it always did, sending nothing',
      fn: async () => {
        await using run = await startRunThatSettled();
        /**
         The messages the watch sent, which nothing running must not produce.
         */
        const sent: string[] = [];
        /**
         The deadlines the work was asked under, which must be none at all when there is nothing to wait for.
         */
        let looks = 0;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + HOLD_MILLISECONDS,
          workLeftRunning: async function workLeftRunning() {
            looks += 1;
            return [];
          },
        },);
        expect(endReason,).toBe('idle',);
        expect(sent,).toEqual([],);
        expect(looks,).toBeGreaterThan(0,);
        /**
         What the run's feed holds, which must name no wait and no wake.
         */
        const feed = await feedOf(run.runFolder,);
        expect(feed.some((entry,) => entry.kind === 'waiting'),).toBe(false,);
        expect(feed.some((entry,) => entry.kind === 'wake'),).toBe(false,);
      },
    },),
    it({
      name: 'asks an agent whose last answer said nothing to go on, and ends idle once the reply carries words',
      fn: async () => {
        await using run = await startRunThatAnsweredNothing();
        /**
         The messages the watch sent, which is the one ask.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: connectionWhosePiReplies({ sent, record: run.record, },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 100),
        },);
        expect(endReason,).toBe('idle',);
        expect(sent.length,).toBe(1,);
        expect(sent[0],).toContain('Your last answer was empty',);
        /**
         What the summary says about the ending, and what the feed says of the ask.
         */
        const summary = run.record.summarize({
          endReason: 'idle',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        // The reply carried the agent's words, so nothing is left of the empty answer.
        expect(summary.emptyAnswer,).toBeUndefined();
        expect(summary.lastAssistantText,).toBe('recovered',);
        const feed = await feedOf(run.runFolder,);
        expect(feed.filter((entry,) => entry.kind === 'asked-to-go-on',).map((entry,) => entry.attempt,),).toEqual([1,],);
      },
    },),
    it({
      name: 'ends a run as empty-answer once the asks are used up, and the summary names how many went',
      fn: async () => {
        await using run = await startRunThatAnsweredNothing();
        /**
         The messages the watch sent, which is one ask for each time it tried.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: connectionWhosePiAnswersNothing({ sent, record: run.record, },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 100),
        },);
        expect(endReason,).toBe('empty-answer',);
        expect(sent.length,).toBe(EMPTY_ANSWER_ASK_LIMIT,);
        /**
         What the summary says about the ending.
         */
        const summary = run.record.summarize({
          endReason: 'empty-answer',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.emptyAnswer,).toEqual({ asked: EMPTY_ANSWER_ASK_LIMIT, },);
        // The summary names the turn whose words it carries, so an earlier turn's sentence is never read as the agent's last.
        expect(summary.lastAssistantText,).toBe(FIRST_WORDS,);
        expect(summary.lastAssistantTextTurn,).toBe(1,);
        expect(summary.driverError,).toBeUndefined();
      },
    },),
    it({
      name: 'a refused ask ends the run on the empty answer, with Pi\'s refusal on the feed',
      fn: async () => {
        await using run = await startRunThatAnsweredNothing();
        /**
         The messages the watch sent, which is the one ask Pi refused.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, refuses: true, },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 100),
        },);
        expect(endReason,).toBe('empty-answer',);
        expect(sent.length,).toBe(1,);
        /**
         What the feed says of the refusal, and what the summary counts.
         */
        const feed = await feedOf(run.runFolder,);
        const asked = feed.filter((entry,) => entry.kind === 'asked-to-go-on',);
        expect(asked.length,).toBe(1,);
        expect(asked[0]?.success,).toBe(false,);
        expect(asked[0]?.error,).toBe('refused',);
        const summary = run.record.summarize({
          endReason: 'empty-answer',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.emptyAnswer,).toEqual({ asked: 1, },);
      },
    },),
    it({
      name: 'a turn that begins while the work is being read is not asked to go on, and the run goes on',
      fn: async () => {
        await using run = await startRunThatAnsweredNothing();
        /**
         The messages the watch sent, which must not be one the turn interrupted.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: connectionThatRefusesWhilePiWorks({
            sent,
            record: run.record,
          },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + HOLD_MILLISECONDS,
          workLeftRunning: async function workLeftRunning() {
            // Pi begins a turn while the work is being read, as a steer delivered earlier fires after the settle that opened
            // the window. The scan found no process, which is what made the old watch act on a Pi that had stopped resting.
            run.record.observe({
              event: { type: 'agent_start', },
              declineDialog: refuseDialog,
            },);
            run.record.observe({
              event: { type: 'turn_start', },
              declineDialog: refuseDialog,
            },);
            return [];
          },
        },);
        // The run was not ended on a refusal the working agent earned, and its turn was left alone.
        expect(endReason,).toBe('wall-clock',);
        expect(sent,).toEqual([],);
        /**
         What the summary says about the ending, and what the feed says of any ask.
         */
        const summary = run.record.summarize({
          endReason: 'wall-clock',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.emptyAnswer,).toEqual({ asked: 0, },);
        const feed = await feedOf(run.runFolder,);
        expect(feed.some((entry,) => entry.kind === 'asked-to-go-on'),).toBe(false,);
      },
    },),
    it({
      name: 'a turn that begins while the work is being read is not ended as idle, and the run goes on',
      fn: async () => {
        await using run = await startRunWhoseLastAnswerHadWords();
        /**
         The messages the watch sent, which this ending sends none of.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + HOLD_MILLISECONDS,
          workLeftRunning: async function workLeftRunning() {
            run.record.observe({
              event: { type: 'agent_start', },
              declineDialog: refuseDialog,
            },);
            run.record.observe({
              event: { type: 'turn_start', },
              declineDialog: refuseDialog,
            },);
            return [];
          },
        },);
        // Idle is the ending of a run whose agent rested; a turn that opened during the read is no rest.
        expect(endReason,).toBe('wall-clock',);
        expect(sent,).toEqual([],);
      },
    },),
    it({
      name: 'a refusal that meets a Pi at work is no ask, so the run does not end on the empty answer',
      fn: async () => {
        await using run = await startRunThatAnsweredNothing();
        /**
         The messages the watch sent, which is the one the turn interrupted as it went.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: connectionThatRefusesWhilePiWorks({
            sent,
            record: run.record,
            beforeAnswer: function piBeginsATurn() {
              // Pi began a turn as the message arrived, which is the one thing the driver cannot see before it sends.
              run.record.observe({
                event: { type: 'agent_start', },
                declineDialog: refuseDialog,
              },);
              run.record.observe({
                event: { type: 'turn_start', },
                declineDialog: refuseDialog,
              },);
            },
          },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + HOLD_MILLISECONDS,
          workLeftRunning: async () => [],
        },);
        expect(endReason,).toBe('wall-clock',);
        expect(sent.length,).toBe(1,);
        /**
         What the summary counts, which is no ask, and what the feed says of the refusal.
         */
        const summary = run.record.summarize({
          endReason: 'wall-clock',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.emptyAnswer,).toEqual({ asked: 0, },);
        const feed = await feedOf(run.runFolder,);
        expect(feed.some((entry,) => entry.kind === 'asked-to-go-on'),).toBe(false,);
        // The feed says why the message was no ask, so a reader of the run sees the race it lost.
        expect(feed.some((entry,) => String(entry.problem,).includes('had begun a turn')),).toBe(true,);
      },
    },),
    it({
      name: 'an ask the run ends before leaves the empty answer standing, under the ending that ended the wait',
      fn: async () => {
        /**
         Ends one run under the ending a wait can meet, and reads what the summary counted.
         */
        async function expectTheAskEndedUnder({
          ending,
          expected,
        }: {
          readonly ending: 'deadline' | 'pi-exited';
          readonly expected: 'wall-clock' | 'pi-exited'
        },): Promise<void> {
          await using run = await startRunThatAnsweredNothing();
          /**
           The messages the watch sent, which is the ask the wait ended under.
           */
          const sent: string[] = [];
          const endReason = await watchUntilEnded({
            connection: connectionWhoseWaitEnds({
              ending,
              sent,
            },),
            record: run.record,
            options: optionsOf({ runFolder: run.runFolder, },),
            inbox: run.inbox,
            deadline: Date.now() + (HOLD_MILLISECONDS * 100),
          },);
          expect(endReason,).toBe(expected,);
          expect(sent.length,).toBe(1,);
          /**
           What the summary says about the empty answer the run ended on.
           */
          const summary = run.record.summarize({
            endReason: expected,
            piExit: {},
            piHasExited: true,
            sessionStats: null,
          },);
          expect(summary.emptyAnswer,).toEqual({ asked: 1, },);
        }

        await expectTheAskEndedUnder({
          ending: 'deadline',
          expected: 'wall-clock',
        },);
        await expectTheAskEndedUnder({
          ending: 'pi-exited',
          expected: 'pi-exited',
        },);
      },
    },),
    it({
      name: 'a model error that lands while the work is being read is answered by an attempt, not ended',
      fn: async () => {
        await using run = await startRunWhoseLastAnswerHadWords();
        /**
         The messages the watch sent, which must include the attempt at the error.
         */
        const sent: string[] = [];
        /**
         How many times the work was read, so the error lands inside the first read.
         */
        let reads = 0;
        const endReason = await watchUntilEnded({
          connection: connectionWhosePiReplies({
            sent,
            record: run.record,
          },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            retryCap: 3,
            retryBaseSeconds: 0,
          },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 100),
          workLeftRunning: async function readWork() {
            reads += 1;
            if (reads === 1) {
              // A turn Pi began during the read settles on a model error another attempt may fix.
              run.record.observe({
                event: { type: 'agent_start', },
                declineDialog: refuseDialog,
              },);
              run.record.observe({
                event: {
                  type: 'message_end',
                  message: {
                    role: 'assistant',
                    content: [],
                    stopReason: 'error',
                    errorMessage: '503 service unavailable: the provider is overloaded',
                  },
                },
                declineDialog: refuseDialog,
              },);
              run.record.observe({
                event: { type: 'agent_settled', },
                declineDialog: refuseDialog,
              },);
            }
            return [];
          },
        },);
        // The error is answered by an attempt rather than ending a run that was resting before the read.
        expect(endReason,).toBe('idle',);
        expect(run.record.retriesTaken(),).toBe(1,);
        expect(run.record.unrecoveredModelError().failed,).toBe(false,);
        expect(sent.length,).toBe(1,);
      },
    },),
    it({
      name: 'a wake refused because Pi began a turn is held, and goes again once Pi rests',
      fn: async () => {
        await using run = await startRunThatSettled();
        /**
         The messages the watch sent: the wake Pi refused at work, then the wake it took at rest.
         */
        const sent: string[] = [];
        /**
         How many times the work was read: the build runs, then its end meets a turn, then it is gone.
         */
        let reads = 0;
        const endReason = await watchUntilEnded({
          connection: connectionWhoseFirstMessageMeetsATurn({
            sent,
            record: run.record,
          },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 100),
          workLeftRunning: async function readWork() {
            reads += 1;
            return (reads === 1) ? ['proc_1 (native-build) 486667 mise run build'] : [];
          },
        },);
        // The wake is no ending: it goes again once the turn that refused it ends.
        expect(endReason,).toBe('idle',);
        expect(run.record.isRunning(),).toBe(false,);
        expect(sent.length,).toBe(2,);
        const feed = await feedOf(run.runFolder,);
        const wakes = feed.filter((entry,) => entry.kind === 'wake',);
        expect(wakes.at(-1)?.success,).toBe(true,);
      },
    },),
    it({
      name: 'a message from the inbox that meets a turn begun is delivered once Pi rests, and never shelved',
      fn: async () => {
        await using run = await startRunThatSettled();
        await writeFile(join(run.inbox, MESSAGE_NAME,), MESSAGE_TEXT,);
        /**
         The messages the watch sent: the message Pi refused at work, then the same message it took at rest.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: connectionWhoseFirstMessageMeetsATurn({
            sent,
            record: run.record,
          },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + (HOLD_MILLISECONDS * 100),
          workLeftRunning: async () => [],
        },);
        expect(endReason,).toBe('idle',);
        // The message went twice: the refusal is no answer about it, and the second try is the one Pi took.
        expect(sent.filter((message,) => message === MESSAGE_TEXT).length,).toBe(2,);
        expect((await readdir(run.inbox,)).some((name,) => name.startsWith('.failed-',)),).toBe(false,);
        /**
         What the summary says about the messages Pi never took, which is none.
         */
        const summary = run.record.summarize({
          endReason: 'idle',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.inboxNotTaken,).toBeUndefined();
      },
    },),
    it({
      name: 'a message Pi refuses while it rests is kept aside and named in the summary',
      fn: async () => {
        await using run = await startRunThatSettled();
        await writeFile(join(run.inbox, MESSAGE_NAME,), MESSAGE_TEXT,);
        /**
         The messages the watch sent: the one message, refused while Pi rested.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: answeringConnection({
            sent,
            refuses: true,
          },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + HOLD_MILLISECONDS,
          workLeftRunning: async () => [],
        },);
        // The message is kept aside and the run goes on as it did before: what is new is that its name is in the summary.
        expect(endReason,).toBe('idle',);
        expect((await readdir(run.inbox,)),).toContain(`.failed-${MESSAGE_NAME}`,);
        /**
         What the summary says about the message Pi never took.
         */
        const summary = run.record.summarize({
          endReason: 'idle',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.inboxNotTaken,).toEqual([MESSAGE_NAME,],);
      },
    },),
    it({
      name: 'a message still waiting when the run ends is kept aside and named in the summary',
      fn: async () => {
        await using run = await startRunThatSettled();
        await writeFile(join(run.inbox, MESSAGE_NAME,), MESSAGE_TEXT,);
        /**
         The messages the watch sent: the message, refused while Pi worked.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          connection: connectionThatRefusesWhilePiWorks({
            sent,
            record: run.record,
            beforeAnswer: function piBeginsATurn() {
              // Pi begins a turn as the message arrives and keeps working, so the run never rests again.
              run.record.observe({
                event: { type: 'agent_start', },
                declineDialog: refuseDialog,
              },);
            },
          },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: Date.now() + HOLD_MILLISECONDS,
          workLeftRunning: async () => [],
        },);
        expect(endReason,).toBe('wall-clock',);
        expect((await readdir(run.inbox,)),).toContain(`.failed-${MESSAGE_NAME}`,);
        /**
         What the summary says about the message the run ended before handing over.
         */
        const summary = run.record.summarize({
          endReason: 'wall-clock',
          piExit: {},
          piHasExited: true,
          sessionStats: null,
        },);
        expect(summary.inboxNotTaken,).toEqual([MESSAGE_NAME,],);
      },
    },),
  ],
},);
