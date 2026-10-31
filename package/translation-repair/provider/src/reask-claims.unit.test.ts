/**
 Tests for the record of models already asked for a seat of one question (ledger B1076, B1080).

 ONE MODEL GIVES AT MOST ONE VOICE PER QUESTION ("one judge alone never decides", 2026-10-25; "Two judges must name it",
 2026-10-25), so seats of one question that run away together share one record and each takes a model before it asks.

 FIXTURES ARE CAT-THEMED.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  alternateModelsFor,
  createReaskClaims,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_OPENROUTER_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Re-ask claims tests

await describe({
  name: createReaskClaims.name,
  concurrency: 1,
  children: [
    it({
      name: 'GIVES A MODEL TO THE FIRST SEAT THAT TAKES IT and refuses it to every later seat, listing the models taken '
        + 'in the order they were taken',
      fn: async () => {
        const claims = createReaskClaims();
        expect(claims.list(),).toEqual([],);
        expect(claims.take({ modelId: SEAT_BEDROCK_ONLY_TEXT, },),).toBe(true,);
        expect(claims.take({ modelId: SEAT_HYPER_TEXT_BEDROCK, },),).toBe(true,);
        expect(claims.take({ modelId: SEAT_BEDROCK_ONLY_TEXT, },),).toBe(false,);
        expect(claims.list(),).toEqual([SEAT_BEDROCK_ONLY_TEXT, SEAT_HYPER_TEXT_BEDROCK,],);
      },
    },),

    it({
      name: 'KEEPS THE RECORD OF ONE QUESTION APART FROM THE RECORD OF ANOTHER',
      fn: async () => {
        const first = createReaskClaims();
        const second = createReaskClaims();
        first.take({ modelId: SEAT_OPENROUTER_ONLY, },);
        expect(second.take({ modelId: SEAT_OPENROUTER_ONLY, },),).toBe(true,);
      },
    },),

    describe({
      name: `${alternateModelsFor.name} (models asked for other seats)`,
      concurrency: 1,
      children: [
        it({
          name: 'LEAVES OUT A MODEL ASKED FOR ANOTHER SEAT OF THE QUESTION, naming it with why, so the next runaway '
            + 'seat takes the next model of the order',
          fn: async () => {
            const { alternates, leftOut, } = alternateModelsFor({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              role: 'roster',
              askedForOtherSeats: [SEAT_HYPER_TEXT_BEDROCK,],
            },);
            expect(alternates[0],).toBe(SEAT_BEDROCK_ONLY_TEXT,);
            expect(alternates.includes(SEAT_HYPER_TEXT_BEDROCK,),).toBe(false,);
            expect(leftOut.find(function wasAsked(left,): boolean {
              return left.modelId === SEAT_HYPER_TEXT_BEDROCK;
            },),).toEqual({
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              why: 'already asked for another seat of this question',
            },);
          },
        },),

        it({
          name: 'NAMES EVERY REASON A MODEL IS LEFT OUT in the order bench, earlier ask for this seat, ask for '
            + 'another seat',
          fn: async () => {
            const { leftOut, } = alternateModelsFor({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              role: 'roster',
              bench: [SEAT_HYPER_TEXT_BEDROCK,],
              askedAlready: [SEAT_HYPER_TEXT_BEDROCK,],
              askedForOtherSeats: [SEAT_HYPER_TEXT_BEDROCK,],
            },);
            expect(leftOut.find(function onAll(left,): boolean {
              return left.modelId === SEAT_HYPER_TEXT_BEDROCK;
            },)?.why,).toBe(
              'seated on the bench of this question; already asked for this seat; '
                + 'already asked for another seat of this question',
            );
          },
        },),
      ],
    },),
  ],
},);

//endregion Re-ask claims tests
