/**
 How long the tool calls of one feed took, and which calls that feed still has open. A run's duration says how long the whole
 job took; its own calls say which single step is taking long.

 Feed lines carry `t`, the milliseconds since the run started, so a call's duration is the difference between its start and its
 end, and an open call's age is the run's elapsed time minus its start.

 @module
 */

import { createReadStream, } from 'node:fs';
import { createInterface, } from 'node:readline';
import {
  isRecord,
  textOf,
} from './read-pi-fields.ts';

/**
 A line of a feed this reader cannot use, as a symbol so no call's kind or fields can be mistaken for it.
 */
const NOT_A_CALL_LINE: unique symbol = Symbol('feed line is not a tool call',);

/**
 An unreadable line of a feed, as a symbol so no record can be mistaken for it.
 */
const NOT_A_RECORD: unique symbol = Symbol('feed line holds no JSON object',);

/**
 A call a feed started and has not ended.
 */
export type OpenToolCall = {
  /**
   The tool the call runs.
   */
  readonly tool: string;

  /**
   When the call started, as milliseconds since the run started.
   */
  readonly startedMilliseconds: number;

  /**
   The arguments the feed kept for the call, as text.
   */
  readonly argumentsPreview: string;
};

/**
 The tool calls of one feed.
 */
export type ToolCallTimes = {
  /**
   How long each call that ended took, in milliseconds. A call the feed started and never ended is not one of them: a run that
   was killed taught nothing about how long its last call would have taken.
   */
  readonly closedMilliseconds: readonly number[];

  /**
   The calls the feed started and has not ended, oldest first, so the first one is the one that has been open longest.
   */
  readonly openCalls: readonly OpenToolCall[];
};

/**
 Which of a feed's two call lines this is, or that it is neither.

 @param kind - the line's kind field

 @returns the kind, or {@link NOT_A_CALL_LINE}

 @example
 ```ts
 const kind = callKindOf('tool_start',);
 ```
 */
function callKindOf(kind: unknown,): 'tool_start' | 'tool_end' | typeof NOT_A_CALL_LINE {
  if (kind === 'tool_start') {
    return 'tool_start';
  }
  return (kind === 'tool_end') ? 'tool_end' : NOT_A_CALL_LINE;
}

/**
 Reads one line of a feed.

 @param line - one line of the feed file

 @returns the line's fields, or {@link NOT_A_RECORD} for a line that is not a JSON object, such as the cut one a running feed
 ends with

 @example
 ```ts
 const record = recordOfLine('{"kind":"tool_start"}',);
 ```
 */
function recordOfLine(line: string,): Readonly<Record<string, unknown>> | typeof NOT_A_RECORD {
  try {
    /**
     The line as it was written.
     */
    const parsed: unknown = JSON.parse(line,);
    return isRecord(parsed,) ? parsed : NOT_A_RECORD;
  }
  catch (error) {
    // A feed being written ends in a line that may be cut in half, and a line this reader cannot read says nothing about the
    // call either way.
    if (error instanceof SyntaxError) {
      return NOT_A_RECORD;
    }
    throw error;
  }
}

/**
 Collects what one feed says about its calls.

 @param feedFile - the feed of one run

 @returns the closed durations and the calls still open

 @throws the error the file gave, such as a missing feed

 @example
 ```ts
 const times = await toolCallTimesOfFeed({ feedFile: '/runs/a-1/feed.jsonl', },);
 ```
 */
export async function toolCallTimesOfFeed({
  feedFile,
}: {
  readonly feedFile: string;
}): Promise<ToolCallTimes> {
  /**
   The calls each tool has started and not yet ended, in the order they started. One list per tool, because two calls of one
   tool can overlap and their lines still pair in order.
   */
  const openByTool = new Map<string, OpenToolCall[]>();
  /**
   How long every call that ended took.
   */
  const closedMilliseconds: number[] = [];
  /**
   The feed's lines, read as they come rather than all at once, so a long run's feed costs the durations and not its text.
   */
  const lines = createInterface({
    input: createReadStream(
      feedFile,
      { encoding: 'utf8', },
    ),
    crlfDelay: Number.POSITIVE_INFINITY,
  },);
  for await (const line of lines) {
    if (line === '') {
      continue;
    }
    /**
     The line's fields.
     */
    const record = recordOfLine(line,);
    if ((typeof record) === 'symbol') {
      continue;
    }
    /**
     When the line was written.
     */
    const at = record.t;
    if (((typeof at) !== 'number') || (!Number.isFinite(at,))) {
      continue;
    }
    /**
     Which call line this is.
     */
    const kind = callKindOf(record.kind,);
    /**
     The tool the line names.
     */
    const tool = textOf(record.tool,);
    if (((typeof kind) === 'symbol') || (tool === '')) {
      continue;
    }
    if (kind === 'tool_start') {
      /**
       The calls of this tool that are still open.
       */
      const open = openByTool.get(tool,) ?? [];
      open.push({
        tool,
        startedMilliseconds: at,
        argumentsPreview: textOf(record.args,),
      },);
      openByTool.set(
        tool,
        open,
      );
      continue;
    }
    /**
     The call of this tool that this line ends, when its start was seen.
     */
    const started = openByTool.get(tool,)
      ?.shift();
    if (started !== undefined) {
      closedMilliseconds.push(at - started.startedMilliseconds,);
    }
  }
  return {
    closedMilliseconds,
    openCalls: [...openByTool.values(),].flat()
      .toSorted(function byStart(
        left,
        right,
      ) {
        return left.startedMilliseconds - right.startedMilliseconds;
      },),
  };
}
