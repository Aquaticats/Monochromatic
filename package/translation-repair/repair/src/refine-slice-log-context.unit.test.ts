/**
 Tests that every line the naturalness phase writes while refining a slice
 names that slice (ledger L12).

 WHY. Ledger A11 put each repair slice's work under a log context, so its
 lines read `[repair slice N]`, but the refinement phase that runs over the
 settled slices afterwards was left outside it. On TianqiChen66621's log, 421
 lines under the repair lane carried no slice, the phase's refiner, select,
 checker and probe round lines and its `decideBestCandidate` and
 `runRefineStage` lines among them, so with slices refined side by side under
 overlap the log alone could not say which slice a round belonged to.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule';
import { sliceTagged, } from '@monochromatic-dev/translation-repair-typography';
import type {
  ChunkRepairOutcome,
  RepairModels,
} from '@monochromatic-dev/translation-repair-assembly';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runRefinePhase, } from '../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Slice the phase refines. Zero, since a slice's index is its position; a
 line written outside any slice context carries no slice tag at all, so zero
 cannot pass by default.
 */
const SLICE_INDEX = 0;

/**
 Repaired slice text, one long single-line paragraph so the lane finds it
 eligible.
 */
const REPAIRED_TEXT = 'The cat is doing the sunbathing on the windowsill in every afternoon, and when the light is '
  + 'moving across the floor she is following it without any hurry at all.';

/**
 Smoother rendering the scripted rewriter returns.
 */
const SMOOTH_TEXT = 'The cat sunbathes on the windowsill every afternoon, and when the light moves across the floor '
  + 'she follows it without hurry.';

/**
 Original of the slice.
 */
const SOURCE_TEXT = '猫猫每天下午都在窗台上晒太阳。';

/**
 Roster with the lane on and refiners disjoint from checkers.
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
 The one slice pair the phase refines.
 */
const SLICES: readonly ChunkPair[] = [
  {
    source: {
      sliceIndex: SLICE_INDEX,
      text: SOURCE_TEXT,
      startOffset: 0,
      endOffset: SOURCE_TEXT.length,
      nodes: [],
    },
    target: {
      sliceIndex: SLICE_INDEX,
      text: REPAIRED_TEXT,
      startOffset: 0,
      endOffset: REPAIRED_TEXT.length,
      nodes: [],
    },
  },
];

/**
 Settled slice with one accepted issue its patch left open, so the recheck
 runs a checker round too.
 */
const OUTCOME: ChunkRepairOutcome = {
  sliceIndex: SLICE_INDEX,
  repairedText: REPAIRED_TEXT,
  changed: false,
  issues: [
    {
      issueId: 'issue/open',
      status: 'accepted',
      severity: 'major',
      claims: [],
      tallies: {},
    },
  ],
  resolvedIssueIds: [],
  candidateResolvedIssueIds: [],
  checkerReadings: {},
  recheckReadings: {},
  repairRegions: [],
  authorship: UNATTRIBUTED_TEXT,
  accuracyPatchSelected: false,
  refined: false,
  rounds: [],
  droppedDeclaredNames: [],
  nonTranslationVotes: 0,
  nonTranslationContradicted: false,
  nonTranslationStanding: false,
  heardCritics: 1,
  heardCriticIds: [],
  claimAttributions: [],
  findings: [],
  silentStages: [],
};

/**
 Client answering every stage the phase runs.
 */
const CLIENT: ModelClient = {
  chatText: async () => {
    throw new Error('chatText unused by the phase',);
  },
  chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
    /**
     Stage name from the structured-output constraint.
     */
    const stage = request.responseFormat?.json_schema.name ?? '';
    /**
     User sheet, for counting a checker sheet's issues.
     */
    const asked = request.messages.at(-1,);
    const content = (asked === undefined) ? '' : messageText({ message: asked, },);
    /**
     Scripted reply for the stage.
     */
    const scripted: unknown = (stage === 'refine_report')
      ? { rewrites: [{ paragraph: 1, newText: SMOOTH_TEXT, },], }
      : (stage === 'candidate_ballot')
        ? { best: 1, assessment: 'scripted', }
        : (stage === 'introduced_defect_report')
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
          : {
            checks: [...Array.from({ length: content.split('\nISSUE ',).length - 1, },).keys(),]
              .map(function toCheck(index,) {
                return { issue: index + 1, verdict: 'not-fixed', };
              },),
          };
    if (!request.validate(scripted,))
      throw new Error(`stub script failed the ${stage} guard`,);
    return {
      kind: 'ok',
      value: scripted,
      rawText: JSON.stringify(scripted,),
    };
  },
  quotas: async () => {
    throw new Error('quotas unused by the phase',);
  },
};

await describe({
  name: 'refinement lines name their slice (ledger L12)',
  children: [
    it({
      name: 'EVERY round line the phase writes while refining a slice carries `[repair slice N]`',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();
        await runRefinePhase({
          declaredNames: [],
          client: CLIENT,
          targetText: REPAIRED_TEXT,
          slices: SLICES,
          outcomes: [OUTCOME,],
          models: MODELS,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          // A logger reading the log context, as `repairPreparedDocument` passes it.
          l: sliceTagged({ l: logger, },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        /**
         Stage round lines the phase wrote.
         */
        const rounds = lines.filter(function isRound(line,) {
          return line.includes(' round: ',);
        },);
        expect({
          rounds: rounds.length > 0,
          allNamed: rounds.every(function names(line,) {
            return line.includes(`[repair slice ${String(SLICE_INDEX,)}]`,);
          },),
        },).toEqual({
          rounds: true,
          allNamed: true,
        },);
      },
    },),
  ],
},);
