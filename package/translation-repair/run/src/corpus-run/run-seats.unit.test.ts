/**
 Tests the provider-aware judge seats.

 THE CASES: Qwen3.8-27B, cut in 30 of 34 translate-lane select rounds when
 Hyper served it (XIEPT2, 2026-09-03) and answering 25 of 28 when Synthetic
 did (Toka_ls, 2026-09-02), withheld while Hyper would serve it; Kimi-K3,
 withheld from the select seats while Hyper would serve it and from every
 seat while only OpenRouter would (the owner's cost decision of 2026-09-03),
 with the next served checker of the measured order keeping the bench's
 width; the full bench when
 the view cannot be read; and the static drops holding either way.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ANTHROPIC_SERVED_IDS,
  ROSTER_MODEL_IDS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type BudgetView,
  OPENROUTER_WITHHELD,
} from '@monochromatic-dev/translation-repair-provider';
import {
  HYPER_SLOW_JUDGES,
  HYPER_SLOW_SELECT_JUDGES,
  judgeSeatsFor,
  OWNER_CULLED,
  RUN_CHECKER_ORDER,
  RUN_DECISION_JUDGES,
  RUN_LATE_JUDGES,
  RUN_MODELS,
  RUN_READER_MODELS,
  RUN_ROSTER,
  RUN_SELECT_JUDGES,
  RUN_TRANSLATORS,
  RUN_WIDE_SEATS,
  RUN_WRITERS,
  TRANSLATOR_DROPPED,
  WRITER_UNMEASURED,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_DECISIONS,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  BEDROCK_ONLY_ROSTER_IDS,
  SEATED_BEDROCK_JUDGES,
  SEATED_OPENROUTER_JUDGES,
} from '@monochromatic-dev/translation-repair-shared-contract/ts/roster-buckets.test-fixture.ts';

/**
 The Hyper-slow judge.
 */
const QWEN = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 The judge Hyper serves too slowly in the select role alone, and the one the
 owner declined to pay OpenRouter's rate on.
 */
const KIMI = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Nobody dry.
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
 Synthetic dry, Hyper serving the shared seats.
 */
const SYNTHETIC_DRY: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  hyper: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: false,
};

/**
 Only OpenRouter left.
 */
const OPENROUTER_ONLY: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  // DRY: no Claude key in this reading, so Haiku does not take the second checker place a served Haiku would; a wet
  // Claude key is held by `run-config-seating.unit.test.ts` (the seat) and `pass-reseat.unit.test.ts` (the pass-level hook).
  anthropic: true,
  openrouter: false,
};

await describe({
  name: judgeSeatsFor.name,
  children: [
    it({
      name: 'WITHHOLDS the Hyper-slow judge from the wide seats when Hyper would serve it and SEATS it there when '
        + 'Synthetic does, KEEPS it off the late-judge bench either way, with the static drops holding either way',
      fn: async () => {
        expect(HYPER_SLOW_JUDGES.has(QWEN,),).toBe(true,);
        expect(RUN_WIDE_SEATS.includes(QWEN,),).toBe(true,);
        // THE OWNER'S ANSWER OF 2026-10-22: "Remove from both benches", the
        // consolidation slate and the late-judge rounds (ledger B815). The
        // consolidation slate's judges are the late judges, so the hold reaches both.
        expect(RUN_LATE_JUDGES.includes(QWEN,),).toBe(false,);

        const dry = judgeSeatsFor({ dry: SYNTHETIC_DRY, },);
        expect(dry.wideSeats.includes(QWEN,),).toBe(false,);
        expect(dry.lateJudges.includes(QWEN,),).toBe(false,);
        expect(dry.wideSeats.length,).toBe(RUN_WIDE_SEATS.length - 1,);
        expect(dry.lateJudges,).toEqual(RUN_LATE_JUDGES,);
        expect(dry.repairModels.criticModelIds,).toEqual(dry.wideSeats,);
        expect(dry.repairModels.panelModelIds,).toEqual(dry.wideSeats,);
        expect(dry.repairModels.judgeModelIds,).toEqual(dry.selectJudges,);
        expect(dry.translateModels.judgeModelIds,).toEqual(dry.selectJudges,);
        expect(dry.wideSeats.includes(SEAT_HYPER_OPENROUTER_VISION_EDITOR,),).toBe(false,);
        expect(dry.withheld.includes(QWEN,),).toBe(true,);

        const wet = judgeSeatsFor({ dry: ALL_WET, },);
        expect(wet.wideSeats,).toEqual(RUN_WIDE_SEATS,);
        expect(wet.lateJudges,).toEqual(RUN_LATE_JUDGES,);
        expect(wet.selectJudges,).toEqual(RUN_SELECT_JUDGES,);
        expect(wet.slateJudges,).toEqual(RUN_LATE_JUDGES,);
        expect(wet.checkers,).toEqual(RUN_MODELS.checkerModelIds,);
        expect(wet.translators,).toEqual(RUN_TRANSLATORS,);
        expect(wet.readers,).toEqual(RUN_READER_MODELS,);
        expect(wet.roster,).toEqual(RUN_ROSTER,);
        expect(wet.translateModels.translatorModelIds,).toEqual(RUN_TRANSLATORS,);
        expect(wet.withheld,).toEqual([],);
        expect(wet.wideSeats.includes(SEAT_HYPER_OPENROUTER_VISION_EDITOR,),).toBe(false,);
      },
    },),

    it({
      name: 'SEATS THE MEASURED BEDROCK-ONLY SIZES WHERE THEY WERE MEASURED AND NOWHERE ELSE while the roster '
        + 'names both: google.gemma-4-e2b judges (11 of 12 on the fidelity probe of 2026-09-07), reads no '
        + 'pictures (two readings no seated reader corroborated, 2026-09-08) and writes in both lanes (30 of '
        + '298 disinterested ballots in the producer calibration of 2026-09-08, not separated from the pooled '
        + 'null); google.gemma-4-31b reads pictures (8 of 8 corroborated), judges nothing and writes nothing; '
        + 'inception/mercury-2.5 judges (14 of 14 on the fidelity probe of 2026-09-09) and writes the '
        + 'consolidation alone since the producer calibration of 2026-09-19 read it below the pooled null '
        + '(10 of 108 disinterested ballots, z -3.00 across the threshold of 2.77); deepseek-v4.1-flash writes '
        + 'in both lanes since that calibration (29 of 121, z +0.79, not separated from the null)',
      fn: async function seatsByMeasurement(): Promise<void> {
        const wet = judgeSeatsFor({ dry: ALL_WET, },);
        for (const candidate of BEDROCK_ONLY_ROSTER_IDS) {
          expect(ROSTER_MODEL_IDS.includes(candidate,),).toBe(true,);
          expect(wet.checkers.includes(candidate,),).toBe(false,);
        }
        expect(wet.translators.includes(SEAT_BEDROCK_ONLY_TEXT,),).toBe(true,);
        expect(wet.writers.includes(SEAT_BEDROCK_ONLY_TEXT,),).toBe(true,);
        expect(wet.translators.includes(SEAT_BEDROCK_ONLY_VISION_UNSEATED,),).toBe(false,);
        expect(wet.writers.includes(SEAT_BEDROCK_ONLY_VISION_UNSEATED,),).toBe(false,);
        expect(wet.readers.includes(SEAT_BEDROCK_ONLY_TEXT,),).toBe(false,);
        expect(wet.readers.includes(SEAT_BEDROCK_ONLY_VISION_UNSEATED,),).toBe(true,);
        for (const seated of SEATED_BEDROCK_JUDGES) {
          expect(RUN_ROSTER.includes(seated,),).toBe(true,);
          expect(wet.roster.includes(seated,),).toBe(true,);
          expect(wet.wideSeats.includes(seated,),).toBe(true,);
          expect(wet.selectJudges.includes(seated,),).toBe(true,);
          expect(wet.lateJudges.includes(seated,),).toBe(true,);
          expect(wet.slateJudges.includes(seated,),).toBe(true,);
        }
        expect(RUN_ROSTER.includes(SEAT_BEDROCK_ONLY_VISION_UNSEATED,),).toBe(false,);
        expect(wet.wideSeats.includes(SEAT_BEDROCK_ONLY_VISION_UNSEATED,),).toBe(false,);
        // A model one provider alone serves holds no seat until measured in.
        for (const seated of SEATED_OPENROUTER_JUDGES) {
          expect(RUN_ROSTER.includes(seated,),).toBe(true,);
          expect(wet.roster.includes(seated,),).toBe(true,);
          expect(wet.wideSeats.includes(seated,),).toBe(true,);
          expect(wet.selectJudges.includes(seated,),).toBe(true,);
          expect(wet.lateJudges.includes(seated,),).toBe(true,);
          expect(wet.slateJudges.includes(seated,),).toBe(true,);
          expect(wet.checkers.includes(seated,),).toBe(false,);
          expect(wet.readers.includes(seated,),).toBe(false,);
        }
        // The calibration read at 20:02 UTC on 2026-09-09 seated the one
        // OpenRouter-only model as a writer; the one read at 16:45 UTC on
        // 2026-09-19 took its translator seat back (z -3.00 below the pooled
        // null, across the Bonferroni threshold of 2.77) and left it the
        // consolidation seat, the same shape as deepseek-v4-pro-0813's exit.
        expect(wet.translators.includes(SEAT_OPENROUTER_ONLY,),).toBe(false,);
        expect(wet.writers.includes(SEAT_OPENROUTER_ONLY,),).toBe(true,);
        // The same calibration read deepseek-v4.1-flash at the null (z +0.79),
        // so by the rule it takes the translator and the consolidation seat.
        expect(wet.translators.includes(SEAT_HYPER_OPENROUTER_UNMEASURED,),).toBe(true,);
        expect(wet.writers.includes(SEAT_HYPER_OPENROUTER_UNMEASURED,),).toBe(true,);
        for (const unmeasured of WRITER_UNMEASURED) {
          expect(wet.writers.includes(unmeasured,),).toBe(false,);
          expect(wet.translators.includes(unmeasured,),).toBe(false,);
        }
        expect(ROSTER_MODEL_IDS.filter(function unseated(modelId,): boolean {
          return !RUN_ROSTER.includes(modelId,);
        },).toSorted(),).toEqual([
          SEAT_BEDROCK_ONLY_VISION_UNSEATED,
          SEAT_OPENROUTER_DECISIONS,
          // The Claude cards other than Haiku 5.5: Sonnet 5.5 is an editor and sits on no derived bench, and the six
          // effort cards wait for the effort bench to name a level.
          ...ANTHROPIC_SERVED_IDS.filter(function notHaiku(id,): boolean {
            return id !== 'claude-haiku-5-5';
          },),
          // Measured as a checker on 2026-09-27 and nowhere else yet.
          SEAT_OPENROUTER_ONLY_CHECKER,
          // Culled from every role by the owner on 2026-09-24; its card
          // stays for the catalogs and this fixture.
          SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        ].toSorted(),);
        expect(wet.writers,).toEqual(RUN_WRITERS,);
        expect(RUN_WRITERS,).toEqual(RUN_ROSTER.filter(function measuredWriter(modelId,): boolean {
          return !WRITER_UNMEASURED.has(modelId,);
        },),);
        // Translators are the writers less the seats dropped from the
        // translator role: stated over the two exported lists, not by
        // restating the definition `run-config.ts` computes them with, so a
        // change to either list's rule shows here.
        expect(RUN_TRANSLATORS,).toEqual(RUN_WRITERS.filter(function keepsTranslatorSeat(modelId,): boolean {
          return !TRANSLATOR_DROPPED.has(modelId,);
        },),);
      },
    },),

    it({
      name: 'SEATS NO DECISION SEAT ON THE CONSOLIDATION SLATE under any reading: its judges\' reasons are the '
        + 'objections the final polish checks against the ORIGINAL, and a decision seat\'s reason is a '
        + 'distribution (ledger B125)',
      fn: async () => {
        for (const view of [
          ALL_WET,
          SYNTHETIC_DRY,
          OPENROUTER_ONLY,
        ]) {
          /**
           Seats under this reading.
           */
          const seats = judgeSeatsFor({ dry: view, },);
          expect(seats.slateJudges.filter(function decides(modelId,): boolean {
            return RUN_DECISION_JUDGES.includes(modelId,);
          },),).toEqual([],);
        }
        // The positive control: the select benches do seat it while OpenRouter reads wet.
        expect(judgeSeatsFor({ dry: ALL_WET, },).selectJudges.includes(SEAT_OPENROUTER_DECISIONS,),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS EVERY WRITER A PRODUCER CALIBRATION MEASURED OUT OF THE TRANSLATOR SEAT in its judge seats '
        + 'and the consolidation seat, unless the owner culled it from every role: gpt-oss-120b (2026-09-01, '
        + 'z -4.5) held those seats until the cull of 2026-09-24; the two DeepSeek V4 models measured '
        + 'out beside it left the roster on 2026-09-16',
      fn: async function keepsDroppedWritersElsewhere(): Promise<void> {
        const wet = judgeSeatsFor({ dry: ALL_WET, },);
        expect(TRANSLATOR_DROPPED.has(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(true,);
        expect(OWNER_CULLED.has(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(true,);
        for (const dropped of [...TRANSLATOR_DROPPED,].filter(function stillSeated(modelId,): boolean {
          return !OWNER_CULLED.has(modelId,);
        },)) {
          expect(wet.translators.includes(dropped,),).toBe(false,);
          expect(wet.writers.includes(dropped,),).toBe(true,);
          expect(wet.wideSeats.includes(dropped,),).toBe(true,);
          expect(wet.selectJudges.includes(dropped,),).toBe(true,);
        }
      },
    },),

    it({
      name: 'SEATS the Hyper-slow judge again in the wide and select seats when Hyper is dry too and OpenRouter would '
        + 'serve it, since the withholding is about Hyper\'s serving speed and not about the model, and KEEPS it off '
        + 'the late-judge bench, which its card holds it from',
      fn: async () => {
        const seats = judgeSeatsFor({ dry: OPENROUTER_ONLY, },);
        expect(seats.wideSeats.includes(QWEN,),).toBe(true,);
        expect(seats.lateJudges.includes(QWEN,),).toBe(false,);
        expect(seats.selectJudges.includes(QWEN,),).toBe(true,);
      },
    },),

    it({
      name: 'WITHHOLDS the select-slow judge from both lanes\' select seats and the consolidation slate when '
        + 'Hyper would serve it, KEEPS it as critic, panel, contest judge and gate, and SEATS it everywhere '
        + 'when Synthetic does',
      fn: async () => {
        expect(HYPER_SLOW_SELECT_JUDGES.has(KIMI,),).toBe(true,);
        expect(HYPER_SLOW_JUDGES.has(KIMI,),).toBe(false,);

        const dry = judgeSeatsFor({ dry: SYNTHETIC_DRY, },);
        expect(dry.selectJudges.includes(KIMI,),).toBe(false,);
        expect(dry.slateJudges.includes(KIMI,),).toBe(false,);
        expect(dry.wideSeats.includes(KIMI,),).toBe(true,);
        expect(dry.lateJudges.includes(KIMI,),).toBe(true,);
        expect(dry.checkers.includes(KIMI,),).toBe(true,);
        /**
         Select bench short of the withheld Kimi, plus the decision seats.
         */
        const selectSeatsExpected = (dry.wideSeats.length - 1) + RUN_DECISION_JUDGES.length;
        expect(dry.selectJudges.length,).toBe(selectSeatsExpected,);
        expect(dry.slateJudges.length,).toBe(dry.lateJudges.length - 1,);
        expect(dry.repairModels.criticModelIds.includes(KIMI,),).toBe(true,);
        expect(dry.repairModels.panelModelIds.includes(KIMI,),).toBe(true,);
        expect(dry.repairModels.judgeModelIds.includes(KIMI,),).toBe(false,);
        expect(dry.translateModels.judgeModelIds.includes(KIMI,),).toBe(false,);

        const wet = judgeSeatsFor({ dry: ALL_WET, },);
        expect(wet.selectJudges.includes(KIMI,),).toBe(true,);
        expect(wet.slateJudges.includes(KIMI,),).toBe(true,);
      },
    },),

    it({
      name: 'WITHHOLDS THE OPENROUTER-WITHHELD MODEL FROM EVERY SEAT when only OpenRouter would serve it, '
        + 'the owner\'s decision of 2026-09-03 on cost, THE WRITING AND READING SEATS INCLUDED, since the '
        + 'first OpenRouter-only pass bought six translations from it while every bench had it out, and '
        + 'SEATS THE SERVED CHECKERS OF THE MEASURED ORDER FIRST so the bench keeps its width',
      fn: async () => {
        expect(OPENROUTER_WITHHELD.has(KIMI,),).toBe(true,);
        expect(RUN_MODELS.checkerModelIds.includes(KIMI,),).toBe(true,);
        expect(RUN_TRANSLATORS.includes(KIMI,),).toBe(true,);
        expect(RUN_READER_MODELS.includes(KIMI,),).toBe(true,);
        for (const checker of RUN_CHECKER_ORDER) {
          expect(RUN_MODELS.editorModelIds.includes(checker,),).toBe(false,);
          expect(RUN_MODELS.refinerModelIds.includes(checker,),).toBe(false,);
        }

        const seats = judgeSeatsFor({ dry: OPENROUTER_ONLY, },);
        expect(seats.wideSeats.includes(KIMI,),).toBe(false,);
        expect(seats.selectJudges.includes(KIMI,),).toBe(false,);
        expect(seats.lateJudges.includes(KIMI,),).toBe(false,);
        expect(seats.slateJudges.includes(KIMI,),).toBe(false,);
        expect(seats.checkers.includes(KIMI,),).toBe(false,);
        expect(seats.checkers.slice(
          0,
          2,
        ),).toEqual([
          SEAT_OPENROUTER_ONLY_CHECKER,
          SEAT_OPENROUTER_ONLY,
        ],);
        expect(seats.checkers.length,).toBe(RUN_MODELS.checkerModelIds.length,);
        expect(seats.repairModels.checkerModelIds,).toEqual(seats.checkers,);
        expect(seats.translators.includes(KIMI,),).toBe(false,);
        expect(seats.translators.length,).toBe(RUN_TRANSLATORS.length - 1,);
        expect(seats.translateModels.translatorModelIds,).toEqual(seats.translators,);
        expect(seats.readers.includes(KIMI,),).toBe(false,);
        expect(seats.readers.length,).toBe(RUN_READER_MODELS.length - 1,);
        expect(RUN_ROSTER.includes(KIMI,),).toBe(true,);
        expect(seats.roster.includes(KIMI,),).toBe(false,);
        expect(seats.roster.length,).toBe(RUN_ROSTER.length - 1,);
        expect(seats.withheld,).toEqual([KIMI,],);
      },
    },),
    it({
      name: 'NAMES AS UNSERVED THE SEATED SEATS NO WET PROVIDER SERVES, once each and in bench order, and none while '
        + 'every seated seat is served or the seat is not seated at all (zheermao20: glm-5.3 and Qwen3.8-27B after '
        + 'Synthetic ran dry with Hyper dry from the start)',
      fn: async () => {
        /** NeuralWatt, Bedrock and OpenRouter wet, Synthetic and Hyper dry. */
        const syntheticAndHyperDry: BudgetView = {
          neuralwatt: false,
          synthetic: true,
          hyper: true,
          bedrock: false,
          deepseek: false,
          xiaomi: false,
          anthropic: false,
          openrouter: false,
        };
        /** Benches read under that view. */
        const seats = judgeSeatsFor({ dry: syntheticAndHyperDry, },);

        expect(seats.unserved,).toEqual([QWEN, SEAT_HYPER_ONLY,],);
        expect(seats.roster.includes(SEAT_HYPER_ONLY,),).toBe(true,);
        expect(judgeSeatsFor({ dry: ALL_WET, },).unserved,).toEqual([],);
        // QWEN is not seated here, since Hyper would serve it too slowly, so it is not named unserved either.
        expect(judgeSeatsFor({ dry: SYNTHETIC_DRY, },).unserved,).toEqual([],);
      },
    },),
  ],
},);
