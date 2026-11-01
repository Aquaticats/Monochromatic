/**
 Tests for stripping the edits a checker voted worse from a winning patch.

 LEDGER L3 (the owner's ruling of 2026-09-28, "Revert worse-voted,
 recheck"): a selected repair patch carried edits whose issues the checkers
 did not confirm, 18 of 51 TianqiChen666 patches; TianqiChen66616 slice 3
 shipped such an edit that one checker had voted worse. An edit whose issue
 the checkers did not confirm and which drew a worse verdict is stripped, and
 the reduced patch faces one more checker round before it ships.

 TWO BALLOTS FOR UNDO TOO (owner, 2026-10-25, "Two ballots for undo too"): the
 worse verdict that strips is the one that sets `regressed` on the issue's
 tally, two or more ballots naming worse with the worse weight above the
 fixed and not-fixed weight together. One worse ballot strips nothing,
 whatever sits beside it; an unconfirmed edit without that verdict stays.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { repairTranslation, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Original with two paragraphs, one mistranslated sentence each.
 */
const SOURCE_TEXT = `## 简介

猫猫喜欢在窗台上晒太阳。

猫猫也喜欢追蝴蝶。
`;

/**
 Translation with both sentences mistranslated.
 */
const TARGET_TEXT = `## Introduction

The cat hates sunbathing on the windowsill.

The cat hates butterflies.
`;

/**
 Repair of the first sentence, which every checker confirms.
 */
const SUN_REPAIR = 'The cat loves sunbathing on the windowsill.';

/**
 Repair of the second sentence, which the checkers split on.
 */
const FLY_REPAIR = 'The cat also loves chasing butterflies.';

/**
 The two issues the critics raise, quoting exact fixture bytes.
 */
const ISSUES: readonly Record<string, unknown>[] = [
  {
    category: 'accuracy/mistranslation',
    severity: 'major',
    summary: 'Sunbathing is rendered as hating it.',
    sourceQuote: '猫猫喜欢在窗台上晒太阳。',
    targetQuote: 'The cat hates sunbathing on the windowsill.',
  },
  {
    category: 'accuracy/mistranslation',
    severity: 'major',
    summary: 'Chasing butterflies is rendered as hating them.',
    sourceQuote: '猫猫也喜欢追蝴蝶。',
    targetQuote: 'The cat hates butterflies.',
  },
];

/**
 Checkers in the order their butterfly verdicts are scripted.
 */
const CHECKERS = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Role roster; three checkers, since fewer cannot decide.
 */
const MODELS: RepairModels = {
  criticModelIds: [...CHECKERS,],
  panelModelIds: [...CHECKERS,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [...CHECKERS,],
  checkerModelIds: [...CHECKERS,],
};

/**
 Numbered blocks of a sheet: each line opening with the marker starts one,
 and the lines after it until the next belong to it.

 @param text - sheet text

 @param marker - line opening, such as `ISSUE `

 @returns Block number and the lines it holds
 */
function numberedBlocks(
  {
    text,
    marker,
  }: {
    readonly text: string;
    readonly marker: string;
  },
): readonly { readonly number: number; readonly lines: readonly string[]; }[] {
  /**
   The sheet's lines.
   */
  const lines = text.split('\n',);
  /**
   Where each block opens, and its number.
   */
  const opens = lines.flatMap(function toOpening(line, at,): readonly { readonly at: number; readonly number: number; }[] {
    /**
     Number after the marker, when the line opens a block.
     */
    const opened = line.startsWith(marker,) ? Number(line.slice(marker.length,).trim(),) : Number.NaN;
    return Number.isInteger(opened,)
      ? [{
        at,
        number: opened,
      },]
      : [];
  },);
  return opens.map(function toBlock({ at, number, }, index,) {
    return {
      number,
      lines: lines.slice(
        at + 1,
        opens[index + 1]?.at ?? lines.length,
      ),
    };
  },);
}

/**
 Client scripting every repair stage; the checkers confirm the sunbathing
 repair and cast the given verdicts, in checker order, on the butterfly one.

 @param flyVerdicts - each checker's verdict on the butterfly issue

 @param resolutionCalls - checker calls, counted

 @returns Scripted client
 */
function scriptedClient(
  {
    flyVerdicts,
    resolutionCalls,
  }: {
    readonly flyVerdicts: readonly string[];
    readonly resolutionCalls: { count: number; };
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the repair pipeline',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage name from the structured-output constraint.
       */
      const stage = request.responseFormat?.json_schema.name ?? '';
      /**
       Last message's text, which the sheets number.
       */
      const last = request.messages.at(-1,);
      const content = (last === undefined) ? '' : messageText({ message: last, },);
      if (stage === 'resolution_report')
        resolutionCalls.count += 1;
      /**
       This checker's place in the scripted order.
       */
      const checkerIndex = (CHECKERS as readonly string[]).indexOf(request.modelId,);
      /**
       Scripted wire reply for the stage.
       */
      const scripted: unknown = (stage === 'critic_report')
        ? { issues: ISSUES, }
        : (stage === 'panel_ballot')
          ? {
            verdicts: numberedBlocks({ text: content, marker: 'CLAIM ', },).map(function toVerdict({ number, },) {
              return { claim: number, reason: 'scripted', vote: 'supported', };
            },),
          }
          : (stage === 'editor_report')
            ? {
              edits: numberedBlocks({ text: content, marker: 'REGION ', },).map(function toEdit({ number, lines, },) {
                return {
                  region: number,
                  newText: lines.some(function isFly(line,) {
                    return line.startsWith('CURRENT TEXT: ',) && line.includes('butterflies',);
                  },)
                    ? FLY_REPAIR
                    : SUN_REPAIR,
                };
              },),
            }
            : (stage === 'resolution_report')
              ? {
                checks: numberedBlocks({ text: content, marker: 'ISSUE ', },).map(function toCheck({ number, lines, },) {
            /**
             Whether this block is the butterfly issue.
             */
                  const isFly = lines.some(function mentions(line,) {
                    return line.includes('butterflies',);
                  },);
                  return {
                    issue: number,
                    verdict: isFly ? (flyVerdicts[checkerIndex] ?? 'fixed') : 'fixed',
                  };
                },),
              }
              : (stage === 'introduced_defect_report')
                ? {
                  checks: numberedBlocks({ text: content, marker: 'REGION ', },).map(function toCheck({ number, },) {
                    return {
                      region: number,
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
                  ? { rewrites: [], }
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
      throw new Error('quotas unused by the repair pipeline',);
    },
  };
}

await describe({
  name: 'stripping worse-voted edits from a winning patch (ledger L3)',
  children: [
    it({
      name: 'STRIPS AN UNCONFIRMED EDIT TWO CHECKERS VOTED WORSE against one fixed and rechecks the reduced '
        + 'patch, which ships the confirmed repair alone',
      fn: async () => {
        /**
         Checker calls, counted.
         */
        const resolutionCalls = { count: 0, };
        const result = await repairTranslation({
          client: scriptedClient({
            flyVerdicts: [
              'fixed',
              'worse',
              'worse',
            ],
            resolutionCalls,
          },),
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          models: MODELS,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect({
          sunRepaired: result.repairedText.includes(SUN_REPAIR,),
          flyStripped: result.repairedText.includes('The cat hates butterflies.',),
          checkerRounds: resolutionCalls.count / CHECKERS.length,
        },).toEqual({
          sunRepaired: true,
          flyStripped: true,
          checkerRounds: 2,
        },);
      },
    },),
    it({
      name: 'KEEPS AN UNCONFIRMED EDIT NO CHECKER VOTED WORSE, on one checker round, as the ruling leaves it',
      fn: async () => {
        /**
         Checker calls, counted.
         */
        const resolutionCalls = { count: 0, };
        const result = await repairTranslation({
          client: scriptedClient({
            flyVerdicts: [
              'fixed',
              'not-fixed',
              'not-fixed',
            ],
            resolutionCalls,
          },),
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          models: MODELS,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect({
          sunRepaired: result.repairedText.includes(SUN_REPAIR,),
          flyKept: result.repairedText.includes(FLY_REPAIR,),
          checkerRounds: resolutionCalls.count / CHECKERS.length,
        },).toEqual({
          sunRepaired: true,
          flyKept: true,
          checkerRounds: 1,
        },);
      },
    },),
    it({
      name: 'KEEPS AN UNCONFIRMED EDIT ONE CHECKER VOTED WORSE beside one fixed and one not-fixed, on one checker '
        + 'round: one worse ballot is one voice, and one voice strips nothing (owner, 2026-10-25, '
        + '"Two ballots for undo too")',
      fn: async () => {
        /**
         Checker calls, counted.
         */
        const resolutionCalls = { count: 0, };
        const result = await repairTranslation({
          client: scriptedClient({
            flyVerdicts: [
              'fixed',
              'not-fixed',
              'worse',
            ],
            resolutionCalls,
          },),
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          models: MODELS,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect({
          sunRepaired: result.repairedText.includes(SUN_REPAIR,),
          flyKept: result.repairedText.includes(FLY_REPAIR,),
          checkerRounds: resolutionCalls.count / CHECKERS.length,
        },).toEqual({
          sunRepaired: true,
          flyKept: true,
          checkerRounds: 1,
        },);
      },
    },),
    it({
      name: 'KEEPS A CONFIRMED EDIT one checker voted worse, since the ruling strips only what the checkers did '
        + 'not confirm',
      fn: async () => {
        /**
         Checker calls, counted.
         */
        const resolutionCalls = { count: 0, };
        const result = await repairTranslation({
          client: scriptedClient({
            flyVerdicts: [
              'fixed',
              'fixed',
              'worse',
            ],
            resolutionCalls,
          },),
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          models: MODELS,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect({
          flyKept: result.repairedText.includes(FLY_REPAIR,),
          checkerRounds: resolutionCalls.count / CHECKERS.length,
        },).toEqual({
          flyKept: true,
          checkerRounds: 1,
        },);
      },
    },),
  ],
},);
