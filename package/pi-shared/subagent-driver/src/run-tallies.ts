/**
 What a run counts: the tools it started, the ones that failed, and the dialogs it was asked.

 @module
 */

import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import type { PiRecord, } from './driver-types.ts';
import {
  PROCESS_TOOL_NAME,
  PROCESSES_RECORD_KIND,
  processesInToolAnswer,
} from './processes-a-tool-answer-names.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { textOf, } from './read-pi-fields.ts';

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
 What a run counted.
 */
export type RunTallies = {
  readonly toolHistogram: () => Readonly<Record<string, number>>;
  readonly toolErrors: () => readonly PiRecord[];
  readonly dialogs: () => readonly PiRecord[];

  /**
   Counts and notes a tool starting.
   */
  readonly countToolStart: (event: PiRecord,) => void;

  /**
   Notes a tool ending, and keeps its error if it failed.
   */
  readonly noteToolEnd: (event: PiRecord,) => void;

  /**
   Notes a dialog and declines it, so it cannot wait for an answer forever.
   */
  readonly declineDialog: (observation: {
    readonly event: PiRecord;
    readonly declineDialog: (id: unknown,) => void;
  },) => void;
};

/**
 Starts counting what a run does.

 @param note - where a feed line goes

 @param millisecondsSoFar - how long the run has lasted, for the records kept in the summary

 @returns the counts, and where each kind of record goes

 @example
 ```ts
 const tallies = startRunTallies({ note, millisecondsSoFar: () => 0, },);
 ```
 */
export function startRunTallies({
  note,
  millisecondsSoFar,
}: {
  readonly note: (entry: PiRecord,) => void;
  readonly millisecondsSoFar: () => number
},): RunTallies {
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
  return {
    toolHistogram: function toolHistogramOfRun() {
      return Object.fromEntries(toolHistogram,);
    },
    toolErrors: function toolErrorsOfRun() {
      return toolErrors;
    },
    dialogs: function dialogsOfRun() {
      return dialogs;
    },
    countToolStart: function countToolStart(event,) {
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
    },
    noteToolEnd: function noteToolEnd(event,) {
      /**
       Whether the tool failed.
       */
      const isError = event.isError === true;
      /**
       The tool's name.
       */
      const tool = textOf(event.toolName,);
      note({
        kind: 'tool_end',
        tool: event.toolName,
        isError,
        result: preview({
          value: event.result,
          length: FEED_PREVIEW_CHARACTERS.toolResult,
        },),
      },);
      // An answer that names processes is kept whole as a record of its own, because the line above holds only a preview of it: text
      // cut inside a JSON string cannot be read back, and the driver asks the feed which of a run's processes still run.
      /**
       What this answer says about the processes, if anything.
       */
      const processes = (isError || (tool !== PROCESS_TOOL_NAME))
        ? []
        : processesInToolAnswer({ answer: event.result, },);
      if (processes.length > 0) {
        note({
          kind: PROCESSES_RECORD_KIND,
          processes,
        },);
      }
      if (isError) {
        toolErrors.push({
          t: millisecondsSoFar(),
          tool: event.toolName,
          result: preview({
            value: event.result,
            length: FEED_PREVIEW_CHARACTERS.toolError,
          },),
        },);
      }
    },
    declineDialog: function declineDialog({
      event,
      declineDialog: decline,
    },) {
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
        t: millisecondsSoFar(),
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
      decline(event.id,);
    },
  };
}
