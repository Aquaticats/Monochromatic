/**
 What a run's events change: the tools it started, the dialogs it was asked, the reply it ended on, and whether it is at work.

 @module
 */

import {
  FEED_PREVIEW_CHARACTERS,
  MODEL_ERROR_WITHOUT_TEXT,
  NO_RUN_FAILURE,
} from './driver-constants.ts';
import type {
  EventObservation,
  PiRecord,
  RunFailure,
} from './driver-types.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { startRunTallies, } from './run-tallies.ts';
import {
  assistantMessageOf,
  isAssistantMessageEnd,
} from './read-assistant-message.ts';
import {
  fieldsOf,
  textOf,
} from './read-pi-fields.ts';

/**
 The events worth a feed line of their own detail: a compaction of the session, how it ended, and the other waits a run makes a
 reader sit through.
 */
const NOTED_EVENT_TYPES: ReadonlySet<string> = new Set([
  'auto_retry_start',
  'compaction_start',
  'compaction_end',
  'queue_update',
],);

/**
 What Pi reports of its own session, measured on the run's own clock rather than the epoch's.
 */
type CompactionInTheRun =
  | { readonly compacting: false }
    | {
      readonly compacting: true;

    /**
     Why Pi began the compaction, as `compaction_start` reports it.
     */
      readonly reason: string;

    /**
     How long the run had lasted when it saw the compaction begin.
     */
      readonly sinceMilliseconds: number;
    };

/**
 What one run has seen.
 */
export type ObservedPiRun = {
  readonly isRunning: () => boolean;
  readonly settledCount: () => number;
  readonly turnCount: () => number;
  readonly lastAssistantText: () => string;
  readonly unrecoveredModelError: () => RunFailure;

  /**
   What Pi reported of its own session, when that is why the run could not go on: a compaction that failed, or one still under
   way as the run ended.
   */
  readonly compactionProblem: () => RunFailure;
  readonly toolHistogram: () => Readonly<Record<string, number>>;
  readonly toolErrors: () => readonly PiRecord[];
  readonly dialogs: () => readonly PiRecord[];
  readonly observe: (observation: EventObservation,) => void;

  /**
   Counts a prompt Pi handled without starting a run, for which no `agent_settled` will come.
   */
  readonly countPromptHandled: () => void;
};

/**
 Starts watching what Pi's events say about a run.

 @param note - where a feed line goes

 @param millisecondsSoFar - how long the run has lasted, for the records kept in the summary

 @param countModelError - told once for each assistant message that ended in an error, before anything recovers it

 @param noteAnswer - told for each assistant message that did not end in an error, which ends the error a run stood on

 @returns what the run has seen, and where its events go

 @example
 ```ts
 const observed = startObservingPiRun({ note, millisecondsSoFar: () => 0, countModelError: () => {}, noteAnswer: () => {}, },);
 ```
 */
export function startObservingPiRun({
  note,
  millisecondsSoFar,
  countModelError,
  noteAnswer,
}: {
  readonly note: (entry: PiRecord,) => void;
  readonly millisecondsSoFar: () => number;
  readonly countModelError: (problem: string,) => void;
  readonly noteAnswer: () => void
},): ObservedPiRun {
  /**
   What the run counts about its tools and dialogs.
   */
  const tallies = startRunTallies({
    note,
    millisecondsSoFar,
  },);
  /**
   What changes during the run.
   */
  const state: {
    running: boolean;
    settled: number;
    turns: number;
    lastAssistantText: string;

    /**
     The error of the last assistant message, which is a model error only if nothing after it succeeds.
     */
    pendingModelError?: string;

    /**
     The error Pi settled on, which is why a run ends as a model error.
     */
    unrecoveredModelError?: string;

    /**
     What Pi reports of its own session, since what it said of the last compaction that failed.
     */
    compaction: CompactionInTheRun;

    /**
     What Pi said when its last compaction failed, which answers why a prompt was never taken.
     */
    compactionFailure?: string;
  } = {
    running: false,
    settled: 0,
    turns: 0,
    lastAssistantText: '',
    compaction: { compacting: false, },
  };

  /**
   Notes an assistant message, keeps its text, and remembers how it ended.

   @param event - `message_end` record
   */
  function observeMessageEnd(event: PiRecord,): void {
    if (!isAssistantMessageEnd(event,)) {
      return;
    }
    /**
     What the message says.
     */
    const message = assistantMessageOf(event,);
    if (message.text !== '') {
      state.lastAssistantText = message.text;
    }
    /**
     Whether the message ended in an error, which is a model error only when nothing after it recovers.
     */
    const endedInError = message.stop === 'error';
    if (endedInError) {
      state.pendingModelError = message.error === '' ? MODEL_ERROR_WITHOUT_TEXT : message.error;
      countModelError(state.pendingModelError,);
    }
    else {
      noteAnswer();
      delete state.pendingModelError;
      delete state.unrecoveredModelError;
    }
    note({
      kind: 'assistant',
      stop: message.stop,
      text: preview({
        value: message.text,
        length: FEED_PREVIEW_CHARACTERS.assistantText,
      },),
      thinkingChars: message.thinkingChars,
      toolCalls: message.parts
        .filter(function isToolCall(part,) {
          return part.type === 'toolCall';
        },)
        .length,
      usage: {
        input: message.usage
          .input,
        output: message.usage
          .output,
        cost: fieldsOf(message.usage
          .cost,)
          .total,
      },
      error: message.error === '' ? null : message.error,
    },);
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
          if (state.pendingModelError !== undefined) {
            state.unrecoveredModelError = state.pendingModelError;
          }
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
          tallies.countToolStart(event,);
        },
      ],
      [
        'tool_execution_end',
        function onToolEnd({ event, },) {
          tallies.noteToolEnd(event,);
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
        function onInterfaceRequest(observation,) {
          tallies.declineDialog(observation,);
        },
      ],
      [
        'compaction_start',
        function onCompactionStart({ event, },) {
          state.compaction = {
            compacting: true,
            reason: textOf(event.reason,),
            sinceMilliseconds: millisecondsSoFar(),
          };
        },
      ],
      [
        'compaction_end',
        function onCompactionEnd({ event, },) {
          state.compaction = { compacting: false, };
          /**
           What Pi said of the failure, which is empty for a compaction that worked.
           */
          const message = textOf(event.errorMessage,);
          if (message !== '') {
            state.compactionFailure = message;
            return;
          }
          if (event.aborted === true) {
            state.compactionFailure = 'Pi aborted the compaction';
          }
        },
      ],
    ],);

  /**
   Notes what an event says, and hands it to whatever it changes.

   @param observation - the event and how to decline a dialog
   */
  function observe(observation: EventObservation,): void {
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
  }

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
    lastAssistantText: function lastAssistantText() {
      return state.lastAssistantText;
    },
    unrecoveredModelError: function unrecoveredModelError() {
      return (state.unrecoveredModelError === undefined) ? NO_RUN_FAILURE : {
        failed: true,
        problem: state.unrecoveredModelError,
      };
    },
    compactionProblem: function compactionProblem() {
      /**
       The compaction under way, or the fact that there is none.
       */
      const { compaction } = state;
      if (compaction.compacting) {
        return {
          failed: true,
          problem: `Pi was still compacting (${compaction.reason}) after ${String(millisecondsSoFar() - compaction.sinceMilliseconds,)} ms`,
        };
      }
      /**
       What Pi said when its last compaction failed, if one did.
       */
      const { compactionFailure } = state;
      return (compactionFailure === undefined) ? NO_RUN_FAILURE : {
        failed: true,
        problem: compactionFailure,
      };
    },
    toolHistogram: tallies.toolHistogram,
    toolErrors: tallies.toolErrors,
    dialogs: tallies.dialogs,
    observe,
    countPromptHandled: function countPromptHandled() {
      state.settled += 1;
    },
  };
}
