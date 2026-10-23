/**
 Tests for what a round does with a seat whose request the route refused as malformed (F5 of the re-check of the second
 fix round): the round ends with that error, before quorum and after it, and does not count the seat as a lost voice.

 A schema written with a keyword the Claude route cannot send is a fault in our own stage code and not a thing the model
 did. Counting it as a lost voice makes every Claude call of the stage fail with one warn line each while the pass goes
 on without the voice, which is the outcome the refusal at build time was meant to replace. An ordinary failure of a
 call stays a lost voice, which the last case pins.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { setImmediate as turnOfTheLoop, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  ClaudeSchemaKeywordError,
  type ChatJsonOutcome,
  type ChatJsonRequest,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runGatherRound, } from '../dist/final/node/index.mjs';
import {
  capturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

/**
 Seat whose request the route refuses.
 */
const FAULTY_SEAT = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Roster the rounds ask, named from the catalog because model identifiers are never invented.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  FAULTY_SEAT,
];

/**
 Client answering every seat at once but one, whose call fails with the given error after the round has had a turn of
 the event loop.

 @param failing - error the faulty seat's call fails with

 @returns Client the round can drive

 @example
 ```ts
 const client = clientFailingOne({ failing: new Error('refused', ), },);
 ```
 */
function clientFailingOne({ failing, }: { readonly failing: Error; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (request.modelId === FAULTY_SEAT) {
        await turnOfTheLoop();
        throw failing;
      }

      /**
       Scripted payload for an answering call.
       */
      const scripted: unknown = { meow: request.modelId, };
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Runs one round over the roster with one seat failing.

 @param failing - error the faulty seat's call fails with

 @param heardNeeded - voices the round waits for before its grace opens

 @param said - array the round's lines land in

 @returns What the round rejected with, undefined when it returned

 @example
 ```ts
 const refusal = await roundWith({ failing, heardNeeded: 3, said, },);
 ```
 */
async function roundWith(
  {
    failing,
    heardNeeded,
    said,
  }: {
    readonly failing: Error;
    readonly heardNeeded: number;
    readonly said: string[];
  },
): Promise<unknown> {
  try {
    await runGatherRound({
      client: clientFailingOne({ failing, },),
      modelIds: ROSTER,
      messages: [{ role: 'user', content: 'meow', },],
      signal: new AbortController().signal,
      exchangeTimeoutMs: HANG_STOP_MS,
      responseFormat: MEOW_FORMAT,
      validate: isMeowReply,
      stage: 'cat-stage',
      l: capturingLogger({ messages: said, },),
      heardNeeded,
      graceMs: HANG_STOP_MS,
    },);
    return undefined;
  }
  catch (error) {
    return error;
  }
}

/**
 Refusal the Claude route raises for a schema that writes a keyword it cannot send.
 */
const SCHEMA_REFUSAL = new ClaudeSchemaKeywordError({ refusedKeyword: 'oneOf', schemaPath: 'root.pick', },);

await describe({
  name: 'a round whose seat the route refused as malformed',
  children: [
    it({
      name: 'ENDS WITH THE ROUTE\'S REFUSAL WHEN THE ROUND WAITS FOR EVERY SEAT, and writes no lost voice for it',
      fn: async () => {
        const said: string[] = [];
        const refusal = await roundWith({ failing: SCHEMA_REFUSAL, heardNeeded: ROSTER.length, said, },);

        expect(refusal,).toBe(SCHEMA_REFUSAL,);
        expect(said.some(function isLoss(line,): boolean {
          return line.includes('voice lost',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'ENDS WITH THE ROUTE\'S REFUSAL EVEN AFTER QUORUM STANDS, where the straggler window would otherwise '
        + 'settle the seat quietly as a voice it stopped waiting for',
      fn: async () => {
        const said: string[] = [];
        const refusal = await roundWith({ failing: SCHEMA_REFUSAL, heardNeeded: 1, said, },);

        expect(refusal,).toBe(SCHEMA_REFUSAL,);
        expect(said.some(function isAbandon(line,): boolean {
          return line.includes('abandoned',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'KEEPS AN ORDINARY FAILURE OF A CALL A LOST VOICE (the control arm): the round returns and writes the loss',
      fn: async () => {
        const said: string[] = [];
        const refusal = await roundWith({ failing: new Error('refused at once',), heardNeeded: ROSTER.length, said, },);

        expect(refusal,).toBeUndefined();
        expect(said.some(function isLoss(line,): boolean {
          return line.includes('voice lost',);
        },),).toBe(true,);
      },
    },),
  ],
},);
