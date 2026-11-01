/**
 Tests that every checker round is SENT the declared names and the cited
 references its callers hold (ledger L14).

 WHY A SEPARATE FILE FROM `resolution-sheet-evidence.unit.test.ts`. That one
 hands the builder its evidence and asserts the sheet renders it, which says
 nothing about whether any caller hands it over. The evidence is an optional
 property spread into an object literal, so a caller that forwards nothing
 compiles, lints and passes its own suite while asking the checkers a smaller
 question. Three callers own a checker round: the stage itself, the
 repair lane's proof and its worse-vote recheck (ledger L3), and the
 refinement recheck (ledger L11).

 BOTH DIRECTIONS ARE PINNED: a page declaring nothing and linking nowhere
 gets neither block, so a caller pasting them unconditionally fails too.

 NO NETWORK. Scripted clients answer every stage and record each checker
 sheet.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
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
  repairPreparedDocument,
  runCheckerStage,
  runRefinePhase,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the drivers under test.
 */
const l = tagged({ tag: 'checker-evidence-threading-test', },);

//region Fixtures

/**
 Marker no prompt constant carries, so a match can only come from the
 declared identity.
 */
const IDENTITY_MARK = 'ZQCHKIDENT';

/**
 Declared identity line carrying the marker.
 */
const IDENTITY_LINE = `- name: ORIGINAL declares "猫猫", TRANSLATION declares "The cat" ${IDENTITY_MARK}`;

/**
 Marker for the cited references, kept apart from the identity one.
 */
const REFERENCE_MARK = 'ZQCHKREF';

/**
 Cited reference line carrying the marker.
 */
const REFERENCE_CONTEXT = `- https://cats.invalid/sill: the cat suns on the sill. ${REFERENCE_MARK}`;

/**
 Invented original with one mistranslated sentence.

 NO HEADING: preparation writes the archive's rendering of a heading into the
 identity context as a line of its own, so a heading gives the control page
 a DECLARED NAMES block, which the checkers are rightly shown.
 */
const SOURCE_TEXT = '猫猫喜欢在窗台上晒太阳。\n';

/**
 Invented archive with that sentence mistranslated.
 */
const TARGET_TEXT = 'The cat hates sunbathing on the windowsill.\n';

/**
 Repair of the sentence, which every checker confirms.
 */
const SUN_REPAIR = 'The cat loves sunbathing on the windowsill.';

/**
 Smoother rendering the refinement case's rewriter returns.
 */
const SMOOTH_TEXT = 'The cat sunbathes on the windowsill every afternoon, and when the light moves across the floor '
  + 'she follows it without hurry.';

/**
 Checkers, three since fewer cannot decide.
 */
const CHECKERS = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Repair roster: one editor, the checkers doubling as critics, panel and
 judges, and a refiner outside the checker bench.
 */
const MODELS: RepairModels = {
  criticModelIds: [...CHECKERS,],
  panelModelIds: [...CHECKERS,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [...CHECKERS,],
  checkerModelIds: [...CHECKERS,],
};

/**
 What the evidence cases hand a caller, and what the controls hand it.
 */
type Evidence = {
  /**
   Declared identity, absent for a page with none.
   */
  readonly identityContext?: string;

  /**
   Cited references, absent when the original links nowhere.
   */
  readonly referenceContext?: string;
};

/**
 Evidence every carrying case threads.
 */
const CARRIED: Evidence = {
  identityContext: IDENTITY_LINE,
  referenceContext: REFERENCE_CONTEXT,
};

//endregion Fixtures

//region Scripted client

/**
 Count of numbered blocks a sheet opens with the marker.

 @param text - sheet text

 @param marker - line opening, such as `ISSUE `

 @returns Numbers one to the count, one per block

 @example
 ```ts
 const numbers = blockNumbers({ text, marker: 'ISSUE ', },);
 ```
 */
function blockNumbers({ text, marker, }: { readonly text: string; readonly marker: string; },): readonly number[] {
  return text
    .split('\n',)
    .filter(function opensBlock(line,) {
      return line.startsWith(marker,) && Number.isInteger(
        Number(line.slice(marker.length,).trim(),),
      );
    },)
    .map(function toNumber(_line, index,) {
      return index + 1;
    },);
}

/**
 Client answering every repair and refinement stage, recording each
 checker sheet in the order it was sent.

 @param checkerSheets - collector the checker sheets land in

 @returns Scripted client

 @example
 ```ts
 const client = recordingClient({ checkerSheets: [], },);
 ```
 */
function recordingClient({ checkerSheets, }: { readonly checkerSheets: string[]; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the checker drivers',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage name from the structured-output constraint.
       */
      const stage = request.responseFormat?.json_schema.name ?? '';
      /**
       Both messages joined, so a sheet's system rules are read too.
       */
      const content = request.messages
        .map(function toText(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);
      if (stage === 'resolution_report')
        checkerSheets.push(content,);
      /**
       Scripted wire reply for the stage.
       */
      const scripted: unknown = (stage === 'critic_report')
        ? {
          issues: [
            {
              category: 'accuracy/mistranslation',
              severity: 'major',
              summary: 'Sunbathing is rendered as hating it.',
              sourceQuote: '猫猫喜欢在窗台上晒太阳。',
              targetQuote: 'The cat hates sunbathing on the windowsill.',
            },
          ],
        }
        : (stage === 'panel_ballot')
          ? {
            verdicts: blockNumbers({ text: content, marker: 'CLAIM ', },).map(function toVerdict(claim,) {
              return { claim, reason: 'scripted', vote: 'supported', };
            },),
          }
          : (stage === 'editor_report')
            ? {
              edits: blockNumbers({ text: content, marker: 'REGION ', },).map(function toEdit(region,) {
                return { region, newText: SUN_REPAIR, };
              },),
            }
            : (stage === 'resolution_report')
              ? {
                checks: blockNumbers({ text: content, marker: 'ISSUE ', },).map(function toCheck(issue,) {
                  return { issue, verdict: 'fixed', };
                },),
              }
              : (stage === 'introduced_defect_report')
                ? {
                  checks: blockNumbers({ text: content, marker: 'REGION ', },).map(function toCheck(region,) {
                    return {
                      region,
                      verdict: 'no-introduced-defect-found',
                      category: '',
                      severity: '',
                      evidence: '',
                      omittedText: '',
                      verdictReason: '',
                    };
                  },),
                }
                : (stage === 'refine_report')
                  ? { rewrites: [{ paragraph: 1, newText: SMOOTH_TEXT, },], }
                  : (stage === 'candidate_ballot')
                    ? { best: 1, assessment: 'scripted', }
                    : undefined;
      if (scripted === undefined)
        throw new Error(`stub has no script for stage ${stage}`,);
      if (!request.validate(scripted,))
        throw new Error(`stub script failed the ${stage} guard`,);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the checker drivers',);
    },
  };
}

//endregion Scripted client

//region Drivers

/**
 Checker sheets the stage sends when handed the evidence directly.

 @param evidence - what the stage is handed

 @returns Every checker sheet sent

 @example
 ```ts
 const sheets = await stageSheets({ evidence: CARRIED, },);
 ```
 */
async function stageSheets({ evidence, }: { readonly evidence: Evidence; },): Promise<readonly string[]> {
  /**
   Checker sheets, in order.
   */
  const checkerSheets: string[] = [];
  await runCheckerStage({
    client: recordingClient({ checkerSheets, },),
    checkerModelIds: [...CHECKERS,],
    sourceText: SOURCE_TEXT,
    patchedText: SUN_REPAIR,
    issues: [
      {
        issueId: 'issue/sun',
        status: 'accepted',
        severity: 'major',
        claims: [],
        tallies: {},
      },
    ],
    authorship: UNATTRIBUTED_TEXT,
    ...evidence,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return checkerSheets;
}

/**
 Checker sheets the repair lane sends over one prepared document.

 @param evidence - what preparation carries into the lane

 @returns Every checker sheet sent

 @example
 ```ts
 const sheets = await repairLaneSheets({ evidence: CARRIED, },);
 ```
 */
async function repairLaneSheets({ evidence, }: { readonly evidence: Evidence; },): Promise<readonly string[]> {
  /**
   Checker sheets, in order.
   */
  const checkerSheets: string[] = [];
  await repairPreparedDocument({
    client: recordingClient({ checkerSheets, },),
    prepared: prepareDocumentPair({
      sourceText: SOURCE_TEXT,
      targetText: TARGET_TEXT,
      contextLines: (evidence.identityContext === undefined) ? [] : [evidence.identityContext,],
      referenceContext: evidence.referenceContext ?? '',
    },),
    models: MODELS,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return checkerSheets;
}

/**
 Repaired text of the refinement case, one long paragraph so it is eligible.
 */
const REPAIRED_TEXT = 'The cat is doing the sunbathing on the windowsill in every afternoon, and when the light is '
  + 'moving across the floor she is following it without any hurry at all.';

/**
 The one slice the refinement case rewrites.
 */
const SLICES: readonly ChunkPair[] = [
  {
    source: {
      sliceIndex: 0,
      text: '猫猫每天下午都在窗台上晒太阳。',
      startOffset: 0,
      endOffset: 15,
      nodes: [],
    },
    target: {
      sliceIndex: 0,
      text: REPAIRED_TEXT,
      startOffset: 0,
      endOffset: REPAIRED_TEXT.length,
      nodes: [],
    },
  },
];

/**
 Settled slice whose accuracy patch lost over one accepted issue, which the
 refinement recheck owes a round (ledger L11).
 */
const LOST_PATCH_OUTCOME: ChunkRepairOutcome = {
  sliceIndex: 0,
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
 Checker sheets the refinement recheck sends.

 @param evidence - what the phase is handed

 @returns Every checker sheet sent

 @example
 ```ts
 const sheets = await refineSheets({ evidence: CARRIED, },);
 ```
 */
async function refineSheets({ evidence, }: { readonly evidence: Evidence; },): Promise<readonly string[]> {
  /**
   Checker sheets, in order.
   */
  const checkerSheets: string[] = [];
  await runRefinePhase({
    declaredNames: [],
    client: recordingClient({ checkerSheets, },),
    targetText: REPAIRED_TEXT,
    slices: SLICES,
    outcomes: [LOST_PATCH_OUTCOME,],
    models: {
      ...MODELS,
      judgeModelIds: [
        SEAT_HYPER_OPENROUTER_VISION_EDITOR,
        ...CHECKERS,
        SEAT_HYPER_OPENROUTER_UNMEASURED,
      ],
      refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
    },
    ...evidence,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return checkerSheets;
}

//endregion Drivers

/**
 Heading line of the declared-identity block, as its fence opens it.
 */
const IDENTITY_HEADING = ' DECLARED NAMES ';

/**
 Heading line of the cited-reference block, as its fence opens it.
 */
const REFERENCE_HEADING = ' CITED REFERENCES, EVIDENCE ONLY ';

/**
 Whether a sheet opens a fenced block under the heading.

 THE FENCED LINE, NOT THE WORDS: the house policy every checker sheet states
 names the DECLARED NAMES block in prose, so the bare words are on a sheet for
 a page declaring nothing.

 @param sheet - one checker sheet

 @param heading - heading between the fences, spaces included

 @returns Whether a fence line carries it

 @example
 ```ts
 const shown = opensFencedBlock({ sheet, heading: IDENTITY_HEADING, },);
 ```
 */
function opensFencedBlock({ sheet, heading, }: { readonly sheet: string; readonly heading: string; },): boolean {
  return sheet
    .split('\n',)
    .some(function isHeading(line,) {
      return line.startsWith('=',) && line.includes(heading,);
    },);
}

/**
 What one set of checker sheets shows of the evidence.

 @param sheets - checker sheets one driver sent

 @returns Whether a round ran, and whether every sheet carried each block

 @example
 ```ts
 const shown = shownIn({ sheets, },);
 ```
 */
function shownIn({ sheets, }: { readonly sheets: readonly string[]; },): {
  readonly asked: boolean;
  readonly identity: boolean;
  readonly references: boolean;
} {
  return {
    asked: sheets.length > 0,
    identity: sheets.every(function carries(sheet,) {
      return opensFencedBlock({ sheet, heading: IDENTITY_HEADING, },) && sheet.includes(IDENTITY_MARK,);
    },),
    references: sheets.every(function carries(sheet,) {
      return opensFencedBlock({ sheet, heading: REFERENCE_HEADING, },) && sheet.includes(REFERENCE_MARK,);
    },),
  };
}

/**
 Whether any checker sheet opened either block.

 @param sheets - checker sheets one driver sent

 @returns Whether a round ran, and whether any sheet opened each block

 @example
 ```ts
 const shown = headingsIn({ sheets, },);
 ```
 */
function headingsIn({ sheets, }: { readonly sheets: readonly string[]; },): {
  readonly asked: boolean;
  readonly identity: boolean;
  readonly references: boolean;
} {
  return {
    asked: sheets.length > 0,
    identity: sheets.some(function shows(sheet,) {
      return opensFencedBlock({ sheet, heading: IDENTITY_HEADING, },);
    },),
    references: sheets.some(function shows(sheet,) {
      return opensFencedBlock({ sheet, heading: REFERENCE_HEADING, },);
    },),
  };
}

/**
 Every driver owning a checker round, by name.
 */
const DRIVERS: readonly {
  readonly name: string;
  readonly sheets: (request: { readonly evidence: Evidence; },) => Promise<readonly string[]>;
}[] = [
  {
    name: 'the checker stage',
    sheets: stageSheets,
  },
  {
    name: 'the repair lane\'s proof',
    sheets: repairLaneSheets,
  },
  {
    name: 'the refinement recheck',
    sheets: refineSheets,
  },
];

await describe({
  name: 'every checker round is sent the declared names and cited references (ledger L14)',
  children: DRIVERS.flatMap(function toCases({ name, sheets, },) {
    return [
      it({
        name: `${name} FORWARDS both blocks to every checker sheet`,
        fn: async () => {
          expect(shownIn({ sheets: await sheets({ evidence: CARRIED, },), },),).toEqual({
            asked: true,
            identity: true,
            references: true,
          },);
        },
      },),
      it({
        name: `${name} SENDS NEITHER block for a page declaring nothing and linking nowhere`,
        fn: async () => {
          expect(headingsIn({ sheets: await sheets({ evidence: {}, },), },),).toEqual({
            asked: true,
            identity: false,
            references: false,
          },);
        },
      },),
    ];
  },),
},);
