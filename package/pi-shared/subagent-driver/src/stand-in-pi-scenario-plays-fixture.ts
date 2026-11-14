/**
 What the stand-in Pi does with a prompt or a steer in a scenario that answers at once and fails on nothing: a refused prompt, a
 prompt handled without a run, a dialog, a tool run, a tool answer longer than a feed line keeps, a noisy line, and a slow
 answer. The plays for a failed attempt are in
 `stand-in-pi-model-error-plays-fixture.ts`, and the dispatcher that chooses between all of them is in
 `stand-in-pi-prompt-plays-fixture.ts`.

 @module
 */

import { stdout, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  FEED_PREVIEW_CHARACTERS,
  POLL_MILLISECONDS,
} from '../dist/final/node/index.mjs';
import {
  seen,
  writeLogLine,
} from './stand-in-pi-log-fixture.ts';
import {
  answer,
  assistantSilence,
  assistantToolCall,
  emit,
  finishRun,
  finishRunSilently,
  FIRST_TURN_REPLY,
  GO_ON_REPLY,
  LONG_REPLY,
} from './stand-in-pi-records-fixture.ts';

/**
 How many looks at the inbox a delayed answer spans, so a driver that sends the next message without waiting for this answer has
 sent it before the answer comes.
 */
const LOOKS_A_DELAYED_ANSWER_SPANS = 3;

/**
 Milliseconds a delayed answer takes.
 */
const SLOW_ANSWER_MILLISECONDS: number = POLL_MILLISECONDS * LOOKS_A_DELAYED_ANSWER_SPANS;

/**
 Turns the many-turns scenario starts, above any cap the tests set.
 */
const MANY_TURNS = 5;

/**
 Turns a run here takes before its agent stops answering: the four tool turns the run of 2026-11-13 took before its fifth turn
 ended with no text and no tool call.
 */
const TOOL_TURNS_BEFORE_SILENCE = 4;

/**
 Which turn of those carries the words the agent began with, which is the turn a summary of that run carried as its last.
 */
const FIRST_TURN = 0;

/**
 The name the stand-in's long answer gives the process it is about, which a reader of the feed matches against.
 */
export const REPORTED_PROCESS_ID = 'proc_long1';

/**
 A `process` tool answer longer than a feed line keeps, about a process that has already ended, so the run has no work to wait
 for.

 A real answer of this kind carries the tail of a build's log, and the feed keeps only a preview of it: text cut inside a JSON
 string, which a reader that parses it back cannot read. The record the driver writes beside that line holds the answer whole.

 @returns the answer, as the tool gives it
 */
function aLongAnswerAboutEndedWork(): Readonly<Record<string, unknown>> {
  return {
    content: [{
      type: 'text',
      text: 'x'.repeat(FEED_PREVIEW_CHARACTERS.toolResult * 2,),
    },],
    details: {
      action: 'list',
      processes: [{
        id: REPORTED_PROCESS_ID,
        name: 'native-build',
        command: 'cargo build',
        cwd: '/',
        status: 'exited',
      },],
    },
  };
}

/**
 Answers a command after a delay, without holding up the reading of the next one, so a test can see whether the driver waited.

 @param id - command's identifier
 */
async function answerSlowly(id: unknown,): Promise<void> {
  await wait(SLOW_ANSWER_MILLISECONDS,);
  seen.answering -= 1;
  answer({ id, },);
}

/**
 Writes the tool turns a run takes before its agent stops answering: the first with the words the agent began with, the rest
 calling a tool and saying nothing.
 */
function emitToolTurns(): void {
  for (let turn = 0; turn < TOOL_TURNS_BEFORE_SILENCE; turn += 1) {
    emit({ type: 'turn_start', },);
    emit(assistantToolCall(turn === FIRST_TURN ? { text: FIRST_TURN_REPLY, } : {},),);
    emit({
      type: 'tool_execution_start',
      toolName: 'bash',
      args: { command: 'ls', },
    },);
    emit({
      type: 'tool_execution_end',
      toolName: 'bash',
      isError: false,
      result: 'ok',
    },);
  }
}

/**
 Writes the tool turns, and then an answer that said nothing, which is the ending a run of 2026-11-13 had.
 */
function emitToolTurnsThenSilence(): void {
  emit({ type: 'agent_start', },);
  emitToolTurns();
  emit({ type: 'turn_start', },);
  emit(assistantSilence(),);
  emit({ type: 'agent_end', },);
  emit({ type: 'agent_settled', },);
}

/**
 What each scenario that answers at once does with a prompt or a steer, apart from the ones that fail on a model error. A scenario
 missing from every map finishes a normal run.
 */
export const scenarioPlays: ReadonlyMap<string, (id: unknown,) => void> = new Map<string, (id: unknown,) => void>([
  [
    'reject',
    function rejectPrompt(id,) {
      emit({
        type: 'response',
        id,
        success: false,
        error: 'no key for this provider',
      },);
    },
  ],
  [
    'reject-inbox',
    function rejectLaterMessages(id,) {
      if (seen.prompts > 1) {
        emit({
          type: 'response',
          id,
          success: false,
          error: 'refused',
        },);
        return;
      }
      answer({ id, },);
      finishRun('done',);
    },
  ],
  [
    'handled',
    function handlePrompt(id,) {
      answer({
        id,
        data: { disposition: 'handled', },
      },);
    },
  ],
  [
    'never-settles',
    function neverSettle(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
    },
  ],
  [
    'silent-inbox',
    function acceptThenStaySilent(id,) {
      if (seen.prompts > 1) {
        return;
      }
      answer({ id, },);
      emit({ type: 'agent_start', },);
    },
  ],
  [
    'settle-gap',
    function endWithoutSettling(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      emit({ type: 'agent_end', },);
      void writeLogLine('marker agent_end',);
    },
  ],
  [
    'many-turns',
    function startManyTurns(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      for (let turn = 0; turn < MANY_TURNS; turn += 1) {
        emit({ type: 'turn_start', },);
      }
    },
  ],
  [
    'dialog',
    function askDialog(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      emit({
        type: 'extension_ui_request',
        id: 'dialog-1',
        method: 'select',
        title: 'Pick one',
        message: 'Which?',
      },);
      emit({
        type: 'extension_ui_request',
        id: 'notice-1',
        method: 'notify',
        message: 'hello',
      },);
    },
  ],
  [
    'tools',
    function useTools(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      emit({
        type: 'tool_execution_start',
        toolName: 'bash',
        args: { command: 'ls', },
      },);
      emit({
        type: 'tool_execution_end',
        toolName: 'bash',
        isError: false,
        result: 'ok',
      },);
      emit({
        type: 'tool_execution_start',
        toolName: 'bash',
        args: { command: 'false', },
      },);
      emit({
        type: 'tool_execution_end',
        toolName: 'bash',
        isError: true,
        result: 'failed',
      },);
      emit({ type: 'auto_retry_start', },);
      emit({ type: 'agent_settled', },);
    },
  ],
  [
    'empty-answer-after-tools',
    function stopAfterToolWork(id,) {
      answer({ id, },);
      if (seen.prompts > 1) {
        // The driver asked the agent to go on after an answer that said nothing, and this is the agent answering with what it
        // owed.
        finishRun(GO_ON_REPLY,);
        return;
      }
      emitToolTurnsThenSilence();
    },
  ],
  [
    'empty-answer-always',
    function answerNothingHoweverOftenAsked(id,) {
      answer({ id, },);
      if (seen.prompts === 1) {
        emitToolTurnsThenSilence();
        return;
      }
      finishRunSilently();
    },
  ],
  [
    'noise',
    function writeNoise(id,) {
      answer({ id, },);
      stdout.write('this line is not JSON\n',);
      finishRun('after the noise',);
    },
  ],
  [
    'process-answer-longer-than-a-line',
    function reportLongAnswerAboutEndedWork(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      emit({ type: 'turn_start', },);
      emit({
        type: 'tool_execution_start',
        toolName: 'process',
        args: { action: 'list', },
      },);
      emit({
        type: 'tool_execution_end',
        toolName: 'process',
        isError: false,
        result: aLongAnswerAboutEndedWork(),
      },);
      finishRun('the build ended and I read what it said',);
    },
  ],
  [
    'long-reply',
    function finishLong(id,) {
      answer({ id, },);
      finishRun(LONG_REPLY,);
    },
  ],
  [
    'crash-on-steer',
    function startThenWait(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
    },
  ],
  [
    'slow-answer',
    function answerAfterWaiting(id,) {
      seen.answering += 1;
      emit({ type: 'agent_start', },);
      void answerSlowly(id,);
    },
  ],
],);
