/**
 Tests for the Charm Hyper catalog.

 THIS FILE PINS MEASUREMENTS, not preferences. Every value it checks came from
 live calls, and each one is a value that a plausible reading of provider
 docs could get wrong: model answer ceiling can sit below the measured answer bound,
 and provider-specific image input support must be read rather than inferred.

 A CHANGED VALUE HERE IS A PROVIDER CHANGE, so these cases are meant to fail
 loudly rather than be updated to match.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  answerCeilingFor,
  HYPER_MODELS,
  NO_SYNTHETIC_COUNTERPART,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  hyperModelsWithoutSyntheticCounterparts,
  hyperModelsWithSyntheticCounterparts,
} from './catalog-lookups.test-fixture.ts';

await describe({
  name: 'hyper-catalog',
  children: [
    it({
      name: 'CARRIES every model the owner allowlisted, and nothing else, so a model reaches the '
        + 'wire because it was chosen rather than because a string resolved',
      fn: async () => {
        expect(Object.keys(HYPER_MODELS,).toSorted(),).toEqual([
          SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID,
          SEAT_HYPER_TEXT_BEDROCK,
          SEAT_HYPER_ONLY,
          'glm-5.3-flash',
          'gpt-oss-120b',
          'kimi-k3',
          SEAT_HYPER_VISION,
          'qwen3.8-27b',
        ],);
      },
    },),

    it({
      name: 'LOWERS the answer ceiling to what a model can actually emit, where a model cannot '
        + 'reach the bound: asking for more than a model emits buys a truncation and reports it as a '
        + 'schema mismatch, sending a reader to the prompt instead of to the ceiling',
      fn: async () => {
        expect(answerCeilingFor({ modelId: 'gpt-oss-120b', },),).toBe(13_107,);
        expect(answerCeilingFor({ modelId: 'kimi-k3', },),).toBe(16_000,);
      },
    },),

    it({
      name: 'HOLDS the measured bound for a model that could emit far more, since the bound is '
        + 'about what an answer should be rather than what a model is capable of',
      fn: async () => {
        expect(answerCeilingFor({ modelId: SEAT_HYPER_ONLY, },),).toBe(32_000,);
        expect(answerCeilingFor({ modelId: SEAT_HYPER_VISION, },),).toBe(32_000,);
      },
    },),

    it({
      name: 'NAMES the four models both providers serve, which are the only ones a non-conformant '
        + 'answer can be re-asked across',
      fn: async () => {
        // glm-5.3-flash joined 2026-09-01 as the Hyper route for Synthetic's
        // GLM-5.3-Flash seat.
        expect(hyperModelsWithSyntheticCounterparts().toSorted(),).toEqual([
          'glm-5.3-flash',
          'gpt-oss-120b',
          'kimi-k3',
          'qwen3.8-27b',
        ],);
      },
    },),

    it({
      name: 'names Hyper-origin identities without a Synthetic counterpart, independently of OpenRouter reach',
      fn: async () => {
        expect(hyperModelsWithoutSyntheticCounterparts().toSorted(),).toEqual([
          SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID,
          SEAT_HYPER_TEXT_BEDROCK,
          SEAT_HYPER_ONLY,
          SEAT_HYPER_VISION,
        ],);
      },
    },),

    it({
      name: 'SPLITS the roster into shared and provider-only with no model in both and none left '
        + 'out, since a model missing from the split would silently lose its recovery path',
      fn: async () => {
        expect(hyperModelsWithSyntheticCounterparts().length + hyperModelsWithoutSyntheticCounterparts().length,)
          .toBe(Object.keys(HYPER_MODELS,).length,);

        for (const id of hyperModelsWithSyntheticCounterparts())
          expect(hyperModelsWithoutSyntheticCounterparts().includes(id,),).toBe(false,);
      },
    },),

    it({
      name: 'PAIRS each shared model with the SAME panelist on the other provider, because '
        + 'provider is not part of panelist identity: a slice judged by that model counts once '
        + 'however it was reached',
      fn: async () => {
        expect(HYPER_MODELS['kimi-k3'].sharedWith,).toBe(SEAT_SYNTHETIC_VISION_WITHHELD,);
        expect(HYPER_MODELS['gpt-oss-120b'].sharedWith,).toBe(SEAT_SYNTHETIC_TEXT_EVERYWHERE,);
        expect(HYPER_MODELS['qwen3.8-27b'].sharedWith,).toBe(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
        expect(HYPER_MODELS[SEAT_HYPER_VISION].sharedWith,).toBe(NO_SYNTHETIC_COUNTERPART,);
      },
    },),

    it({
      name: 'reports image-capable serving paths including approved V4.1 Flash without deciding reader seating',
      fn: async () => {
        /**
         Models this provider says can be sent an image.
         */
        const readers = Object
          .values(HYPER_MODELS,)
          .filter(function reads(info,): boolean {
            return info.readsImages;
          },)
          .map(function toId(info,): string {
            return info.id;
          },);

        expect(readers.toSorted(),).toEqual([
          SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID,
          'glm-5.3-flash',
          'kimi-k3',
          SEAT_HYPER_VISION,
          'qwen3.8-27b',
        ],);
      },
    },),
  ],
},);
