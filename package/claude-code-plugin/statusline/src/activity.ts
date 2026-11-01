/**
 Activity word extraction from Claude transcripts.
 
 @module
 */

import { openAsBlob, } from 'node:fs';
import { readFile, } from 'node:fs/promises';

import { findGerundInText, } from './gerund.ts';

//region Transcript tail

/**
 Number of bytes to read from the end of the transcript.
 Large enough to hold several typical JSONL entries (median entry is under 1 KB,
 95th percentile about 6 KB), small enough to stay a single cheap read.
 */
const TAIL_BYTES = 65_536;

/**
 Substring that marks a JSONL entry as an assistant message before paying for `JSON.parse`.
 */
const ASSISTANT_ENTRY_MARKER = '"type":"assistant"';

//endregion Transcript tail

//region Gerund extraction

/**
 Fields this module reads from transcript JSON objects, each unverified until narrowed.
 */
type TranscriptObject = {
  /**
   Content blocks of an assistant message.
   */
  readonly content?: unknown;
  /**
   Message of a transcript entry.
   */
  readonly message?: unknown;
  /**
   Text of a `text` content block.
   */
  readonly text?: unknown;
  /**
   Summary text of a `thinking` content block.
   */
  readonly thinking?: unknown;
  /**
   Kind of a transcript entry or content block.
   */
  readonly type?: unknown;
};

/**
 Reads the complete JSONL lines in the transcript tail without loading the entire file.
 
 @param transcriptPath - path to session transcript JSONL
 
 @returns complete lines in file order; the first line is dropped when the read began mid-file, since it is cut off
 
 @example
 ```ts
 await readTranscriptTailLines({ transcriptPath: '/tmp/session.jsonl' });
 ```
 */
async function readTranscriptTailLines({
  transcriptPath,
}: Readonly<{
  transcriptPath: string;
}>,): Promise<readonly string[]> {
  /**
   Blob view of the transcript.
   */
  const blob = await openAsBlob(transcriptPath,);
  /**
   Slice offset clamped to zero for short transcripts.
   */
  const start = Math.max(
    0,
    blob.size - TAIL_BYTES,
  );
  /**
   Tail of the transcript decoded as UTF-8 text.
   */
  const tail = await blob
    .slice(
    start,
    blob.size,
    )
    .text();
  /**
   Non-empty lines, including a possibly cut-off first one.
   */
  const lines = tail.split('\n',)
    .filter(function isNotBlank(line,): boolean {
      return line.length > 0;
    },);

  if (start === 0)
    return lines;
  return lines.slice(1,);
}

/**
 Parses one JSONL line.
 
 @param line - one transcript JSONL line
 
 @returns parsed value, or `undefined` when the line is not valid JSON, such as a line cut off by the tail window
 
 @example
 ```ts
 parseJsonLine('{"type":"assistant"}');
 ```
 */
function parseJsonLine(line: string,): unknown {
  try {
    return JSON.parse(line,);
  }
  catch (_error: unknown) {
    return undefined;
  }
}

/**
 Narrows a parsed JSON value to an object whose fields are read by name.
 
 @param value - parsed JSON value
 
 @returns whether the value is a non-null object
 
 @example
 ```ts
 isRecord({ type: 'text' });
 ```
 */
function isRecord(value: unknown,): value is TranscriptObject {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Wraps a value in a list when it is a string.
 
 @param value - parsed JSON field
 
 @returns the string alone, or nothing for any other value
 
 @example
 ```ts
 stringsOf('Compiling');
 ```
 */
function stringsOf(value: unknown,): readonly string[] {
  if ((typeof value) !== 'string')
    return [];
  return [value,];
}

/**
 Collects the prose of one assistant content block.
 
 @param block - parsed content block
 
 @returns the block's text for `text` blocks and its summary for `thinking` blocks, otherwise nothing; tool_use and tool_result blocks carry no prose, and the thinking signature is an opaque token
 
 @example
 ```ts
 proseOfBlock({ type: 'text', text: 'Compiling' });
 ```
 */
function proseOfBlock(block: unknown,): readonly string[] {
  if (!isRecord(block,))
    return [];
  if (block.type === 'text')
    return stringsOf(block.text,);
  if (block.type === 'thinking')
    return stringsOf(block.thinking,);
  return [];
}

/**
 Collects the prose of an assistant JSONL entry.
 
 Text blocks and thinking summaries both describe what Claude is doing.
 Tool calls, tool results, and every non-assistant entry carry no such words, so they are ignored.
 
 @param line - one transcript JSONL line
 
 @returns joined assistant text and thinking blocks, or empty string for other entries and unparsable lines
 
 @example
 ```ts
 assistantTextOfLine('{"type":"assistant","message":{"content":[{"type":"text","text":"Compiling"}]}}');
 ```
 */
function assistantTextOfLine(line: string,): string {
  if (!line.includes(ASSISTANT_ENTRY_MARKER,))
    return '';

  /**
   Parsed entry.
   */
  const entry = parseJsonLine(line,);
  if (!isRecord(entry,))
    return '';
  if (entry.type !== 'assistant')
    return '';

  /**
   Assistant message of the entry.
   */
  const { message, } = entry;
  if (!isRecord(message,))
    return '';

  /**
   Content blocks of the assistant message.
   */
  const { content, } = message;
  if (!Array.isArray(content,))
    return '';

  return content
    .flatMap(proseOfBlock,)
    .join('\n',);
}

/**
 Extracts a context-aware activity word from transcript path.
 
 Walks the tail entries from newest to oldest and stops at the first assistant text holding a gerund.
 
 @param transcriptPath - path to session transcript JSONL
 
 @returns capitalized activity word, or empty string when unavailable
 
 @example
 ```ts
 await readActivityWord({ transcriptPath: '/tmp/session.jsonl' });
 ```
 */
async function readActivityWord({
  transcriptPath,
}: Readonly<{
  transcriptPath: string;
}>,): Promise<string> {
  try {
    /**
     Complete JSONL lines from the end of the transcript.
     */
    const lines = await readTranscriptTailLines({ transcriptPath, },);

    for (const line of lines.toReversed()) {
      /**
       Gerund found in this entry's assistant prose.
       */
      const word = findGerundInText(assistantTextOfLine(line,),);
      if (word.length > 0)
        return word;
    }
    return '';
  }
  catch (_error: unknown) {
    return '';
  }
}

//endregion Gerund extraction

export {
  assistantTextOfLine,
  readActivityWord,
  readTranscriptTailLines,
};
