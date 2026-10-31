/**
 Tests for the order of the models a role has a recorded standing for (ledger B1144).

 THE OWNER'S "Order by role standing (Recommended)" (2026-10-31) orders the stand-ins of an editor or a refiner by their
 standing in that role at the editor calibration of 2026-09-01, best first, and leaves every other role to its own order.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  standingOrderOf,
  ROLE_STANDINGS,
  type RosterModelId,
} from '../dist/final/node/index.mjs';

//region Seat role standing tests

/**
 Every model the calibration measured in either role, in no particular order.
 */
const MEASURED: readonly RosterModelId[] = [
  'gemma-4-26b-a4b-it',
  'hf:Qwen/Qwen3.8-27B',
  'minimax-m3',
  'hf:moonshotai/Kimi-K3',
  'glm-5.3',
  'hf:zai-org/GLM-5.3-Flash',
];

await describe({
  name: standingOrderOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'ORDERS THE EDITOR STAND-INS BY THEIR SHARE OF THE EDITOR CALIBRATION, best first',
      fn: async () => {
        expect(standingOrderOf({ role: 'editor', modelIds: MEASURED, },),).toEqual([
          'hf:zai-org/GLM-5.3-Flash',
          'glm-5.3',
          'hf:Qwen/Qwen3.8-27B',
          'hf:moonshotai/Kimi-K3',
          'minimax-m3',
          'gemma-4-26b-a4b-it',
        ],);
      },
    },),

    it({
      name: 'ORDERS THE REFINER STAND-INS BY THEIR SHARE OF THE REFINER CALIBRATION, which is not the editor order',
      fn: async () => {
        expect(standingOrderOf({ role: 'refiner', modelIds: MEASURED, },),).toEqual([
          'hf:zai-org/GLM-5.3-Flash',
          'glm-5.3',
          'minimax-m3',
          'hf:moonshotai/Kimi-K3',
          'hf:Qwen/Qwen3.8-27B',
          'gemma-4-26b-a4b-it',
        ],);
      },
    },),

    it({
      name: 'KEEPS ONLY THE MODELS HANDED IN, and none the calibration never measured',
      fn: async () => {
        expect(standingOrderOf({
          role: 'editor',
          modelIds: ['google.gemma-4-e2b', 'minimax-m3', 'hf:Qwen/Qwen3.8-27B',],
        },),).toEqual(['hf:Qwen/Qwen3.8-27B', 'minimax-m3',],);
      },
    },),

    it({
      name: 'ANSWERS NONE FOR A ROLE WITH NO STANDING, so its caller keeps the order of its own',
      fn: async () => {
        expect(standingOrderOf({ role: 'checker', modelIds: MEASURED, },),).toEqual([],);
        expect(Object.keys(ROLE_STANDINGS,),).toEqual(['editor', 'refiner',],);
      },
    },),
  ],
},);

//endregion Seat role standing tests
