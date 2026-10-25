/**
 Tests that the naturalness lane's slice settler does not stop on the critics'
 non-translation votes (ledger L15), and that a rewrite ships only past a
 recheck its checkers were heard on and a damage probe its probers were heard
 on (ledger L11).

 WHY THIS FILE WAS WRONG. It was written on 2026-08-24 to defend an early
 return that deleting left the whole suite green, on the reading that a slice
 the critics ruled non-translation "shipped deliberately untouched". That had
 stopped being true on 2026-08-16: question 3, answer B
 (`doc/decision/translation-repair-question-answers.md`) keeps critics as
 evidence "rather than deciding anything", removes every early return they
 owned, and `05ff9791e` stopped `repairChunk` returning its input unchanged
 when the votes stand. This early return was the one left: over every
 artifact, 10 of 6,150 slices had standing votes, the repair lane changed 8
 of them, and none of the 9 in runs that refine was refined.

 SO BOTH CASES REACH THE CLIENT. A client that throws on any exchange makes
 reaching a model observable; a slice standing as non-translation must reach
 it exactly as the same slice without the ruling does.

 THE SCRIPTED CASES RUN THE WHOLE FLOW against a client that answers each
 stage by the name of its structured-output constraint, with a reply that
 stage's own wire guard accepts, and each asserts the settlement's findings
 whole, so a stage the script failed to answer shows as a lost voice in the
 case that ran it. The recheck gate's own verdicts, round by round, are in
 `refine-recheck.unit.test.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ResolutionVerdict, } from '@monochromatic-dev/translation-repair-adjudication';
import type {
  ChunkRepairOutcome,
  RepairModels,
} from '@monochromatic-dev/translation-repair-assembly';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { settleRefinedSlice, } from '../dist/final/node/index.mjs';
import {
  type AskedSheet,
  askedSheetOf,
  CITED_REFERENCES_HEADING,
  DECLARED_NAMES_HEADING,
  stagesAsked,
  stagesCarrying,
} from '@monochromatic-dev/translation-repair-provider/ts/asked-sheets.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  ADDED_WORDING,
  CHECKERS,
  everyCheckerSays,
  REPAIRED_TEXT,
  REWRITTEN_TEXT,
  SOURCE_TEXT,
  SUNBATHING_ISSUE,
  sunbathingOutcome,
} from '@monochromatic-dev/translation-repair-assembly/ts/sunbathing-recheck.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the settler under test.
 */
const l = tagged({ tag: 'refine-slice-settle-test', },);

/**
 Message the refusing client throws with, so a case can tell its own refusal
 apart from any other failure.
 */
const CLIENT_WAS_REACHED = 'the refusing client was asked for a completion';

/**
 Model this lane hands a paragraph to for rewriting, named once so the roster
 and the finding a case reads both spell the same id.
 */
const REFINER: RosterModelId = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Refiner roster, one model so a lost voice leaves no quorum and the stage's
 own account of what happened is unambiguous.
 */
const REFINERS: readonly RosterModelId[] = [REFINER,];

/**
 Roster with the lane on and refiners disjoint from checkers, as production
 runs it.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
  ],
  refinerModelIds: REFINERS,
  checkerModelIds: CHECKERS,
};

/**
 Structured-output name of the rewriter's exchange, which is how the scripted
 client and a case tell the stages apart.
 */
const REFINER_STAGE = 'refine_report';

/**
 Structured-output name of a selection judge's exchange.
 */
const JUDGE_STAGE = 'candidate_ballot';

/**
 Structured-output name of a recheck checker's exchange.
 */
const CHECKER_STAGE = 'resolution_report';

/**
 Structured-output name of a damage prober's exchange.
 */
const PROBER_STAGE = 'introduced_defect_report';

/**
 Stages a scripted run asks when the slice carries no accepted issue, one
 entry per exchange: the one refiner, the three judges, and the three probers.
 No checker is asked, since the recheck buys no round for a slice with
 nothing to rule on.
 */
const STAGES_WITHOUT_RECHECK: readonly string[] = [
  REFINER_STAGE,
  JUDGE_STAGE,
  JUDGE_STAGE,
  JUDGE_STAGE,
  PROBER_STAGE,
  PROBER_STAGE,
  PROBER_STAGE,
];

/**
 Stages a scripted run asks when the slice carries the open issue and every
 stage is heard: the three checkers rule between the judges and the probers.
 */
const STAGES_WITH_RECHECK: readonly string[] = [
  REFINER_STAGE,
  JUDGE_STAGE,
  JUDGE_STAGE,
  JUDGE_STAGE,
  CHECKER_STAGE,
  CHECKER_STAGE,
  CHECKER_STAGE,
  PROBER_STAGE,
  PROBER_STAGE,
  PROBER_STAGE,
];

/**
 Findings every scripted run leaves before its recheck: the one refiner heard
 and proposing, its own ballot discounted, and its rewrite selected.
 */
const SELECTED_FINDINGS: readonly string[] = [
  'refine-candidates (1/1 heard, 1 proposing)',
  `select-self-vote (${REFINER})`,
  'refine-selected (weight 2.5 of 3 ballots)',
];

/**
 What a scripted prober answers: a report raising no claim, a reply the
 probe stage's guard refuses so the stage never hears it, or a report whose
 one claim quotes wording the rewrite added.
 */
type ProberAnswer = 'no-claim' | 'unreadable' | 'added-claim';

/**
 Report each prober answer sends, every one but the unreadable one shaped as
 the probe stage's guard accepts.
 */
const PROBER_REPORTS: ReadonlyMap<ProberAnswer, unknown> = new Map<ProberAnswer, unknown>([
  [
    'no-claim',
    { checks: [], },
  ],
  [
    'unreadable',
    { checks: 'none', },
  ],
  [
    'added-claim',
    {
      checks: [{
        region: 1,
        verdict: 'introduced-defect',
        category: 'fluency/addition',
        severity: 'minor',
        evidence: ADDED_WORDING,
        omittedText: '',
        verdictReason: 'the cat never said it was rewritten',
      },],
    },
  ],
],);

/**
 Prober answers of a damage probe that heard none of its three probers.
 */
const NO_PROBER_HEARD: ReadonlyMap<RosterModelId, ProberAnswer> = new Map(CHECKERS.map(function toEntry(
  modelId,
): readonly [
  RosterModelId,
  ProberAnswer,
] {
  return [
    modelId,
    'unreadable',
  ];
},),);

/**
 Lost voices the damage probe reports for a round that heard none of its
 three probers, in the bench's seat order.
 */
const EVERY_PROBER_LOST: readonly string[] = CHECKERS.map(function toFinding(modelId,): string {
  return `stage-voice-lost (introduced-defect-probe ${modelId})`;
},);

/**
 Heading the damage probe's sheet prints over the neighbouring original.
 */
const NEARBY_ORIGINAL_HEADING = 'NEARBY ORIGINAL, CONTEXT ONLY';

/**
 Heading the damage probe's sheet prints over the neighbouring archive text.
 */
const NEARBY_TRANSLATION_HEADING = 'NEARBY EXISTING TRANSLATION, CONTEXT ONLY';

/**
 Declared identity a case hands in.
 */
const IDENTITY_CONTEXT = 'The translator signs as 喵工作室.';

/**
 Cited reference a case hands in.
 */
const REFERENCE_CONTEXT = 'Cat naps are documented in the glossary.';

/**
 Original of the neighbouring slice a case hands in.
 */
const NEIGHBOUR_SOURCE = '邻猫每天下午都在窗台上晒太阳。';

/**
 Archive English of that neighbouring slice.
 */
const NEIGHBOUR_INCUMBENT = 'The neighbouring cat naps there too.';

/**
 Client that throws on any exchange, so reaching a model is observable.

 NOT A RECORDING CLIENT. A counter would say how many calls happened and
 would let a case pass while quietly buying something; throwing makes the
 first call end the settlement, which is what the assertion is about.
 */
const REFUSING_CLIENT: ModelClient = {
  chatText(): never {
    throw new Error(CLIENT_WAS_REACHED,);
  },
  chatJson(): never {
    throw new Error(CLIENT_WAS_REACHED,);
  },
  quotas(): never {
    throw new Error(CLIENT_WAS_REACHED,);
  },
};

/**
 Client that answers every stage the settle flow asks and records each
 exchange's stage and sheet, so a case can read what the flow asked in order.

 SCRIPTED BY STAGE, each stage getting the reply its own wire guard accepts:
 the rewriter one rewrite of the one paragraph, a judge a ballot for it, a
 prober a report raising no claim unless a case scripts another answer, and
 a checker its verdict on the one issue. A checker given no verdict sends an
 empty report, which the checker stage's guard refuses (ledger L8), so the
 stage never hears that checker; a prober scripted unreadable sends a report
 the probe stage's guard refuses. A reply its stage refuses comes back as a
 schema mismatch, as a provider's would, and shows as a lost voice in the
 findings every scripted case asserts whole.

 @param asked - sink receiving each exchange in the order the flow asked

 @param checkerVerdicts - verdict each checker casts on the one issue, by
 model id; a checker it leaves out is never heard

 @param proberAnswers - what each prober answers, by model id; a prober it
 leaves out raises no claim

 @returns Client usable by the settle flow

 @example
 ```ts
 const client = scriptedSettleClient({ asked: [], },);
 ```
 */
function scriptedSettleClient(
  {
    asked,
    checkerVerdicts = new Map<RosterModelId, ResolutionVerdict>(),
    proberAnswers = new Map<RosterModelId, ProberAnswer>(),
  }: {
    readonly asked: AskedSheet[];
    readonly checkerVerdicts?: ReadonlyMap<RosterModelId, ResolutionVerdict>;
    readonly proberAnswers?: ReadonlyMap<RosterModelId, ProberAnswer>;
  },
): ModelClient {
  return {
    chatText(): never {
      throw new Error(CLIENT_WAS_REACHED,);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage asking and the sheet it composed.
       */
      const exchange = askedSheetOf({ request, },);
      asked.push(exchange,);
      /**
       Verdict this checker casts, absent for a checker never heard and for
       every other stage's models.
       */
      const verdict = checkerVerdicts.get(request.modelId,);
      /**
       Reply each stage's own wire guard accepts, by stage name.
       */
      const replies: ReadonlyMap<string, unknown> = new Map<string, unknown>([
        [
          REFINER_STAGE,
          {
            rewrites: [{
              paragraph: 1,
              newText: REWRITTEN_TEXT,
            },],
          },
        ],
        [
          JUDGE_STAGE,
          {
            best: 1,
            assessment: 'scripted',
          },
        ],
        [
          CHECKER_STAGE,
          {
            checks: (verdict === undefined)
              ? []
              : [{
                issue: 1,
                verdict,
              },],
          },
        ],
        [
          PROBER_STAGE,
          PROBER_REPORTS.get(proberAnswers.get(request.modelId,) ?? 'no-claim',),
        ],
      ],);
      /**
       Reply scripted for the stage asking.
       */
      const reply = replies.get(exchange.stage,);
      if (request.validate(reply,))
        return {
          kind: 'ok',
          value: reply,
          rawText: JSON.stringify(reply,),
        };
      return {
        kind: 'schema-mismatch',
        rawText: '',
        detail: `the ${exchange.stage} guard refused the scripted reply`,
      };
    },
    quotas(): never {
      throw new Error(CLIENT_WAS_REACHED,);
    },
  };
}

/**
 Settles one slice, against the refusing client unless a case scripts one.

 @param nonTranslationStanding - whether this slice stands as non-translation

 @param client - client the flow asks, the refusing one by default

 @param neighbouringSourceText - original of the passages either side

 @param neighbouringIncumbentText - archive English of those passages

 @param identityContext - declared names and handles

 @param referenceContext - what the pages the original cites say

 @param issues - issues the accuracy lane settled for the slice

 @returns What the lane settled on

 @example
 ```ts
 const settled = await settleWith({ nonTranslationStanding: true, },);
 ```
 */
async function settleWith(
  {
    nonTranslationStanding,
    client = REFUSING_CLIENT,
    neighbouringSourceText,
    neighbouringIncumbentText,
    identityContext,
    referenceContext,
    issues,
  }: {
    readonly nonTranslationStanding: boolean;
    readonly client?: ModelClient;
    readonly neighbouringSourceText?: string;
    readonly neighbouringIncumbentText?: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly issues?: ChunkRepairOutcome['issues'];
  },
): Promise<Awaited<ReturnType<typeof settleRefinedSlice>>> {
  return await settleRefinedSlice({
    client,
    outcome: {
      ...sunbathingOutcome({ nonTranslationStanding, },),
      ...(issues === undefined ? {} : { issues, }),
    },
    sourceText: SOURCE_TEXT,
    incumbentText: REPAIRED_TEXT,
    definitions: '',
    models: MODELS,
    refinerModelIds: REFINERS,
    declaredNames: [],
    ...(neighbouringSourceText === undefined ? {} : { neighbouringSourceText, }),
    ...(neighbouringIncumbentText === undefined ? {} : { neighbouringIncumbentText, }),
    ...(identityContext === undefined ? {} : { identityContext, }),
    ...(referenceContext === undefined ? {} : { referenceContext, }),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: settleRefinedSlice.name,
  children: [
    it({
      name: 'REACHES THE CLIENT for a slice standing as non-translation: the votes are evidence for the judges, '
        + 'not a stop (question 3, answer B), so the lane asks its rewriter as it would for any slice',
      fn: async () => {
        /**
         Settlement of a slice the critics ruled non-translation.
         */
        const settled = await settleWith({ nonTranslationStanding: true, },);

        expect(settled.asked,).toBe(true,);
        expect(settled.findings,)
          .toContain(`stage-voice-lost (refiner ${REFINER})`,);
        expect(settled.refinedBy,).toEqual([],);
        expect(settled.refinersHeard,).toEqual([],);
      },
    },),

    it({
      name: 'REACHES THE CLIENT for the same slice without the ruling, the same way. The refiner stage '
        + 'catches the refusal and records it as a lost voice, so what the client saw is legible in the '
        + 'findings rather than in an exception',
      fn: async () => {
        /**
         Settlement of the same slice with the ruling lifted.
         */
        const settled = await settleWith({ nonTranslationStanding: false, },);

        expect(settled.asked,).toBe(true,);
        expect(settled.findings,)
          .toContain(`stage-voice-lost (refiner ${REFINER})`,);
        expect(settled.refinedBy,).toEqual([],);
        // The one refiner's voice was lost, so nobody was heard either.
        expect(settled.refinersHeard,).toEqual([],);
      },
    },),

    it({
      name: 'CARRIES the neighbouring slice\'s source and incumbent onto the three damage probers\' sheets '
        + 'and no other stage\'s, each under its NEARBY heading, so a rewrite dropping repetition reads '
        + 'against its neighbour, and ships the rewrite every stage was heard on',
      fn: async () => {
        /**
         Exchanges the flow asked, in order.
         */
        const asked: AskedSheet[] = [];
        /**
         Settlement of a slice with a neighbour on each side.
         */
        const settled = await settleWith({
          nonTranslationStanding: true,
          client: scriptedSettleClient({ asked, },),
          neighbouringSourceText: NEIGHBOUR_SOURCE,
          neighbouringIncumbentText: NEIGHBOUR_INCUMBENT,
        },);
        for (
          const text of [
            NEARBY_ORIGINAL_HEADING,
            NEIGHBOUR_SOURCE,
            NEARBY_TRANSLATION_HEADING,
            NEIGHBOUR_INCUMBENT,
          ]
        ) {
          expect(stagesCarrying({
            asked,
            text,
          },),).toEqual([
            PROBER_STAGE,
            PROBER_STAGE,
            PROBER_STAGE,
          ],);
        }
        expect(settled.findings,).toEqual(SELECTED_FINDINGS,);
        expect(settled.outcome.refined,).toBe(true,);
        // The slice stands as non-translation, and the rewrite still ships
        // under its refiner's name (ledger L15).
        expect(settled.refinedBy,).toEqual(REFINERS,);
      },
    },),

    it({
      name: 'PRINTS neither NEARBY heading on any sheet of a slice that stands alone, the three damage '
        + 'probers\' sheets among them',
      fn: async () => {
        /**
         Exchanges the flow asked with no neighbour handed in.
         */
        const asked: AskedSheet[] = [];
        /**
         Settlement of a slice with no neighbour.
         */
        const settled = await settleWith({
          nonTranslationStanding: true,
          client: scriptedSettleClient({ asked, },),
        },);
        expect(stagesAsked({ asked, },),).toEqual(STAGES_WITHOUT_RECHECK,);
        for (const heading of [NEARBY_ORIGINAL_HEADING, NEARBY_TRANSLATION_HEADING,]) {
          expect(stagesCarrying({
            asked,
            text: heading,
          },),).toEqual([],);
        }
        expect(settled.findings,).toEqual(SELECTED_FINDINGS,);
      },
    },),

    it({
      name: 'CARRIES the identity context under the DECLARED NAMES heading onto the sheets of all four '
        + 'stages, the three recheck checkers\' among them (ledger L14), and the reference context under '
        + 'the CITED REFERENCES heading onto the rewriter\'s, the judges\' and the checkers\' sheets, the '
        + 'damage probers\' sheets taking no reference',
      fn: async () => {
        /**
         Exchanges the flow asked with both contexts in.
         */
        const asked: AskedSheet[] = [];
        /**
         Settlement of a slice with an open issue, so the recheck asks its
         checkers, every one of them answering.
         */
        const settled = await settleWith({
          nonTranslationStanding: true,
          client: scriptedSettleClient({
            asked,
            checkerVerdicts: everyCheckerSays({ verdict: 'not-fixed', },),
          },),
          issues: [SUNBATHING_ISSUE,],
          identityContext: IDENTITY_CONTEXT,
          referenceContext: REFERENCE_CONTEXT,
        },);
        for (const text of [DECLARED_NAMES_HEADING, IDENTITY_CONTEXT,]) {
          expect(stagesCarrying({
            asked,
            text,
          },),).toEqual(STAGES_WITH_RECHECK,);
        }
        for (const text of [CITED_REFERENCES_HEADING, REFERENCE_CONTEXT,]) {
          expect(stagesCarrying({
            asked,
            text,
          },),).toEqual([
            REFINER_STAGE,
            JUDGE_STAGE,
            JUDGE_STAGE,
            JUDGE_STAGE,
            CHECKER_STAGE,
            CHECKER_STAGE,
            CHECKER_STAGE,
          ],);
        }
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          'refine-recheck-passed (1 issue)',
        ],);
      },
    },),

    it({
      name: 'PRINTS neither the DECLARED NAMES nor the CITED REFERENCES heading on any sheet of the four '
        + 'stages where no context was handed in',
      fn: async () => {
        /**
         Exchanges the same flow asked with no context in.
         */
        const asked: AskedSheet[] = [];
        /**
         Settlement of the slice with the open issue and no context.
         */
        const settled = await settleWith({
          nonTranslationStanding: true,
          client: scriptedSettleClient({
            asked,
            checkerVerdicts: everyCheckerSays({ verdict: 'not-fixed', },),
          },),
          issues: [SUNBATHING_ISSUE,],
        },);
        expect(stagesAsked({ asked, },),).toEqual(STAGES_WITH_RECHECK,);
        for (const heading of [DECLARED_NAMES_HEADING, CITED_REFERENCES_HEADING,]) {
          expect(stagesCarrying({
            asked,
            text: heading,
          },),).toEqual([],);
        }
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          'refine-recheck-passed (1 issue)',
        ],);
      },
    },),

    it({
      name: 'ROLLS BACK a rewrite whose recheck heard none of its three checkers: the findings carry the '
        + 'checker stage\'s three lost voices and its unmet quorum and name the round unheard, the open '
        + 'issue\'s reading holds no ballot, and no prober is asked about a rewrite that does not ship '
        + '(ledger L11)',
      fn: async () => {
        /**
         Exchanges the flow asked.
         */
        const asked: AskedSheet[] = [];
        /**
         Settlement of a round in which every checker sent an empty report.
         */
        const settled = await settleWith({
          nonTranslationStanding: false,
          client: scriptedSettleClient({ asked, },),
          issues: [SUNBATHING_ISSUE,],
        },);
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          `stage-voice-lost (checker ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER})`,
          `stage-voice-lost (checker ${SEAT_SYNTHETIC_VISION_WITHHELD})`,
          `stage-voice-lost (checker ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
          'stage-quorum-unmet (checker 0/3)',
          'refine-recheck-unheard (0 of 3 checkers heard)',
        ],);
        expect(settled.outcome.recheckReadings,).toEqual({
          [SUNBATHING_ISSUE.issueId]: {
            ballots: [],
            configuredCheckers: 3,
            tally: {
              fixed: 0,
              notFixed: 0,
              worse: 0,
              resolved: false,
              regressed: false,
            },
          },
        },);
        expect(settled.outcome.refined,).toBe(false,);
        expect(settled.outcome.repairedText,).toBe(REPAIRED_TEXT,);
        expect(settled.refinedBy,).toEqual([],);
        // Each checker is asked once and, its empty report unreadable, once
        // more in the gather's recovery round; the flow ends there.
        expect(stagesAsked({ asked, },),).toEqual([
          REFINER_STAGE,
          JUDGE_STAGE,
          JUDGE_STAGE,
          JUDGE_STAGE,
          CHECKER_STAGE,
          CHECKER_STAGE,
          CHECKER_STAGE,
          CHECKER_STAGE,
          CHECKER_STAGE,
          CHECKER_STAGE,
        ],);
      },
    },),

    it({
      name: 'SHIPS a rewrite whose recheck every checker answered not-fixed on the open issue, under '
        + 'refine-recheck-passed and with the three ballots in the issue\'s reading',
      fn: async () => {
        /**
         Settlement of a round in which every checker found the open issue as
         it stood.
         */
        const settled = await settleWith({
          nonTranslationStanding: false,
          client: scriptedSettleClient({
            asked: [],
            checkerVerdicts: everyCheckerSays({ verdict: 'not-fixed', },),
          },),
          issues: [SUNBATHING_ISSUE,],
        },);
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          'refine-recheck-passed (1 issue)',
        ],);
        expect(settled.outcome.recheckReadings,).toEqual({
          [SUNBATHING_ISSUE.issueId]: {
            ballots: CHECKERS.map(function toBallot(modelId,) {
              return {
                modelId,
                verdict: 'not-fixed',
                wroteTheText: false,
              };
            },),
            configuredCheckers: 3,
            tally: {
              fixed: 0,
              notFixed: 3,
              worse: 0,
              resolved: false,
              regressed: false,
            },
          },
        },);
        expect(settled.outcome.refined,).toBe(true,);
        expect(settled.outcome.repairedText,).toBe(REWRITTEN_TEXT,);
        expect(settled.refinedBy,).toEqual(REFINERS,);
      },
    },),

    it({
      name: 'ROLLS BACK a rewrite two checkers of three found worse on the open issue, naming the issue, '
        + 'though the third answered not-fixed (owner, 2026-10-25, "Two ballots for undo too")',
      fn: async () => {
        /**
         Settlement of a round with two worse ballots beside one not-fixed.
         */
        const settled = await settleWith({
          nonTranslationStanding: false,
          client: scriptedSettleClient({
            asked: [],
            checkerVerdicts: new Map<RosterModelId, ResolutionVerdict>([
              ...everyCheckerSays({ verdict: 'not-fixed', },),
              [
                SEAT_SYNTHETIC_VISION_WITHHELD,
                'worse',
              ],
              [
                SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                'worse',
              ],
            ],),
          },),
          issues: [SUNBATHING_ISSUE,],
        },);
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          `refine-rolled-back (${SUNBATHING_ISSUE.issueId})`,
        ],);
        expect(settled.outcome.recheckReadings,).toEqual({
          [SUNBATHING_ISSUE.issueId]: {
            ballots: [
              {
                modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                verdict: 'not-fixed',
                wroteTheText: false,
              },
              {
                modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
                verdict: 'worse',
                wroteTheText: false,
              },
              {
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                verdict: 'worse',
                wroteTheText: false,
              },
            ],
            configuredCheckers: 3,
            tally: {
              fixed: 0,
              notFixed: 1,
              worse: 2,
              resolved: false,
              regressed: true,
            },
          },
        },);
        expect(settled.outcome.refined,).toBe(false,);
        expect(settled.outcome.repairedText,).toBe(REPAIRED_TEXT,);
        expect(settled.refinedBy,).toEqual([],);
      },
    },),

    it({
      name: 'ROLLS BACK a rewrite whose damage probe heard none of its three probers: the findings carry the '
        + 'probe stage\'s three lost voices and its unmet quorum and name the probe round unheard at 0 of 3, '
        + 'not damage the rewrite added, and the text before the rewrite ships',
      fn: async () => {
        /**
         Settlement of a slice with no accepted issue, so no recheck runs, whose
         three probers each send a report the probe stage refuses.
         */
        const settled = await settleWith({
          nonTranslationStanding: false,
          client: scriptedSettleClient({
            asked: [],
            proberAnswers: NO_PROBER_HEARD,
          },),
        },);
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          ...EVERY_PROBER_LOST,
          'stage-quorum-unmet (introduced-defect-probe 0/3)',
          'refine-probe-unheard (0 of 3 probers heard)',
        ],);
        expect(settled.outcome.refined,).toBe(false,);
        expect(settled.outcome.repairedText,).toBe(REPAIRED_TEXT,);
        expect(settled.refinedBy,).toEqual([],);
      },
    },),

    it({
      name: 'ROLLS BACK a rewrite whose damage probe heard one prober of three, short of the stage\'s quorum '
        + 'of two, though that prober raised no claim: refine-probe-unheard at 1 of 3 beside the two lost '
        + 'voices and the unmet quorum',
      fn: async () => {
        /**
         Settlement of the slice whose first prober alone is heard, raising
         nothing.
         */
        const settled = await settleWith({
          nonTranslationStanding: false,
          client: scriptedSettleClient({
            asked: [],
            proberAnswers: new Map<RosterModelId, ProberAnswer>([
              ...NO_PROBER_HEARD,
              [
                SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                'no-claim',
              ],
            ],),
          },),
        },);
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          `stage-voice-lost (introduced-defect-probe ${SEAT_SYNTHETIC_VISION_WITHHELD})`,
          `stage-voice-lost (introduced-defect-probe ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
          'stage-quorum-unmet (introduced-defect-probe 1/3)',
          'refine-probe-unheard (1 of 3 probers heard)',
        ],);
        expect(settled.outcome.refined,).toBe(false,);
        expect(settled.outcome.repairedText,).toBe(REPAIRED_TEXT,);
        expect(settled.refinedBy,).toEqual([],);
      },
    },),

    it({
      name: 'NAMES the damage a lone heard prober\'s admitted claim shows, though its round fell short of '
        + 'quorum: refine-rolled-back-by-probe counts the claim beside the probe stage\'s unmet quorum, and '
        + 'no unheard finding replaces it',
      fn: async () => {
        /**
         Settlement of the slice whose first prober alone is heard, quoting
         the wording the rewrite added.
         */
        const settled = await settleWith({
          nonTranslationStanding: false,
          client: scriptedSettleClient({
            asked: [],
            proberAnswers: new Map<RosterModelId, ProberAnswer>([
              ...NO_PROBER_HEARD,
              [
                SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                'added-claim',
              ],
            ],),
          },),
        },);
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          `stage-voice-lost (introduced-defect-probe ${SEAT_SYNTHETIC_VISION_WITHHELD})`,
          `stage-voice-lost (introduced-defect-probe ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
          'stage-quorum-unmet (introduced-defect-probe 1/3)',
          'refine-rolled-back-by-probe (1 added-damage and 0 removal claims admitted against the rewrite)',
        ],);
        expect(settled.outcome.refined,).toBe(false,);
        expect(settled.outcome.repairedText,).toBe(REPAIRED_TEXT,);
      },
    },),

    it({
      name: 'SHIPS a rewrite whose damage probe heard two probers of three, the stage\'s quorum, neither '
        + 'raising a claim: the lost prober and the short roster are named beside the selection\'s findings',
      fn: async () => {
        /**
         Settlement of the slice whose third prober alone is never heard.
         */
        const settled = await settleWith({
          nonTranslationStanding: false,
          client: scriptedSettleClient({
            asked: [],
            proberAnswers: new Map<RosterModelId, ProberAnswer>([
              [
                SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                'unreadable',
              ],
            ],),
          },),
        },);
        expect(settled.findings,).toEqual([
          ...SELECTED_FINDINGS,
          `stage-voice-lost (introduced-defect-probe ${SEAT_SYNTHETIC_TEXT_EVERYWHERE})`,
          'stage-roster-incomplete (introduced-defect-probe 2/3)',
        ],);
        expect(settled.outcome.refined,).toBe(true,);
        expect(settled.outcome.repairedText,).toBe(REWRITTEN_TEXT,);
        expect(settled.refinedBy,).toEqual(REFINERS,);
      },
    },),
  ],
},);
