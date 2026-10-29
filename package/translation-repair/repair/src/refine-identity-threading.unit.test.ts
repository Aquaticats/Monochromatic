/**
 Tests that the repair driver's refinement STEP hands the declared-name block
 down to the sheet the rewriters actually read.

 WHY A SEPARATE FILE FROM `refine-window-threading.unit.test.ts`. That one
 drives `runRefinePhase`, one layer below this. The link measured missing here
 is `refineSettledSlices`, the wrapper the repair driver calls, and every case
 in that file passes whether or not this wrapper forwards anything: it calls
 the phase itself.

 WHAT WAS MEASURED. On 2026-08-25, inverting this wrapper's conditional spread
 so the identity block is forwarded only when it is ABSENT failed no test in
 this package. The rewriters would then be asked to improve how a memorial
 page reads while being told nothing about which names and handles must
 survive exactly, which is the protection the declared-names guard puts there.

 THE FAILURE MODE IS INVISIBLE TO EVERY OTHER KIND OF TEST, for the reason
 already found once: the block is an optional property spread into an object
 literal, TypeScript does not excess-property-check a spread, and the sheet
 still renders without it. A wrapper that forwarded nothing compiled, linted
 and passed its own suite while asking the models a strictly smaller question.

 BOTH DIRECTIONS ARE PINNED. A document declaring nothing must get no block at
 all rather than an empty heading, so the case that asserts the heading is
 present is read against a control that asserts it is absent; without the
 control, a wrapper that pasted the heading unconditionally would pass.

 NO NETWORK. The client scripts the rewriter, the judges and the probe, and
 records every rewriter sheet.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
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
import {
  buildRefineSelectionContext,
  refineSettledSlices,
  type RefineStageMode,
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
const l = tagged({ tag: 'refine-identity-threading-test', },);

//region Fixtures

/**
 Heading the rewriter sheet gives the declared-name block.
 */
const IDENTITY_FENCE = 'DECLARED NAMES, which must survive exactly:';

/**
 Marker no prompt constant and no other fixture carries, so a match in a sheet
 can only have come from the identity block.
 */
const IDENTITY_MARK = 'ZQIDENT';

/**
 Declared names as front matter yields them, carrying the marker.
 */
const IDENTITY_CONTEXT = `- name: Mittens ${IDENTITY_MARK}\n- alias: sunbeam-cat`;

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
 Sheets one run of the step asked, by who was asked.

 @example
 ```ts
 const sheets: StepSheets = { rewriter: [], judges: [], };
 ```
 */
type StepSheets = {
  /**
   User sheet of every rewriter exchange, in order.
   */
  readonly rewriter: readonly string[];
  /**
   Whole text of every judge exchange choosing among the rewrites, in order.
   */
  readonly judges: readonly string[];
};

/**
 Runs the step and returns every sheet its rewriters and judges were asked.

 @param identityContext - declared names to thread, absent for the control

 @returns Sheets of the rewriter and judge exchanges, in order

 @example
 ```ts
 const { rewriter, judges, } = await stepSheets({ identityContext: IDENTITY_CONTEXT, },);
 ```
 */
async function stepSheets(
  { identityContext, }: { readonly identityContext?: string; },
): Promise<StepSheets> {
  /**
   User sheet of every rewriter exchange, in order.
   */
  const asked: string[] = [];
  /**
   Whole text of every judge exchange, in order: the slate carries its
   evidence in the user message and its rules in the system one.
   */
  const judged: string[] = [];

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
       User prompt of this exchange.
       */
      const last = request.messages.at(-1,);
      const content = (last === undefined) ? '' : messageText({ message: last, },);
      if (stage === 'refine_report')
        asked.push(content,);
      if (stage === 'candidate_ballot') {
        judged.push(request.messages
          .map(function textOf(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',),);
      }

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

  await refineSettledSlices({
    client,
    targetText: ARCHIVE_PARAGRAPH,
    slices: SLICES,
    outcomes: OUTCOMES,
    models: MODELS,
    ...(identityContext === undefined ? {} : { identityContext, }),
    declaredNames: [],
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return {
    rewriter: asked,
    judges: judged,
  };
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: refineSettledSlices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HANDS THE DECLARED NAMES DOWN to the rewriter sheet, so a lane asked to improve how a '
            + 'memorial page reads is told which names and handles must survive it exactly',
          fn: async () => {
            /**
             Sheets the rewriters were asked with the block threaded.
             */
            const { rewriter: sheets, } = await stepSheets({ identityContext: IDENTITY_CONTEXT, },);

            expect(sheets.length,).toBeGreaterThan(0,);
            for (const sheet of sheets) {
              expect(sheet.includes(IDENTITY_FENCE,),).toBe(true,);
              expect(sheet.includes(IDENTITY_MARK,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'SENDS NO BLOCK AT ALL when the document declares nothing, which is the control that '
            + 'makes the HANDS THE DECLARED NAMES DOWN case legible: a step pasting the heading '
            + 'unconditionally would satisfy it',
          fn: async () => {
            /**
             Sheets the rewriters were asked with nothing declared.
             */
            const { rewriter: sheets, } = await stepSheets({},);

            expect(sheets.length,).toBeGreaterThan(0,);
            for (const sheet of sheets) {
              expect(sheet.includes(IDENTITY_FENCE,),).toBe(false,);
              expect(sheet.includes(IDENTITY_MARK,),).toBe(false,);
            }
          },
        },),
        it({
          name: 'HANDS THE DECLARED NAMES to the judges choosing among the rewrites too (ledger B28): the '
            + 'rewriter was told a handle survives exactly, and a judge not told so can prefer a rewrite '
            + 'that changes it',
          fn: async () => {
            /**
             Judge sheets with the block threaded, and with nothing declared.
             */
            const [declared, bare,] = await Promise.all([
              stepSheets({ identityContext: IDENTITY_CONTEXT, },),
              stepSheets({},),
            ],);

            expect(declared.judges.length,).toBeGreaterThan(0,);
            expect(bare.judges.length,).toBeGreaterThan(0,);
            expect({
              declared: declared.judges.every(function carries(sheet,): boolean {
                return sheet.includes(IDENTITY_MARK,);
              },),
              bare: bare.judges.some(function carries(sheet,): boolean {
                return sheet.includes(IDENTITY_MARK,);
              },),
            },).toEqual({
              declared: true,
              bare: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${buildRefineSelectionContext.name} (ledger B28)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES the refine judges the declared names as labelled evidence in every mode, and none when the '
            + 'page declares nothing',
          fn: async () => {
            /**
             One mode of each kind the refine stage asks.
             */
            const modes: readonly RefineStageMode[] = [
              { kind: 'comparative', },
              {
                kind: 'objection-correction',
                groups: [{
                  origin: 'consolidation gate',
                  objections: ['The tense shifts mid-paragraph.',],
                },],
              },
            ];
            /**
             Per mode: whether the declared names reach the evidence with them, and without.
             */
            const readings = modes.map(function readingOf(mode,): string {
              /**
               Evidence with the page's declared names.
               */
              const declared = buildRefineSelectionContext({
                mode,
                sourceText: SOURCE_PARAGRAPH,
                repairedText: SMOOTH_TEXT,
                identityContext: IDENTITY_CONTEXT,
              },).evidence;
              /**
               Evidence for a page that declares nothing.
               */
              const bare = buildRefineSelectionContext({
                mode,
                sourceText: SOURCE_PARAGRAPH,
                repairedText: SMOOTH_TEXT,
              },).evidence;
              return `${mode.kind}: declared ${String(declared.some(function namesThem(entry,): boolean {
            return entry.label.startsWith('DECLARED NAMES',) && entry.text.includes(IDENTITY_MARK,);
          },),)}, bare ${String(bare.some(function namesAny(entry,): boolean {
            return entry.label.startsWith('DECLARED NAMES',);
          },),)}`;
            },);
            expect(readings,).toEqual(modes.map(function expected(mode,): string {
              return `${mode.kind}: declared true, bare false`;
            },),);
          },
        },),
      ],
    },),
  ],
},);
