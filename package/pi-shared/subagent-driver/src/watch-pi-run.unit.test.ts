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
  type CommandRequest,
  type ConnectionToPi,
  COMMAND_ANSWER_MILLISECONDS,
  type DriverOptions,
  isRecord,
  MODEL_ERROR_CONCURRENCY_WAIT_SECONDS,
  type PiRecord,
  POLL_MILLISECONDS,
  type RecordOfPiRun,
  startRecordOfPiRun,
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

 @param concurrencyWaitSeconds - seconds the run waits for a request slot

 @param idleSeconds - seconds of silence after which a settled run ends

 @returns options of this run
 */
function optionsOf({
  runFolder,
  milliseconds = RUN_MILLISECONDS,
  retryCap = 0,
  retryBaseSeconds = 1,
  concurrencyWaitSeconds = MODEL_ERROR_CONCURRENCY_WAIT_SECONDS,
  idleSeconds = IDLE_SECONDS,
}: {
  readonly runFolder: string;
  readonly milliseconds?: number;
  readonly retryCap?: number;
  readonly retryBaseSeconds?: number;
  readonly concurrencyWaitSeconds?: number;
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
    concurrencyWaitSeconds,
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
 Starts a run whose Pi settled on a failure another attempt may fix, and then is at work again, which is what a message it took
 from the inbox starts.

 @param atWork - whether Pi is at work on a run that did not produce an assistant message

 @returns the run, and the folder that goes with it
 */
async function startRunThatFailed({ atWork }: { readonly atWork: boolean },): Promise<RunUnderTest> {
  const run = await startRunAtWork();
  observeFailedAttempt(run.record, FAILURE_TEXT,);
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

 @returns the connection a watch reads
 */
function answeringConnection({
  sent = [],
  disposition,
}: {
  readonly sent?: string[];
  readonly disposition?: string
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
  ],
},);
