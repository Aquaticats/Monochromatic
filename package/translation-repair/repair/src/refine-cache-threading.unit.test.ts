/**
 Tests that the refinement step HANDS ITS CACHE DOWN to the phase that fills
 it.

 WHY THE CACHE IS LOAD-BEARING. The naturalness lane buys a rewriter round, a
 ballot and a defect check per refinable slice. Without a cache,
 a resumed run republishes nothing and rebuys all of it, and
 every cached stage now republishes its findings rather than going quiet.
 The cache is how a run that was interrupted costs what it already paid.

 WHAT WAS MEASURED. On 2026-08-25, inverting this step's conditional spread so
 the cache is forwarded only when it is ABSENT failed no test in this package.
 The lane would then settle correctly, log correctly, and persist nothing, so
 the defect shows up only on the next run and only as a bill.

 READ OFF THE CACHE, not off the settlement. A step that forwarded nothing
 still returns the right text; what it stops doing is writing.

 THE SECOND CASE IS THE ABSENT ONE, since a step that manufactured a cache of
 its own would satisfy the first on its own. A lane given none must still
 settle rather than refuse.

 NO NETWORK. Each stage is scripted by the schema it asks for, and the cache
 records the keys it was asked to persist rather than writing any file.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import type {
  ChunkRepairOutcome,
  RepairModels,
} from '@monochromatic-dev/translation-repair-assembly';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type RefinedSliceSettlement,
  refineSettledSlices,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the step under test.
 */
const l = tagged({ tag: 'refine-cache-threading-test', },);

//region Fixtures

/**
 Long single-line paragraph, which is the shape the lane finds refinable.
 */
const ARCHIVE_PARAGRAPH =
  'The cat is doing the sunbathing on the windowsill in every afternoon, and when the light is moving across the floor she is following it without any hurry at all.';

/**
 Invented original of the same slice.
 */
const SOURCE_PARAGRAPH = '猫猫每天下午都在窗台上晒太阳。';

/**
 Smoother rendering the scripted rewriter returns.
 */
const SMOOTH_TEXT =
  'The cat sunbathes on the windowsill every afternoon, and when the light moves across the floor she follows it without hurry.';

/**
 Roster with the lane on, refiners disjoint from checkers as the phase
 requires.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
  refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
};

/**
 The one slice every case here refines.
 */
const SLICES: readonly ChunkPair[] = [
  {
    source: {
      sliceIndex: 0,
      text: SOURCE_PARAGRAPH,
      startOffset: 0,
      endOffset: SOURCE_PARAGRAPH.length,
      nodes: [],
    },
    target: {
      sliceIndex: 0,
      text: ARCHIVE_PARAGRAPH,
      startOffset: 0,
      endOffset: ARCHIVE_PARAGRAPH.length,
      nodes: [],
    },
  },
];

/**
 Settled accuracy outcome the step refines.
 */
const OUTCOMES: readonly ChunkRepairOutcome[] = [
  {
    sliceIndex: 0,
    repairedText: ARCHIVE_PARAGRAPH,
    changed: false,
    issues: [],
    resolvedIssueIds: [],
    candidateResolvedIssueIds: [],
    // No checker round in this fixture, so nothing was said about any issue.
    checkerReadings: {},
    recheckReadings: {},
    repairRegions: [],
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    // Hand-written fixture text, so nothing here has a model author.
    authorship: {
      perIssue: {},
      everyIssue: [],
    },
    nonTranslationVotes: 0,
    nonTranslationContradicted: false,
    nonTranslationStanding: false,
    heardCritics: 1,
    heardCriticIds: [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  },
];

/**
 Runs the step and reports what its cache was asked to keep.

 @param withCache - whether to hand the step a cache at all

 @returns Keys the step persisted, empty when it was given no cache

 @example
 ```ts
 const kept = await keysKept({ withCache: true, },);
 ```
 */
async function keysKept(
  { withCache, }: { readonly withCache: boolean; },
): Promise<readonly string[]> {
  /**
   Every key the step asked to have written, in order.
   */
  const persisted: string[] = [];

  /**
   Cache that records rather than writes, and starts holding nothing so every
   slice is settled fresh.
   */
  const recording: SliceCache<RefinedSliceSettlement> = {
    resumed: new Map(),
    persist: async ({ key, }: { readonly key: string; readonly serialized: string; },) => {
      persisted.push(key,);
    },
  };

  /**
   Client scripting each stage by the schema it asks for.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText unused by the step',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage name from the structured-output constraint.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name
        ?? '';

      /**
       Scripted reply for the stage.
       */
      const scripted: unknown = stage === 'refine_report'
        ? {
          rewrites: [
            {
              paragraph: 1,
              newText: SMOOTH_TEXT,
            },
          ],
        }
        : stage === 'candidate_ballot'
          ? {
            best: 1,
            assessment: 'scripted',
          }
          : stage === 'introduced_defect_report'
            ? {
              checks: [
                {
                  region: 1,
                  verdict: 'no-introduced-defect-found',
                  category: '',
                  severity: '',
                  evidence: '',
                  omittedText: '',
                  verdictReason: '',
                },
              ],
            }
            : { checks: [], };
      if (!request.validate(scripted,))
        throw new Error(`stub script failed the ${stage} guard`,);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the step',);
    },
  };

  /**
   What the step settled on, read only to prove the rewriters were reached:
   a run that refined nothing would persist nothing for an innocent reason.
   */
  const phase = await refineSettledSlices({
    client,
    targetText: ARCHIVE_PARAGRAPH,
    slices: SLICES,
    outcomes: OUTCOMES,
    models: MODELS,
    declaredNames: [],
    ...(withCache ? { refineCache: recording, } : {}),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  expect(phase.askedRewriters,).toBe(true,);

  return persisted;
}

//endregion Fixtures

await describe({
  name: refineSettledSlices.name,
  children: [
    it({
      name: 'KEEPS WHAT IT SETTLED, handing its cache down to the phase that fills it, since a lane '
        + 'that forwards nothing settles correctly and persists nothing, and the bill for that arrives '
        + 'only on the next run',
      fn: async () => {
        /**
         Keys the step asked its cache to keep.
         */
        const kept = await keysKept({ withCache: true, },);

        expect(kept.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'SETTLES WITH NO CACHE AT ALL, which is the control: a step manufacturing one of its own '
        + 'would satisfy the "KEEPS WHAT IT SETTLED" case without ever having been handed anything',
      fn: async () => {
        /**
         Keys kept when the step was handed no cache.
         */
        const kept = await keysKept({ withCache: false, },);

        expect(kept,).toStrictEqual([],);
      },
    },),
  ],
},);
