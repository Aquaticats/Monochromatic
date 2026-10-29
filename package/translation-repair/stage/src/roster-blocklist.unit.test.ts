/**
 Tests that the owner blocklist bars every blocked spelling from the
 compiled catalogs and labels family arrivals, while leaving the eligible
 neighbours (dated DeepSeek aliases, the Qwen3.8 line short of Max, the
 GLM-5.3 pair) untouched.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { SYNTHETIC_MODELS, } from '@monochromatic-dev/translation-repair-provider';
import {
  HYPER_MODELS,
  ROSTER_MODEL_IDS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  blocklistVerdictFor,
  ROSTER_BLOCKLIST,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: blocklistVerdictFor.name,
  children: [
    it({
      name: 'BARS EVERY BLOCKED SPELLING from the roster and both compiled catalogs',
      fn: async () => {
        /**
         Every id the pipeline could seat.
         */
        const seated = [
          ...ROSTER_MODEL_IDS,
          ...Object.keys(SYNTHETIC_MODELS,),
          ...Object.keys(HYPER_MODELS,),
        ];
        for (const id of seated)
          expect(blocklistVerdictFor({ id, },),).toEqual({ blocked: false, },);
      },
    },),

    it({
      name: 'RETURNS THE OWNER REASON for exact entries under both provider spellings',
      fn: async () => {
        expect(blocklistVerdictFor({ id: 'qwen3.8-max', },),).toEqual({
          blocked: true,
          reason: 'absurd cost in money',
        },);
        expect(blocklistVerdictFor({ id: 'hf:zai-org/GLM-5.2', },),).toEqual({
          blocked: true,
          reason: 'too outdated',
        },);
        expect(blocklistVerdictFor({ id: 'glm-5.2', },),).toEqual({
          blocked: true,
          reason: 'too outdated',
        },);
        expect(blocklistVerdictFor({ id: 'syn:small:text', },),).toEqual({
          blocked: true,
          reason: 'too outdated',
        },);
        expect(blocklistVerdictFor({ id: 'deepseek-v4-pro', },),).toEqual({
          blocked: true,
          reason: 'the undated versions',
        },);
        expect(
          blocklistVerdictFor({ id: 'hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4', },),
        ).toEqual({
          blocked: true,
          reason: "can't stick to its own viewpoint",
        },);
      },
    },),

    it({
      name: 'CATCHES FAMILY ARRIVALS the owner phrased as series, on both provider spellings',
      fn: async () => {
        // A name the provider adds later inside a blocked family must not
        // surface as a candidate at the next refresh.
        expect(blocklistVerdictFor({ id: 'llama-5-800b-instruct', },),).toEqual({
          blocked: true,
          reason: 'too outdated',
        },);
        expect(blocklistVerdictFor({ id: 'hf:meta-llama/Llama-4-Scout', },),).toEqual({
          blocked: true,
          reason: 'too outdated',
        },);
        expect(blocklistVerdictFor({ id: 'qwen3.6-turbo', },),).toEqual({
          blocked: true,
          reason: 'too outdated',
        },);
        expect(blocklistVerdictFor({ id: 'hf:Qwen/Qwen3.7-Omni', },),).toEqual({
          blocked: true,
          reason: 'too outdated',
        },);
        expect(blocklistVerdictFor({ id: 'qwen3-vl-235b', },),).toEqual({
          blocked: true,
          reason: 'too outdated',
        },);
      },
    },),

    it({
      name: 'BARS THE DATED DEEPSEEK V4 PAIR UNDER BOTH PROVIDERS\' SPELLINGS on the owner\'s words of 2026-09-16',
      fn: async () => {
        for (const id of [
          'deepseek-v4-pro-0813',
          'deepseek-v4-flash-0731',
          'deepseek/deepseek-v4-pro-0813',
          'deepseek/deepseek-v4-flash-0731',
        ]) {
          expect(blocklistVerdictFor({ id, },),).toEqual({
            blocked: true,
            reason: 'DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash',
          },);
        }
      },
    },),

    it({
      name: 'LEAVES ELIGIBLE NEIGHBOURS ALONE: DeepSeek V4.1 Flash, the Qwen3.8 line short of Max, the GLM-5.3 pair, Kimi K3',
      fn: async () => {
        // The dot after qwen3 is the family boundary, and the owner's single
        // exact entries must not leak onto their newer siblings. qwen3.8-flash
        // and qwen3.8-2.4t-a95b stay blocklist-eligible even though the
        // 2026-09-01 conformance probe culled them: non-conformance is a
        // catalog fact, never an owner instruction.
        expect(blocklistVerdictFor({ id: SEAT_HYPER_OPENROUTER_UNMEASURED, },),).toEqual({ blocked: false, },);
        expect(blocklistVerdictFor({ id: 'qwen3.8-flash', },),).toEqual({ blocked: false, },);
        expect(blocklistVerdictFor({ id: 'qwen3.8-2.4t-a95b', },),).toEqual({ blocked: false, },);
        expect(blocklistVerdictFor({ id: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },),).toEqual({ blocked: false, },);
        expect(blocklistVerdictFor({ id: SEAT_HYPER_ONLY, },),).toEqual({ blocked: false, },);
        expect(blocklistVerdictFor({ id: 'glm-5.3-flash', },),).toEqual({ blocked: false, },);
        expect(blocklistVerdictFor({ id: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },),).toEqual({ blocked: false, },);
        expect(blocklistVerdictFor({ id: SEAT_SYNTHETIC_VISION_WITHHELD, },),).toEqual({ blocked: false, },);
        expect(blocklistVerdictFor({ id: 'kimi-k2.8', },),).toEqual({ blocked: false, },);
      },
    },),

    it({
      name: 'CARRIES A REASON ON EVERY ENTRY, since the constant exists to record authority',
      fn: async () => {
        for (const entry of ROSTER_BLOCKLIST)
          expect(entry.reason.length > 0,).toBe(true,);
      },
    },),
  ],
},);
