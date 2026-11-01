/**
 Tests that the benches a re-seating hands a stage carry the writing benches the reading derived, backfill included.

 THE CASE: corpus pass XingZ6024 (2026-11-01, build e9b966af55) read NeuralWatt, Hyper, OpenRouter and Bedrock dry and
 Synthetic, DeepSeek, Xiaomi and Anthropic wet. The owner's answer of 2026-11-01 to "what should a run do when a writing
 bench has fewer than two reachable seats" was "Backfill, and add sonnet". A hook that re-seats under a hold hands the
 repair lane its editors and refiners and the consolidation its polish refiners, so a reader that kept the static list
 would ask the dark seats the reading had replaced.

 NOTHING HERE REACHES A PROVIDER.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  HYPER_SLOW_JUDGES,
  judgeSeatsFor,
  RUN_MODELS,
} from '@monochromatic-dev/translation-repair-run';
import type { BudgetView, } from '@monochromatic-dev/translation-repair-provider';
import {
  consolidationHooksFor,
  lanesHooksFor,
} from '../../dist/final/node/index.mjs';
import { metersAllWet, } from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';
import { expectedWritingBench, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/writing-bench-expectation.test-fixture.ts';

//region Re-seated writing bench tests

/**
 The reading of corpus pass XingZ6024: Synthetic, DeepSeek, Xiaomi and Anthropic wet, the other four dry.
 */
const XINGZ6024: BudgetView = {
  neuralwatt: true,
  synthetic: false,
  hyper: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: true,
};

/**
 A hold a hook sees running before its first reading, which makes it read the benches again.
 */
const SHORT_HOLD_MS = 40;

/**
 Logger the hooks write to.
 */
const l = tagged({ tag: 'pass-reseat-writing-bench-test', },);

/**
 A two-paragraph page the consolidation's naturalness roles read.
 */
const PREPARED = prepareDocumentPair({
  sourceText: '猫在窗边睡着了。\n\n它梦见了鱼。',
  targetText: 'The cat fell asleep by the window.\n\nIt dreamed of fish.',
},);

/**
 Builds a client that answers the XingZ6024 view with Hyper held until the first dryness read, so a hook reads the
 benches again once and no reading waits a hold out.

 @returns Client with only the seat reader's surface

 @example
 ```ts
 const client = heldClient();
 ```
 */
function heldClient() {
  /**
   Dryness reads so far, after the first of which no hold runs.
   */
  const reads = { count: 0, };
  return {
    providerDryness: async (): Promise<BudgetView> => {
      reads.count += 1;
      return XINGZ6024;
    },
    providerMeterStates: metersAllWet,
    providerHolds: () => ({
      neuralwatt: 0,
      synthetic: 0,
      bedrock: 0,
      deepseek: 0,
      xiaomi: 0,
      anthropic: 0,
      hyper: (reads.count === 0) ? SHORT_HOLD_MS : 0,
      openrouter: 0,
    }),
  };
}

/**
 The benches the XingZ6024 reading must derive for both writing roles.

 @returns Expected editor and refiner benches

 @example
 ```ts
 const { refiners, } = expectedBenches();
 ```
 */
function expectedBenches() {
  /**
   Checkers the reading seats.
   */
  const { checkers, } = judgeSeatsFor({ dry: XINGZ6024, },);
  return {
    editors: expectedWritingBench({
      role: 'editor',
      dry: XINGZ6024,
      staticSeats: RUN_MODELS.editorModelIds,
      checkerBench: checkers,
      hyperSlow: HYPER_SLOW_JUDGES,
      selfCertificationPermitted: RUN_MODELS.checkerSelfCertificationPermitted,
    },).bench,
    refiners: expectedWritingBench({
      role: 'refiner',
      dry: XINGZ6024,
      staticSeats: RUN_MODELS.refinerModelIds,
      checkerBench: checkers,
      hyperSlow: HYPER_SLOW_JUDGES,
      selfCertificationPermitted: RUN_MODELS.checkerSelfCertificationPermitted,
    },).bench,
  };
}

await describe({
  name: 'writing bench handed to a re-seated stage',
  concurrency: 1,
  children: [
    it({
      name: 'HANDS THE REPAIR LANE THE DERIVED EDITORS AND REFINERS under a hold, so a chunk after a re-seating asks the '
        + 'backfilled seats and not the dark static ones',
      fn: async () => {
        const hooks = lanesHooksFor({
          client: heldClient(),
          signal: new AbortController().signal,
          entryId: 'mittens',
        },);
        /**
         Roster the hook hands the driver for the next chunk.
         */
        const seating = await hooks.beforeSlice({ lane: 'repair', },);
        const expected = expectedBenches();
        expect(seating.repairModels?.editorModelIds,).toEqual(expected.editors,);
        expect(seating.repairModels?.refinerModelIds,).toEqual(expected.refiners,);
      },
    },),
    it({
      name: 'HANDS THE CONSOLIDATION POLISH THE DERIVED REFINERS under a hold, so the final naturalness rewrite asks the '
        + 'backfilled seats too',
      fn: async () => {
        const hooks = consolidationHooksFor({
          client: heldClient(),
          signal: new AbortController().signal,
          prepared: PREPARED,
          l,
        },);
        /**
         Seating the hook hands the slice under the hold.
         */
        const seating = await hooks.beforeSlice();
        expect(seating.roster?.polishConfig?.refinerModelIds,).toEqual(expectedBenches().refiners,);
      },
    },),
  ],
},);

//endregion Re-seated writing bench tests
