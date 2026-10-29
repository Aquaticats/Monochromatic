/**
 Transcript tail reading for the thinking-dismissal hook.

 Reads only the tail so a stop stays fast on transcripts
 that grow without bound.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { open, } from 'node:fs/promises';

import type {
  TranscriptContentBlock,
  TranscriptRecord,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';

import { NOT_PARSED, } from './jsonl.ts';

/**
 Module logger for rows the reader skips,
 so a swallowed parse failure stays visible.
 */
const loggerForRows = tagged({ tag: 'thinking-dismissal-transcript-read', },);

/**
 Bytes of the transcript tail read per stop,
 large enough for the finishing turn's thinking blocks.
 */
const TAIL_BYTES = 262_144;

/**
 Thinking blocks found in a transcript tail,
 with the count of rows the tail scan examined.
 */
type ThinkingScan = {
  /**
   Thinking blocks in tail order.
   */
  readonly blocks: readonly string[];

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
function parseRecord(line: string,): TranscriptRecord | typeof NOT_PARSED {
  try {
    /* oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing a machine-written transcript line after its parse succeeds */
    return JSON.parse(line,) as TranscriptRecord;
  } catch (error) {
    loggerForRows.warn(`skipping an unparseable transcript row: ${String(error,)}`,);
    return NOT_PARSED;
  }
}

/**
 Reads thinking blocks from the tail of a transcript.

 Reads only the last {@link TAIL_BYTES} bytes so a stop stays fast on
 transcripts that grow without bound; the finishing turn's blocks live there.
 Rows that fail to parse are skipped, since the hook must never fail a stop.

 @param transcriptPath - Transcript JSONL to read

 @returns Thinking blocks of the tail and the rows examined

 @example
 ```ts
 await readThinkingBlocks('/session.jsonl');
 ```
 */
async function readThinkingBlocks(transcriptPath: string,): Promise<ThinkingScan> {
  /**
   Thinking blocks collected in tail order.
   */
  const blocks: string[] = [];
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
     Message of the row, absent on attachments and session metadata.
     */
    const { message, } = record;
    /**
     Message content of the row, an array of blocks for assistant turns.
     */
    const content = message?.content;
    if (!Array.isArray(content)) continue;
    for (const item of content as readonly TranscriptContentBlock[]) {
      if (isString(item.thinking)) blocks.push(item.thinking,);
    }
  }
  /**
   Scan result assembled from the collected blocks and row count.
   */
  const scan: ThinkingScan = {
    blocks,
    rows: examined,
  };
  return scan;
}

export type { ThinkingScan, };

export {
  readThinkingBlocks,
  TAIL_BYTES,
};
