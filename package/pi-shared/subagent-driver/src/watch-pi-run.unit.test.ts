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
  type ConnectionToPi,
  type DriverOptions,
  isRecord,
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
 Seconds of silence after which a settled run would end. Every run here is one Pi is at work on, so this never decides anything;
 the wall clock, which each test sets, is what ends them.
 */
const IDLE_SECONDS = 0;

/**
 Seconds of wall clock the options name, derived from the deadline a test passes, so the two never disagree.
 */
const RUN_WALL_SECONDS = RUN_MILLISECONDS / MILLISECONDS_PER_SECOND;

/**
 Text of the message a test drops in the inbox.
 */
const MESSAGE_TEXT = 'a message nothing answers';

/**
 Name the message is published under.
 */
const MESSAGE_NAME = 'message.txt';

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
 The options of a run, which the watch reads for its turn cap and its idle limit; the wall clock a test passes is the deadline
 the watch keeps, and these carry the same number so a reader cannot find two answers to one question.

 @param runFolder - folder of the run

 @returns options of this run
 */
function optionsOf({ runFolder, }: { readonly runFolder: string; },): DriverOptions {
  return {
    runFolder,
    workingFolder: runFolder,
    promptFile: join(runFolder, 'prompt.md',),
    provider: 'stand-in',
    model: 'stand-in-model',
    piArguments: [],
    wallSeconds: RUN_WALL_SECONDS,
    idleSeconds: IDLE_SECONDS,
    turnCap: Number.MAX_SAFE_INTEGER,
  };
}

/**
 Starts a run whose Pi is at work, with a message waiting in the inbox.

 @returns the run, and the folder that goes with it
 */
async function startRunWithAMessage(): Promise<RunUnderTest> {
  const folder = await makeFolder();
  const runFolder = folder.path;
  const inbox = join(runFolder, 'inbox',);
  await mkdir(inbox,);
  await writeFile(join(inbox, MESSAGE_NAME,), MESSAGE_TEXT,);
  const record = startRecordOfPiRun({ feedFile: join(runFolder, 'feed.jsonl',), },);
  record.observe({
    event: { type: 'agent_start', },
    declineDialog: function refuseDialog() {
      throw new Error('no dialog was expected',);
    },
  },);
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
 A connection that never answers, stalling for exactly the bound the watch asked for, and recording that bound.

 @param boundsAsked - where the bounds the watch asked for are recorded

 @returns the connection a watch reads
 */
function stallingConnection({ boundsAsked, }: { readonly boundsAsked: number[]; },): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async function stall({ answerMilliseconds, }: { readonly answerMilliseconds?: number },) {
      // A watch that asks for an answer with no bound would wait for ever, which is exactly what this file is about.
      if (answerMilliseconds === undefined) {
        throw new Error('the watch asked for an answer with no bound',);
      }
      boundsAsked.push(answerMilliseconds,);
      await wait(answerMilliseconds, undefined, { ref: false, },);
      return {
        success: false,
        error: `no answer in ${String(answerMilliseconds,)} ms`,
      };
    },
    shutDown: async () => ({ stopped: true, }),
  };
}

/**
 A connection whose commands are all answered at once.

 @returns the connection a watch reads
 */
function answeringConnection(): ConnectionToPi {
  return {
    hasExited: () => false,
    exitDetails: () => ({}),
    write: function write() {
      throw new Error('the watch was not expected to write without waiting for an answer',);
    },
    send: async () => ({ success: true, }),
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
  ],
},);
