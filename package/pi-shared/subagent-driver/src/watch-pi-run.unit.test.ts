/**
 The watch of a run, after its prompt was sent, against a connection and a feed a test owns. The bounds that keep a stalled
 delivery from holding a run are judged here, where no process starts and no start-up can spend the run's own clock, and each
 test owns the run's clock itself: a run ends when the test moves that clock, never because a look happened to cost more on a busy
 machine than a number of milliseconds the test wrote down.

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
  type TestContext,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  answerModelError,
  type CommandRequest,
  type ConnectionToPi,
  COMMAND_ANSWER_MILLISECONDS,
  type DriverOptions,
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
 Milliseconds of wall clock a run here gets when its bound is not the subject: far above every look and every bound a fixture
 waits out, so a watch that never reaches the end its test means is reported as one that hit its wall clock instead of holding the
 suite. A look was measured at about half a second under `CPUQuota=1%` on 2026-11-15, so this is a hundred looks' worth, and the
 decision a test asserts never rests on it: a test that wants the run's bound to pass moves its own clock onto it.
 */
const BOUND_ABOVE_EVERY_LOOK_MILLISECONDS = 60_000;

/**
 Milliseconds a run here may last when the bound itself is the subject: enough for the watch to take one look, deliver one
 message and take another look. The watch looks every `POLL_MILLISECONDS`, so no part of this is a sleep whose length a test
 guessed.
 */
const RUN_MILLISECONDS = 2 * POLL_MILLISECONDS;

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

  /**
   Finishes writing the feed, once, so a reader reads a file its writer has done with rather than one it still holds.
   */
  readonly closeFeed: () => Promise<void>;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A run's clock, which one test owns: the machine's own clock plus a distance the test adds, so the run's wall clock passes at the
 look the test means and never because a look cost more on a busy machine before the test got there.
 */
type RunClock = {
  readonly now: () => number;
  readonly advance: (milliseconds: number) => void;
};

/**
 The machine's clock, with a distance the test adds to it.

 A driver measures a run's wall clock against the time of the day, so a run whose bound is short is over before a busy machine has
 taken the looks its test needs: that is how the 429 and wake tests failed on 2026-11-15. Each test here gives its run a bound far
 above every look it needs, and the test that wants that bound to pass moves this clock onto it, which ends the run at the look the
 test means whatever the machine's speed. A clock that is never moved is the machine's own, so a watch that never reaches the end
 its test means is ended by the run's bound rather than holding the suite. The test's sandbox puts the machine's clock back when
 the test ends.

 @param ctx - the test's own context, whose sandbox restores the machine's clock

 @returns the clock this test moves

 @example
 ```ts
 const clock = ownTheClock(ctx,);
 clock.advance(BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,);
 ```
 */
function ownTheClock(ctx: TestContext,): RunClock {
  /**
   The machine's clock, which this test replaces and whose sandbox puts back.
   */
  const machineNow: () => number = Date.now;
  /**
   How far this test has moved the clock, which is nothing until it moves it.
   */
  const distance: { milliseconds: number } = { milliseconds: 0, };

  /**
   Where this test's clock stands.

   @returns milliseconds since the epoch, as the machine's clock reads them plus the distance this test added
   */
  function now(): number {
    return machineNow() + distance.milliseconds;
  }

  /**
   Moves this test's clock forward.

   @param milliseconds - how far, as a length of time
   */
  function advance(milliseconds: number,): void {
    distance.milliseconds += milliseconds;
  }

  ctx.sinon.stub(Date, 'now',)
    .callsFake(now,);
  return {
    now,
    advance,
  };
}

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
  milliseconds = BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
  /**
   The one close of this run's feed, so a test that reads the feed and the disposer never close it twice.
   */
  const closed: { writing?: Promise<void> } = {};

  /**
   Finishes writing the feed, once.

   @returns what ends when the feed's last line is written
   */
  function closeFeed(): Promise<void> {
    closed.writing ??= record.close();
    return closed.writing;
  }
  return {
    runFolder,
    inbox,
    record,
    closeFeed,
    async [Symbol.asyncDispose]() {
      await closeFeed();
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

 @param onLook - what the test does at the start of a look, counted from one

 @param onAnswer - what the test does once the stall is over, such as moving its clock past the run's own bound

 @returns the connection a watch reads
 */
function stallingConnection({
  boundsAsked,
  onLook,
  onAnswer,
}: {
  readonly boundsAsked: number[];
  readonly onLook?: (look: number) => void;
  readonly onAnswer?: () => void
},): ConnectionToPi {
  /**
   Which look the watch is taking, counting from one, which only the watch's own question changes.
   */
  const looks: { taken: number } = { taken: 0, };
  return {
    hasExited: function hasExited(): boolean {
      looks.taken += 1;
      onLook?.(looks.taken,);
      return false;
    },
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
      onAnswer?.();
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

 @param onLook - what the test does at the start of a look, counted from one, such as moving its clock past the run's own bound or
 letting a reply land between two looks

 @returns the connection a watch reads
 */
function answeringConnection({
  sent = [],
  disposition,
  refuses = false,
  onLook,
}: {
  readonly sent?: string[];
  readonly disposition?: string;
  readonly refuses?: boolean;
  readonly onLook?: (look: number) => void
} = {},): ConnectionToPi {
  /**
   Which look the watch is taking, counting from one, which only the watch's own question changes.
   */
  const looks: { taken: number } = { taken: 0, };
  return {
    hasExited: function hasExited(): boolean {
      looks.taken += 1;
      onLook?.(looks.taken,);
      return false;
    },
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

 @param onLook - what the test does at the start of a look, counted from one

 @returns the connection a watch reads
 */
function connectionWhosePiReplies({
  sent,
  record,
  onLook,
}: {
  readonly sent: string[];
  readonly record: RecordOfPiRun;
  readonly onLook?: (look: number) => void
},): ConnectionToPi {
  /**
   Which look the watch is taking, counting from one, which only the watch's own question changes.
   */
  const looks: { taken: number } = { taken: 0, };
  return {
    hasExited: function hasExited(): boolean {
      looks.taken += 1;
      onLook?.(looks.taken,);
      return false;
    },
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
 What the feed holds, read once the run has finished writing it.

 A reader that reads a feed its writer still holds sees the file as it stood a moment ago, whatever the machine's speed: the 429
 test of 2026-11-15 read a feed without the `problem` line the watch had just noted, and the wake test read one without its `wake`
 line. Closing the record is the event that every line the watch noted has been written, so it is waited for rather than a length
 of time.

 @param run - the run whose feed is read

 @returns the feed's records, in the order they were written
 */
async function feedOf(run: RunUnderTest,): Promise<readonly PiRecord[]> {
  await run.closeFeed();
  return (await readFile(join(run.runFolder, 'feed.jsonl',), 'utf8',)).split('\n',)
    .filter((line,) => line !== '')
    .map(recordOf,);
}

/**
 What a connection does at the start of the look a test means: moves the test's clock onto the run's own bound, which ends the run
 at that look whatever the machine's speed.

 @param clock - the clock the test owns

 @param look - which look, counting from one, the run's wall clock passes at

 @returns the hook a connection calls at the start of every look

 @example
 ```ts
 const onLook = passTheBoundAtLook({ clock, look: 2, },);
 ```
 */
function passTheBoundAtLook({ clock, look, }: {
  readonly clock: RunClock;
  readonly look: number
},): (eachLook: number) => void {
  return function onLook(eachLook,) {
    if (eachLook === look) {
      clock.advance(BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,);
    }
  };
}

await describe({
  name: watchUntilEnded.name,
  children: [
    it({
      name: 'bounds a stalled delivery by what the run has left, and still ends at the wall clock',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunWithAMessage();
        /**
         The bounds the watch asked Pi for, in the order it asked.
         */
        const boundsAsked: number[] = [];
        /**
         When the run must be over: far above every look, so the delivery is reached whatever the first look costs. The test spends
         that bound up to one look pair of its end at the first look, so the delivery is made with only that much of the run left,
         which is the subject here.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        /**
         What the run had left when the watch started, which no bound the watch asks for can exceed.
         */
        const leftAtStart = deadline - clock.now();
        const endReason = await watchUntilEnded({
          connection: stallingConnection({
            boundsAsked,
            onLook: function onLook(look,) {
              if (look === 1) {
                // One look pair of the bound is kept for the delivery, whatever the first look cost on this machine.
                clock.advance(Math.max(0, (deadline - clock.now()) - RUN_MILLISECONDS,),);
              }
            },
            // What the run had left when the stalled delivery was asked for is spent by the delivery itself, so the run ends at its
            // wall clock on the next look whatever the machine's speed.
            onAnswer: function onAnswer() {
              clock.advance(RUN_MILLISECONDS,);
            },
          },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline,
        },);
        expect(endReason,).toBe('wall-clock',);
        // One message, asked for within the run's own limit: a delivery that waited for its answer by an ordinary command
        // bound would outlast the run it is part of.
        expect(boundsAsked,).toHaveLength(1,);
        // The wait is cut to what the run had left, never to the bound an ordinary command is given: a delivery that waited by that
        // bound would outlast the run it is part of.
        expect(boundsAsked[0],).toBeLessThan(COMMAND_ANSWER_MILLISECONDS,);
        expect(boundsAsked[0],).toBeLessThanOrEqual(leftAtStart,);
        /**
         What the feed says about the delivery.
         */
        const delivered = (await feedOf(run,)).find((entry,) => entry.kind === 'inbox',);
        expect(delivered?.success,).toBe(false,);
        expect(String(delivered?.error,),).toContain('no answer',);
        // The keep-it-aside path is what makes a stalled message neither lost nor sent again.
        expect(await readdir(run.inbox,),).toEqual([`.failed-${MESSAGE_NAME}`,],);
        expect(await readFile(join(run.inbox, `.failed-${MESSAGE_NAME}`,), 'utf8',),).toBe(MESSAGE_TEXT,);
      },
    },),
    it({
      name: 'takes what Pi answered off the inbox and goes on watching',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunWithAMessage();
        /**
         When the run must be over: far above every look and every bound a fixture waits out, so it is a bound and never the thing
         that decides how this run ends.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        const endReason = await watchUntilEnded({
          // The message is taken on the first look, and the clock passes at the second, so the run ends at its wall clock once the
          // look that took the message has had its say.
          connection: answeringConnection({ onLook: passTheBoundAtLook({ clock, look: 2, },), },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline,
        },);
        expect(endReason,).toBe('wall-clock',);
        /**
         What the feed says about the delivery.
         */
        const delivered = (await feedOf(run,)).find((entry,) => entry.kind === 'inbox',);
        expect(delivered?.success,).toBe(true,);
        expect(await readdir(run.inbox,),).toEqual([],);
      },
    },),
    it({
      name: 'holds a retry back while Pi is at work on a run the retry did not start',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunThatFailed({ atWork: true, },);
        /**
         The messages the watch sent, which a retry sent into the run Pi is at work on would join.
         */
        const sent: string[] = [];
        /**
         When the run must be over: far above every look and every bound a fixture waits out, so it is a bound and never the thing
         that decides how this run ends.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, onLook: passTheBoundAtLook({ clock, look: 2, },), },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         When the run must be over: far above every look and every bound a fixture waits out, so it is a bound and never the thing
         that decides how this run ends.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ disposition: 'handled', },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunThatFailed({
          atWork: false,
          text: RATE_LIMIT_FAILURE_TEXT,
        },);
        /**
         The messages the watch sent, which a 429 leaves to Pi's own retry.
         */
        const sent: string[] = [];
        /**
         When the run must be over: far above every look and every bound a fixture waits out, so it is a bound and never the thing
         that decides how this run ends.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
        const feed = await feedOf(run,);
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
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         The messages the watch sent, which a wait cut off before its end never puts one in.
         */
        const sent: string[] = [];
        /**
         When the run must be over: far above every look and every bound a fixture waits out, so it is a bound and never the thing
         that decides how this run ends.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, onLook: passTheBoundAtLook({ clock, look: 2, },), },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
        expect((await feedOf(run,)).some((entry,) => entry.kind === 'retry'),).toBe(false,);
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
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunThatFailed({ atWork: true, },);
        /**
         The messages the watch sent, which no attempt at this failure is left to send.
         */
        const sent: string[] = [];
        /**
         When the run must be over: far above every look and every bound a fixture waits out, so it is a bound and never the thing
         that decides how this run ends.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        const endReason = await watchUntilEnded({
          connection: answeringConnection({ sent, onLook: passTheBoundAtLook({ clock, look: 2, },), },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         The messages the watch sent, which a recovered failure owes nothing to.
         */
        const sent: string[] = [];
        /**
         When the run must be over: far above every look and every bound a fixture waits out, so it is a bound and never the thing
         that decides how this run ends.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        /**
         What the watch does: its first look schedules the attempt the failure earns, and the reply lands at the start of the
         second, before that attempt would go.
         */
        const watching = watchUntilEnded({
          connection: answeringConnection({
            sent,
            // The reply lands at the start of the second look, before the watch answers the failure it scheduled an attempt
            // for: the same moment the wait would have ended, whatever the machine's speed.
            onLook: function onLook(look,) {
              if (look === 2) {
                observeSuccessfulReply(run.record,);
              }
            },
          },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
            retryCap: 3,
            retryBaseSeconds: WAIT_LONGER_THAN_ONE_LOOK_SECONDS,
          },),
          inbox: run.inbox,
          deadline,
        },);
        const endReason = await watching;
        expect(endReason,).toBe('idle',);
        expect(sent,).toEqual([],);
        expect(run.record.retriesTaken(),).toBe(0,);
      },
    },),
    it({
      name: 'drops an owed attempt whose failure another one replaced, and reads the new one afresh',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunThatFailed({ atWork: false, },);
        /**
         The messages the watch sent, which the failure that replaced the first one earns none of.
         */
        const sent: string[] = [];
        /**
         When the run must be over: far above every look and every bound a fixture waits out, so it is a bound and never the thing
         that decides how this run ends.
         */
        const deadline = clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS;
        /**
         What the watch does: its first look schedules the attempt the failure earns, and the second failure lands at the start
         of the second look, before that attempt would go.
         */
        const watching = watchUntilEnded({
          connection: answeringConnection({
            sent,
            // The second failure lands at the start of the second look, before the watch sends the attempt the first one owed.
            onLook: function onLook(look,) {
              if (look === 2) {
                observeFailedAttempt(run.record, PERMANENT_FAILURE_TEXT,);
              }
            },
          },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
            retryCap: 3,
            retryBaseSeconds: WAIT_LONGER_THAN_ONE_LOOK_SECONDS,
          },),
          inbox: run.inbox,
          deadline,
        },);
        const endReason = await watching;
        // What stands now is a failure no attempt can fix, whatever the failure the owed attempt was scheduled for.
        expect(endReason,).toBe('model-error',);
        expect(sent,).toEqual([],);
        expect(run.record.retriesTaken(),).toBe(0,);
      },
    },),
    it({
      name: 'keeps a run whose own work still runs, then sends the agent back when that work ends',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
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
          deadline: clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
        const feed = await feedOf(run,);
        expect(feed.filter((entry,) => entry.kind === 'waiting').length,).toBe(1,);
        expect(String(feed.find((entry,) => entry.kind === 'waiting')?.problem,),).toContain('native-build-2',);
        expect(feed.some((entry,) => (entry.kind === 'wake') && (entry.success === true)),).toBe(true,);
      },
    },),
    it({
      name: 'ends on its wall clock while its work still runs, and its feed says what it waited for',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
        await using run = await startRunThatSettled();
        /**
         The messages the watch sent, which a kept run must not send while it waits.
         */
        const sent: string[] = [];
        const endReason = await watchUntilEnded({
          // The work is named on the first look; the clock passes at the second, while the run waits for work that never ends.
          connection: answeringConnection({ sent, onLook: passTheBoundAtLook({ clock, look: 2, },), },),
          record: run.record,
          options: optionsOf({ runFolder: run.runFolder, },),
          inbox: run.inbox,
          deadline: clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
          workLeftRunning: async function workLeftRunning() {
            return ['proc_7adb (native-build-2) 486667 systemd-run --user --scope',];
          },
        },);
        expect(endReason,).toBe('wall-clock',);
        expect(sent,).toEqual([],);
        /**
         What the run's feed holds.
         */
        const feed = await feedOf(run,);
        expect(feed.some((entry,) => entry.kind === 'waiting'),).toBe(true,);
        // A run the wall clock ended while it waited for work did not end with nothing to do, and the feed says so.
        expect(String(feed.at(-1)?.problem,),).toContain('waited for the work it left running',);
      },
    },),
    it({
      name: 'does not end a run as idle before the agent answers the message that woke it, and ends it on the wall clock'
        + ' instead',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
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
          // Pi takes the message and says nothing more, which is the state a driver that ended a woken run as idle left it in: the
          // agent's turn never opened, so the report it would have written was never read. The clock passes at the fourth look,
          // after the look that could have ended the run as idle and did not.
          connection: answeringConnection({ sent, onLook: passTheBoundAtLook({ clock, look: 4, },), },),
          record: run.record,
          options: optionsOf({
            runFolder: run.runFolder,
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
          },),
          inbox: run.inbox,
          deadline: clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
        const feed = await feedOf(run,);
        expect(feed.some((entry,) => (entry.kind === 'wake') && (entry.success === true)),).toBe(true,);
      },
    },),
    it({
      name: 'ends a run as idle when Pi handled the message that woke it without starting a turn',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
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
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
          },),
          inbox: run.inbox,
          deadline: clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
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
            milliseconds: BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
          },),
          inbox: run.inbox,
          deadline: clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
        const feed = await feedOf(run,);
        expect(feed.find((entry,) => entry.kind === 'wake')?.success,).toBe(false,);
      },
    },),
    it({
      name: 'ends a run with nothing running as it always did, sending nothing',
      fn: async (ctx,) => {
        /** The run's clock, which this test moves: see `ownTheClock`. */
        const clock = ownTheClock(ctx,);
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
          deadline: clock.now() + BOUND_ABOVE_EVERY_LOOK_MILLISECONDS,
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
        const feed = await feedOf(run,);
        expect(feed.some((entry,) => entry.kind === 'waiting'),).toBe(false,);
        expect(feed.some((entry,) => entry.kind === 'wake'),).toBe(false,);
      },
    },),
  ],
},);
