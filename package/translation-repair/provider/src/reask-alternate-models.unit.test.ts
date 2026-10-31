/**
 Tests for the list of models a reasoning runaway is re-asked of (ledger B1072, B1074, B1075, B1076).

 THE LIST IS THE ROSTER'S MODELS OTHER THAN THE RUNAWAY, ordered by the share of their calls that ran away (the cap
 census of 2026-10-30), lowest first, without a decision-only seat, without the Claude route, without a model the
 card holds out of the seat's role and without a model on the bench of the question.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  alternateModelsFor,
  describeLeftOut,
  MIN_CALLS_TO_MEASURE,
  RUNAWAY_READINGS,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_DECISIONS,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Re-ask alternate model tests

/**
 The order the census of 2026-10-30 gives every chat model other than `deepseek-v4.1-flash`, before any hold or bench
 takes one out.
 */
const CENSUS_ORDER_WITHOUT_DEEPSEEK = [
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_VISION,
  SEAT_HYPER_ONLY,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
] as const;

await describe({
  name: alternateModelsFor.name,
  concurrency: 1,
  children: [
    it({
      name: 'ORDERS THE OTHER MODELS BY THE SHARE OF THEIR CALLS THAT RAN AWAY, lowest first, as the cap census of '
        + '2026-10-30 read it pooled over providers, and breaks a tie in roster order',
      fn: async () => {
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },).alternates,).toEqual([
          SEAT_HYPER_TEXT_BEDROCK,
          SEAT_BEDROCK_ONLY_TEXT,
          SEAT_OPENROUTER_ONLY,
          SEAT_SYNTHETIC_VISION_WITHHELD,
          SEAT_HYPER_VISION,
          SEAT_HYPER_ONLY,
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_HYPER_OPENROUTER_VISION_EDITOR,
        ],);
        expect(CENSUS_ORDER_WITHOUT_DEEPSEEK.length,).toBe(11,);
      },
    },),

    it({
      name: 'STATES EACH READING THE ORDER RESTS ON: calls since the caps and the ones that ran to the cap with no '
        + 'content, for the twelve chat models the census seated, none of them under the calls that can be measured',
      fn: async () => {
        expect(MIN_CALLS_TO_MEASURE,).toBe(100,);
        expect(RUNAWAY_READINGS.length,).toBe(12,);
        for (const reading of RUNAWAY_READINGS) {
          expect(reading.calls >= MIN_CALLS_TO_MEASURE,).toBe(true,);
          expect(reading.runaways <= reading.calls,).toBe(true,);
        }
        expect(RUNAWAY_READINGS.find(function isDeepseek(reading,) {
          return reading.modelId === SEAT_HYPER_OPENROUTER_UNMEASURED;
        },),).toEqual({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, calls: 78_100, runaways: 438, },);
      },
    },),

    it({
      name: 'PLACES A MODEL WITH TOO FEW CALLS TO MEASURE AFTER EVERY MEASURED MODEL, in roster order, since a share '
        + 'read from fewer than 100 calls says less than any measured one',
      fn: async () => {
        /**
         Readings with Kimi-K3 one call under what can be measured and every model not named left with no reading.
         */
        const readings = [
          { modelId: SEAT_HYPER_TEXT_BEDROCK, calls: 1_000, runaways: 0, },
          { modelId: SEAT_SYNTHETIC_VISION_WITHHELD, calls: MIN_CALLS_TO_MEASURE - 1, runaways: 0, },
          { modelId: SEAT_HYPER_VISION, calls: 1_000, runaways: 5, },
        ] as const;
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, readings, },).alternates.slice(0, 4,),)
          .toEqual([
            SEAT_HYPER_TEXT_BEDROCK,
            SEAT_HYPER_VISION,
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          ],);
      },
    },),

    it({
      name: 'LEAVES OUT THE MODEL THAT RAN AWAY wherever it stands in the order, so the lowest share is skipped for '
        + 'itself and the highest for itself',
      fn: async () => {
        expect(alternateModelsFor({ modelId: SEAT_HYPER_TEXT_BEDROCK, },).alternates[0],).toBe(SEAT_BEDROCK_ONLY_TEXT,);
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },).alternates.at(-1,),).toBe(
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        );
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },).alternates.includes(
          SEAT_HYPER_OPENROUTER_UNMEASURED,
        ),).toBe(false,);
      },
    },),

    it({
      name: 'ASKS NO DECISION-ONLY SEAT AND NO CLAUDE SEAT even when the model that ran away is one of them, and holds '
        + 'the owner\'s cull and the two models no judge probe seated out of the roster-wide role, nine models of the '
        + 'twelve chat models',
      fn: async () => {
        for (const ranAway of [SEAT_OPENROUTER_DECISIONS, 'claude-sonnet-5-5-effort-high',] as const) {
          /**
           The list for this model.
           */
          const { alternates, leftOut, } = alternateModelsFor({ modelId: ranAway, },);
          expect(alternates.length,).toBe(9,);
          expect(alternates.some(function onClaudeOrDecides(seat,): boolean {
            return seat.startsWith('claude-',) || (seat === SEAT_OPENROUTER_DECISIONS);
          },),).toBe(false,);
          expect(alternates.includes(SEAT_HYPER_OPENROUTER_UNMEASURED,),).toBe(true,);
          expect(leftOut.map(function nameOf(left,) {
            return left.modelId;
          },),).toEqual([
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            SEAT_BEDROCK_ONLY_VISION_UNSEATED,
            SEAT_OPENROUTER_ONLY_CHECKER,
          ],);
        }
      },
    },),

    it({
      name: 'HONOURS THE HOLDS OF THE ROLE ASKED: for a wide judge the three models the card drops from the wide seats '
        + 'are left out, each with the hold named; for a checker the two the judge probe withheld are asked',
      fn: async () => {
        const wide = alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, role: 'wide-judge', },);
        expect(wide.alternates,).toEqual([
          SEAT_HYPER_TEXT_BEDROCK,
          SEAT_BEDROCK_ONLY_TEXT,
          SEAT_OPENROUTER_ONLY,
          SEAT_SYNTHETIC_VISION_WITHHELD,
          SEAT_HYPER_VISION,
        ],);
        expect(wide.leftOut.filter(function isHeld(left,): boolean {
          return left.why.includes('wide-seat-dropped',);
        },),).toEqual([
          {
            modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            why: 'held out of the wide-judge role by wide-seat-dropped',
          },
          {
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            why: 'held out of the wide-judge role by wide-seat-dropped',
          },
          {
            modelId: SEAT_HYPER_ONLY,
            why: 'held out of the wide-judge role by wide-seat-dropped',
          },
        ],);

        const checker = alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, role: 'checker', },);
        expect(checker.alternates.includes(SEAT_OPENROUTER_ONLY_CHECKER,),).toBe(true,);
        expect(checker.alternates.includes(SEAT_BEDROCK_ONLY_VISION_UNSEATED,),).toBe(true,);
        expect(checker.alternates.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(false,);

        const translator = alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, role: 'translator', },);
        expect(translator.alternates.includes(SEAT_OPENROUTER_ONLY,),).toBe(false,);
        expect(translator.alternates.includes(SEAT_HYPER_OPENROUTER_VISION_EDITOR,),).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES OUT EVERY MODEL ON THE BENCH OF THE QUESTION, each named with why, since its answer recorded as the '
        + 'voice of the seat that ran away would count one model twice',
      fn: async () => {
        const { alternates, leftOut, } = alternateModelsFor({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          bench: [SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_TEXT,],
        },);
        expect(alternates[0],).toBe(SEAT_OPENROUTER_ONLY,);
        expect(alternates.includes(SEAT_HYPER_TEXT_BEDROCK,),).toBe(false,);
        expect(leftOut.filter(function onBench(left,): boolean {
          return left.why === 'seated on the bench of this question';
        },).map(function nameOf(left,) {
          return left.modelId;
        },),).toEqual([SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_TEXT,],);
      },
    },),

    it({
      name: 'LEAVES OUT A MODEL ALREADY ASKED FOR THE SEAT, so a second ask does not replay the first model\'s claimed '
        + 'reply as if it were another voice',
      fn: async () => {
        const { alternates, leftOut, } = alternateModelsFor({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          askedAlready: [SEAT_HYPER_TEXT_BEDROCK,],
        },);
        expect(alternates[0],).toBe(SEAT_BEDROCK_ONLY_TEXT,);
        expect(leftOut.find(function wasAsked(left,): boolean {
          return left.modelId === SEAT_HYPER_TEXT_BEDROCK;
        },),).toEqual({ modelId: SEAT_HYPER_TEXT_BEDROCK, why: 'already asked for this seat', },);
      },
    },),

    it({
      name: 'LEAVES NO ALTERNATE where the bench holds every model that could take the re-ask, and says why for each',
      fn: async () => {
        const { alternates, leftOut, } = alternateModelsFor({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          bench: CENSUS_ORDER_WITHOUT_DEEPSEEK,
        },);
        expect(alternates,).toEqual([],);
        expect(leftOut.length,).toBe(11,);
      },
    },),

    it({
      name: 'WRITES THE MODELS LEFT OUT AS ONE PHRASE, each with its reason, in the order they were left out',
      fn: async () => {
        expect(describeLeftOut({
          leftOut: [
            { modelId: SEAT_HYPER_TEXT_BEDROCK, why: 'seated on the bench of this question', },
            { modelId: SEAT_OPENROUTER_ONLY, why: 'held out of the wide-judge role by wide-seat-dropped', },
          ],
        },),).toBe(
          `${SEAT_HYPER_TEXT_BEDROCK} (seated on the bench of this question), `
            + `${SEAT_OPENROUTER_ONLY} (held out of the wide-judge role by wide-seat-dropped)`,
        );
        expect(describeLeftOut({ leftOut: [], },),).toBe('none',);
      },
    },),
  ],
},);

//endregion Re-ask alternate model tests
