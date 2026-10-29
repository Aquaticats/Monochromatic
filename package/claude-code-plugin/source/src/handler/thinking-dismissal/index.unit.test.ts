import {
  describe,
  expect as verify,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  mkdtemp,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  createThinkingDismissalHandler,
  findSizeDismissal,
  thinkingDismissalParser,
  thinkingDismissalWriter,
} from './index.ts';

/**
 Scratch directory for this suite's fixture transcripts and shadow logs.
 */
const SCRATCH = await mkdtemp(join(scratchRoot(), 'thinking-dismissal-',),);

/**
 Transcript line with one thinking block.

 @param thinking - Reasoning text of the block

 @returns one JSONL line

 @example
 ```ts
 thinkingLine('Quadratic, but the lines are short.');
 ```
 */
function thinkingLine(thinking: string,): string {
  return JSON.stringify({
    type: 'assistant',
    message: {
      content: [{ type: 'thinking', thinking, },],
    },
  },);
}

/**
 Runs the handler against a fixture transcript,
 returning the shadow log contents it produced.

 @param id - Case identifier, so cases never share fixtures

 @param lines - Transcript JSONL lines to write

 @returns the shadow log text after the run

 @example
 ```ts
 await runHandler('case-1', [thinkingLine('...')]);
 ```
 */
async function runHandler(
  id: string,
  lines: readonly string[],
): Promise<string> {
  const transcript = join(SCRATCH, `transcript-${id}.jsonl`,);
  await writeFile(
    transcript,
    `${lines.join('\n',)}\n`,
  );
  const logPath = join(SCRATCH, `shadow-${id}.jsonl`,);
  const handler = createThinkingDismissalHandler({
    env: { MONOCHROMATIC_THINKING_DISMISSAL_LOG: logPath, },
  },);
  await handler({
    hook_event_name: 'Stop',
    stop_hook_active: false,
    session_id: `session-${id}`,
    transcript_path: transcript,
    cwd: SCRATCH,
    permission_mode: 'default',
  },);
  return readFile(logPath, 'utf8',);
}

await describe({
  name: 'thinking-dismissal shadow',
  children: [
    it({
      name: 'flags a sentence that carries cost and a dismissal together',
      fn: async () => {
        const hits = findSizeDismissal('The scan is quadratic, but the lines are short.',);
        verify(hits.length,).toBe(1,);
        verify(hits[0]?.pattern,).toBe('size-dismissal',);
        verify(hits[0]?.sentenceIndex,).toBe(0,);
        verify(hits[0]?.blockHash.length,).toBe(16,);
      },
    },),
    it({
      name: 'leaves a block that names cost without dismissing it unflagged',
      fn: async () => {
        const hits = findSizeDismissal('The scan is quadratic. Rewrite it as one linear pass.',);
        verify(hits.length,).toBe(0,);
      },
    },),
    it({
      name: 'records a hit with hashes and indexes and never thinking text',
      fn: async () => {
        const raw = await runHandler('hit', [
          thinkingLine('Quadratic, but the lines are short.'),
        ],);
        const record = JSON.parse(raw.trimEnd(),) as {
          readonly thinkingBlocks: number;
          readonly hits: readonly { readonly pattern: string, }[];
        };
        verify(record.thinkingBlocks,).toBe(1,);
        verify(record.hits.length,).toBe(1,);
        verify(raw.includes('lines are short'),).toBe(false,);
        verify(raw.includes('Quadratic'),).toBe(false,);
      },
    },),
    it({
      name: 'records the stop with zero hits when thinking dismisses nothing',
      fn: async () => {
        const raw = await runHandler('clean', [
          thinkingLine('One linear pass over the rows.'),
        ],);
        const record = JSON.parse(raw.trimEnd(),) as {
          readonly thinkingBlocks: number;
          readonly hits: readonly unknown[];
        };
        verify(record.thinkingBlocks,).toBe(1,);
        verify(record.hits.length,).toBe(0,);
      },
    },),
    it({
      name: 'does not re-record a hit a previous stop already recorded',
      fn: async () => {
        const transcript = join(SCRATCH, 'transcript-dedupe.jsonl',);
        await writeFile(
          transcript,
          `${thinkingLine('Quadratic, but the lines are short.')}\n`,
        );
        const logPath = join(SCRATCH, 'shadow-dedupe.jsonl',);
        const handler = createThinkingDismissalHandler({
          env: { MONOCHROMATIC_THINKING_DISMISSAL_LOG: logPath, },
        },);
        await handler({
          hook_event_name: 'Stop',
          stop_hook_active: false,
          session_id: 'session-dedupe',
          transcript_path: transcript,
          cwd: SCRATCH,
          permission_mode: 'default',
        },);
        await handler({
          hook_event_name: 'Stop',
          stop_hook_active: true,
          session_id: 'session-dedupe',
          transcript_path: transcript,
          cwd: SCRATCH,
          permission_mode: 'default',
        },);
        const raw = await readFile(logPath, 'utf8',);
        const records = raw.trimEnd()
          .split('\n',)
          .map((line,) => JSON.parse(line,) as { readonly hits: readonly unknown[], },);
        verify(records.length,).toBe(2,);
        verify(records[0]?.hits.length,).toBe(1,);
        verify(records[1]?.hits.length,).toBe(0,);
      },
    },),
    it({
      name: 'skips malformed transcript rows and still records the stop',
      fn: async () => {
        const raw = await runHandler('malformed', [
          'not json at all',
          thinkingLine('Quadratic, but the lines are short.'),
          '{also broken',
        ],);
        const record = JSON.parse(raw.trimEnd(),) as {
          readonly hits: readonly unknown[];
        };
        verify(record.hits.length,).toBe(1,);
      },
    },),
    it({
      name: 'parses the Stop payload and serializes an unblocking output',
      fn: async () => {
        const event = thinkingDismissalParser('{"hook_event_name":"Stop","stop_hook_active":false}',);
        verify(event.hook_event_name,).toBe('Stop',);
        verify(thinkingDismissalWriter({},),).toBe('{}',);
      },
    },),
  ],
},);
