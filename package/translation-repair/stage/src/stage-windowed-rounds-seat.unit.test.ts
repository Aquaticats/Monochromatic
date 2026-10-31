/**
 Tests for the seat that windowed rounds name on every ask (ledger B1079, B1080): a round has no recovery round, so
 a runaway of one of its seats is re-asked inside the provider, which can honour the holds of the seat's role only if
 the ask carries the role, and can ask no model for two seats only if the asks of one question share a record.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
  ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';
import { runWindowedRounds, } from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Windowed rounds seat tests

/**
 The bench of the rounds.
 */
const BENCH: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
];

/**
 Runs the rounds over the bench with every seat answering, and returns the seat each ask carried.

 @returns Seat of each ask, in the order the asks were made

 @example
 ```ts
 const seats = await seatsOfAsks();
 ```
 */
async function seatsOfAsks(): Promise<readonly ReaskSeat[]> {
  /**
   Seat each ask carried; an ask that carried none adds nothing, so a missing seat shows as a short list.
   */
  const seats: ReaskSeat[] = [];

  /**
   Client whose every seat answers.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (request.reaskSeat !== undefined)
        seats.push(request.reaskSeat,);

      /**
       Scripted payload for the answering call.
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
  await runWindowedRounds({
    client,
    modelIds: BENCH,
    messages: [{ role: 'user', content: 'meow', },],
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    responseFormat: MEOW_FORMAT,
    validate: isMeowReply,
    stage: 'gate',
    l: levelCapturingLogger({ lines: [], },),
    graceMs: HANG_STOP_MS,
    fanOut: 'whole-bench',
    role: 'late-judge',
  },);
  return seats;
}

await describe({
  name: `${runWindowedRounds.name} (the seat of each ask)`,
  concurrency: 1,
  children: [
    it({
      name: 'NAMES ITS ROLE AND ITS BENCH ON EVERY ASK, so a runaway of one of its seats is re-asked of a model the role '
        + 'does not hold out and that is none of the seats',
      fn: async () => {
        const seats = await seatsOfAsks();
        expect(seats,).toHaveLength(BENCH.length,);
        for (const seat of seats) {
          expect(seat.role,).toBe('late-judge',);
          expect(seat.bench,).toEqual(BENCH,);
        }
      },
    },),

    it({
      name: 'SHARES ONE RECORD OF THE MODELS ASKED ACROSS EVERY SEAT OF THE ROUNDS, so a model one seat takes is not '
        + 'taken again by another',
      fn: async () => {
        const seats = await seatsOfAsks();
        expect(seats,).toHaveLength(BENCH.length,);
        const [first,] = seats;
        expect(first?.claims === undefined,).toBe(false,);
        for (const seat of seats) {
          expect(seat.claims === first?.claims,).toBe(true,);
        }
      },
    },),
  ],
},);

//endregion Windowed rounds seat tests
