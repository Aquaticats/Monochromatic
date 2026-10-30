/**
 Tests for the list of models a reasoning runaway is re-asked of (ledger B1072).

 THE LIST IS THE ROSTER'S, in its own order, without the model that ran away, without decision-only seats and without
 the Claude route.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { alternateModelsFor, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_DECISIONS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Re-ask alternate model tests

await describe({
  name: alternateModelsFor.name,
  concurrency: 1,
  children: [
    it({
      name: 'LISTS THE ROSTER IN ITS OWN ORDER without the model that ran away, and without any decision-only seat or '
        + 'seat on the Claude route',
      fn: async () => {
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),).toEqual([
          'hf:zai-org/GLM-5.3-Flash',
          'hf:Qwen/Qwen3.8-27B',
          'hf:moonshotai/Kimi-K3',
          'hf:openai/gpt-oss-120b',
          'minimax-m3',
          'gemma-4-26b-a4b-it',
          'glm-5.3',
          'google.gemma-4-e2b',
          'google.gemma-4-31b',
          'inception/mercury-2.5',
          'xiaomi/mimo-v2.6-flash',
        ],);
      },
    },),

    it({
      name: 'LEAVES OUT THE MODEL THAT RAN AWAY wherever it stands in the roster, so the first seat is skipped for the '
        + 'second and the second for the first',
      fn: async () => {
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },).slice(0, 2,),).toEqual([
          'hf:Qwen/Qwen3.8-27B',
          'hf:moonshotai/Kimi-K3',
        ],);
        expect(alternateModelsFor({ modelId: 'hf:Qwen/Qwen3.8-27B', },).slice(0, 2,),).toEqual([
          'hf:zai-org/GLM-5.3-Flash',
          'hf:moonshotai/Kimi-K3',
        ],);
        expect(alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },).includes(
          SEAT_HYPER_OPENROUTER_UNMEASURED,
        ),).toBe(false,);
      },
    },),

    it({
      name: 'ASKS NO DECISION-ONLY SEAT AND NO CLAUDE SEAT even when the model that ran away is one of them: the list for '
        + 'either holds all twelve chat models, the eleven another chat seat leaves and the one that seat was',
      fn: async () => {
        for (const ranAway of [SEAT_OPENROUTER_DECISIONS, 'claude-sonnet-5-5-effort-high',] as const) {
          /**
           The list for this model.
           */
          const list = alternateModelsFor({ modelId: ranAway, },);
          expect(list.length,).toBe(12,);
          expect(list.some(function onClaudeOrDecides(seat,): boolean {
            return seat.startsWith('claude-',) || (seat === SEAT_OPENROUTER_DECISIONS);
          },),).toBe(false,);
          expect(list.includes(SEAT_HYPER_OPENROUTER_UNMEASURED,),).toBe(true,);
        }
      },
    },),
  ],
},);

//endregion Re-ask alternate model tests
