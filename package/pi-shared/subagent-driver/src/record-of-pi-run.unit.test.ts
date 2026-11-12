/**
 What the record keeps from a run's events.

 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
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
  preview,
  type RecordOfPiRun,
  type RunFailure,
  type RunSummary,
  startRecordOfPiRun,
} from '../dist/final/node/index.mjs';

/** Milliseconds a record's clock is moved by, so a silence is measured on a clock the test owns. */
const SILENCE_MILLISECONDS = 30;
/** Milliseconds between looks at what a test waits for. */
const LOOK_MILLISECONDS = 10;
/** Milliseconds a test waits for the feed's failure to show before giving up on it. */
const FEED_FAILURE_DEADLINE_MILLISECONDS = 5_000;

type DisposableFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

async function makeFolder(): Promise<DisposableFolder> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-record-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Makes a record whose clock only moves when a test says so, so nothing about its counts depends on how long a test took.

 @param feedFile - file the record writes

 @returns the record and the clock it reads
 */
function recordOnOwnClock(feedFile: string,): { readonly record: RecordOfPiRun; readonly clock: { at: number }; } {
  const clock = { at: 1_000, };
  return {
    record: startRecordOfPiRun({
      feedFile,
      now: () => clock.at,
    },),
    clock,
  };
}

/**
 The problem a failure names, or the empty text when nothing failed.

 @param failure - what the record reported

 @returns the problem's words
 */
function problemOf(failure: RunFailure,): string {
  return failure.failed ? failure.problem : '';
}

async function summaryOf(record: RecordOfPiRun,): Promise<RunSummary> {
  await record.close();
  return record.summarize({ endReason: 'idle', piExit: {}, piHasExited: true, sessionStats: null, },);
}

function declineNothing(): void {
  throw new Error('no dialog was expected',);
}

/**
 A rate limit, shaped as a gateway sends one, and the error the record's own tests end a run on.
 */
const RATE_LIMIT_ERROR_TEXT = '429 {"error":{"message":"Too many requests"}}';

/**
 A plan limit sent as a rate limit: Pi's own retry leaves it to the run, so a run can end on one after a rate limit it retried.
 */
const QUOTA_ERROR_TEXT = '429 {"error":{"code":"insufficient_quota","message":"You exceeded your current quota"}}';

function assistantMessage(
  content: readonly Record<string, unknown>[],
  ending: Record<string, unknown> = {},
): Record<string, unknown> {
  return {
    type: 'message_end',
    message: {
      role: 'assistant',
      content,
      ...ending,
    },
  };
}

await describe({
  name: 'the record of a run',
  children: [
    it({
      name: `${preview.name} keeps short text, cuts long text and says how long it was`,
      fn: async () => {
        expect(preview({ value: 'abc', length: 3, },),).toBe('abc',);
        expect(preview({ value: 'abcd', length: 3, },),).toBe('abc...[4]',);
        expect(preview({ value: { a: 1, }, length: 20, },),).toBe('{"a":1}',);
        expect(preview({ value: undefined, length: 20, },),).toBe('',);
      },
    },),
    it({
      name: 'a note counts as activity, so a message just delivered does not look like a long silence',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, clock, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        clock.at += SILENCE_MILLISECONDS;
        expect(record.silentMilliseconds(),).toBe(SILENCE_MILLISECONDS,);
        record.note({ kind: 'inbox', },);
        expect(record.silentMilliseconds(),).toBe(0,);
        await record.close();
      },
    },),
    it({
      name: 'keeps the assistant text, ignoring other roles and messages without text',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({ event: assistantMessage([{ type: 'text', text: 'first', },],), declineDialog: declineNothing, },);
        record.observe({ event: { type: 'message_end', message: { role: 'user', content: [{ type: 'text', text: 'asked', },], }, }, declineDialog: declineNothing, },);
        record.observe({ event: assistantMessage([{ type: 'toolCall', },],), declineDialog: declineNothing, },);
        expect((await summaryOf(record,)).lastAssistantText,).toBe('first',);
      },
    },),
    it({
      name: 'a later reply with text replaces the one before it',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({ event: assistantMessage([{ type: 'text', text: 'first', },],), declineDialog: declineNothing, },);
        record.observe({ event: assistantMessage([{ type: 'text', text: 'second', },],), declineDialog: declineNothing, },);
        expect((await summaryOf(record,)).lastAssistantText,).toBe('second',);
      },
    },),
    it({
      name: 'notes the thinking length and the tool calls of a message',
      fn: async () => {
        await using folder = await makeFolder();
        const feedFile = join(folder.path, 'feed.jsonl',);
        const { record, } = recordOnOwnClock(feedFile,);
        record.observe({
          event: assistantMessage([{ type: 'thinking', thinking: 'abcd', }, { type: 'toolCall', }, { type: 'toolCall', },],),
          declineDialog: declineNothing,
        },);
        await record.close();
        const line = await readFile(feedFile, 'utf8',);
        expect(line,).toContain('"thinkingChars":4',);
        expect(line,).toContain('"toolCalls":2',);
      },
    },),
    it({
      name: 'only agent_settled ends a run: agent_end leaves it running',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({ event: { type: 'agent_start', }, declineDialog: declineNothing, },);
        record.observe({ event: { type: 'agent_end', }, declineDialog: declineNothing, },);
        expect(record.isRunning(),).toBe(true,);
        record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        expect(record.isRunning(),).toBe(false,);
        expect(record.settledCount(),).toBe(1,);
        await record.close();
      },
    },),
    it({
      name: 'a model error of the last reply is known only once Pi settles, because a retry may still come',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: assistantMessage([], { stopReason: 'error', errorMessage: 'no key', },),
          declineDialog: declineNothing,
        },);
        expect(record.unrecoveredModelError().failed,).toBe(false,);
        record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        expect(record.unrecoveredModelError(),).toEqual({
          failed: true,
          problem: 'no key',
        },);
        await record.close();
      },
    },),
    it({
      name: 'a reply that succeeded after a model error leaves no model error',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: assistantMessage([], { stopReason: 'error', errorMessage: 'no key', },),
          declineDialog: declineNothing,
        },);
        record.observe({ event: { type: 'auto_retry_start', }, declineDialog: declineNothing, },);
        record.observe({ event: assistantMessage([{ type: 'text', text: 'recovered', },],), declineDialog: declineNothing, },);
        record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        expect(record.unrecoveredModelError().failed,).toBe(false,);
        await record.close();
      },
    },),
    it({
      name: 'a later reply that succeeded clears a model error Pi settled on',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: assistantMessage([], { stopReason: 'error', errorMessage: 'no key', },),
          declineDialog: declineNothing,
        },);
        record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        record.observe({ event: assistantMessage([{ type: 'text', text: 'steered back', },],), declineDialog: declineNothing, },);
        expect(record.unrecoveredModelError().failed,).toBe(false,);
        await record.close();
      },
    },),
    it({
      name: 'keeps what Pi said when its own automatic retry gave up on the error the run ended on',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: assistantMessage([], { stopReason: 'error', errorMessage: RATE_LIMIT_ERROR_TEXT, },),
          declineDialog: declineNothing,
        },);
        record.observe({ event: { type: 'auto_retry_start', attempt: 1, }, declineDialog: declineNothing, },);
        record.observe({
          event: {
            type: 'auto_retry_end',
            success: false,
            attempt: 3,
            finalError: RATE_LIMIT_ERROR_TEXT,
          },
          declineDialog: declineNothing,
        },);
        record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        const summary = await summaryOf(record,);
        expect(summary.piRetryGaveUp,).toEqual({
          attempts: 3,
          problem: RATE_LIMIT_ERROR_TEXT,
        },);
        expect(summary.modelError,).toBe(RATE_LIMIT_ERROR_TEXT,);
        expect(summary.retries,).toBe(0,);
        // An event that names no attempt leaves the count at none rather than inventing one.
        const unnamed = recordOnOwnClock(join(folder.path, 'unnamed.jsonl',),);
        unnamed.record.observe({
          event: assistantMessage([], { stopReason: 'error', errorMessage: RATE_LIMIT_ERROR_TEXT, },),
          declineDialog: declineNothing,
        },);
        unnamed.record.observe({
          event: { type: 'auto_retry_end', success: false, },
          declineDialog: declineNothing,
        },);
        unnamed.record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        expect((await summaryOf(unnamed.record,)).piRetryGaveUp,).toEqual({
          attempts: 0,
          problem: RATE_LIMIT_ERROR_TEXT,
        },);
      },
    },),
    it({
      name: 'leaves nothing of Pi\'s own retry once a reply succeeded',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: assistantMessage([], { stopReason: 'error', errorMessage: RATE_LIMIT_ERROR_TEXT, },),
          declineDialog: declineNothing,
        },);
        record.observe({
          event: {
            type: 'auto_retry_end',
            success: false,
            attempt: 1,
            finalError: RATE_LIMIT_ERROR_TEXT,
          },
          declineDialog: declineNothing,
        },);
        record.observe({
          event: assistantMessage([{ type: 'text', text: 'recovered', },],),
          declineDialog: declineNothing,
        },);
        record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        const summary = await summaryOf(record,);
        expect('piRetryGaveUp' in summary,).toBe(false,);
        expect('modelError' in summary,).toBe(false,);
      },
    },),
    it({
      name: 'names no retry of an earlier failure once a failure of its own stands',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: assistantMessage([], { stopReason: 'error', errorMessage: RATE_LIMIT_ERROR_TEXT, },),
          declineDialog: declineNothing,
        },);
        record.observe({
          event: {
            type: 'auto_retry_end',
            success: false,
            attempt: 1,
            finalError: RATE_LIMIT_ERROR_TEXT,
          },
          declineDialog: declineNothing,
        },);
        // The failure the run ends on is a later one Pi never retried, such as a quota it leaves to the run.
        record.observe({
          event: assistantMessage([], { stopReason: 'error', errorMessage: QUOTA_ERROR_TEXT, },),
          declineDialog: declineNothing,
        },);
        record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        const summary = await summaryOf(record,);
        expect(summary.modelError,).toBe(QUOTA_ERROR_TEXT,);
        expect('piRetryGaveUp' in summary,).toBe(false,);
      },
    },),
    it({
      name: 'a model error with no message is still an error the run reports',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: assistantMessage([], { stopReason: 'error', },),
          declineDialog: declineNothing,
        },);
        record.observe({ event: { type: 'agent_settled', }, declineDialog: declineNothing, },);
        expect(record.unrecoveredModelError(),).toEqual({
          failed: true,
          problem: 'the model reported an error and said nothing about it',
        },);
        await record.close();
      },
    },),
    it({
      name: 'a feed that fails is named at once, and a note after it is still safe',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'missing', 'feed.jsonl',),);
        /**
         When to give up waiting for the feed's open to fail.
         */
        const deadline = Date.now() + FEED_FAILURE_DEADLINE_MILLISECONDS;
        while ((!record.feedProblem().failed) && (Date.now() <= deadline)) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- the feed's open fails on its own schedule, so the test looks again after waiting; see doc/lint/sequential-awaits.md
          await wait(LOOK_MILLISECONDS,);
        }
        if (!record.feedProblem().failed) {
          throw new Error(
            `the feed's failure was named nowhere within ${String(FEED_FAILURE_DEADLINE_MILLISECONDS,)} ms`,
          );
        }
        expect(
          problemOf(record.feedProblem(),),
        ).toContain('ENOENT',);
        expect(() => {
          record.note({ kind: 'state', },);
        },).not.toThrow();
        expect((await summaryOf(record,)).feedProblem,).toContain('ENOENT',);
      },
    },),
    it({
      name: 'names a failure of the driver in the summary and on the feed',
      fn: async () => {
        await using folder = await makeFolder();
        const feedFile = join(folder.path, 'feed.jsonl',);
        const { record, } = recordOnOwnClock(feedFile,);
        record.noteDriverFailure('it broke',);
        expect((await summaryOf(record,)).driverError,).toBe('it broke',);
        expect(await readFile(feedFile, 'utf8',),).toContain('"kind":"driver-failed"',);
      },
    },),
    it({
      name: 'leaves how Pi ended out of the summary when Pi outlasted every signal',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        await record.close();
        const summary = record.summarize({ endReason: 'idle', piExit: {}, piHasExited: false, sessionStats: null, },);
        expect('piExit' in summary,).toBe(false,);
      },
    },),
    it({
      name: 'names the compaction Pi could not finish, which no reply of the run recovers',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, clock, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: { type: 'compaction_start', reason: 'threshold', },
          declineDialog: declineNothing,
        },);
        clock.at += SILENCE_MILLISECONDS;
        record.observe({
          event: {
            type: 'compaction_end',
            reason: 'threshold',
            aborted: false,
            errorMessage: 'Auto-compaction failed: the request is larger than the model window',
          },
          declineDialog: declineNothing,
        },);
        expect((await summaryOf(record,)).compactionProblem,)
          .toBe('Auto-compaction failed: the request is larger than the model window',);
      },
    },),
    it({
      name: 'notes a compaction still running as the run ended, naming it and how long it had run',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, clock, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: { type: 'compaction_start', reason: 'threshold', },
          declineDialog: declineNothing,
        },);
        clock.at += SILENCE_MILLISECONDS;
        expect((await summaryOf(record,)).compactionProblem,)
          .toBe(`Pi was still compacting (threshold) after ${String(SILENCE_MILLISECONDS,)} ms`,);
      },
    },),
    it({
      name: 'a compaction that worked and one Pi aborted leave no problem behind',
      fn: async () => {
        await using folder = await makeFolder();
        const { record, } = recordOnOwnClock(join(folder.path, 'feed.jsonl',),);
        record.observe({
          event: { type: 'compaction_start', reason: 'threshold', },
          declineDialog: declineNothing,
        },);
        record.observe({
          event: { type: 'compaction_end', reason: 'threshold', aborted: false, },
          declineDialog: declineNothing,
        },);
        expect('compactionProblem' in (await summaryOf(record,)),).toBe(false,);
        const aborted = recordOnOwnClock(join(folder.path, 'aborted.jsonl',),);
        aborted.record.observe({
          event: { type: 'compaction_start', reason: 'manual', },
          declineDialog: declineNothing,
        },);
        aborted.record.observe({
          event: { type: 'compaction_end', reason: 'manual', aborted: true, },
          declineDialog: declineNothing,
        },);
        expect((await summaryOf(aborted.record,)).compactionProblem,).toBe('Pi aborted the compaction',);
      },
    },),
  ],
},);
