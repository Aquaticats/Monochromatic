/**
 Test for a wide-judge gather after a reasoning runaway, over the cards of the roster as they stand (ledger B1079).

 THE BENCH IS BUILT FROM THE CARDS, not listed, so the case stays true as cards move: every chat model no wide-judge
 hold keeps out of the wide seats. The models left are the ones the cards drop from those seats, which the roster-wide
 role used to ask first, so a runaway critic, panelist or select judge was re-asked of a model that was never one of
 the wide seats.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  holdsKeepingOutOf,
  isDecisionSeat,
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { gatherStageVoices, } from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Wide judge runaway tests

/**
 Chat models the cards drop from the wide seats, which no wide gather seats.
 */
const HELD_OUT_OF_WIDE_SEATS: readonly RosterModelId[] = ROSTER_MODEL_IDS.filter(
  function heldOut(modelId,): boolean {
    return (!isDecisionSeat({ modelId, },))
      && (holdsKeepingOutOf({
        modelId,
        role: 'wide-judge',
      },).length
        > 0);
  },
);

/**
 Chat models no wide-judge hold keeps out of the wide seats: the bench of a wide gather.
 */
const WIDE_BENCH: readonly RosterModelId[] = ROSTER_MODEL_IDS.filter(
  function seated(modelId,): boolean {
    return (!isDecisionSeat({ modelId, },)) && (!HELD_OUT_OF_WIDE_SEATS.includes(modelId,));
  },
);

await describe({
  name: `${gatherStageVoices.name} (a wide-judge runaway on the cards of the roster)`,
  concurrency: 1,
  children: [
    it({
      name: 'ASKS NO MODEL THE CARDS HOLD OUT OF THE WIDE SEATS FOR A RUNAWAY WIDE JUDGE, and asks the runaway\'s own '
        + 'model again, nudged, on a warning, when every other model is on the bench or held out',
      fn: async () => {
        // A POSITIVE CONTROL: with no model held out of the wide seats the case would show nothing.
        expect(HELD_OUT_OF_WIDE_SEATS.length > 0,).toBe(true,);
        expect(WIDE_BENCH.includes(SEAT_HYPER_OPENROUTER_UNMEASURED,),).toBe(true,);

        /**
         Models asked, in order.
         */
        const asked: RosterModelId[] = [];

        /**
         Lines the gather logged.
         */
        const lines: string[] = [];

        /**
         Whether the runaway has run away yet.
         */
        let ranAway = false;

        /**
         Client whose runaway seat answers a length stop with no content once, and every other ask is answered.
         */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            asked.push(request.modelId,);
            if ((request.modelId === SEAT_HYPER_OPENROUTER_UNMEASURED) && (!ranAway)) {
              ranAway = true;
              return {
                kind: 'schema-mismatch',
                rawText: '',
                reason: 'truncated-completion',
                detail: 'provider reported a truncating completion',
              };
            }

            /**
             Scripted payload for the answering call, naming the model that wrote it.
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
        const gather = await gatherStageVoices({
          client,
          modelIds: WIDE_BENCH,
          messages: [{ role: 'user', content: 'meow', },],
          signal: new AbortController().signal,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'panel',
          l: levelCapturingLogger({ lines, },),
          graceMs: HANG_STOP_MS,
          fanOut: 'whole-bench',
          role: 'wide-judge',
        },);
        expect(asked.filter(function isHeldOut(modelId,): boolean {
          return HELD_OUT_OF_WIDE_SEATS.includes(modelId,);
        },),).toEqual([],);
        expect(asked.filter(function isRunaway(modelId,): boolean {
          return modelId === SEAT_HYPER_OPENROUTER_UNMEASURED;
        },),).toHaveLength(2,);
        expect(gather.voices.find(function isRunawaySeat(voice,): boolean {
          return voice.modelId === SEAT_HYPER_OPENROUTER_UNMEASURED;
        },)?.value.meow,).toBe(SEAT_HYPER_OPENROUTER_UNMEASURED,);
        expect(lines.some(function warnsOfAnEmptyList(line,): boolean {
          return line.includes('no other model could take the re-ask of a reasoning runaway (0 tried)',);
        },),).toBe(true,);
      },
    },),
  ],
},);

//endregion Wide judge runaway tests
