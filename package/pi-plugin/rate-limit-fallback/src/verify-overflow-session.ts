/**
 The three-turn session whose third request outgrows a stand-in member's enforced window, and whose
 after-compaction retry fits inside it.
 @module
 */

import assert from 'node:assert/strict';
import { join, } from 'node:path';
import { ENFORCED_CHARS, } from './verify-stand-in-member-extension.ts';
import {
  assistantMessages,
  runPiSession,
  type PiMessage,
  type PiRun,
} from './verify-pi-command-line-session.ts';

//region The overflow session and its control

/**
 Characters each padded prompt carries, as a share of the enforced window.

 The third request is the three prompts together, so their shares sum above one; the turn 3 prompt
 alone stays under Pi's recent-transcript budget, so compaction cuts before it and keeps a tail every
 member can accept.
 */
const FIRST_TURN_SHARE = 0.4;

/**
 Share of the enforced window the second prompt carries.
 */
const SECOND_TURN_SHARE = 0.31;

/**
 Share of the enforced window the third prompt carries.
 */
const THIRD_TURN_SHARE = 0.34;

/**
 Shares every turn's prompt takes from the enforced window, in order.
 */
const TURN_SHARES = [
  FIRST_TURN_SHARE,
  SECOND_TURN_SHARE,
  THIRD_TURN_SHARE,
] as const;

/**
 Characters each prompt carries, derived from the window the stand-in enforces.

 @returns one character count per turn

 @example
 ```ts
 turnChars()[2];
 ```
 */
function turnChars(): number[] {
  return TURN_SHARES.map(function characters(share,) {
    return Math.round(ENFORCED_CHARS * share,);
  },);
}

/**
 Prompts the three turns send, labelled so the transcript shows which turn failed.

 @param chars - one character count per turn

 @returns one padded prompt per turn

 @example
 ```ts
 overflowPrompts(turnChars());
 ```
 */
function overflowPrompts(chars: readonly number[],): string[] {
  /**
   Filler one padded prompt is built from.
   */
  const filler = 'The stand-in transcript pads this turn so the request can outgrow a window. ';
  return chars.map(function prompt(
    count,
    index,
  ) {
    /**
     Filler repeated enough to cover the turn, before the exact trim.
     */
    const padding = filler.repeat(Math.ceil(count / filler.length,),)
      .slice(
      0,
      count,
      );
    return `Turn ${index + 1}. ${padding}`;
  },);
}

/**
 Run one three-turn overflow session and return its last turn.

 @param home - disposable home the session runs in

 @param sessionId - session the three prompts share

 @param model - model the session selects

 @returns the last turn's run and each turn's assistant messages

 @example
 ```ts
 await overflowSession({ home, sessionId: 'hybrid', model: WRITER_SELECTION });
 ```
 */
export async function overflowSession({
  home,
  sessionId,
  model,
}: {
  readonly home: string;
  readonly sessionId: string;
  readonly model: string;
},): Promise<{
  readonly run: PiRun;
  readonly turns: readonly (readonly PiMessage[])[];
}> {
  /**
   Prompts the turns send, in order.
   */
  const prompts = overflowPrompts(turnChars(),);
  /**
   Assistant messages each completed turn produced.
   */
  const turns: PiMessage[][] = [];
  for (const [index, prompt,] of prompts.entries()) {
    // The turns carry one session, so each prompt waits for the previous one to settle; the
    // suppression is documented in doc/lint/sequential-awaits.md.
    /**
     That turn's session.
     */
    // oxlint-disable-next-line no-await-in-loop -- the transcript is built by one turn after another
    const run = await runPiSession({
      home,
      behaviour: 'overflow',
      args: [
        '--mode',
        'json',
        '--model',
        model,
        '--no-tools',
        '--no-context-files',
        '--no-skills',
        '--no-prompt-templates',
        '--no-mcp',
        '--session-dir',
        join(
          home,
          'sessions',
        ),
        '--session-id',
        sessionId,
        prompt,
      ],
    },);
    turns.push(assistantMessages(run.records,),);
    if (index === (prompts.length - 1))
      return {
        run,
        turns,
      };
    assert.equal(
      run.exitCode,
      0,
      `overflow turn ${index + 1}: Pi exited ${run.exitCode}: ${run.stderr}`,
    );
  }
  throw new Error('The overflow session ran no turn',);
}

//endregion The overflow session and its control
