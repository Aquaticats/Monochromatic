/**
 Tests for the final polish's configuration.

 WHAT THIS FILE PINS: a roster with refiners, which every corpus run seats
 (`RunRepairModels`), configures the polish from its refiners and judges, the
 gate bench and the prepared document. A roster without refiners has no
 builder: the one that answered it with a disabled kind lost its last
 production caller when the pass began building through this one (ledger T8).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { configuredConsolidationPolish, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 A prepared page pair with no declared names.
 */
const PREPARED = prepareDocumentPair({
  sourceText: '猫在窗边睡着了。\n\n它梦见了鱼。',
  targetText: 'The cat fell asleep by the window.\n\nIt dreamed of fish.',
},);

/**
 A repair roster with every required role and its refiners seated.
 */
const WITH_REFINERS: RepairModels & Required<Pick<RepairModels, 'refinerModelIds'>> = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
  checkerModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
};

/**
 The final gate's bench.
 */
const GATE = [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,] as const;

await describe({
  name: configuredConsolidationPolish.name,
  children: [
    it({
      name: 'CONFIGURES THE POLISH from a roster with refiners: its refiners and judges, the gate bench, and '
        + 'the prepared document\'s declared names',
      fn: async () => {
        /**
         The configuration that roster gives.
         */
        const configured = configuredConsolidationPolish({
          prepared: PREPARED,
          models: WITH_REFINERS,
          gateModelIds: GATE,
        },);
        expect({
          refinerModelIds: configured.refinerModelIds,
          judgeModelIds: configured.judgeModelIds,
          gateModelIds: configured.gateModelIds,
          declaredNames: configured.declaredNames,
        },).toEqual({
          refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          judgeModelIds: WITH_REFINERS.judgeModelIds,
          gateModelIds: GATE,
          declaredNames: PREPARED.declaredNames,
        },);
      },
    },),
  ],
},);
