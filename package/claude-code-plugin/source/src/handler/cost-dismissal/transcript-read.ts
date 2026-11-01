/**
 Transcript tail reading for the cost-dismissal hook.

 Reads only the tail so a hook call stays fast on transcripts that grow
 without bound. Supports both transcript dialects:
 Claude Code message rows and Codex response-item rollouts.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { open, } from 'node:fs/promises';

import type { TranscriptContentBlock, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type { ReadonlyDeep, } from 'type-fest';

import { NOT_PARSED, } from './jsonl.ts';

/**
 One parsed transcript row,
 read as the machine wrote it.
 */
type TranscriptRow = ReadonlyDeep<Record<string, unknown>>;

/**
 Module logger for rows the reader skips,
 so a swallowed parse failure stays visible.
 */
const loggerForRows = tagged({ tag: 'cost-dismissal-transcript-read', },);

/**
 Bytes of the transcript tail read per hook call,
 large enough for the finishing turn's text.
 */
const TAIL_BYTES = 262_144;

/**
 Assistant text found in a transcript tail,
 with the count of rows the tail scan examined.
 One unit is one thinking block,
 one summary entry,
 or one assistant message text block:
 whatever prose the agent produced.
 */
type TextScan = {
  /**
   Assistant text units in tail order.
   */
  readonly units: readonly string[];

  /**
   Rows the scan parsed out of the tail.
   */
  readonly rows: number;
};

/**
 Whether a value is a text string,
 narrowing it for the callers that keep only text.

 @param value - Value under test

 @returns True when the value is a string

 @example
 ```ts
 isString('x'); // => true
 ```
 */
function isString(value: unknown,): value is string {
  return (typeof value) === 'string';
}

/**
 Parses one transcript JSONL line,
 or returns {@link NOT_PARSED} when the line is not a record.

 @param line - One JSONL line

 @returns The parsed record, or {@link NOT_PARSED}

 @example
 ```ts
 parseRecord('{"type":"assistant"}');
 ```
 */
function parseRecord(line: string,): TranscriptRow | typeof NOT_PARSED {
  try {
    /* oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing a machine-written transcript line after its parse succeeds */
    return JSON.parse(line,) as TranscriptRow;
  } catch (error) {
    loggerForRows.warn(`skipping an unparseable transcript row: ${String(error,)}`,);
    return NOT_PARSED;
  }
}

/**
 Collects the assistant prose of one Claude Code message row:
 thinking blocks and visible text blocks.

 @param record - Parsed transcript row

 @returns Text units of the row

 @example
 ```ts
 claudeUnitsOf({ type: 'assistant', message: { content: [] } });
 ```
 */
function claudeUnitsOf(record: TranscriptRow,): readonly string[] {
  /**
   Message of the row, absent on attachments and session metadata.
   */
  const { message, } = record as { readonly message?: { readonly content?: unknown, }, };
  /**
   Message content of the row, an array of blocks for assistant turns.
   */
  const content = message?.content;
  if (!Array.isArray(content)) return [];
  /**
   Text units collected in block order.
   */
  const units: string[] = [];
  for (const item of content as readonly TranscriptContentBlock[]) {
    if (isString(item.thinking)) units.push(item.thinking,);
    if (isString(item.text)) units.push(item.text,);
  }
  return units;
}

/**
 Reads the text of one content or summary entry list.

 @param list - Entry list of a reasoning or message payload

 @returns The entry texts in order, empty when the list carries none

 @example
 ```ts
 textsOf([{ text: 'done' }]); // => ['done']
 ```
 */
function textsOf(list: unknown,): readonly string[] {
  if (!Array.isArray(list)) return [];
  /**
   Texts of the entries.
   */
  const texts: string[] = [];
  for (const item of list as readonly Record<string, unknown>[]) {
    if (isString(item.text)) texts.push(item.text,);
  }
  return texts;
}

/**
 Collects the assistant prose of one Codex response item:
 reasoning summary and content entries,
 and assistant message text.

 @param record - Parsed rollout row

 @returns Text units of the row

 @example
 ```ts
 codexUnitsOf({ type: 'response_item', payload: { type: 'reasoning', summary: [] } });
 ```
 */
function codexUnitsOf(record: TranscriptRow,): readonly string[] {
  if (record.type !== 'response_item') return [];
  /**
   Response item payload, the reasoning or message body.
   */
  const { payload, } = record as {
    readonly payload?: {
      readonly type?: string;
      readonly role?: string;
      readonly summary?: unknown;
      readonly content?: unknown;
    },
  };
  if (payload === undefined) return [];
  /**
   Text units collected in entry order.
   */
  const units: string[] = [];
  if (payload.type === 'reasoning') {
    /**
     Summary texts of the reasoning item.
     */
    const summaryTexts = textsOf(payload.summary,);
    /**
     Content texts of the reasoning item.
     */
    const contentTexts = textsOf(payload.content,);
    units.push(
      ...summaryTexts,
      ...contentTexts,
    );
    return units;
  }
  /**
   True when the item is a visible message at all.
   */
  const isMessage = payload.type === 'message';
  if (!isMessage) return units;
  /**
   True when the message is the agent's own.
   */
  const isAssistantRole = payload.role === 'assistant';
  if (!isAssistantRole) return units;
  units.push(...textsOf(payload.content,),);
  return units;
}

/**
 Reads assistant text from the tail of a transcript.

 Reads only the last {@link TAIL_BYTES} bytes so a hook call stays fast on
 transcripts that grow without bound; the finishing turn's text lives there.
 Rows that fail to parse are skipped, since the hook must never fail its event.

 @param transcriptPath - Transcript JSONL to read

 @returns Assistant text of the tail and the rows examined

 @example
 ```ts
 await readTextUnits('/session.jsonl');
 ```
 */
async function readTextUnits(transcriptPath: string,): Promise<TextScan> {
  /**
   Text units collected in tail order.
   */
  const units: string[] = [];
  /**
   Open transcript handle, closed by scope exit even when reading throws.
   */
  await using handle = await open(
    transcriptPath,
    'r',
  );
  /**
   Byte size of the transcript, used to read only its tail.
   */
  const { size, } = await handle.stat();
  /**
   Offset of the first byte read; clamped so small transcripts read whole.
   */
  const start = Math.max(
    0,
    size - TAIL_BYTES,
  );
  /**
   Buffer holding the tail bytes.
   */
  const buffer = Buffer.alloc(size - start,);
  await handle.read(
    buffer,
    0,
    buffer.length,
    start,
  );
  /**
   Tail text with a partial first row dropped when the tail starts mid-row.
   */
  const text = buffer.toString('utf8',);
  /**
   Tail rows before a partial leading row is dropped.
   */
  const splitRows = text.split('\n',);
  /**
   Tail rows, with a partial first row dropped when the tail starts mid-row.
   */
  const rows = (start === 0)
    ? splitRows
    : splitRows.slice(1,);
  /**
   Rows the scan parsed, excluding blanks.
   */
  let examined = 0;
  for (const row of rows) {
    if (row === '') continue;
    examined += 1;
    /**
     Parsed record, or the skip sentinel for a malformed row.
     */
    const record = parseRecord(row,);
    if (record === NOT_PARSED) continue;
    /**
     Assistant prose of this row, whichever dialect wrote it.
     */
    const claudeUnits = claudeUnitsOf(record,);
    /**
     Codex-dialect prose of this row, empty on Claude Code rows.
     */
    const codexUnits = codexUnitsOf(record,);
    units.push(
      ...claudeUnits,
      ...codexUnits,
    );
  }
  /**
   Scan result assembled from the collected units and row count.
   */
  const scan: TextScan = {
    units,
    rows: examined,
  };
  return scan;
}

export type { TextScan, };

export {
  readTextUnits,
  TAIL_BYTES,
};
