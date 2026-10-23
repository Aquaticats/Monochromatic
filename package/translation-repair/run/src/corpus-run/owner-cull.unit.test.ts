/**
 Guards the owner's cull of 2026-09-24: "That particular model got cause and
 effect wrong. Cull it." and then "cull it from every role", said of the
 critic that filed, alone, an accepted omission claim reading 服用激素，成为跨
 性别的原因 as hormones being the reason for transitioning (CuspariaKLSY10
 slice 1), the same seat that abstained on a slate over "quoted lines in
 present tense" where no line was quoted (zheermao8 slice 9). The card stays
 for the catalogs and the unit fixture; the seats go. Cat-themed invention in
 every fixture; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ROSTER_MODEL_IDS, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import { assertCheckerQuorumReachable, } from '@monochromatic-dev/translation-repair-assembly';
import {
  judgeSeatsFor,
  OWNER_CULLED,
  RECALL_JUDGE_MODEL_IDS,
  RUN_LATE_JUDGES,
  RUN_MODELS,
  RUN_READER_MODELS,
  RUN_ROSTER,
  RUN_SELECT_JUDGES,
  RUN_TRANSLATORS,
  RUN_WIDE_SEATS,
  RUN_WRITERS,
} from '../../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Every provider wet, the reading that seats the widest benches.
 */
const ALL_WET: BudgetView = {
  neuralwatt: false,
  synthetic: false,
  hyper: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: false,
};

/**
 Every provider dry, the reading that seats the substitute checker.
 */
const ALL_DRY: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  openrouter: true,
};

await describe({
  name: 'the owner\'s cull of 2026-09-24',
  children: [
    it({
      name: 'HOLDS THE CULLED SEAT OUT OF EVERY STATIC BENCH while its card stays on the catalogs',
      fn: async () => {
        expect(OWNER_CULLED.has(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(true,);
        expect(ROSTER_MODEL_IDS.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(true,);
        for (
          const bench of [
            RUN_ROSTER,
            RUN_WIDE_SEATS,
            RUN_LATE_JUDGES,
            RUN_SELECT_JUDGES,
            RUN_TRANSLATORS,
            RUN_WRITERS,
            RUN_READER_MODELS,
            RUN_MODELS.checkerModelIds,
            RUN_MODELS.editorModelIds,
            RUN_MODELS.refinerModelIds,
            // The recall benchmark's judges (ledger P4): its default named the
            // culled seat two days after the cull, since nothing derived it.
            RECALL_JUDGE_MODEL_IDS,
          ]
        ) {
          expect(bench.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(false,);
        }
        expect(RECALL_JUDGE_MODEL_IDS.length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'SEATS NOBODY CULLED ON ANY READING, wet or dry, and keeps three checkers on the static bench',
      fn: async () => {
        for (const dry of [ALL_WET, ALL_DRY,]) {
          /**
           Seats derived from this reading.
           */
          const seats = judgeSeatsFor({ dry, },);
          for (
            const bench of [
              seats.roster,
              seats.wideSeats,
              seats.selectJudges,
              seats.lateJudges,
              seats.slateJudges,
              seats.checkers,
              seats.translators,
              seats.readers,
              seats.writers,
              seats.repairModels.criticModelIds,
              seats.repairModels.panelModelIds,
              seats.repairModels.checkerModelIds,
            ]
          ) {
            expect(bench.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(false,);
          }
        }
        // THE CONTRACT'S OWN FLOOR rather than a literal count (ledger E9).
        expect(function checkerFloorHolds(): void {
          assertCheckerQuorumReachable({ checkerModelIds: RUN_MODELS.checkerModelIds, },);
        },).not.toThrow();
      },
    },),
  ],
},);
