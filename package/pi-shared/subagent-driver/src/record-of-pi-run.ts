/**
 What the driver knows about a run: the feed it writes as events arrive, and the counts that decide when the run is over.

 @module
 */

import { appendFileSync, } from 'node:fs';
import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import type {
  EndReason,
  PiExit,
  PiRecord,
  RunSummary,
} from './driver-types.ts';

/**
 The dialog methods that wait for an answer; the other interface methods need none.
 */
const DIALOG_METHODS: ReadonlySet<string> = new Set(['select', 'confirm', 'input', 'editor',],);

/**
 The events worth a feed line of their own detail.
 */
const NOTED_EVENT_TYPES: ReadonlySet<string> = new Set(['auto_retry_start', 'compaction_start', 'queue_update',],);

/**
 Shortens a value for a feed line, saying how long it was.

 @param request - value and longest text to keep

 @returns the value as text, cut if long

 @example
 ```ts
 const text = preview({ value: 'abc', length: 2, },);
 ```
 */
export function preview({ value, length, }: { readonly value: unknown; readonly length: number; },): string {
  /**
   The value as text.
   */
  const text = (typeof value === 'string') ? value : (JSON.stringify(value,) ?? '');
  return (text.length > length) ? `${text.slice(0, length,)}...[${String(text.length,)}]` : text;
}

/**
 Reads a field that holds an object.

 @param value - field's value

 @returns the object, or an empty one when field holds anything else
 */
function asRecord(value: unknown,): PiRecord {
  return ((typeof value === 'object') && (value !== null)) ? value as PiRecord : {};
}

/**
 Reads a field that holds a list of objects.

 @param value - field's value

 @returns the list, or an empty one when field holds anything else
 */
function asRecordList(value: unknown,): readonly PiRecord[] {
  return Array.isArray(value,) ? value as readonly PiRecord[] : [];
}

/**
 The feed and the counts of one run.
 */
export class RecordOfPiRun {
  readonly #feedFile: string;
  readonly #startedAt = Date.now();
  readonly #toolHistogram = new Map<string, number>();
  readonly #toolErrors: PiRecord[] = [];
  readonly #dialogs: PiRecord[] = [];
  #running = false;
  #settled = 0;
  #turns = 0;
  #lastActivityAt = Date.now();
  #lastAssistantText = '';

  /**
   Starts a record.

   @param feedFile - file that receives one JSON line per noted event

   @example
   ```ts
   const record = new RecordOfPiRun('feed.jsonl',);
   ```
   */
  constructor(feedFile: string,) {
    this.#feedFile = feedFile;
  }

  /**
   Whether Pi is at work. Only `agent_settled` clears this: `agent_end` ends one low-level run, and retries or follow-up work can
   still come.

   @returns true from `agent_start` until `agent_settled`
   */
  get running(): boolean {
    return this.#running;
  }

  /**
   How many times Pi has come to rest.

   @returns the count, including prompts Pi handled without a run
   */
  get settled(): number {
    return this.#settled;
  }

  /**
   How many model turns have started.

   @returns the count
   */
  get turns(): number {
    return this.#turns;
  }

  /**
   How long Pi has been silent.

   @returns milliseconds since the last event
   */
  get silentMilliseconds(): number {
    return Date.now() - this.#lastActivityAt;
  }

  /**
   Writes one feed line.

   @param entry - fields of the line

   @example
   ```ts
   record.note({ kind: 'state', model: 'x', },);
   ```
   */
  note(entry: PiRecord,): void {
    appendFileSync(this.#feedFile, `${JSON.stringify({ t: Date.now() - this.#startedAt, ...entry, },)}\n`,);
  }

  /**
   Counts a prompt Pi handled without starting a run, for which no `agent_settled` will come.

   @example
   ```ts
   record.countHandledPrompt();
   ```
   */
  countHandledPrompt(): void {
    this.#settled += 1;
  }

  /**
   Takes in one event.

   @param request - event, and how to decline a dialog so it cannot stall the session

   @example
   ```ts
   record.observe({ event, declineDialog: (id,) => { connection.write({ id, cancelled: true, },); }, },);
   ```
   */
  observe(
    { event, declineDialog, }: { readonly event: PiRecord; readonly declineDialog: (id: unknown,) => void; },
  ): void {
    this.#lastActivityAt = Date.now();
    /**
     The kind of event.
     */
    const type = String(event.type,);
    switch (type) {
      case 'agent_start': {
        this.#running = true;
        return;
      }
      case 'agent_settled': {
        this.#running = false;
        this.#settled += 1;
        this.note({ kind: 'settled', n: this.#settled, },);
        return;
      }
      case 'turn_start': {
        this.#turns += 1;
        this.note({ kind: 'turn_start', n: this.#turns, },);
        return;
      }
      case 'tool_execution_start': {
        this.#observeToolStart(event,);
        return;
      }
      case 'tool_execution_end': {
        this.#observeToolEnd(event,);
        return;
      }
      case 'message_end': {
        this.#observeMessageEnd(event,);
        return;
      }
      case 'extension_ui_request': {
        this.#observeInterfaceRequest({ event, declineDialog, },);
        return;
      }
      default: {
        if (NOTED_EVENT_TYPES.has(type,)) {
          this.note({ kind: type, detail: preview({ value: event, length: FEED_PREVIEW_CHARACTERS.eventDetail, },), },);
        }
      }
    }
  }

  /**
   Counts and notes a tool starting.

   @param event - `tool_execution_start` record
   */
  #observeToolStart(event: PiRecord,): void {
    /**
     The tool's name.
     */
    const tool = String(event.toolName,);
    this.#toolHistogram.set(tool, (this.#toolHistogram.get(tool,) ?? 0) + 1,);
    this.note({ kind: 'tool_start', tool, args: preview({ value: event.args, length: FEED_PREVIEW_CHARACTERS.toolArguments, },), },);
  }

  /**
   Notes a tool ending, and keeps its error if it failed.

   @param event - `tool_execution_end` record
   */
  #observeToolEnd(event: PiRecord,): void {
    /**
     Whether the tool failed.
     */
    const isError = event.isError === true;
    this.note({
      kind: 'tool_end',
      tool: event.toolName,
      isError,
      result: preview({ value: event.result, length: FEED_PREVIEW_CHARACTERS.toolResult, },),
    },);
    if (isError) {
      this.#toolErrors.push({
        t: Date.now() - this.#startedAt,
        tool: event.toolName,
        result: preview({ value: event.result, length: FEED_PREVIEW_CHARACTERS.toolError, },),
      },);
    }
  }

  /**
   Notes an assistant message and keeps its text.

   @param event - `message_end` record
   */
  #observeMessageEnd(event: PiRecord,): void {
    /**
     The finished message.
     */
    const message = asRecord(event.message,);
    if (message.role !== 'assistant') {
      return;
    }
    /**
     The message's parts.
     */
    const parts = asRecordList(message.content,);
    /**
     The message's text.
     */
    const text = parts.filter((part,) => part.type === 'text').map((part,) => String(part.text ?? '',)).join('\n',);
    /**
     The message's usage.
     */
    const usage = asRecord(message.usage,);
    if (text !== '') {
      this.#lastAssistantText = text;
    }
    this.note({
      kind: 'assistant',
      stop: message.stopReason,
      text: preview({ value: text, length: FEED_PREVIEW_CHARACTERS.assistantText, },),
      thinkingChars: parts.filter((part,) => part.type === 'thinking').reduce(
        (sum, part,) => sum + String(part.thinking ?? '',).length,
        0,
      ),
      toolCalls: parts.filter((part,) => part.type === 'toolCall').length,
      usage: { input: usage.input, output: usage.output, cost: asRecord(usage.cost,).total, },
      error: message.errorMessage ?? null,
    },);
  }

  /**
   Declines a dialog, which would otherwise wait forever, and leaves the other interface requests alone.

   @param request - event and how to decline
   */
  #observeInterfaceRequest(
    { event, declineDialog, }: { readonly event: PiRecord; readonly declineDialog: (id: unknown,) => void; },
  ): void {
    /**
     The interface method asked for.
     */
    const method = String(event.method,);
    if (!DIALOG_METHODS.has(method,)) {
      return;
    }
    /**
     The dialog's title for the record.
     */
    const title = preview({ value: event.title, length: FEED_PREVIEW_CHARACTERS.dialogTitle, },);
    this.#dialogs.push({
      t: Date.now() - this.#startedAt,
      method,
      title,
      message: preview({ value: event.message, length: FEED_PREVIEW_CHARACTERS.dialogMessage, },),
    },);
    this.note({ kind: 'dialog', method, title, },);
    declineDialog(event.id,);
  }

  /**
   Builds the summary of the finished run.

   @param request - how the run ended, how Pi exited, and Pi's session statistics

   @returns what `summary.json` holds

   @example
   ```ts
   const summary = record.summarize({ endReason: 'idle', piExit: null, sessionStats: {}, },);
   ```
   */
  summarize(
    { endReason, piExit, sessionStats, }: { readonly endReason: EndReason; readonly piExit: PiExit | null; readonly sessionStats: unknown; },
  ): RunSummary {
    return {
      endReason,
      piExit,
      totalMilliseconds: Date.now() - this.#startedAt,
      turns: this.#turns,
      settled: this.#settled,
      toolHistogram: Object.fromEntries(this.#toolHistogram,),
      toolErrors: this.#toolErrors,
      dialogs: this.#dialogs,
      sessionStats,
      lastAssistantText: this.#lastAssistantText,
    };
  }
}
