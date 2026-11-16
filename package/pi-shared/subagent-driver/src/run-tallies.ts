/**
 What a run counts: the tools it started, the ones that failed, and the dialogs it was asked.

 @module
 */

import { FEED_PREVIEW_CHARACTERS, } from './driver-constants.ts';
import type {
  PiRecord,
  SilentCommand,
} from './driver-types.ts';
import {
  PROCESS_TOOL_NAME,
  PROCESSES_RECORD_KIND,
  processesInToolAnswer,
} from './processes-a-tool-answer-names.ts';
import { preview, } from './previews-of-pi-messages.ts';
import {
  fieldsOf,
  textOf,
} from './read-pi-fields.ts';
import { declaredBoundOf, } from './a-command-that-says-nothing.ts';

/**
 How many calls that ended are kept for the driver to name.

 One operation-wide abort ends what is in flight, and a call that starts while the abort is on the wire ends in that same window, so
 the few calls kept are every call one abort can end, with room to spare.
 */
const ENDED_CALLS_KEPT = 8;

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

  /**
   The tool commands in flight, oldest first, each with how long it has run and the bound it declared for itself. Empty when the
   run is sitting in none.
   */
  readonly commandsInFlight: () => readonly SilentCommand[];

  /**
   The calls that ended at or after a moment of the run, newest last, each with how long it had run when it ended. A driver naming
   the work an operation-wide abort ended asks from the moment it asked for that abort, because a call that starts while the abort
   is on the wire ends after it.
   */
  readonly commandsEndedSince: (endedAfterMilliseconds: number) => readonly SilentCommand[];
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
  /**
   The tool commands started and not yet ended, oldest first. A tool that starts another leaves both open, and two calls of one tool
   can be in flight at once.
   */
  const inFlight: {
    readonly toolCallId: string;
    readonly tool: string;
    readonly args: unknown;
    readonly sinceMilliseconds: number
  }[] = [];
  /**
   The calls that ended, newest last, kept to at most `ENDED_CALLS_KEPT`.
   */
  const ended: {
    readonly call: SilentCommand;
    readonly endedAtMilliseconds: number
  }[] = [];
  return {
    commandsEndedSince: function commandsEndedSince(endedAfterMilliseconds,) {
      return ended.filter(function endedAfter(entry,) {
        return entry.endedAtMilliseconds >= endedAfterMilliseconds;
      },)
        .map(function callOf(entry,) {
          return entry.call;
        },);
    },
    commandsInFlight: function commandsInFlightOfRun() {
      return inFlight.map(function running(started,) {
        return {
          ...((started.toolCallId === '') ? {} : { toolCallId: started.toolCallId, }),
          tool: started.tool,
          args: started.args,
          milliseconds: millisecondsSoFar() - started.sinceMilliseconds,
          ...declaredBoundOf(started.args,),
        };
      },);
    },
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
      inFlight.push({
        toolCallId: textOf(event.toolCallId,),
        tool,
        args: event.args,
        sinceMilliseconds: millisecondsSoFar(),
      },);
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
      /**
       The identifier Pi gave the call this ends, which is empty when it gave none.
       */
      const toolCallId = textOf(event.toolCallId,);
      /**
       The call this ends: the one Pi named. Pi names every call on its start and on its end
       (`docs/json.md` in the installed Pi), so a second call of the same tool is never taken for the first.

       A Pi that names none, or names one no start carried, leaves only the tool's name to go on, and the entry taken is the last
       one of that name still open: with two calls of one tool in flight, that can take the wrong one, which is the case the
       identifier is here to remove.
       */
      const named = (toolCallId === '')
        ? (-1)
        : inFlight.findIndex(function sameCall(entry,) {
          return entry.toolCallId === toolCallId;
        },);
      /**
       Where the command this ends was started.
       */
      const startedAt = (named === (-1))
        ? inFlight.findLastIndex(function sameTool(entry,) {
          return entry.tool === tool;
        },)
        : named;
      /**
       The call this ends, when the driver saw it start.
       */
      const started = (startedAt === (-1)) ? undefined : inFlight.splice(
        startedAt,
        1,
      )[0];
      if (started !== undefined) {
        /**
         When it ended, on the run's clock, which is what a driver naming an abort's work reads.
         */
        const endedAtMilliseconds = millisecondsSoFar();
        ended.push({
          call: {
            ...((started.toolCallId === '') ? {} : { toolCallId: started.toolCallId, }),
            tool: started.tool,
            args: started.args,
            milliseconds: endedAtMilliseconds - started.sinceMilliseconds,
            ...declaredBoundOf(started.args,),
          },
          endedAtMilliseconds,
        },);
        // A run may make thousands of calls, and only the last few are the work one abort can have ended.
        ended.splice(
          0,
          Math.max(
            0,
            ended.length - ENDED_CALLS_KEPT,
          ),
        );
      }
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
