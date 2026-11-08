/**
 What the driver knows about a run: the feed it writes as events arrive, and the counts that decide when the run is over.

 @module
 */

import { createWriteStream, } from 'node:fs';
import { finished, } from 'node:stream/promises';
import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import type {
  EventObservation,
  PiRecord,
  RecordOfPiRun,
} from './driver-types.ts';
import {
  preview,
  textOfParts,
  thinkingLengthOf,
} from './previews-of-pi-messages.ts';
import {
  fieldsOf,
  recordsIn,
  textOf,
} from './read-pi-fields.ts';

/**
 The dialog methods that wait for an answer; the other interface methods need none.
 */
const DIALOG_METHODS: ReadonlySet<string> = new Set([
  'select',
  'confirm',
  'input',
  'editor',
],);

/**
 The events worth a feed line of their own detail.
 */
const NOTED_EVENT_TYPES: ReadonlySet<string> = new Set([
  'auto_retry_start',
  'compaction_start',
  'queue_update',
],);

/**
 Starts the record of a run.

 @param feedFile - file that receives one JSON line per noted event

 @returns the record

 @example
 ```ts
 const record = startRecordOfPiRun('feed.jsonl',);
 ```
 */
export function startRecordOfPiRun(feedFile: string,): RecordOfPiRun {
  /**
   When the run started.
   */
  const startedAt = Date.now();
  /**
   Where feed lines go.
   */
  const feed = createWriteStream(
    feedFile,
    { flags: 'a', },
  );
  /**
   How many times each tool started.
   */
  const toolHistogram = new Map<string, number>();
  /**
   The tools that failed.
   */
  const toolErrors: PiRecord[] = [];
  /**
   The dialogs declined.
   */
  const dialogs: PiRecord[] = [];
  /**
   What changes during the run.
   */
  const state: {
    running: boolean;
    settled: number;
    turns: number;
    lastActivityAt: number;
    lastAssistantText: string;
    feedProblem?: string;
    driverError?: string;
  } = {
    running: false,
    settled: 0,
    turns: 0,
    lastActivityAt: startedAt,
    lastAssistantText: '',
  };

  feed.on(
    'error',
    function onFeedError({ message, }: { readonly message: string; },) {
      state.feedProblem = message;
    },
  );

  /**
   Writes one feed line.

   @param entry - fields of the line
   */
  function note(entry: PiRecord,): void {
    state.lastActivityAt = Date.now();
    if (state.feedProblem === undefined) {
      feed.write(`${JSON.stringify({
        t: Date.now() - startedAt,
        ...entry,
      },)}\n`,);
    }
  }

  /**
   Counts and notes a tool starting.

   @param event - `tool_execution_start` record
   */
  function observeToolStart(event: PiRecord,): void {
    /**
     The tool's name.
     */
    const tool = textOf(event.toolName,);
    toolHistogram.set(
      tool,
      (toolHistogram.get(tool,) ?? 0) + 1,
    );
    note({
      kind: 'tool_start',
      tool,
      args: preview({
        value: event.args,
        length: FEED_PREVIEW_CHARACTERS.toolArguments,
      },),
    },);
  }

  /**
   Notes a tool ending, and keeps its error if it failed.

   @param event - `tool_execution_end` record
   */
  function observeToolEnd(event: PiRecord,): void {
    /**
     Whether the tool failed.
     */
    const isError = event.isError === true;
    note({
      kind: 'tool_end',
      tool: event.toolName,
      isError,
      result: preview({
        value: event.result,
        length: FEED_PREVIEW_CHARACTERS.toolResult,
      },),
    },);
    if (isError) {
      toolErrors.push({
        t: Date.now() - startedAt,
        tool: event.toolName,
        result: preview({
          value: event.result,
          length: FEED_PREVIEW_CHARACTERS.toolError,
        },),
      },);
    }
  }

  /**
   Notes an assistant message and keeps its text.

   @param event - `message_end` record
   */
  function observeMessageEnd(event: PiRecord,): void {
    /**
     The finished message.
     */
    const message = fieldsOf(event.message,);
    if (message.role !== 'assistant') {
      return;
    }
    /**
     The message's parts.
     */
    const parts = recordsIn(message.content,);
    /**
     The message's text.
     */
    const text = textOfParts(parts,);
    /**
     The message's usage.
     */
    const usage = fieldsOf(message.usage,);
    if (text !== '') {
      state.lastAssistantText = text;
    }
    note({
      kind: 'assistant',
      stop: message.stopReason,
      text: preview({
        value: text,
        length: FEED_PREVIEW_CHARACTERS.assistantText,
      },),
      thinkingChars: thinkingLengthOf(parts,),
      toolCalls: parts.filter(function isToolCall(part,) {
        return part.type === 'toolCall';
      },)
        .length,
      usage: {
        input: usage.input,
        output: usage.output,
        cost: fieldsOf(usage.cost,)
          .total,
      },
      error: message.errorMessage ?? null,
    },);
  }

  /**
   Declines a dialog, which would otherwise wait forever, and leaves the other interface requests alone.

   @param event - `extension_ui_request` record

   @param declineDialog - how to decline
   */
  function observeInterfaceRequest({
    event,
    declineDialog,
  }: EventObservation,): void {
    /**
     The interface method asked for.
     */
    const method = textOf(event.method,);
    if (!DIALOG_METHODS.has(method,)) {
      return;
    }
    /**
     The dialog's title for the record.
     */
    const title = preview({
      value: event.title,
      length: FEED_PREVIEW_CHARACTERS.dialogTitle,
    },);
    dialogs.push({
      t: Date.now() - startedAt,
      method,
      title,
      message: preview({
        value: event.message,
        length: FEED_PREVIEW_CHARACTERS.dialogMessage,
      },),
    },);
    note({
      kind: 'dialog',
      method,
      title,
    },);
    declineDialog(event.id,);
  }

  /**
   What each kind of event changes.
   */
  const eventHandlers: ReadonlyMap<string, (observation: EventObservation,) => void> = new Map<
    string,
    (observation: EventObservation,) => void
    >([
      [
        'agent_start',
        function onAgentStart() {
          state.running = true;
        },
      ],
      [
        'agent_settled',
        function onAgentSettled() {
          state.running = false;
          state.settled += 1;
          note({
            kind: 'settled',
            n: state.settled,
          },);
        },
      ],
      [
        'turn_start',
        function onTurnStart() {
          state.turns += 1;
          note({
            kind: 'turn_start',
            n: state.turns,
          },);
        },
      ],
      [
        'tool_execution_start',
        function onToolStart({ event, },) {
          observeToolStart(event,);
        },
      ],
      [
        'tool_execution_end',
        function onToolEnd({ event, },) {
          observeToolEnd(event,);
        },
      ],
      [
        'message_end',
        function onMessageEnd({ event, },) {
          observeMessageEnd(event,);
        },
      ],
      [
        'extension_ui_request',
        observeInterfaceRequest,
      ],
    ],);

  return {
    isRunning: function isRunning() {
      return state.running;
    },
    settledCount: function settledCount() {
      return state.settled;
    },
    turnCount: function turnCount() {
      return state.turns;
    },
    silentMilliseconds: function silentMilliseconds() {
      return Date.now() - state.lastActivityAt;
    },
    note,
    countHandledPrompt: function countHandledPrompt() {
      state.settled += 1;
    },
    noteDriverFailure: function noteDriverFailure(failure,) {
      state.driverError = failure;
      note({
        kind: 'driver-failed',
        failure,
      },);
    },
    observe: function observe(observation,) {
      state.lastActivityAt = Date.now();
      /**
       The kind of event.
       */
      const type = textOf(observation.event
        .type,);
      if (NOTED_EVENT_TYPES.has(type,)) {
        note({
          kind: type,
          detail: preview({
            value: observation.event,
            length: FEED_PREVIEW_CHARACTERS.eventDetail,
          },),
        },);
      }
      eventHandlers.get(type,)?.(observation,);
    },
    summarize: function summarize({
      endReason,
      piExit,
      piHasExited,
      sessionStats,
    },) {
      return {
        endReason,
        ...(piHasExited ? { piExit, } : {}),
        totalMilliseconds: Date.now() - startedAt,
        turns: state.turns,
        settled: state.settled,
        toolHistogram: Object.fromEntries(toolHistogram,),
        toolErrors,
        dialogs,
        sessionStats,
        lastAssistantText: state.lastAssistantText,
        ...(state.feedProblem === undefined ? {} : { feedProblem: state.feedProblem, }),
        ...(state.driverError === undefined ? {} : { driverError: state.driverError, }),
      };
    },
    close: async function close() {
      try {
        feed.end();
        await finished(feed,);
      }
      catch (error) {
        state.feedProblem = String(error,);
      }
    },
  };
}
