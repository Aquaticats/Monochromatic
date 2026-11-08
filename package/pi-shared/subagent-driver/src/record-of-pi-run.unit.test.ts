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
  type RunSummary,
  startRecordOfPiRun,
} from '../dist/final/node/index.mjs';

/** Milliseconds a record is left silent, so a later note can be seen to reset it. */
const SILENCE_MILLISECONDS = 30;

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

async function summaryOf(record: RecordOfPiRun,): Promise<RunSummary> {
  await record.close();
  return record.summarize({ endReason: 'idle', piExit: {}, piHasExited: true, sessionStats: null, },);
}

function declineNothing(): void {
  throw new Error('no dialog was expected',);
}

function assistantMessage(content: readonly Record<string, unknown>[],): Record<string, unknown> {
  return { type: 'message_end', message: { role: 'assistant', content, }, };
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
        const record = startRecordOfPiRun(join(folder.path, 'feed.jsonl',),);
        await wait(SILENCE_MILLISECONDS,);
        const before = record.silentMilliseconds();
        record.note({ kind: 'inbox', },);
        expect(record.silentMilliseconds(),).toBeLessThan(before,);
        await record.close();
      },
    },),
    it({
      name: 'keeps the assistant text, ignoring other roles and messages without text',
      fn: async () => {
        await using folder = await makeFolder();
        const record = startRecordOfPiRun(join(folder.path, 'feed.jsonl',),);
        record.observe({ event: assistantMessage([{ type: 'text', text: 'first', },],), declineDialog: declineNothing, },);
        record.observe({ event: { type: 'message_end', message: { role: 'user', content: [{ type: 'text', text: 'asked', },], }, }, declineDialog: declineNothing, },);
        record.observe({ event: assistantMessage([{ type: 'toolCall', },],), declineDialog: declineNothing, },);
        expect((await summaryOf(record,)).lastAssistantText,).toBe('first',);
      },
    },),
    it({
      name: 'notes the thinking length and the tool calls of a message',
      fn: async () => {
        await using folder = await makeFolder();
        const feedFile = join(folder.path, 'feed.jsonl',);
        const record = startRecordOfPiRun(feedFile,);
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
        const record = startRecordOfPiRun(join(folder.path, 'feed.jsonl',),);
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
      name: 'a feed that cannot be written does not stop the run and is named in the summary',
      fn: async () => {
        await using folder = await makeFolder();
        const record = startRecordOfPiRun(join(folder.path, 'missing', 'feed.jsonl',),);
        await wait(SILENCE_MILLISECONDS,);
        record.note({ kind: 'state', },);
        expect((await summaryOf(record,)).feedProblem,).toContain('ENOENT',);
      },
    },),
    it({
      name: 'names a failure of the driver in the summary and on the feed',
      fn: async () => {
        await using folder = await makeFolder();
        const feedFile = join(folder.path, 'feed.jsonl',);
        const record = startRecordOfPiRun(feedFile,);
        record.noteDriverFailure('it broke',);
        expect((await summaryOf(record,)).driverError,).toBe('it broke',);
        expect(await readFile(feedFile, 'utf8',),).toContain('"kind":"driver-failed"',);
      },
    },),
    it({
      name: 'leaves how Pi ended out of the summary when Pi outlasted every signal',
      fn: async () => {
        await using folder = await makeFolder();
        const record = startRecordOfPiRun(join(folder.path, 'feed.jsonl',),);
        await record.close();
        const summary = record.summarize({ endReason: 'idle', piExit: {}, piHasExited: false, sessionStats: null, },);
        expect('piExit' in summary,).toBe(false,);
      },
    },),
  ],
},);
