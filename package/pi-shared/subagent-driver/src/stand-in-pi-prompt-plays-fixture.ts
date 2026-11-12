/**
 Which play a test's scenario gets: the ones that take time over an answer, the ones that answer at once, and the ones about a
 failed attempt. The plays themselves are in the files beside this one.

 @module
 */

import { stdout, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { compactionPlays, } from './stand-in-pi-compaction-plays-fixture.ts';
import { scenario, } from './stand-in-pi-log-fixture.ts';
import { modelErrorPlays, } from './stand-in-pi-model-error-plays-fixture.ts';
import { rateLimitPlays, } from './stand-in-pi-rate-limit-plays-fixture.ts';
import {
  answer,
  emit,
  finishRun,
} from './stand-in-pi-records-fixture.ts';
import { scenarioPlays, } from './stand-in-pi-scenario-plays-fixture.ts';

/**
 Status the crashing scenarios exit with.
 */
export const CRASH_STATUS = 3;

/**
 Milliseconds between the halves of a record written in two pieces.
 */
const SPLIT_WRITE_MILLISECONDS = 50;

/**
 Bytes into the two-byte character where the record is cut.
 */
const SPLIT_OFFSET_AFTER_CHARACTER_START = 1;

/**
 Code point of the Unicode line separator.
 */
const LINE_SEPARATOR_CODE_POINT = 0x20_28;

/**
 A Unicode line separator, valid inside a JSON string and not a line ending there.
 */
export const LINE_SEPARATOR: string = String.fromCodePoint(LINE_SEPARATOR_CODE_POINT,);

/**
 Writes a record in two pieces, split inside a two-byte character, holding a Unicode line separator.

 @param text - assistant's reply, which must hold the two-byte character
 */
async function emitFinalReplyInTwoPieces(text: string,): Promise<void> {
  /**
   The record's bytes.
   */
  const bytes = Buffer.from(
    `${JSON.stringify({
      type: 'message_end',
      message: {
        role: 'assistant',
        content: [{
          type: 'text',
          text,
        },],
      },
    },)}\n`,
  );
  /**
   Where the record is cut.
   */
  const cutAt = bytes.indexOf(Buffer.from('é',),) + SPLIT_OFFSET_AFTER_CHARACTER_START;
  stdout.write(bytes.subarray(
    0,
    cutAt,
  ),);
  await wait(SPLIT_WRITE_MILLISECONDS,);
  stdout.write(bytes.subarray(cutAt,),);
}

/**
 Scenarios that take time over a prompt or a steer.
 */
const waitingPromptPlays: ReadonlyMap<string, (id: unknown,) => Promise<void>> = new Map<string, (id: unknown,) => Promise<void>>([
  [
    'split-record',
    async function splitRecord(id,) {
      answer({ id, },);
      emit({ type: 'agent_start', },);
      await emitFinalReplyInTwoPieces(`café line${LINE_SEPARATOR}separator`,);
      emit({ type: 'agent_settled', },);
    },
  ],
],);

/**
 Plays out a prompt or a steer according to the scenario.

 @param id - command's identifier

 @example
 ```ts
 await playPrompt('c1',);
 ```
 */
export async function playPrompt(id: unknown,): Promise<void> {
  /**
   The scenario's play if it compacts the session first.
   */
  const compactionPlay = compactionPlays.get(scenario,);
  if (compactionPlay !== undefined) {
    // Not awaited: Pi answers other commands while its compaction runs, so the stand-in must go on reading them.
    compactionPlay(id,);
    return;
  }
  /**
   The scenario's play if it takes time.
   */
  const waitingPlay = waitingPromptPlays.get(scenario,);
  if (waitingPlay !== undefined) {
    await waitingPlay(id,);
    return;
  }
  /**
   The scenario's play if it has one, whether it answers at once, fails first, or meets a rate limit.
   */
  const play = scenarioPlays.get(scenario,)
    ?? modelErrorPlays.get(scenario,)
    ?? rateLimitPlays.get(scenario,);
  if (play !== undefined) {
    play(id,);
    return;
  }
  answer({ id, },);
  finishRun('done',);
}
