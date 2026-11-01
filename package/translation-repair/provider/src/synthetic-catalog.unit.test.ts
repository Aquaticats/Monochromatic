/**
 Tests for the verified model catalog.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  SYNTHETIC_MODELS,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: 'synthetic catalog',
  children: [
    it({
      name: 'keeps every catalog entry internally coherent',
      fn: async () => {
        for (const [key, info,] of Object.entries(SYNTHETIC_MODELS,)) {
          expect(info.id,).toBe(key,);
          expect(info.contextLength > 0,).toBe(true,);
          expect(info.maxOutputLength > 0,).toBe(true,);
          expect(info.promptDollarsPerToken > 0,).toBe(true,);
          expect(info.completionDollarsPerToken > 0,).toBe(true,);
        }
      },
    },),

    it({
      name: 'RECORDS THE LIVE GLM-5.3-FLASH WIRE FACTS and leaves retiring GLM-5.2 uncallable',
      fn: async () => {
        const replacement = SYNTHETIC_MODELS[SEAT_HYPER_OPENROUTER_VISION_EDITOR];
        expect(replacement,).toEqual({
          id: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          readsImages: true,
          family: 'zai',
          contextLength: 524_288,
          maxOutputLength: 65_536,
          promptDollarsPerToken: 0.00000015,
          completionDollarsPerToken: 0.0000005,
        },);
        expect(Object.hasOwn(SYNTHETIC_MODELS, 'hf:zai-org/GLM-5.2',),).toBe(false,);
      },
    },),

    it({
      name: 'spans one model per vendor family, so no alias counts one model twice',
      fn: async () => {
        /** Distinct families in the catalog. */
        const families = new Set(
          Object.values(SYNTHETIC_MODELS,).map(function toFamily(info,) {
            return info.family;
          },),
        );
        expect([...families,].toSorted(),).toEqual([
          'moonshot',
          'openai',
          'qwen',
          'zai',
        ],);
        // Not every id the models endpoint lists: syn:large:text,
        // syn:large:vision, syn:small:text, and syn:small:vision each alias a
        // model already counted here. Admitting one would seat a single model
        // twice on a voting panel and count one opinion as two confirmations,
        // and would put a second entry under a family already present.
        //
        // HISTORY, NOT AN ASSERTION: five rather than six from 2026-08-24, when
        // the owner blocklisted `zai-org/GLM-4.7-Flash`, then four from
        // 2026-08-29 when the owner removed Nemotron from every stage. The Z.ai
        // family remains represented by GLM-5.3-Flash; the NVIDIA family leaves
        // the callable catalog.
        expect(families.size,).toBe(Object.keys(SYNTHETIC_MODELS,).length,);
        expect(Object.hasOwn(
          SYNTHETIC_MODELS,
          'hf:zai-org/GLM-4.7-Flash',
        ),).toBe(false,);
        expect(Object.hasOwn(
          SYNTHETIC_MODELS,
          'hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4',
        ),).toBe(false,);
      },
    },),
  ],
},);
