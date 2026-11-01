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

import { MIN_PROVIDER_CALLS, } from '@monochromatic-dev/translation-repair-shared-contract';
import * as providerExports from '../dist/final/node/index.mjs';
import {
  alternateModelsFor,
  describeLeftOut,
  RUNAWAY_READINGS,
} from '../dist/final/node/index.mjs';
import {
  HYPER_WET_ONLY,
  SYNTHETIC_WET_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
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
 Qwen3.8-27B, the one model whose card holds `hyper-slow`.
 */
const QWEN = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

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
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, role: 'roster', },).alternates,).toEqual([
          SEAT_HYPER_TEXT_BEDROCK,
          SEAT_BEDROCK_ONLY_TEXT,
          SEAT_OPENROUTER_ONLY,
          SEAT_SYNTHETIC_VISION_WITHHELD,
          SEAT_HYPER_VISION,
          SEAT_HYPER_ONLY,
          SEAT_HYPER_OPENROUTER_VISION_EDITOR,
        ],);
        expect(CENSUS_ORDER_WITHOUT_DEEPSEEK.length,).toBe(11,);
      },
    },),

    it({
      name: 'STATES EACH READING THE ORDER RESTS ON: calls since the caps and the ones that ran to the cap with no '
        + 'content, for the twelve chat models the census seated, none of them under the calls that can be measured',
      fn: async () => {
        // ONE FLOOR, the completion-cap rule's own: the provider package states no second constant for it.
        expect('MIN_CALLS_TO_MEASURE' in providerExports,).toBe(false,);
        expect(MIN_PROVIDER_CALLS,).toBe(100,);
        expect(RUNAWAY_READINGS.length,).toBe(12,);
        for (const reading of RUNAWAY_READINGS) {
          expect(reading.calls >= MIN_PROVIDER_CALLS,).toBe(true,);
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
          { modelId: SEAT_SYNTHETIC_VISION_WITHHELD, calls: MIN_PROVIDER_CALLS - 1, runaways: 0, },
          { modelId: SEAT_HYPER_VISION, calls: 1_000, runaways: 5, },
        ] as const;
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, role: 'roster', readings, },).alternates.slice(0, 3,),)
          .toEqual([
            SEAT_HYPER_TEXT_BEDROCK,
            SEAT_HYPER_VISION,
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          ],);
      },
    },),

    it({
      name: 'LEAVES OUT THE MODEL THAT RAN AWAY wherever it stands in the order, so the lowest share is skipped for '
        + 'itself and the highest for itself',
      fn: async () => {
        expect(alternateModelsFor({ modelId: SEAT_HYPER_TEXT_BEDROCK, role: 'roster', },).alternates[0],).toBe(SEAT_BEDROCK_ONLY_TEXT,);
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, role: 'roster', },).alternates.at(-1,),).toBe(
          SEAT_HYPER_ONLY,
        );
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, role: 'roster', },).alternates.includes(
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
          const { alternates, leftOut, } = alternateModelsFor({ modelId: ranAway, role: 'roster', },);
          expect(alternates.length,).toBe(8,);
          expect(alternates.some(function onClaudeOrDecides(seat,): boolean {
            return seat.startsWith('claude-',) || (seat === SEAT_OPENROUTER_DECISIONS);
          },),).toBe(false,);
          expect(alternates.includes(SEAT_HYPER_OPENROUTER_UNMEASURED,),).toBe(true,);
          expect(leftOut.map(function nameOf(left,) {
            return left.modelId;
          },),).toEqual([
            // Qwen3.8-27B: the run withholds it from the roster seats while Hyper serves it (ledger B1143).
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
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
            why: 'held out of the wide-judge role by wide-seat-dropped and hyper-slow',
          },
          {
            modelId: SEAT_HYPER_ONLY,
            why: 'held out of the wide-judge role by wide-seat-dropped',
          },
        ],);

        const checker = alternateModelsFor({ modelId: SEAT_OPENROUTER_ONLY, role: 'checker', },);
        expect(checker.alternates.includes(SEAT_OPENROUTER_ONLY_CHECKER,),).toBe(true,);
        expect(checker.alternates.includes(SEAT_BEDROCK_ONLY_VISION_UNSEATED,),).toBe(true,);
        expect(checker.alternates.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(false,);

        const translator = alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, role: 'translator', },);
        expect(translator.alternates.includes(SEAT_OPENROUTER_ONLY,),).toBe(false,);
        expect(translator.alternates.includes(SEAT_HYPER_OPENROUTER_VISION_EDITOR,),).toBe(true,);
      },
    },),

    it({
      name: 'ASKS A CHECKER ONLY OF A MODEL THE CHECKER BENCHMARK MEASURED IN: the measured order less the runaway, '
        + 'the Claude route and the bench, and every other roster model is left out with why, the two Gemma sizes the '
        + 'benchmark measured out of the role among them',
      fn: async () => {
        const { alternates, leftOut, } = alternateModelsFor({
          modelId: SEAT_OPENROUTER_ONLY_CHECKER,
          role: 'checker',
          bench: [SEAT_OPENROUTER_ONLY_CHECKER, SEAT_SYNTHETIC_VISION_WITHHELD, 'claude-haiku-5-5',],
        },);
        expect(alternates,).toEqual([SEAT_BEDROCK_ONLY_VISION_UNSEATED, SEAT_OPENROUTER_ONLY,],);
        expect(leftOut.find(function isGemma(left,): boolean {
          return left.modelId === SEAT_HYPER_TEXT_BEDROCK;
        },),).toEqual({
          modelId: SEAT_HYPER_TEXT_BEDROCK,
          why: 'not on the measured list of the checker role',
        },);
        expect(leftOut.find(function isKimi(left,): boolean {
          return left.modelId === SEAT_SYNTHETIC_VISION_WITHHELD;
        },),).toEqual({
          modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
          why: 'seated on the bench of this question',
        },);
      },
    },),

    it({
      name: 'ASKS AN EDITOR OR A REFINER OF THE WRITERS THE EDITOR CALIBRATION SEATS, not of a model that hold keeps '
        + 'out of writing, and never of a model on the bench of the question',
      fn: async () => {
        for (const role of ['editor', 'refiner',] as const) {
          const { alternates, } = alternateModelsFor({
            modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            role,
            bench: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_HYPER_ONLY, SEAT_HYPER_OPENROUTER_UNMEASURED,],
          },);
          expect(alternates.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(false,);
          expect(alternates.includes(SEAT_HYPER_ONLY,),).toBe(false,);
          expect(alternates.includes(SEAT_HYPER_TEXT_BEDROCK,),).toBe(true,);
        }
      },
    },),

    it({
      name: 'ORDERS THE STAND-INS OF AN EDITOR OR A REFINER BY THEIR STANDING IN THAT ROLE, best first, then the '
        + 'models never measured in it by the share of runaways (the owner\'s "Order by role standing (Recommended)", '
        + '2026-10-31, ledger B1144): the editor calibration of 2026-09-01 puts Qwen3.8-27B (17.3) before Kimi-K3 '
        + '(14.2), minimax-m3 (8.4) and gemma-4-26b (7.0) as an editor, and glm-5.3 (12.1) before Kimi-K3 (7.8), '
        + 'Qwen3.8-27B (7.7) and gemma-4-26b (3.4) as a refiner',
      fn: async () => {
        expect(alternateModelsFor({
          modelId: 'hf:zai-org/GLM-5.3-Flash',
          role: 'editor',
          bench: ['hf:zai-org/GLM-5.3-Flash', 'claude-sonnet-5-5', 'glm-5.3', 'deepseek-v4.1-flash',],
          routeTimeHolds: { applied: 'in-reading', dry: SYNTHETIC_WET_ONLY, },
        },).alternates,).toEqual([
          'hf:Qwen/Qwen3.8-27B',
          'hf:moonshotai/Kimi-K3',
          'minimax-m3',
          'gemma-4-26b-a4b-it',
          'google.gemma-4-e2b',
          'inception/mercury-2.5',
        ],);
        expect(alternateModelsFor({
          modelId: 'hf:zai-org/GLM-5.3-Flash',
          role: 'refiner',
          bench: ['hf:zai-org/GLM-5.3-Flash', 'deepseek-v4.1-flash', 'minimax-m3',],
          routeTimeHolds: { applied: 'in-reading', dry: SYNTHETIC_WET_ONLY, },
        },).alternates,).toEqual([
          'glm-5.3',
          'hf:moonshotai/Kimi-K3',
          'hf:Qwen/Qwen3.8-27B',
          'gemma-4-26b-a4b-it',
          'google.gemma-4-e2b',
          'inception/mercury-2.5',
        ],);
      },
    },),

    it({
      name: 'NEVER ASKS A MODEL THE RUN WITHHOLDS FROM AN EDITOR OR REFINER SEAT (ledger B1330): Qwen3.8-27B is left '
        + 'out of both roles with hyper-slow named where the reading has Hyper serving it, is first for an editor and '
        + 'third for a refiner where Synthetic serves it and Hyper is dry, and is left out of both where the reading '
        + 'is unknown',
      fn: async () => {
        for (const [role, position, bench,] of [
          ['editor', 0, ['hf:zai-org/GLM-5.3-Flash', 'claude-sonnet-5-5', 'glm-5.3', 'deepseek-v4.1-flash',],],
          ['refiner', 2, ['hf:zai-org/GLM-5.3-Flash', 'deepseek-v4.1-flash', 'minimax-m3',],],
        ] as const) {
          /**
           The question every case asks: a GLM runaway with the static seats of the role on the bench.
           */
          const asked = {
            modelId: 'hf:zai-org/GLM-5.3-Flash',
            role,
            bench,
          } as const;
          for (const routeTimeHolds of [
            { applied: 'always', },
            { applied: 'in-reading', dry: HYPER_WET_ONLY, },
          ] as const) {
            const withheld = alternateModelsFor({ ...asked, routeTimeHolds, },);
            expect(withheld.alternates.includes(QWEN,),).toBe(false,);
            expect(withheld.leftOut.find(function isQwen(left,): boolean {
              return left.modelId === QWEN;
            },)?.why,).toBe(`held out of the ${role} role by hyper-slow`,);
          }
          // NO RULE GIVEN IS THE SAME AS THE STATIC ONE: a caller that never learned the reading withholds.
          expect(alternateModelsFor(asked,).alternates.includes(QWEN,),).toBe(false,);
          expect(alternateModelsFor({
            ...asked,
            routeTimeHolds: { applied: 'in-reading', dry: SYNTHETIC_WET_ONLY, },
          },).alternates.indexOf(QWEN,),).toBe(position,);
          /**
           The alternates of a caller that applies the reading itself.
           */
          const { alternates: byTheCaller, } = alternateModelsFor({
            ...asked,
            routeTimeHolds: { applied: 'by-the-caller', },
          },);
          expect(byTheCaller.includes(QWEN,),).toBe(true,);
        }
      },
    },),

    it({
      name: 'NEVER ASKS KIMI-K3 FOR A SELECT OR SLATE SEAT, nor QWEN3.8-27B FOR A SEAT THE RUN WITHHOLDS IT FROM WHILE '
        + 'HYPER SERVES IT (ledger B1143): with Synthetic dry and Hyper serving Kimi-K3 the run keeps it off the '
        + 'select and slate benches, and a runaway select or slate judge is not re-asked of it on Hyper',
      fn: async () => {
        for (const role of ['select-judge', 'slate-judge',] as const) {
          const { alternates, leftOut, } = alternateModelsFor({ modelId: 'minimax-m3', role, },);
          expect(alternates.includes('hf:moonshotai/Kimi-K3',),).toBe(false,);
          expect(leftOut.find(function isKimi(left,): boolean {
            return left.modelId === 'hf:moonshotai/Kimi-K3';
          },)?.why,).toContain('hyper-slow-select',);
        }
        expect(alternateModelsFor({ modelId: 'minimax-m3', role: 'wide-judge', },).alternates.includes('hf:moonshotai/Kimi-K3',),)
          .toBe(true,);
        for (const role of ['roster', 'translator', 'writer',] as const) {
          expect(alternateModelsFor({ modelId: 'minimax-m3', role, },).alternates.includes('hf:Qwen/Qwen3.8-27B',),).toBe(false,);
        }
      },
    },),

    it({
      name: 'KEEPS THE SHARE OF RUNAWAYS AS THE ORDER OF EVERY OTHER ROLE, the roles with no recorded standing',
      fn: async () => {
        for (const role of ['roster', 'writer', 'wide-judge', 'checker',] as const) {
          const { alternates, } = alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, role, },);
          const byShare = alternates.map(function shareOf(modelId,): number | 'unmeasured' {
            return providerExports.runawayShareOf({ modelId, readings: providerExports.RUNAWAY_READINGS, },);
          },).filter(function measured(share,): share is number {
            return share !== 'unmeasured';
          },);
          expect(byShare,).toEqual(byShare.toSorted(function ascending(left, right,): number {
            return left - right;
          },),);
        }
      },
    },),

    it({
      name: 'LEAVES OUT EVERY MODEL ON THE BENCH OF THE QUESTION, each named with why, since its answer recorded as '
        + 'the voice of the seat that ran away would count one model twice',
      fn: async () => {
        const { alternates, leftOut, } = alternateModelsFor({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          role: 'roster',
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
          role: 'roster',
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
          role: 'roster',
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
