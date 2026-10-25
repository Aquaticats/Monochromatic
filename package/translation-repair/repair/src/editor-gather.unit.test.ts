/**
 Tests for the editor gather: the part of the editor stage that builds the sheet, asks the editors and turns what they
 said into patches through the apply gate, before any judge is involved.

 The editor fidelity probe seats one card through this function and nothing after it, so the cases pin two promises:
 it asks only the editors it is handed, and it asks them the sheet the stage asks them.

 Fixtures are cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type {
  AdjudicatedIssue,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  buildEditorMessages,
  gatherEditorProposals,
  runEditorStage,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the gather under test.
 */
const l = tagged({ tag: 'editor-gather-test', },);

/**
 Translation chunk the editors propose against.
 */
const TARGET_TEXT = 'The cat is doing the sleeping on the windowsill.';

/**
 Original the edits answer to.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉。';

/**
 Single editable envelope covering the whole chunk.
 */
const ENVELOPES: readonly EditableEnvelope[] = [
  {
    envelopeId: 'envelope/0',
    startOffset: 0,
    endOffset: TARGET_TEXT.length,
    baseText: TARGET_TEXT,
    baseHash: hashContent({ content: TARGET_TEXT, },),
    issueIds: ['adjudicated/tense',],
  },
];

/**
 Accepted issue the edit answers.
 */
const ISSUES: readonly AdjudicatedIssue[] = [
  {
    issueId: 'adjudicated/tense',
    status: 'accepted' as const,
    severity: 'major' as const,
    claims: [],
    tallies: {},
  },
];

/**
 What a scripted client saw and answers with.
 */
type ScriptedAsk = {
  /**
   Model ids asked, in order.
   */
  readonly modelIds: string[];

  /**
   Message lists sent, in order.
   */
  readonly sheets: (ChatJsonRequest<unknown>['messages'])[];
};

/**
 Client answering every exchange with one scripted report.

 @param report - report each exchange returns

 @param seen - record the exchanges are appended to

 @returns Client honoring that script

 @example
 ```ts
 const client = scriptedClient({ report: { edits: [], }, seen, },);
 ```
 */
function scriptedClient(
  {
    report,
    seen,
  }: {
    readonly report: unknown;
    readonly seen: ScriptedAsk;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      seen.modelIds.push(request.modelId,);
      seen.sheets.push(request.messages,);
      if (!request.validate(report,)) {
        return {
          kind: 'schema-mismatch',
          rawText: JSON.stringify(report,),
          detail: 'scripted report failed the editor guard',
        };
      }
      return {
        kind: 'ok',
        value: report,
        rawText: JSON.stringify(report,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Inputs both the gather and the stage are handed in a case.
 */
const COMMON = {
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT,
  envelopes: ENVELOPES,
  issues: ISSUES,
  perCallTimeoutMs: HANG_STOP_MS,
  l,
  stageSettings: SCRIPTED_STAGE_SETTINGS,
} as const;

await describe({
  name: gatherEditorProposals.name,
  children: [
    it({
      name: 'ASKS ONLY THE EDITOR IT IS HANDED, and returns that editor\'s patch through the apply gate, with no judge '
        + 'in the call at all, which is what lets a probe seat one card and spend on nothing else',
      fn: async () => {
        /**
         Exchanges the scripted client saw.
         */
        const seen: ScriptedAsk = {
          modelIds: [],
          sheets: [],
        };
        const gathered = await gatherEditorProposals({
          ...COMMON,
          client: scriptedClient({
            report: {
              edits: [
                {
                  region: 1,
                  newText: 'The cat sleeps on the windowsill.',
                },
              ],
            },
            seen,
          },),
          editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          signal: new AbortController().signal,
        },);

        expect(seen.modelIds,).toStrictEqual([SEAT_HYPER_OPENROUTER_VISION_EDITOR,],);
        expect(gathered.gather.voices.length,).toBe(1,);
        expect(gathered.built.candidates.length,).toBe(1,);
        expect(gathered.built.candidates[0]?.patch.patchedText,).toBe('The cat sleeps on the windowsill.',);
        expect(gathered.built.candidates[0]?.patch.applied.length,).toBe(1,);
        expect(gathered.unchanged.patchedText,).toBe(TARGET_TEXT,);
      },
    },),

    it({
      name: 'SENDS THE SHEET THE STAGE SENDS: the editor of the gather and the editor of `runEditorStage` are handed '
        + 'byte-identical messages, and both match `buildEditorMessages`, so the probe reads the seat as production '
        + 'uses it and not a copy of it',
      fn: async () => {
        /**
         Exchanges the gather made.
         */
        const gatherSeen: ScriptedAsk = {
          modelIds: [],
          sheets: [],
        };
        /**
         Exchanges the stage made.
         */
        const stageSeen: ScriptedAsk = {
          modelIds: [],
          sheets: [],
        };
        /**
         Report both clients return.
         */
        const report = { edits: [], };
        await gatherEditorProposals({
          ...COMMON,
          client: scriptedClient({
            report,
            seen: gatherSeen,
          },),
          editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          signal: new AbortController().signal,
        },);
        await runEditorStage({
          ...COMMON,
          client: scriptedClient({
            report,
            seen: stageSeen,
          },),
          editorModelIds: [
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            SEAT_SYNTHETIC_VISION_WITHHELD,
          ],
          judgeModelIds: [
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
          ],
          signal: new AbortController().signal,
        },);

        /**
         The sheet the prompt builder gives for these inputs.
         */
        const built = buildEditorMessages({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          envelopes: ENVELOPES,
          issues: ISSUES,
        },);
        expect(gatherSeen.sheets,).toStrictEqual([built.messages,],);
        expect(stageSeen.sheets.length,).toBeGreaterThan(0,);
        expect(stageSeen.sheets.every(function same(sheet,): boolean {
          return JSON.stringify(sheet,) === JSON.stringify(built.messages,);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'HEARS NOBODY when the editor\'s reply cannot be read, and builds no candidate, so the caller can count '
        + 'an unusable reply apart from a refusal',
      fn: async () => {
        /**
         Exchanges the scripted client saw.
         */
        const seen: ScriptedAsk = {
          modelIds: [],
          sheets: [],
        };
        const gathered = await gatherEditorProposals({
          ...COMMON,
          client: scriptedClient({
            report: { unrelated: true, },
            seen,
          },),
          editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          signal: new AbortController().signal,
        },);

        expect(gathered.gather.voices,).toStrictEqual([],);
        expect(gathered.built.candidates,).toStrictEqual([],);
        expect(seen.modelIds.length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'BUILDS A CANDIDATE THAT APPLIED NOTHING when the editor answered with no edit, which is a refusal and '
        + 'not a loss',
      fn: async () => {
        const gathered = await gatherEditorProposals({
          ...COMMON,
          client: scriptedClient({
            report: { edits: [], },
            seen: {
              modelIds: [],
              sheets: [],
            },
          },),
          editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          signal: new AbortController().signal,
        },);

        expect(gathered.gather.voices.length,).toBe(1,);
        expect(gathered.built.candidates[0]?.patch.applied,).toStrictEqual([],);
        expect(gathered.built.candidates[0]?.patch.rejected,).toStrictEqual([],);
      },
    },),
  ],
},);
